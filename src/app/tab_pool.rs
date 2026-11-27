use std::collections::{HashMap, VecDeque};
use std::sync::atomic::Ordering;
use std::time::{Duration, Instant};

#[cfg(target_os = "windows")]
use wry::{MemoryUsageLevel, WebViewExtWindows};

use crate::tabs::TabItem;

use super::webview::{ContentBounds, TabWebView};
#[cfg(target_os = "windows")]
use super::webview::{
    bring_hud_to_top, enable_clip_siblings, place_behind_hud, sync_webview_bounds_rect,
};

const PAUSE_MEDIA_JS: &str =
    "document.querySelectorAll('video, audio').forEach(el => { try { el.pause(); } catch(_) {} });";

#[cfg(target_os = "windows")]
#[inline]
fn bring_webview_below_hud(webview: &wry::WebView, hud_wv: Option<&wry::WebView>) {
    use windows::Win32::UI::WindowsAndMessaging::{
        SetWindowPos, HWND_TOP, SWP_NOACTIVATE, SWP_NOMOVE, SWP_NOSIZE,
    };
    if let Some(hud) = hud_wv {
        enable_clip_siblings(hud);
        bring_hud_to_top(hud);
        place_behind_hud(webview, hud);
    } else {
        let hwnd = webview.hwnd();
        unsafe {
            let _ = SetWindowPos(
                hwnd,
                Some(HWND_TOP),
                0,
                0,
                0,
                0,
                SWP_NOMOVE | SWP_NOSIZE | SWP_NOACTIVATE,
            );
        }
    }
}

#[cfg(target_os = "windows")]
#[inline]
fn place_behind_visible(pending: &wry::WebView, visible: &wry::WebView) {
    use windows::Win32::UI::WindowsAndMessaging::{
        SetWindowPos, ShowWindow, SWP_NOACTIVATE, SWP_NOMOVE, SWP_NOSIZE, SW_SHOWNA,
    };
    enable_clip_siblings(visible);
    enable_clip_siblings(pending);
    let pending_hwnd = pending.hwnd();
    let visible_hwnd = visible.hwnd();
    unsafe {
        let _ = SetWindowPos(
            pending_hwnd,
            Some(visible_hwnd),
            0,
            0,
            0,
            0,
            SWP_NOMOVE | SWP_NOSIZE | SWP_NOACTIVATE,
        );
        let _ = ShowWindow(pending_hwnd, SW_SHOWNA);
        let _ = pending.controller().SetIsVisible(true);
    }
}

/// Manages live per-tab WebViews, LRU eviction for normal tabs, a single recycled `about:blank`
/// controller, and flicker-free foreground visibility swaps.
pub struct TabPool {
    live_webviews: HashMap<usize, TabWebView>,
    lru_order: VecDeque<usize>,
    last_active_at: HashMap<usize, Instant>,
    runtime_hashes: HashMap<usize, u64>,
    idle_webview: Option<TabWebView>,
    visible_tab_id: Option<usize>,
    secondary_visible_tab_id: Option<usize>,
    pending_reveal: Option<(usize, u64)>,
    reveal_gen: u64,
}

impl TabPool {
    pub fn new(initial_id: usize, initial_tv: TabWebView) -> Self {
        let mut live_webviews = HashMap::new();
        live_webviews.insert(initial_id, initial_tv);
        let mut lru_order = VecDeque::new();
        lru_order.push_back(initial_id);
        let mut last_active_at = HashMap::new();
        last_active_at.insert(initial_id, Instant::now());
        Self {
            live_webviews,
            lru_order,
            last_active_at,
            runtime_hashes: HashMap::new(),
            idle_webview: None,
            visible_tab_id: Some(initial_id),
            secondary_visible_tab_id: None,
            pending_reveal: None,
            reveal_gen: 0,
        }
    }

    pub fn should_push_runtime(&mut self, id: usize, hash: u64) -> bool {
        if self.runtime_hashes.get(&id).copied() == Some(hash) {
            return false;
        }
        self.runtime_hashes.insert(id, hash);
        true
    }

    pub fn clear_runtime_hash(&mut self, id: usize) {
        self.runtime_hashes.remove(&id);
    }

    pub fn contains(&self, id: usize) -> bool {
        self.live_webviews.contains_key(&id)
    }

    pub fn live_ids(&self) -> Vec<usize> {
        self.live_webviews.keys().copied().collect()
    }

    pub fn live_count(&self) -> usize {
        self.live_webviews.len()
    }

    pub fn get_webview(&self, id: usize) -> Option<&wry::WebView> {
        self.live_webviews.get(&id).map(|tv| &tv.webview)
    }

    pub fn visible_id(&self) -> Option<usize> {
        self.visible_tab_id
    }

    pub fn visible_webview(&self) -> Option<&wry::WebView> {
        self.visible_tab_id
            .and_then(|vid| self.live_webviews.get(&vid))
            .map(|tv| &tv.webview)
    }

    pub fn active_or_visible_webview(&self, active_id: usize) -> Option<&wry::WebView> {
        self.get_webview(active_id).or_else(|| self.visible_webview())
    }

    pub fn pending_reveal_id(&self) -> Option<usize> {
        self.pending_reveal.map(|(id, _)| id)
    }

    pub fn pending_reveal_gen(&self, id: usize) -> Option<u64> {
        match self.pending_reveal {
            Some((pid, gen)) if pid == id => Some(gen),
            _ => None,
        }
    }

    pub fn is_pending_match(&self, id: usize, gen: u64) -> bool {
        self.pending_reveal == Some((id, gen))
    }

    pub fn set_pending_reveal(&mut self, id: usize) -> u64 {
        self.reveal_gen = self.reveal_gen.wrapping_add(1);
        self.pending_reveal = Some((id, self.reveal_gen));
        self.reveal_gen
    }

    pub fn clear_pending(&mut self) {
        if let Some((pid, _)) = self.pending_reveal.take() {
            if self.visible_tab_id != Some(pid) {
                if let Some(tv) = self.live_webviews.get(&pid) {
                    let _ = tv.webview.set_visible(false);
                }
            }
        }
    }

    /// Shows `id` behind the currently visible WebView (without focus or visual bleed) so WebView2
    /// can composite its first frame before swapping to the foreground.
    pub fn prepare_pending_behind(&self, id: usize, bounds: ContentBounds) {
        #[cfg(target_os = "windows")]
        {
            if let (Some(pending_tv), Some(vis_wv)) =
                (self.live_webviews.get(&id), self.visible_webview())
            {
                let _ = pending_tv.webview.set_memory_usage_level(MemoryUsageLevel::Normal);
                sync_webview_bounds_rect(&pending_tv.webview, bounds);
                place_behind_visible(&pending_tv.webview, vis_wv);
            }
        }
        #[cfg(not(target_os = "windows"))]
        let _ = (id, bounds);
    }

    pub fn resize_all(&self, bounds: ContentBounds) {
        #[cfg(target_os = "windows")]
        {
            // Only resize the visible and pending-reveal WebViews immediately so drag-resizing
            // never forces hidden background renderers to relayout; cached tabs sync on reveal.
            if let Some(vis) = self.visible_webview() {
                sync_webview_bounds_rect(vis, bounds);
            }
            if let Some((pid, _)) = self.pending_reveal {
                if self.visible_tab_id != Some(pid) {
                    if let Some(pending_tv) = self.live_webviews.get(&pid) {
                        sync_webview_bounds_rect(&pending_tv.webview, bounds);
                    }
                }
            }
        }
        #[cfg(not(target_os = "windows"))]
        let _ = bounds;
    }

    #[cfg(target_os = "windows")]
    pub fn show_secondary_tab(
        &mut self,
        id: usize,
        bounds: ContentBounds,
        hud_wv: Option<&wry::WebView>,
    ) {
        if let Some(prev) = self.secondary_visible_tab_id {
            if prev != id && Some(prev) != self.visible_tab_id {
                if let Some(prev_tv) = self.live_webviews.get(&prev) {
                    let _ = prev_tv.webview.set_visible(false);
                }
            }
        }
        if let Some(tv) = self.live_webviews.get(&id) {
            let _ = tv.webview.set_memory_usage_level(MemoryUsageLevel::Normal);
            enable_clip_siblings(&tv.webview);
            sync_webview_bounds_rect(&tv.webview, bounds);
            bring_webview_below_hud(&tv.webview, hud_wv);
            let _ = tv.webview.set_visible(true);
            self.secondary_visible_tab_id = Some(id);
            self.last_active_at.insert(id, Instant::now());
        }
    }

    #[cfg(target_os = "windows")]
    pub fn hide_secondary_tab(&mut self) {
        if let Some(prev) = self.secondary_visible_tab_id.take() {
            if Some(prev) != self.visible_tab_id {
                if let Some(prev_tv) = self.live_webviews.get(&prev) {
                    let _ = prev_tv.webview.set_visible(false);
                }
            }
        }
    }

    #[cfg(target_os = "windows")]
    pub fn resize_split(&self, left_bounds: ContentBounds, right_bounds: ContentBounds) {
        if let Some(vis) = self.visible_webview() {
            sync_webview_bounds_rect(vis, left_bounds);
        }
        if let Some(sid) = self.secondary_visible_tab_id {
            if Some(sid) != self.visible_tab_id {
                if let Some(sec_tv) = self.live_webviews.get(&sid) {
                    sync_webview_bounds_rect(&sec_tv.webview, right_bounds);
                }
            }
        }
    }

    pub fn notify_parent_moved(&self) {
        #[cfg(target_os = "windows")]
        if let Some(vis) = self.visible_webview() {
            unsafe {
                let _ = vis.controller().NotifyParentWindowPositionChanged();
            }
        }
    }

    pub fn touch_lru(&mut self, id: usize) {
        self.lru_order.retain(|&x| x != id);
        self.lru_order.push_back(id);
        self.last_active_at.insert(id, Instant::now());
    }

    pub fn insert_live(&mut self, id: usize, tv: TabWebView) {
        self.runtime_hashes.remove(&id);
        if let Some(prev) = self.live_webviews.insert(id, tv) {
            self.recycle_or_drop(prev);
        }
        self.touch_lru(id);
    }

    pub fn take_idle_for(&mut self, new_tab_id: usize) -> Option<TabWebView> {
        let tv = self.idle_webview.take()?;
        tv.bound_tab_id.store(new_tab_id, Ordering::Release);
        Some(tv)
    }

    /// Android `wry` owns a single `RustWebView` per `WryActivity` (creating another replaces
    /// `setContentView`, `set_visible` is a no-op, and `MainPipe` is `bounded(8)`). Rebind the
    /// single live WebView to `new_tab_id` instead of spawning or blanking secondary instances.
    #[cfg(target_os = "android")]
    pub fn rebind_single_webview(&mut self, new_tab_id: usize) -> Option<&wry::WebView> {
        let tv = self
            .live_webviews
            .remove(&new_tab_id)
            .or_else(|| {
                let first_key = self.live_webviews.keys().next().copied()?;
                self.live_webviews.remove(&first_key)
            })
            .or_else(|| self.idle_webview.take())?;
        tv.bound_tab_id.store(new_tab_id, Ordering::Release);
        self.live_webviews.clear();
        self.lru_order.clear();
        self.last_active_at.clear();
        self.runtime_hashes.clear();
        self.pending_reveal = None;
        self.live_webviews.insert(new_tab_id, tv);
        self.lru_order.push_back(new_tab_id);
        self.last_active_at.insert(new_tab_id, Instant::now());
        self.visible_tab_id = Some(new_tab_id);
        self.get_webview(new_tab_id)
    }

    pub fn pause_media(&self, id: usize) {
        if let Some(tv) = self.live_webviews.get(&id) {
            let _ = tv.webview.evaluate_script(PAUSE_MEDIA_JS);
        }
    }

    pub fn park_inactive(&self, id: usize, keep_media: bool, bridge_tok_js: &str) {
        if let Some(tv) = self.live_webviews.get(&id) {
            let mut script = format!(
                "window.__fnx_set_background__ && window.__fnx_set_background__(true, {}); window.__fynexo_finish_progress__ && window.__fynexo_finish_progress__(true, {});",
                bridge_tok_js, bridge_tok_js
            );
            if !keep_media {
                script.push(' ');
                script.push_str(PAUSE_MEDIA_JS);
            }
            let _ = tv.webview.evaluate_script(&script);
        }
    }

    pub fn unpark_active(&self, id: usize, bridge_tok_js: &str) {
        if let Some(tv) = self.live_webviews.get(&id) {
            let script = format!(
                "window.__fnx_set_background__ && window.__fnx_set_background__(false, {}); window.__fynexo_finish_progress__ && window.__fynexo_finish_progress__(true, {});",
                bridge_tok_js, bridge_tok_js
            );
            let _ = tv.webview.evaluate_script(&script);
        }
    }

    /// Shows `id`'s WebView immediately behind `hud_wv` before hiding the previously visible
    /// WebView, avoiding any empty/black intermediate frame.
    pub fn reveal_tab(
        &mut self,
        id: usize,
        bounds: ContentBounds,
        hud_wv: Option<&wry::WebView>,
        focus_tab: bool,
        bridge_tok_js: &str,
    ) {
        if !self.live_webviews.contains_key(&id) {
            return;
        }
        if self.pending_reveal_id() == Some(id) {
            self.pending_reveal = None;
        } else {
            self.clear_pending();
        }
        if let Some(next_tv) = self.live_webviews.get(&id) {
            #[cfg(target_os = "windows")]
            {
                use windows::Win32::UI::WindowsAndMessaging::IsWindowVisible;
                let _ = next_tv.webview.set_memory_usage_level(MemoryUsageLevel::Normal);
                enable_clip_siblings(&next_tv.webview);
                sync_webview_bounds_rect(&next_tv.webview, bounds);
                bring_webview_below_hud(&next_tv.webview, hud_wv);
                if unsafe { IsWindowVisible(next_tv.webview.hwnd()).as_bool() } {
                    unsafe {
                        let _ = next_tv.webview.controller().SetIsVisible(true);
                    }
                } else {
                    let _ = next_tv.webview.set_visible(true);
                }
            }
            #[cfg(not(target_os = "windows"))]
            {
                let _ = (bounds, hud_wv);
                let _ = next_tv.webview.set_visible(true);
            }
            if focus_tab {
                let _ = next_tv.webview.focus();
            }
        }
        let clear_script = format!(
            "window.__fynexo_finish_progress__ && window.__fynexo_finish_progress__(true, {});",
            bridge_tok_js
        );
        let prev_id_opt = self.visible_tab_id;
        let sec_id_opt = self.secondary_visible_tab_id;
        for (&other_id, other_tv) in &self.live_webviews {
            if other_id != id && Some(other_id) != sec_id_opt {
                if prev_id_opt == Some(other_id) {
                    let _ = other_tv.webview.evaluate_script(&clear_script);
                }
                let _ = other_tv.webview.set_visible(false);
            }
        }
        if let Some(prev_id) = prev_id_opt {
            if prev_id != id {
                self.last_active_at.insert(prev_id, Instant::now());
            }
        }
        self.visible_tab_id = Some(id);
        self.last_active_at.insert(id, Instant::now());
    }

    fn recycle_or_drop(&mut self, tv: TabWebView) {
        let _ = tv.webview.evaluate_script(PAUSE_MEDIA_JS);
        tv.bound_tab_id.store(0, Ordering::Release);
        let _ = tv.webview.set_visible(false);
        if self.idle_webview.is_none() {
            #[cfg(target_os = "windows")]
            {
                let _ = tv.webview.set_memory_usage_level(MemoryUsageLevel::Low);
            }
            let _ = tv.webview.load_url("about:blank");
            self.idle_webview = Some(tv);
        }
    }

    pub fn evict_excess_normal_tabs(
        &mut self,
        tabs: &[TabItem],
        active_id: usize,
        max_cached_normal: usize,
    ) -> usize {
        #[cfg(target_os = "android")]
        {
            let _ = (tabs, active_id, max_cached_normal);
            0
        }
        #[cfg(not(target_os = "android"))]
        {
        let visible_id = self.visible_tab_id;
        let pending_id = self.pending_reveal_id();
        let is_bg = |tid: usize| tabs.iter().any(|t| t.id == tid && t.background_play);
        let limit = max_cached_normal.max(1);

        let mut cached_normal = self
            .live_webviews
            .keys()
            .filter(|&&tid| !is_bg(tid))
            .count();
        let mut evicted = 0usize;

        while cached_normal > limit {
            let victim_pos = self.lru_order.iter().position(|&tid| {
                self.live_webviews.contains_key(&tid)
                    && tid != active_id
                    && visible_id != Some(tid)
                    && pending_id != Some(tid)
                    && !is_bg(tid)
            });
            let Some(pos) = victim_pos else {
                break;
            };
            if let Some(victim_id) = self.lru_order.remove(pos) {
                self.last_active_at.remove(&victim_id);
                self.runtime_hashes.remove(&victim_id);
                if let Some(tv) = self.live_webviews.remove(&victim_id) {
                    self.recycle_or_drop(tv);
                    cached_normal -= 1;
                    evicted += 1;
                }
            }
        }
        evicted
        }
    }

    /// Puts idle background tabs that exceed `max_idle` to sleep to reclaim RAM automatically.
    pub fn sleep_idle_tabs(
        &mut self,
        tabs: &[TabItem],
        active_id: usize,
        max_idle: Duration,
    ) -> usize {
        #[cfg(target_os = "android")]
        {
            let _ = (tabs, active_id, max_idle);
            0
        }
        #[cfg(not(target_os = "android"))]
        {
        if max_idle.is_zero() {
            return 0;
        }
        let now = Instant::now();
        let visible_id = self.visible_tab_id;
        let pending_id = self.pending_reveal_id();
        let victims: Vec<usize> = self
            .live_webviews
            .keys()
            .copied()
            .filter(|&tid| {
                if tid == active_id || visible_id == Some(tid) || pending_id == Some(tid) {
                    return false;
                }
                if tabs
                    .iter()
                    .any(|t| t.id == tid && (t.background_play || t.playing_audio))
                {
                    return false;
                }
                self.last_active_at
                    .get(&tid)
                    .map(|t| now.duration_since(*t) >= max_idle)
                    .unwrap_or(false)
            })
            .collect();
        let mut slept = 0usize;
        for tid in victims {
            self.lru_order.retain(|&x| x != tid);
            self.last_active_at.remove(&tid);
            self.runtime_hashes.remove(&tid);
            if let Some(tv) = self.live_webviews.remove(&tid) {
                self.recycle_or_drop(tv);
                slept += 1;
            }
        }
        slept
        }
    }

    /// Keeps only the most recent `hot_limit` tabs at `MemoryUsageLevel::Normal` (hot frame buffer)
    /// when `efficiency_mode` is enabled, cooling older cached WebViews to `MemoryUsageLevel::Low`.
    pub fn apply_memory_tiers(&self, active_id: usize, hot_limit: usize, efficiency_mode: bool) {
        #[cfg(target_os = "windows")]
        {
            if !efficiency_mode {
                for tv in self.live_webviews.values() {
                    let _ = tv.webview.set_memory_usage_level(MemoryUsageLevel::Normal);
                }
                return;
            }
            let target_hot = hot_limit.max(1);
            let mut hot_ids: Vec<usize> = Vec::with_capacity(target_hot + 1);
            if active_id != 0 && self.live_webviews.contains_key(&active_id) {
                hot_ids.push(active_id);
            }
            if let Some(vid) = self.visible_tab_id {
                if !hot_ids.contains(&vid) && self.live_webviews.contains_key(&vid) {
                    hot_ids.push(vid);
                }
            }
            if let Some(pid) = self.pending_reveal_id() {
                if !hot_ids.contains(&pid) && self.live_webviews.contains_key(&pid) {
                    hot_ids.push(pid);
                }
            }
            for &tid in self.lru_order.iter().rev() {
                if hot_ids.len() >= target_hot {
                    break;
                }
                if !hot_ids.contains(&tid) && self.live_webviews.contains_key(&tid) {
                    hot_ids.push(tid);
                }
            }
            for (&tid, tv) in &self.live_webviews {
                let level = if hot_ids.contains(&tid) {
                    MemoryUsageLevel::Normal
                } else {
                    MemoryUsageLevel::Low
                };
                let _ = tv.webview.set_memory_usage_level(level);
            }
        }
        #[cfg(not(target_os = "windows"))]
        {
            let _ = (active_id, hot_limit, efficiency_mode);
        }
    }

    pub fn prune_closed_tabs(&mut self, tabs: &[TabItem]) {
        #[cfg(target_os = "android")]
        {
            let _ = tabs;
        }
        #[cfg(not(target_os = "android"))]
        {
        let dead_ids: Vec<usize> = self
            .live_webviews
            .keys()
            .copied()
            .filter(|tid| !tabs.iter().any(|t| t.id == *tid))
            .collect();
        for tid in dead_ids {
            self.lru_order.retain(|&x| x != tid);
            self.last_active_at.remove(&tid);
            self.runtime_hashes.remove(&tid);
            if self.visible_tab_id == Some(tid) {
                self.visible_tab_id = None;
            }
            if self.pending_reveal_id() == Some(tid) {
                self.pending_reveal = None;
            }
            if let Some(tv) = self.live_webviews.remove(&tid) {
                self.recycle_or_drop(tv);
            }
        }
        }
    }

    pub fn reset_all(&mut self) {
        #[cfg(target_os = "android")]
        {
            self.pending_reveal = None;
            self.runtime_hashes.clear();
        }
        #[cfg(not(target_os = "android"))]
        {
        self.visible_tab_id = None;
        self.pending_reveal = None;
        self.lru_order.clear();
        self.last_active_at.clear();
        self.runtime_hashes.clear();
        let drained: Vec<TabWebView> = self.live_webviews.drain().map(|(_, tv)| tv).collect();
        for tv in drained {
            self.recycle_or_drop(tv);
        }
        }
    }

    pub fn clear_all(&mut self) {
        self.visible_tab_id = None;
        self.pending_reveal = None;
        self.lru_order.clear();
        self.last_active_at.clear();
        self.runtime_hashes.clear();
        self.live_webviews.clear();
        self.idle_webview = None;
    }

    pub fn drop_crashed(&mut self, tab_id: usize) {
        self.lru_order.retain(|&x| x != tab_id);
        self.last_active_at.remove(&tab_id);
        self.runtime_hashes.remove(&tab_id);
        if self.visible_tab_id == Some(tab_id) {
            self.visible_tab_id = None;
        }
        if self.pending_reveal_id() == Some(tab_id) {
            self.pending_reveal = None;
        }
        if let Some(tv) = self.live_webviews.remove(&tab_id) {
            tv.bound_tab_id.store(0, std::sync::atomic::Ordering::Release);
            drop(tv);
        }
    }

    pub fn all_webviews(&self) -> impl Iterator<Item = &wry::WebView> {
        self.live_webviews.values().map(|tv| &tv.webview)
    }

    #[cfg(target_os = "windows")]
    pub fn set_all_memory_low(&self) {
        for tv in self.live_webviews.values() {
            let _ = tv.webview.set_memory_usage_level(MemoryUsageLevel::Low);
        }
        if let Some(ref idle) = self.idle_webview {
            let _ = idle.webview.set_memory_usage_level(MemoryUsageLevel::Low);
        }
    }
}
