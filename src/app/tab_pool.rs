use std::collections::{HashMap, VecDeque};
use std::sync::atomic::Ordering;

#[cfg(target_os = "windows")]
use wry::{MemoryUsageLevel, WebViewExtWindows};

use crate::tabs::TabItem;

use super::webview::TabWebView;
#[cfg(target_os = "windows")]
use super::webview::{enable_clip_siblings, sync_webview_bounds};

const PAUSE_MEDIA_JS: &str =
    "document.querySelectorAll('video, audio').forEach(el => { try { el.pause(); } catch(_) {} });";

#[cfg(target_os = "windows")]
#[inline]
fn bring_webview_to_front(webview: &wry::WebView) {
    use windows::Win32::UI::WindowsAndMessaging::{
        SetWindowPos, HWND_TOP, SWP_NOACTIVATE, SWP_NOMOVE, SWP_NOSIZE,
    };
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
    idle_webview: Option<TabWebView>,
    visible_tab_id: Option<usize>,
    pending_reveal: Option<(usize, u64)>,
    reveal_gen: u64,
}

impl TabPool {
    pub fn new(initial_id: usize, initial_tv: TabWebView) -> Self {
        let mut live_webviews = HashMap::new();
        live_webviews.insert(initial_id, initial_tv);
        let mut lru_order = VecDeque::new();
        lru_order.push_back(initial_id);
        Self {
            live_webviews,
            lru_order,
            idle_webview: None,
            visible_tab_id: Some(initial_id),
            pending_reveal: None,
            reveal_gen: 0,
        }
    }

    pub fn contains(&self, id: usize) -> bool {
        self.live_webviews.contains_key(&id)
    }

    pub fn live_ids(&self) -> Vec<usize> {
        self.live_webviews.keys().copied().collect()
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
    pub fn prepare_pending_behind(&self, id: usize, win_size: (u32, u32)) {
        #[cfg(target_os = "windows")]
        {
            if let (Some(pending_tv), Some(vis_wv)) =
                (self.live_webviews.get(&id), self.visible_webview())
            {
                let _ = pending_tv.webview.set_memory_usage_level(MemoryUsageLevel::Normal);
                sync_webview_bounds(&pending_tv.webview, win_size.0, win_size.1);
                place_behind_visible(&pending_tv.webview, vis_wv);
            }
        }
        #[cfg(not(target_os = "windows"))]
        let _ = (id, win_size);
    }

    pub fn resize_all(&self, width: u32, height: u32) {
        #[cfg(target_os = "windows")]
        {
            // Only resize the visible and pending-reveal WebViews immediately so drag-resizing
            // never forces hidden background renderers to relayout; cached tabs sync on reveal.
            if let Some(vis) = self.visible_webview() {
                sync_webview_bounds(vis, width, height);
            }
            if let Some((pid, _)) = self.pending_reveal {
                if self.visible_tab_id != Some(pid) {
                    if let Some(pending_tv) = self.live_webviews.get(&pid) {
                        sync_webview_bounds(&pending_tv.webview, width, height);
                    }
                }
            }
        }
        #[cfg(not(target_os = "windows"))]
        let _ = (width, height);
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
    }

    pub fn insert_live(&mut self, id: usize, tv: TabWebView) {
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

    /// Shows `id`'s WebView in front before hiding the previously visible WebView,
    /// avoiding any empty/black intermediate frame.
    pub fn reveal_tab(&mut self, id: usize, win_size: (u32, u32), bridge_tok_js: &str) {
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
                sync_webview_bounds(&next_tv.webview, win_size.0, win_size.1);
                bring_webview_to_front(&next_tv.webview);
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
                let _ = win_size;
                let _ = next_tv.webview.set_visible(true);
            }
            let _ = next_tv.webview.focus();
        }
        if let Some(prev_id) = self.visible_tab_id {
            if prev_id != id {
                if let Some(prev_tv) = self.live_webviews.get(&prev_id) {
                    let clear_script = format!(
                        "window.__fynexo_finish_progress__ && window.__fynexo_finish_progress__(true, {});",
                        bridge_tok_js
                    );
                    let _ = prev_tv.webview.evaluate_script(&clear_script);
                    let _ = prev_tv.webview.set_visible(false);
                }
            }
        }
        self.visible_tab_id = Some(id);
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
    ) {
        let visible_id = self.visible_tab_id;
        let pending_id = self.pending_reveal_id();
        let is_bg = |tid: usize| tabs.iter().any(|t| t.id == tid && t.background_play);
        let limit = max_cached_normal.max(1);

        let mut cached_normal = self
            .live_webviews
            .keys()
            .filter(|&&tid| !is_bg(tid))
            .count();

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
                if let Some(tv) = self.live_webviews.remove(&victim_id) {
                    self.recycle_or_drop(tv);
                    cached_normal -= 1;
                }
            }
        }
    }

    /// Keeps only the most recent `hot_limit` tabs at `MemoryUsageLevel::Normal` (hot frame buffer),
    /// while cooling older cached/background WebViews to `MemoryUsageLevel::Low` to release bitmaps.
    pub fn apply_memory_tiers(&self, active_id: usize, hot_limit: usize) {
        #[cfg(target_os = "windows")]
        {
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
            let _ = (active_id, hot_limit);
        }
    }

    pub fn prune_closed_tabs(&mut self, tabs: &[TabItem]) {
        let dead_ids: Vec<usize> = self
            .live_webviews
            .keys()
            .copied()
            .filter(|tid| !tabs.iter().any(|t| t.id == *tid))
            .collect();
        for tid in dead_ids {
            self.lru_order.retain(|&x| x != tid);
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

    pub fn reset_all(&mut self) {
        self.visible_tab_id = None;
        self.pending_reveal = None;
        self.lru_order.clear();
        let drained: Vec<TabWebView> = self.live_webviews.drain().map(|(_, tv)| tv).collect();
        for tv in drained {
            self.recycle_or_drop(tv);
        }
    }

    pub fn clear_all(&mut self) {
        self.visible_tab_id = None;
        self.pending_reveal = None;
        self.lru_order.clear();
        self.live_webviews.clear();
        self.idle_webview = None;
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
