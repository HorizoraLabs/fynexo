use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

use crate::config::data_dir;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct TabItem {
    pub id: usize,
    pub title: String,
    pub url: String,
    pub is_active: bool,
    #[serde(default)]
    pub scroll_y: f64,
    #[serde(default)]
    pub pinned: bool,
    #[serde(default)]
    pub background_play: bool,
    #[serde(default)]
    pub muted: bool,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct WorkspaceSnapshot {
    pub name: String,
    pub tabs: Vec<TabItem>,
    pub active_id: usize,
    pub updated_at: u64,
}

#[derive(Clone, Debug)]
pub struct ClosedTabEntry {
    pub title: String,
    pub url: String,
    pub scroll_y: f64,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct TabState {
    pub tabs: Vec<TabItem>,
    pub active_id: usize,
    #[serde(default)]
    pub closed_count: usize,
}

pub struct TabManager {
    pub tabs: Vec<TabItem>,
    pub active_id: usize,
    counter: usize,
    closed_tabs: Vec<ClosedTabEntry>,
    incognito: bool,
}

#[inline]
fn initial_title_for(url: &str) -> String {
    if url.starts_with("fynexo://newtab") {
        "Start page".to_string()
    } else if url.starts_with("fynexo://settings") {
        "Preferences".to_string()
    } else {
        "New tab".to_string()
    }
}

impl TabManager {
    pub fn session_path() -> PathBuf {
        data_dir().join("fynexo_session.json")
    }

    pub fn new(homepage: &str, incognito: bool) -> Self {
        let first_tab = TabItem {
            id: 1,
            title: initial_title_for(homepage),
            url: homepage.to_string(),
            is_active: true,
            scroll_y: 0.0,
            pinned: false,
            background_play: false,
            muted: false,
        };
        Self {
            tabs: vec![first_tab],
            active_id: 1,
            counter: 1,
            closed_tabs: Vec::new(),
            incognito,
        }
    }

    pub fn load_or_new(homepage: &str, restore_session: bool, incognito: bool) -> Self {
        if restore_session {
            let path = Self::session_path();
            if path.exists() {
                if let Ok(raw) = fs::read_to_string(&path) {
                    if let Ok(mut state) = serde_json::from_str::<TabState>(&raw) {
                        if !state.tabs.is_empty() {
                            let max_id = state.tabs.iter().map(|t| t.id).max().unwrap_or(1);
                            let active_exists = state.tabs.iter().any(|t| t.id == state.active_id);
                            if !active_exists {
                                state.active_id = state.tabs.last().map(|t| t.id).unwrap_or(1);
                            }
                            for t in &mut state.tabs {
                                t.is_active = t.id == state.active_id;
                            }
                            state.tabs.sort_by_key(|t| !t.pinned);
                            return Self {
                                tabs: state.tabs,
                                active_id: state.active_id,
                                counter: max_id.max(1),
                                closed_tabs: Vec::new(),
                                incognito,
                            };
                        }
                    }
                }
            }
        }
        let fresh = Self::new(homepage, incognito);
        fresh.save();
        fresh
    }

    pub fn set_incognito(&mut self, incognito: bool) {
        self.incognito = incognito;
    }

    pub fn save(&self) {
        if self.incognito {
            return;
        }
        let state = self.get_state();
        if let Ok(json) = serde_json::to_string_pretty(&state) {
            let _ = fs::write(Self::session_path(), json);
        }
    }

    pub fn get_state(&self) -> TabState {
        TabState {
            tabs: self.tabs.clone(),
            active_id: self.active_id,
            closed_count: self.closed_tabs.len(),
        }
    }

    pub fn active_url(&self) -> String {
        self.tabs
            .iter()
            .find(|t| t.id == self.active_id)
            .map(|t| t.url.clone())
            .unwrap_or_else(|| "fynexo://newtab".to_string())
    }

    pub fn active_scroll_y(&self) -> f64 {
        self.tabs
            .iter()
            .find(|t| t.id == self.active_id)
            .map(|t| t.scroll_y)
            .unwrap_or(0.0)
    }

    pub fn new_tab(&mut self, url: &str) -> (usize, String) {
        self.counter += 1;
        let id = self.counter;
        for t in &mut self.tabs {
            t.is_active = false;
        }
        self.tabs.push(TabItem {
            id,
            title: initial_title_for(url),
            url: url.to_string(),
            is_active: true,
            scroll_y: 0.0,
            pinned: false,
            background_play: false,
            muted: false,
        });
        self.active_id = id;
        self.save();
        (id, url.to_string())
    }

    pub fn toggle_pin_tab(&mut self, id: usize) -> Option<bool> {
        let new_pinned = {
            let tab = self.tabs.iter_mut().find(|t| t.id == id)?;
            tab.pinned = !tab.pinned;
            tab.pinned
        };
        self.tabs.sort_by_key(|t| !t.pinned);
        self.save();
        Some(new_pinned)
    }

    pub fn switch_tab(&mut self, id: usize) -> Option<String> {
        if self.tabs.iter().any(|t| t.id == id) {
            for t in &mut self.tabs {
                t.is_active = t.id == id;
            }
            self.active_id = id;
            self.save();
            Some(self.active_url())
        } else {
            None
        }
    }

    fn push_closed(&mut self, item: &TabItem) {
        if self.incognito {
            return;
        }
        let u = item.url.trim();
        if u.is_empty()
            || u == "about:blank"
            || u.starts_with("fynexo://newtab")
            || u.starts_with("fynexo://settings")
        {
            return;
        }
        self.closed_tabs.push(ClosedTabEntry {
            title: item.title.clone(),
            url: u.to_string(),
            scroll_y: item.scroll_y,
        });
        if self.closed_tabs.len() > 12 {
            self.closed_tabs.remove(0);
        }
    }

    pub fn reopen_closed_tab(&mut self) -> Option<(String, f64)> {
        let entry = self.closed_tabs.pop()?;
        self.counter += 1;
        let id = self.counter;
        for t in &mut self.tabs {
            t.is_active = false;
        }
        self.tabs.push(TabItem {
            id,
            title: entry.title,
            url: entry.url.clone(),
            is_active: true,
            scroll_y: entry.scroll_y,
            pinned: false,
            background_play: false,
            muted: false,
        });
        self.active_id = id;
        self.save();
        Some((entry.url, entry.scroll_y))
    }

    pub fn duplicate_tab(&mut self, id: usize) -> Option<(usize, String)> {
        let (pos, target) = self.tabs.iter().enumerate().find(|(_, t)| t.id == id)?;
        let clone_item = target.clone();
        self.counter += 1;
        let new_id = self.counter;
        for t in &mut self.tabs {
            t.is_active = false;
        }
        let new_tab = TabItem {
            id: new_id,
            title: clone_item.title,
            url: clone_item.url.clone(),
            is_active: true,
            scroll_y: clone_item.scroll_y,
            pinned: false,
            background_play: clone_item.background_play,
            muted: clone_item.muted,
        };
        self.tabs.insert(pos + 1, new_tab);
        self.active_id = new_id;
        self.save();
        Some((new_id, clone_item.url))
    }

    pub fn close_other_tabs(&mut self, keep_id: usize, default_url: &str) -> Option<String> {
        let to_close: Vec<usize> = self.tabs.iter()
            .filter(|t| t.id != keep_id && !t.pinned)
            .map(|t| t.id)
            .collect();
        for id in to_close {
            let _ = self.close_tab(id, default_url, true);
        }
        if self.active_id != keep_id {
            self.switch_tab(keep_id)
        } else {
            None
        }
    }

    pub fn close_tabs_to_right(&mut self, from_id: usize, default_url: &str) -> Option<String> {
        let pos = self.tabs.iter().position(|t| t.id == from_id)?;
        let to_close: Vec<usize> = self.tabs.iter()
            .skip(pos + 1)
            .filter(|t| !t.pinned)
            .map(|t| t.id)
            .collect();
        for id in to_close {
            let _ = self.close_tab(id, default_url, true);
        }
        if !self.tabs.iter().any(|t| t.id == self.active_id) {
            self.switch_tab(from_id)
        } else {
            None
        }
    }

    pub fn close_all_tabs(&mut self, default_url: &str) -> String {
        let snapshots = self.tabs.clone();
        for t in &snapshots {
            self.push_closed(t);
        }
        self.tabs.clear();
        self.counter += 1;
        let id = self.counter;
        self.tabs.push(TabItem {
            id,
            title: initial_title_for(default_url),
            url: default_url.to_string(),
            is_active: true,
            scroll_y: 0.0,
            pinned: false,
            background_play: false,
            muted: false,
        });
        self.active_id = id;
        self.save();
        default_url.to_string()
    }

    pub fn close_tab(
        &mut self,
        id: usize,
        default_url: &str,
        force_unpin: bool,
    ) -> Result<Option<String>, &'static str> {
        if let Some(target) = self.tabs.iter().find(|t| t.id == id) {
            if target.pinned && !force_unpin {
                return Err("Tab is pinned — right-click or tap pin to unpin first");
            }
            let snapshot = target.clone();
            self.push_closed(&snapshot);
        } else {
            return Ok(None);
        }

        if self.tabs.len() <= 1 {
            if let Some(t) = self.tabs.first_mut() {
                t.title = initial_title_for(default_url);
                t.url = default_url.to_string();
                t.scroll_y = 0.0;
                t.pinned = false;
                self.save();
                return Ok(Some(default_url.to_string()));
            }
            return Ok(None);
        }

        let was_active = self.active_id == id;
        let pos = self.tabs.iter().position(|t| t.id == id);
        self.tabs.retain(|t| t.id != id);

        let res = if was_active {
            let next_idx = match pos {
                Some(idx) if idx < self.tabs.len() => idx,
                Some(_) if !self.tabs.is_empty() => self.tabs.len() - 1,
                _ => 0,
            };
            if let Some(next) = self.tabs.get_mut(next_idx) {
                next.is_active = true;
                self.active_id = next.id;
                Some(next.url.clone())
            } else {
                None
            }
        } else {
            None
        };
        self.save();
        Ok(res)
    }

    pub fn update_active_info(&mut self, title: Option<&str>, url: Option<&str>) -> bool {
        let mut changed = false;
        if let Some(t) = self.tabs.iter_mut().find(|t| t.id == self.active_id) {
            if let Some(url) = url {
                let trimmed = url.trim();
                if !trimmed.is_empty() && t.url != trimmed {
                    t.url.clear();
                    t.url.push_str(trimmed);
                    t.scroll_y = 0.0;
                    changed = true;
                }
            }
            if let Some(title) = title {
                let trimmed = title.trim();
                if !trimmed.is_empty() && t.title != trimmed {
                    t.title.clear();
                    t.title.push_str(trimmed);
                    changed = true;
                }
            }
        }
        if changed {
            self.save();
        }
        changed
    }

    pub fn update_active_scroll(&mut self, scroll_y: f64) -> bool {
        let clamped = if scroll_y.is_finite() && scroll_y >= 0.0 {
            scroll_y
        } else {
            0.0
        };
        if let Some(t) = self.tabs.iter_mut().find(|t| t.id == self.active_id) {
            if (t.scroll_y - clamped).abs() > 24.0 {
                t.scroll_y = clamped;
                return true;
            }
        }
        false
    }

    pub fn toggle_background_play(&mut self, id: usize) -> Option<bool> {
        let t = self.tabs.iter_mut().find(|t| t.id == id)?;
        t.background_play = !t.background_play;
        let state = t.background_play;
        self.save();
        Some(state)
    }

    pub fn toggle_mute_tab(&mut self, id: usize) -> Option<bool> {
        let t = self.tabs.iter_mut().find(|t| t.id == id)?;
        t.muted = !t.muted;
        let state = t.muted;
        self.save();
        Some(state)
    }

    pub fn set_tab_muted(&mut self, id: usize, muted: bool) -> Option<bool> {
        let t = self.tabs.iter_mut().find(|t| t.id == id)?;
        t.muted = muted;
        self.save();
        Some(muted)
    }

    pub fn workspaces_path() -> PathBuf {
        data_dir().join("fynexo_workspaces.json")
    }

    pub fn list_workspaces() -> Vec<WorkspaceSnapshot> {
        let path = Self::workspaces_path();
        if path.exists() {
            if let Ok(raw) = fs::read_to_string(&path) {
                if let Ok(list) = serde_json::from_str::<Vec<WorkspaceSnapshot>>(&raw) {
                    return list;
                }
            }
        }
        Vec::new()
    }

    pub fn save_workspace(&self, name: &str) {
        let trimmed = name.trim();
        if trimmed.is_empty() {
            return;
        }
        let mut list = Self::list_workspaces();
        list.retain(|w| w.name != trimmed);
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        list.insert(
            0,
            WorkspaceSnapshot {
                name: trimmed.to_string(),
                tabs: self.tabs.clone(),
                active_id: self.active_id,
                updated_at: now,
            },
        );
        if let Ok(json) = serde_json::to_string_pretty(&list) {
            let _ = fs::write(Self::workspaces_path(), json);
        }
    }

    pub fn load_workspace(&mut self, name: &str) -> Option<String> {
        let list = Self::list_workspaces();
        let target = list.into_iter().find(|w| w.name == name)?;
        if target.tabs.is_empty() {
            return None;
        }
        self.tabs = target.tabs;
        self.active_id = target.active_id;
        let max_id = self.tabs.iter().map(|t| t.id).max().unwrap_or(1);
        self.counter = max_id.max(self.counter);
        self.save();
        Some(self.active_url())
    }

    pub fn delete_workspace(name: &str) -> bool {
        let mut list = Self::list_workspaces();
        let before_len = list.len();
        list.retain(|w| w.name != name);
        if list.len() < before_len {
            if let Ok(json) = serde_json::to_string_pretty(&list) {
                let _ = fs::write(Self::workspaces_path(), json);
                return true;
            }
        }
        false
    }
}
