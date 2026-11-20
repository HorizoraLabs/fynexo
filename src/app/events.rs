#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct HudInsetLogical {
    pub top: f64,
    pub left: f64,
    pub right: f64,
    pub bottom: f64,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct HudRegionRectLogical {
    pub x: f64,
    pub y: f64,
    pub w: f64,
    pub h: f64,
    pub r: f64,
}

#[derive(Debug, Clone)]
pub enum UserEvent {
    SyncHudLayout {
        win_id: usize,
        inset: HudInsetLogical,
        regions: Vec<HudRegionRectLogical>,
        has_overlay: bool,
    },
    ContentAction {
        win_id: usize,
        action: String,
        payload: serde_json::Value,
    },
    HudAction {
        win_id: usize,
        action: String,
    },
    ContentPointerDown {
        win_id: usize,
    },
    ContentPageMeta {
        win_id: usize,
        tab_id: usize,
        meta: serde_json::Value,
    },
    NewWindow { url: Option<String>, incognito: bool },
    Navigate { win_id: usize, url: String },
    SwitchTab { win_id: usize, id: usize, url: String },
    ResetWorkspace { win_id: usize, active_id: usize, active_url: String },
    RevealPendingTab { win_id: usize, id: usize, gen: u64 },
    TabPaintReady { win_id: usize, tab_id: usize },
    ToggleBackgroundPlay { win_id: usize, id: usize, active: bool },
    ToggleTabMute { win_id: usize, id: usize, muted: bool },
    BootSync { win_id: usize, tab_id: usize },
    SyncNavState { win_id: usize, tab_id: usize },
    SyncConfig,
    SyncTabs(usize),
    PageStarted { win_id: usize, tab_id: usize, url: String },
    PageFinished { win_id: usize, tab_id: usize, url: String },
    DownloadUpdate { name: String, status: String },
    Toast(String),
    ToggleFullscreen { win_id: usize, explicit: Option<bool> },
    ApplyZoom(f64),
    ClearSiteData(usize),
    SyncAdblockFilters,
    SyncAutofill(bool),
    RendererCrashed { win_id: usize, tab_id: usize, unresponsive: bool },
    UpdateUserAgent(Option<String>),
    WindowMinimize(usize),
    WindowMaximize(usize),
    WindowClose(usize),
    WindowDrag(usize),
    GoBack(usize),
    GoForward(usize),
    Reload(usize),
    ReloadTab { win_id: usize, id: usize },
    OpenDevtools(usize),
    EvalScript { win_id: usize, script: String },
    RequestResourceUsage(usize),
    TrimWorkingSet(usize),
    SweepIdleTabs,
    InstallChromeExtUnpacked {
        entry: Box<crate::chrome_ext::ChromeExtensionEntry>,
    },
    ChromeExtRuntimeReady {
        entry_id: String,
        runtime_id: String,
        enabled: bool,
        error: Option<String>,
    },
    ChromeExtSynced(Vec<(String, String, bool)>),
    ToggleChromeExt {
        id: String,
        enabled: bool,
    },
    RemoveChromeExt {
        id: String,
    },
    OpenChromeExtPopup {
        win_id: usize,
        id: String,
    },
    OpenChromeExtOptions {
        win_id: usize,
        id: String,
    },
    CloseChromeExtPopup,
    OpenUrlInWindow {
        win_id: usize,
        url: String,
    },
}
