use super::defaults::DEFAULTS_SCRIPT;
use super::icons::ICONS_SCRIPT;
use super::styles::BASE_STYLES_CSS;

pub fn assemble_hud_script(initial_json: &str) -> String {
    let base_css_json = serde_json::to_string(BASE_STYLES_CSS).unwrap_or_else(|_| "\"\"".to_string());
    format!(
        r##"
(function() {{
    // 1. Iframe Isolation: Only the top-level browsing context hosts Fynexo Chrome
    if (window.top !== window.self) return;


{icons_script}

{defaults_script}

    function mergeStyles(raw) {{
        const s = raw || {{}};
        return {{
            colors: Object.assign({{}}, DEFAULT_STYLES.colors, s.colors || {{}}),
            topbar: Object.assign({{}}, DEFAULT_STYLES.topbar, s.topbar || {{}}),
            tabs: Object.assign({{}}, DEFAULT_STYLES.tabs, s.tabs || {{}}),
            address_bar: Object.assign({{}}, DEFAULT_STYLES.address_bar, s.address_bar || {{}}),
            search_modal: Object.assign({{}}, DEFAULT_STYLES.search_modal, s.search_modal || {{}}),
            mobile_bar: Object.assign({{}}, DEFAULT_STYLES.mobile_bar, s.mobile_bar || {{}}),
            tab_drawer: Object.assign({{}}, DEFAULT_STYLES.tab_drawer, s.tab_drawer || {{}}),
            settings_page: Object.assign({{}}, DEFAULT_STYLES.settings_page, s.settings_page || {{}}),
            newtab: Object.assign({{}}, DEFAULT_STYLES.newtab, s.newtab || {{}})
        }};
    }}

    // Frame-0 Instant Cache Hydration
    function readSessionCache(key) {{
        try {{
            const v = window.sessionStorage && window.sessionStorage.getItem(key);
            return v ? JSON.parse(v) : null;
        }} catch (_) {{
            return null;
        }}
    }}

    function writeSessionCache(key, obj) {{
        try {{
            if (window.sessionStorage) window.sessionStorage.setItem(key, JSON.stringify(obj));
        }} catch (_) {{}}
    }}

    const cachedRt = readSessionCache('__fnx_rt__');
    const cachedTabs = readSessionCache('__fnx_tabs__');
    const bootPayload = cachedRt || {initial_json} || {{}};

    let config = Object.assign({{
        theme: 'dark',
        enforce_perf: true,
        adblock: true,
        restore_session: true,
        force_dark: false,
        incognito: false,
        default_zoom: 1.0,
        user_agent_mode: 'default',
        custom_user_agent: '',
        search_engine: 'google',
        homepage: 'fynexo://newtab',
        search_engines: DEFAULT_ENGINES,
        bangs: DEFAULT_BANGS
    }}, bootPayload.config || {{}});
    let styles = mergeStyles(bootPayload.styles);
    let activeTheme = Object.assign({{}}, DEFAULT_STYLES.colors, bootPayload.active_theme || {{}});
    let keybinds = Object.assign({{}}, DEFAULT_KEYBINDS, bootPayload.keybinds || {{}});
    let enabledRules = Array.isArray(bootPayload.enabled_rules)
        ? bootPayload.enabled_rules
        : (Array.isArray(bootPayload.site_rules) ? bootPayload.site_rules.filter(r => r && r.enabled) : []);
    let bookmarks = Array.isArray(bootPayload.bookmarks) ? bootPayload.bookmarks : [];
    let historyItems = Array.isArray(bootPayload.history) ? bootPayload.history : [];

    function getEngines() {{
        return (Array.isArray(config.search_engines) && config.search_engines.length > 0)
            ? config.search_engines
            : DEFAULT_ENGINES;
    }}

    function getBangs() {{
        return Array.isArray(config.bangs) ? config.bangs : DEFAULT_BANGS;
    }}

    function currentEngineObj(engineId) {{
        const list = getEngines();
        const targetId = engineId || config.search_engine;
        for (let i = 0; i < list.length; i++) {{
            if (list[i] && list[i].id === targetId) return list[i];
        }}
        return list[0] || DEFAULT_ENGINES[0];
    }}

    function cacheSlimRuntime() {{
        writeSessionCache('__fnx_rt__', {{
            config: config,
            styles: styles,
            active_theme: activeTheme,
            keybinds: keybinds,
            enabled_rules: enabledRules,
            bookmarks: bookmarks,
            history: historyItems.slice(0, 25)
        }});
    }}

    let tabState = cachedTabs || (bootPayload.tabs && Array.isArray(bootPayload.tabs.tabs) ? bootPayload.tabs : null) || {{
        tabs: [],
        active_id: 1,
        closed_count: 0
    }};

    let shadowRoot = null;
    let pageRulesEl = null;
    let siteRulesEl = null;
    let isPageKeyboardOpen = false;
    let isDrawerOpen = false;
    let isFullscreen = false;
    let isTopbarPeek = false;
    let isFindOpen = false;
    let isReaderOpen = false;
    let isInlineActive = false;
    let isModalActive = false;
    let closeOmniboxRef = null;
    let readerFontPx = 18;
    let readerWide = false;
    let readerSavedScroll = 0;
    let findRanges = [];
    let findIdx = -1;
    let appliedRuleJsIds = Object.create(null);
    let lastDeskTabsSig = '';
    let lastDrawerTabsSig = '';
    let suggestItems = [];
    let suggestIndex = -1;
    let hudToastTimer = null;

    function sendIpc(data) {{
        const payload = JSON.stringify(data);
        if (window.ipc && typeof window.ipc.postMessage === 'function') {{
            window.ipc.postMessage(payload);
        }} else if (window.__TAURI_INTERNALS__ && window.__TAURI_INTERNALS__.ipc) {{
            window.__TAURI_INTERNALS__.ipc.postMessage(payload);
        }}
    }}

    function normalizeUrlKey(u) {{
        if (!u) return '';
        const t = String(u).trim();
        return (t.length > 8 && t.endsWith('/') && (t.match(/\//g) || []).length === 3) ? t.slice(0, -1).toLowerCase() : t.toLowerCase();
    }}

    function isUrlBookmarked(u) {{
        if (!u || isInternalAny(u)) return false;
        const key = normalizeUrlKey(u);
        for (let i = 0; i < bookmarks.length; i++) {{
            if (normalizeUrlKey(bookmarks[i].url) === key) return true;
        }}
        return false;
    }}

    function isInternalSettings(rawUrl) {{
        const u = rawUrl || window.location.href || '';
        const attr = document.documentElement && document.documentElement.getAttribute('data-fynexo-internal');
        if (!rawUrl && (attr === 'settings' || attr === 'true' || !!document.getElementById('__fynexo_settings_app__'))) return true;
        return u.startsWith('fynexo://settings') || u.includes('fynexo.localhost/settings') || u.includes('fynexo.settings');
    }}

    function isInternalNewtab(rawUrl) {{
        const u = rawUrl || window.location.href || '';
        const attr = document.documentElement && document.documentElement.getAttribute('data-fynexo-internal');
        if (!rawUrl && (attr === 'newtab' || !!document.getElementById('__fynexo_newtab_app__'))) return true;
        return u.startsWith('fynexo://newtab') || u.includes('fynexo.localhost/newtab') || u.includes('fynexo.newtab');
    }}

    function isInternalAny(rawUrl) {{
        return isInternalSettings(rawUrl) || isInternalNewtab(rawUrl);
    }}

    function canonicalCurrentUrl() {{
        if (isInternalSettings()) return 'fynexo://settings';
        if (isInternalNewtab()) return 'fynexo://newtab';
        return window.location.href || '';
    }}

    function formatHostLabel(rawUrl) {{
        if (!rawUrl && isInternalNewtab()) return 'Search or enter URL';
        if (!rawUrl && isInternalSettings()) return 'Preferences';
        if (isInternalNewtab(rawUrl)) return 'Search or enter URL';
        if (isInternalSettings(rawUrl)) return 'Preferences';
        if (!rawUrl || rawUrl === 'about:blank') return 'Search or enter URL';
        try {{
            const parsed = new URL(rawUrl);
            return parsed.hostname.replace(/^www\./, '') || rawUrl;
        }} catch (_) {{
            return rawUrl;
        }}
    }}

    function extractMonogram(rawUrl, title) {{
        if (isInternalNewtab(rawUrl)) return 'âœ¦';
        if (!rawUrl || isInternalSettings(rawUrl)) return 'âš™';
        try {{
            const host = new URL(rawUrl).hostname.replace(/^(www|m)\./, '');
            if (host) return host.charAt(0).toUpperCase();
        }} catch (_) {{}}
        const clean = (title || '').trim();
        return clean ? clean.charAt(0).toUpperCase() : 'â€¢';
    }}

    function monogramAccentColor(rawUrl) {{
        if (isInternalNewtab(rawUrl)) return 'var(--fnx-accent)';
        if (!rawUrl || isInternalSettings(rawUrl)) return 'var(--fnx-secondary)';
        let host = '';
        try {{
            host = new URL(rawUrl).hostname.replace(/^(www|m)\./, '').toLowerCase();
        }} catch (_) {{
            host = String(rawUrl).toLowerCase();
        }}
        if (host.includes('youtube') || host.includes('gmail') || host.includes('mail.google') || host.includes('netflix')) return '#f87171';
        if (host.includes('bilibili')) return '#fb7185';
        if (host.includes('github') || host.includes('discord') || host.includes('gemini')) return '#818cf8';
        if (host.includes('translate') || host.includes('wikipedia') || host.includes('bing') || host.includes('zhihu')) return '#38bdf8';
        if (host.includes('maps') || host.includes('spotify') || host.includes('duckduckgo') || host.includes(' cooling')) return '#34d399';
        if (host.includes('reddit') || host.includes('stackoverflow') || host.includes('amazon') || host.includes('taobao')) return '#fb923c';
        const palette = ['#7a88f7', '#38bdf8', '#34d399', '#f472b6', '#fb923c', '#a78bfa'];
        let h = 0;
        for (let i = 0; i < host.length; i++) {{
            h = ((h << 5) - h) + host.charCodeAt(i);
            h |= 0;
        }}
        return palette[Math.abs(h) % palette.length];
    }}

    function resolvePalette() {{
        const src = (styles.colors && styles.colors.use_custom_colors) ? styles.colors : activeTheme;
        return {{
            bg: src.bg || '#19191c',
            panel: src.panel || '#29292e',
            surface: src.surface || src.panel || '#212126',
            elevated: src.elevated || '#35353c',
            border: src.border || '#37373f',
            borderHover: src.border_hover || src.borderHover || '#4b4b56',
            fg: src.fg || '#f0f0f3',
            secondary: src.secondary || '#c2c2cb',
            muted: src.muted || '#8a8a95',
            accent: src.accent || '#7a88f7',
            accentBg: src.accent_bg || src.accentBg || '#262942',
            hover: src.hover || '#33333a',
            tabActive: src.tab_active || src.tabActive || '#29292e',
            tabInactive: src.tab_inactive || src.tabInactive || 'transparent'
        }};
    }}

    function isBookmarksBarShown() {{
        const mode = ((styles.topbar && styles.topbar.bookmarks_bar) || 'always').toLowerCase();
        if (mode === 'hidden') return false;
        if (mode === 'newtab') return isInternalNewtab();
        return true;
    }}

    function isMobilePlatform() {{
        return /Android|iPhone|iPad|iPod/i.test(navigator.userAgent);
    }}

    function computeDesktopTopHeight() {{
        const tb = styles.topbar || {{}};
        const isDeck = (tb.layout || 'deck').toLowerCase() !== 'compact';
        const navH = tb.height || '38px';
        const titlebarH = tb.titlebar_height || '40px';
        const bmShown = isBookmarksBarShown();
        if (isDeck) {{
            return bmShown ? `calc(${{titlebarH}} + ${{navH}} + 28px)` : `calc(${{titlebarH}} + ${{navH}})`;
        }}
        return bmShown ? `calc(${{navH}} + 28px)` : navH;
    }}

    function isPageAlreadyDark() {{
        try {{
            const meta = document.querySelector('meta[name="color-scheme"]');
            if (meta) {{
                const c = (meta.getAttribute('content') || '').toLowerCase();
                if (c.includes('dark') && !c.includes('light')) return true;
            }}

            const pr = document.getElementById('__fynexo_page_rules__');
            const wasDisabled = pr ? pr.disabled : false;
            if (pr) pr.disabled = true;

            const checkEl = (el) => {{
                if (!el || !window.getComputedStyle) return null;
                const bg = window.getComputedStyle(el).backgroundColor || '';
                const m = bg.match(/rgba?\(\s*(\d+)\s*,\s*(\d+)\s*,\s*(\d+)(?:\s*,\s*([\d.]+))?\s*\)/i);
                if (!m) return null;
                const a = m[4] !== undefined ? parseFloat(m[4]) : 1;
                if (a < 0.12) return null;
                return 0.299 * Number(m[1]) + 0.587 * Number(m[2]) + 0.114 * Number(m[3]);
            }};

            let dark = false;
            const lumBody = checkEl(document.body);
            if (lumBody !== null) {{
                dark = lumBody < 112;
            }} else {{
                const lumHtml = checkEl(document.documentElement);
                if (lumHtml !== null) {{
                    dark = lumHtml < 112;
                }}
            }}

            if (pr) pr.disabled = wasDisabled;
            return dark;
        }} catch (_) {{}}
        return false;
    }}

    function domainMatchesRule(hostname, pattern) {{
        const p = String(pattern || '*').trim().toLowerCase();
        if (!p || p === '*') return true;
        const clean = p.startsWith('*.') ? p.slice(2) : p.replace(/^www\./, '');
        const h = String(hostname || '').toLowerCase();
        return h === clean || h.endsWith('.' + clean);
    }}

    function applySiteRules() {{
        if (isInternalAny()) {{
            if (siteRulesEl && siteRulesEl.isConnected) siteRulesEl.remove();
            return;
        }}
        const host = window.location.hostname || '';
        let combinedCss = '';
        for (let i = 0; i < enabledRules.length; i++) {{
            const r = enabledRules[i];
            if (!r || !r.enabled) continue;
            if (!domainMatchesRule(host, r.domain)) continue;
            if (r.css && r.css.trim()) {{
                combinedCss += '\n/* Rule: ' + (r.name || r.id) + ' */\n' + r.css;
            }}
            if (r.js && r.js.trim() && !appliedRuleJsIds[r.id]) {{
                appliedRuleJsIds[r.id] = true;
                const code = r.js;
                const runRuleJs = () => {{
                    try {{
                        (new Function(code))();
                    }} catch (err) {{
                        console.warn('[Fynexo Site Rule JS Error]:', r.name || r.id, err);
                    }}
                }};
                if (document.readyState === 'loading') {{
                    document.addEventListener('DOMContentLoaded', runRuleJs, {{ once: true }});
                }} else {{
                    runRuleJs();
                }}
            }}
        }}

        if (!combinedCss) {{
            if (siteRulesEl && siteRulesEl.isConnected) siteRulesEl.remove();
            siteRulesEl = null;
            return;
        }}

        if (!siteRulesEl || !siteRulesEl.isConnected) {{
            siteRulesEl = document.getElementById('__fynexo_site_rules__');
            if (!siteRulesEl) {{
                siteRulesEl = document.createElement('style');
                siteRulesEl.id = '__fynexo_site_rules__';
                (document.documentElement || document.head || document.body).appendChild(siteRulesEl);
            }}
        }}
        if (siteRulesEl.textContent !== combinedCss) {{
            siteRulesEl.textContent = combinedCss;
        }}
    }}

    function applyPageRules() {{
        if (!pageRulesEl || !pageRulesEl.isConnected) {{
            pageRulesEl = document.getElementById('__fynexo_page_rules__');
            if (!pageRulesEl) {{
                pageRulesEl = document.createElement('style');
                pageRulesEl.id = '__fynexo_page_rules__';
                (document.documentElement || document.head || document.body).appendChild(pageRulesEl);
            }}
        }}

        const topH = computeDesktopTopHeight();
        const mobH = styles.mobile_bar.height || '54px';
        const deskTopVisible = (styles.topbar.visible_on_desktop !== false) && !isFullscreen;
        const mobTopVisible = (styles.topbar.visible_on_mobile === true) && !isFullscreen;
        const mobBarVisible = (styles.mobile_bar.visible !== false) && !isFullscreen;
        const mobPos = (styles.mobile_bar.position || 'bottom').toLowerCase();

        const highlightCss = `
            ::highlight(fnx-find) {{
                background-color: rgba(250, 204, 21, 0.38);
                color: inherit;
            }}
            ::highlight(fnx-find-active) {{
                background-color: #f59e0b;
                color: #09090b;
            }}
        `;

        const perfCss = '';

        const isDarkAlready = isPageAlreadyDark();
        const forceDarkCss = (config.force_dark && !isInternalAny() && !isDarkAlready) ? `
            html:not([data-fynexo-internal]) {{
                background-color: #111113 !important;
            }}
            html:not([data-fynexo-internal]) > body {{
                filter: invert(0.92) hue-rotate(180deg) !important;
                background-color: #ffffff;
            }}
            html:not([data-fynexo-internal]) > body img,
            html:not([data-fynexo-internal]) > body video,
            html:not([data-fynexo-internal]) > body picture,
            html:not([data-fynexo-internal]) > body canvas,
            html:not([data-fynexo-internal]) > body iframe,
            html:not([data-fynexo-internal]) > body svg:not(:root),
            html:not([data-fynexo-internal]) > body [style*="background-image"] {{
                filter: invert(1) hue-rotate(180deg) !important;
            }}
        ` : '';

        const adblockCss = (config.adblock !== false && !isInternalAny()) ? `
            ins.adsbygoogle,
            [id^="div-gpt-ad"],
            [id^="google_ads_iframe"],
            iframe[src*="doubleclick.net"],
            iframe[src*="googlesyndication.com"],
            .taboola-placeholder,
            .OUTBRAIN,
            ytd-ad-slot-renderer,
            ytd-in-feed-ad-layout-renderer,
            ytd-banner-promo-renderer,
            ytd-statement-banner-renderer,
            #player-ads,
            .ytp-ad-overlay-container,
            .ytp-ad-message-container,
            ytd-promoted-video-renderer,
            ytd-compact-promoted-video-renderer {{
                display: none !important;
                height: 0 !important;
                min-height: 0 !important;
                pointer-events: none !important;
            }}
        ` : '';

        const isMob = isMobilePlatform();
        const deskOffsetCss = (deskTopVisible && !isMob) ? `
            :root {{
                --fnx-host-top-offset: ${{topH}};
            }}
            ::-webkit-scrollbar {{
                width: 10px !important;
                height: 10px !important;
            }}
            ::-webkit-scrollbar-button {{
                display: none !important;
                width: 0 !important;
                height: 0 !important;
            }}
            ::-webkit-scrollbar-track {{
                background: transparent !important;
            }}
            ::-webkit-scrollbar-thumb {{
                background: rgba(128, 128, 128, 0.38) !important;
                border-radius: 5px !important;
                border: 2px solid transparent !important;
                background-clip: content-box !important;
            }}
            ::-webkit-scrollbar-thumb:hover {{
                background: rgba(128, 128, 128, 0.65) !important;
                border: 2px solid transparent !important;
                background-clip: content-box !important;
            }}
            ::-webkit-scrollbar-corner {{
                background: transparent !important;
            }}
            html:not([data-fynexo-internal]) {{
                overflow: hidden !important;
                height: 100vh !important;
                max-height: 100vh !important;
            }}
            html:not([data-fynexo-internal]) > body {{
                position: fixed !important;
                top: ${{topH}} !important;
                left: 0 !important;
                right: 0 !important;
                bottom: 0 !important;
                overflow-y: auto !important;
                overflow-x: auto !important;
            }}

            /* Game & Interactive Canvas Viewport Isolation (Zero-Scroll / Lock Viewport) */
            html.fnx-game-mode,
            html.fnx-game-mode > body,
            html > body.fnx-game-mode,
            html:not([data-fynexo-internal]) > body.fnx-game-mode,
            html.fnx-game-mode:not([data-fynexo-internal]) > body {{
                overflow: hidden !important;
                overflow-x: hidden !important;
                overflow-y: hidden !important;
            }}
            html[data-fynexo-internal] {{
                overflow: hidden !important;
                height: 100vh !important;
                max-height: 100vh !important;
                --fnx-host-top-offset: 0px !important;
            }}
            html[data-fynexo-internal] > body {{
                position: fixed !important;
                top: ${{topH}} !important;
                left: 0 !important;
                right: 0 !important;
                bottom: 0 !important;
                width: 100% !important;
                max-width: 100% !important;
                height: calc(100vh - ${{topH}}) !important;
                max-height: calc(100vh - ${{topH}}) !important;
                overflow-y: auto !important;
                overflow-x: hidden !important;
                box-sizing: border-box !important;
            }}
            /* YouTube Desktop Layout Harmonization & Fixes */
            ytd-app {{
                --ytd-masthead-height: 56px !important;
            }}
            #masthead-container {{
                top: var(--fnx-host-top-offset, ${{topH}}) !important;
                background-color: var(--yt-spec-base-background, #0f0f0f);
            }}
            ytd-mini-guide-renderer,
            ytd-app[guide-persistent-and-visible] #guide,
            ytd-app[guide-persistent-and-visible] #guide-wrapper {{
                top: calc(var(--fnx-host-top-offset, ${{topH}}) + 56px) !important;
                max-height: calc(100vh - var(--fnx-host-top-offset, ${{topH}}) - 56px) !important;
            }}
            /* Drawer mode: starts at topH and overlays masthead with 0 duplicate headers */
            tp-yt-app-drawer#guide,
            ytd-app:not([guide-persistent-and-visible]) #guide,
            tp-yt-app-drawer {{
                top: var(--fnx-host-top-offset, ${{topH}}) !important;
                height: calc(100vh - var(--fnx-host-top-offset, ${{topH}})) !important;
                max-height: calc(100vh - var(--fnx-host-top-offset, ${{topH}})) !important;
                z-index: 2200 !important;
            }}
            ytd-watch-grid #fixed-secondary,
            ytd-watch-grid #panels.ytd-watch-grid,
            ytd-watch-grid ytd-watch-engagement-panels,
            ytd-watch-flexy #panels.ytd-watch-flexy,
            ytd-watch-flexy #playlist.ytd-watch-flexy {{
                top: calc(var(--fnx-host-top-offset, ${{topH}}) + 56px) !important;
                max-height: calc(100vh - var(--fnx-host-top-offset, ${{topH}}) - 56px) !important;
            }}
            /* GitHub Primer Global Navigation Drawer & SidePanels Harmonization */
            header.Header,
            header.AppHeader,
            .AppHeader-globalBar,
            [aria-label="Global"],
            dialog#dialog-global-nav,
            dialog.SidePanel,
            dialog.Overlay--drawer,
            dialog.Overlay--placement-left,
            dialog.Overlay--placement-right,
            .Overlay.SidePanel,
            .Overlay--drawer,
            .Overlay--placement-left,
            .Overlay--placement-right,
            [data-component="SidePanel"],
            aside.SidePanel {{
                top: var(--fnx-host-top-offset, ${{topH}}) !important;
                height: calc(100vh - var(--fnx-host-top-offset, ${{topH}})) !important;
                max-height: calc(100vh - var(--fnx-host-top-offset, ${{topH}})) !important;
            }}
            dialog#dialog-global-nav::backdrop,
            dialog.SidePanel::backdrop,
            dialog.Overlay--placement-left::backdrop,
            dialog.Overlay--placement-right::backdrop,
            .Overlay-backdrop,
            .Overlay-backdrop--side {{
                top: var(--fnx-host-top-offset, ${{topH}}) !important;
                height: calc(100vh - var(--fnx-host-top-offset, ${{topH}})) !important;
            }}
        ` : `
            :root {{ --fnx-host-top-offset: 0px; }}
        `;

        let mobPaddingRules = '';
        if (mobTopVisible) {{
            mobPaddingRules += `padding-top: ${{topH}} !important;`;
        }}
        if (mobBarVisible) {{
            if (mobPos === 'top') {{
                mobPaddingRules += `padding-top: calc(${{mobH}} + env(safe-area-inset-top, 0px)) !important;`;
            }} else {{
                mobPaddingRules += `padding-bottom: calc(${{mobH}} + env(safe-area-inset-bottom, 0px)) !important;`;
            }}
        }}

        const mobOffsetCss = isMob ? `
            :root {{
                --fnx-host-top-offset: ${{mobTopVisible ? topH : (mobBarVisible && mobPos === 'top' ? mobH : '0px')}};
                --fnx-host-bottom-offset: ${{mobBarVisible && mobPos !== 'top' ? mobH : '0px'}};
            }}
            html:not([data-fynexo-internal]) body:not(.fnx-kb-active) {{
                ${{mobPaddingRules}}
            }}
            html:not([data-fynexo-internal]) body.fnx-kb-active {{
                padding-bottom: 0px !important;
            }}
        ` : '';

        const nextCss = highlightCss + perfCss + forceDarkCss + adblockCss + deskOffsetCss + mobOffsetCss;
        if (pageRulesEl.textContent !== nextCss) {{
            pageRulesEl.textContent = nextCss;
        }}
    }}

    function applyShadowStyles() {{
        if (!shadowRoot) return;
        const p = resolvePalette();

        let dynStyle = shadowRoot.getElementById('__fynexo_dynamic_styles__');
        if (!dynStyle) {{
            dynStyle = document.createElement('style');
            dynStyle.id = '__fynexo_dynamic_styles__';
            shadowRoot.appendChild(dynStyle);
        }}

        const tb = styles.topbar;
        const ts = styles.tabs;
        const ab = styles.address_bar;
        const sm = styles.search_modal;
        const mb = styles.mobile_bar;
        const td = styles.tab_drawer;

        const isDeck = (tb.layout || 'deck').toLowerCase() !== 'compact';
        const bmShown = isBookmarksBarShown();
        const mobIsTop = (mb.position || 'bottom').toLowerCase() === 'top';
        const showDeskTop = (tb.visible_on_desktop !== false) && (!isFullscreen || isTopbarPeek);
        const liveTopOffset = showDeskTop ? computeDesktopTopHeight() : '0px';
        const addrWidth = ab.width || '340px';
        const tabRadius = ts.border_radius || '6px';

        dynStyle.textContent = `
            :host, #__fynexo_shell__ {{
                --fnx-bg: ${{p.bg}};
                --fnx-panel: ${{p.panel}};
                --fnx-surface: ${{p.surface}};
                --fnx-elevated: ${{p.elevated}};
                --fnx-border: ${{p.border}};
                --fnx-border-hover: ${{p.borderHover}};
                --fnx-fg: ${{p.fg}};
                --fnx-secondary: ${{p.secondary}};
                --fnx-muted: ${{p.muted}};
                --fnx-accent: ${{p.accent}};
                --fnx-accent-bg: ${{p.accentBg}};
                --fnx-hover: ${{p.hover}};
                --fnx-tab-active: ${{p.tabActive}};
                --fnx-tab-inactive: ${{p.tabInactive}};
                --fnx-topbar-live: ${{liveTopOffset}};
            }}

            #__fynexo_topbar__ {{
                background: ${{tb.background || 'var(--fnx-panel)'}};
                border-bottom: ${{tb.border_bottom || '1px solid var(--fnx-border)'}};
                ${{tb.custom_css || ''}}
            }}

            #__fynexo_tab_row__ {{
                display: ${{isDeck ? 'flex' : 'none'}};
                height: ${{tb.titlebar_height || '40px'}};
                padding: 0;
                background: var(--fnx-bg);
                border-bottom: 1px solid var(--fnx-border);
            }}

            #__fynexo_win_controls__ {{
                height: ${{isDeck ? (tb.titlebar_height || '40px') : '100%'}};
            }}

            #__fynexo_nav_row__ {{
                height: ${{tb.height || '38px'}};
                padding: 0 ${{isDeck ? (tb.padding_x || '6px') : '0'}} 0 5px;
                background: ${{tb.background || 'var(--fnx-panel)'}};
            }}

            #__fynexo_bm_row__ {{
                display: ${{bmShown ? 'flex' : 'none'}};
                height: 28px;
                padding: 0 ${{tb.padding_x || '6px'}} 0 4px;
                background: ${{tb.background || 'var(--fnx-panel)'}};
                border-top: 1px solid rgba(128, 128, 128, 0.14);
            }}

            .fnx-icon-btn {{
                width: ${{tb.button_size || '28px'}};
                height: ${{tb.button_size || '28px'}};
                border-radius: ${{tb.button_radius || '6px'}};
            }}

            #__fynexo_tabs_container__ {{
                gap: ${{typeof ts.gap === 'string' && ts.gap !== '' ? ts.gap : '0px'}};
                align-items: ${{isDeck ? 'flex-end' : 'center'}};
                height: 100%;
            }}

            .fnx-tab-item {{
                height: ${{ts.height || (isDeck ? '36px' : '28px')}};
                width: ${{ts.width || '240px'}};
                flex: 0 1 ${{ts.width || '240px'}};
                max-width: ${{ts.max_width || ts.width || '240px'}};
                min-width: ${{ts.min_width || '120px'}};
                border-radius: ${{isDeck ? `${{tabRadius}} ${{tabRadius}} 0 0` : tabRadius}};
                font-size: ${{ts.font_size || '14px'}};
                margin: ${{isDeck ? '0 0 -1px 0' : '0'}};
                box-sizing: border-box;
                ${{ts.custom_css || ''}}
            }}

            .fnx-tab-item:not(.active) {{
                height: ${{ts.height || (isDeck ? '36px' : '28px')}};
                background: var(--fnx-tab-inactive);
                border: 1px solid transparent;
                ${{isDeck ? 'border-bottom: 1px solid var(--fnx-border);' : ''}}
                color: var(--fnx-secondary);
            }}

            .fnx-tab-item.active {{
                height: ${{ts.height || (isDeck ? '36px' : '28px')}};
                border: 1px solid var(--fnx-border);
                ${{isDeck ? 'border-bottom: 1px solid var(--fnx-tab-active);' : ''}}
                background: var(--fnx-tab-active);
                color: var(--fnx-fg);
                z-index: 2;
            }}

            .fnx-tab-item.pinned {{
                width: 36px !important;
                min-width: 36px !important;
                max-width: 36px !important;
                flex: 0 0 36px !important;
                padding: 0 !important;
                justify-content: center !important;
            }}

            .fnx-tab-monogram {{
                display: ${{ts.show_indicator_dot !== false ? 'inline-flex' : 'none'}};
                width: ${{ts.icon_size || '20px'}};
                height: ${{ts.icon_size || '20px'}};
                font-size: ${{ts.icon_font_size || '11.5px'}};
            }}
            .fnx-tab-item.pinned .fnx-tab-monogram {{
                display: inline-flex !important;
            }}

            .fnx-tab-item:not(.active) .fnx-tab-close {{
                opacity: ${{ts.close_on_hover_only ? '0' : '0.65'}};
                pointer-events: ${{ts.close_on_hover_only ? 'none' : 'auto'}};
            }}
            .fnx-tab-item:not(.active):hover .fnx-tab-close {{
                opacity: 0.95;
                pointer-events: auto;
            }}

            #__fynexo_addr_slot__ {{
                height: ${{ab.height || '30px'}};
                width: ${{isDeck ? 'auto' : addrWidth}};
                flex: ${{isDeck ? '1 1 auto' : '0 0 auto'}};
                max-width: ${{isDeck ? 'none' : '38vw'}};
                border-radius: ${{ab.border_radius || '9px'}};
                background: ${{ab.background || 'var(--fnx-bg)'}};
                border: ${{ab.border || '1px solid var(--fnx-border)'}};
                font-size: ${{ab.font_size || '12.5px'}};
                ${{ab.custom_css || ''}}
            }}
            #__fynexo_addr_slot__.editing {{
                width: ${{isDeck ? 'auto' : `max(${{addrWidth}}, 380px)`}};
                max-width: ${{isDeck ? 'none' : '46vw'}};
                border-color: var(--fnx-accent);
                background: var(--fnx-bg);
            }}

            #__fynexo_address_btn__,
            #__fynexo_inline_box__ {{
                height: 100%;
                font-size: ${{ab.font_size || '12.5px'}};
            }}

            #__fynexo_hud_root__ {{
                background: ${{sm.backdrop_color || 'rgba(0, 0, 0, 0.56)'}};
            }}

            #__fynexo_hud_card__ {{
                width: ${{sm.width || '580px'}};
                margin-top: ${{sm.top_offset || '66px'}};
                border-radius: ${{sm.border_radius || '12px'}};
                background: ${{sm.background || 'var(--fnx-panel)'}};
                border: ${{sm.border || '1px solid var(--fnx-border-hover)'}};
                ${{sm.custom_css || ''}}
            }}

            #__fynexo_input__ {{
                font-size: ${{sm.input_font_size || '14.5px'}};
            }}

            #__fynexo_bottom_bar__ {{
                height: calc(${{mb.height || '54px'}} + ${{mobIsTop ? 'env(safe-area-inset-top, 0px)' : 'env(safe-area-inset-bottom, 0px)'}});
                padding-left: max(env(safe-area-inset-left, 0px), ${{mb.padding_x || '8px'}});
                padding-right: max(env(safe-area-inset-right, 0px), ${{mb.padding_x || '8px'}});
                padding-bottom: ${{mobIsTop ? '0px' : 'env(safe-area-inset-bottom, 0px)'}};
                padding-top: ${{mobIsTop ? 'env(safe-area-inset-top, 0px)' : '0px'}};
                top: ${{mobIsTop ? '0' : 'auto'}};
                bottom: ${{mobIsTop ? 'auto' : '0'}};
                background: ${{mb.background || 'var(--fnx-panel)'}};
                border-top: ${{mobIsTop ? 'none' : (mb.border || '1px solid var(--fnx-border)')}};
                border-bottom: ${{mobIsTop ? (mb.border || '1px solid var(--fnx-border)') : 'none'}};
                ${{mb.custom_css || ''}}
            }}

            .fnx-mb-btn {{
                width: ${{mb.button_size || '38px'}};
                height: ${{mb.button_size || '38px'}};
            }}

            #__fynexo_mb_pill__ {{
                height: ${{mb.pill_height || '38px'}};
                border-radius: ${{mb.pill_radius || '11px'}};
            }}

            #__fynexo_drawer_sheet__ {{
                max-height: ${{td.max_height || '80vh'}};
                border-radius: ${{td.border_radius || '16px'}} ${{td.border_radius || '16px'}} 0 0;
            }}

            .fnx-drawer-card {{
                border-radius: ${{td.card_radius || '10px'}};
                padding: ${{td.card_padding || '11px 13px'}};
                ${{td.custom_css || ''}}
            }}

            ${{!isMobilePlatform() ? `
                #__fynexo_topbar__ {{
                    display: ${{showDeskTop ? 'flex' : 'none'}} !important;
                }}
                #__fynexo_bottom_bar__ {{
                    display: none !important;
                }}
            ` : `
                #__fynexo_topbar__ {{
                    display: ${{(tb.visible_on_mobile === true && !isFullscreen) ? 'flex' : 'none'}} !important;
                }}
                #__fynexo_bottom_bar__ {{
                    display: ${{(mb.visible !== false && !isPageKeyboardOpen && !isFullscreen) ? 'flex' : 'none'}} !important;
                }}
                #__fynexo_hud_card__ {{
                    width: calc(100vw - 20px) !important;
                    max-width: calc(100vw - 20px) !important;
                    margin-top: 10px !important;
                }}
            `}}
        `;

        // Sync layout between 2-Row Layered Deck ("deck") and Single-Row ("compact")
        const tabRow = shadowRoot.getElementById('__fynexo_tab_row__');
        const navRow = shadowRoot.getElementById('__fynexo_nav_row__');
        const tabsDeckCluster = shadowRoot.getElementById('__fynexo_tabs_deck_cluster__');
        const winControls = shadowRoot.getElementById('__fynexo_win_controls__');
        const addrSlot = shadowRoot.getElementById('__fynexo_addr_slot__');
        const leftZone = shadowRoot.getElementById('__fynexo_left_zone__');
        const centerZone = shadowRoot.getElementById('__fynexo_center_zone__');
        const rightZone = shadowRoot.getElementById('__fynexo_right_zone__');

        if (tabRow && navRow && tabsDeckCluster && winControls && addrSlot && leftZone && centerZone && rightZone) {{
            if (isDeck) {{
                if (tabsDeckCluster.parentNode !== tabRow) {{
                    tabRow.insertBefore(tabsDeckCluster, winControls);
                }}
                if (winControls.parentNode !== tabRow) {{
                    tabRow.appendChild(winControls);
                }}
                if (addrSlot.parentNode !== centerZone) {{
                    centerZone.appendChild(addrSlot);
                }}
            }} else {{
                if (tabsDeckCluster.parentNode !== centerZone) {{
                    centerZone.insertBefore(tabsDeckCluster, centerZone.firstChild);
                }}
                if (winControls.parentNode !== rightZone) {{
                    rightZone.appendChild(winControls);
                }}
                const pos = (ab.position || 'right').toLowerCase();
                const targetParent = pos === 'left' ? leftZone : (pos === 'center' ? centerZone : rightZone);
                if (addrSlot.parentNode !== targetParent) {{
                    if (pos === 'left') leftZone.appendChild(addrSlot);
                    else targetParent.insertBefore(addrSlot, targetParent.firstChild);
                }}
            }}
        }}

        const bmToggleBtn = shadowRoot.getElementById('__fynexo_btn_bm_toggle__');
        if (bmToggleBtn) {{
            bmToggleBtn.classList.toggle('active-route', bmShown);
        }}
    }}

    function renderBookmarksBar() {{
        if (!shadowRoot) return;
        const bmList = shadowRoot.getElementById('__fynexo_bm_list__');
        if (!bmList) return;
        bmList.textContent = '';

        const frag = document.createDocumentFragment();
        const items = Array.isArray(bookmarks) ? bookmarks : [];

        if (items.length === 0) {{
            const emptyHint = document.createElement('span');
            emptyHint.style.cssText = 'font-size: 11.5px; color: var(--fnx-muted); padding: 0 6px;';
            emptyHint.textContent = 'No shortcuts saved â€” click â˜… in the address bar to pin sites here';
            frag.appendChild(emptyHint);
        }} else {{
            items.forEach((b) => {{
                if (!b || !b.url) return;
                const pill = document.createElement('button');
                pill.type = 'button';
                pill.className = 'fnx-bm-pill';
                const label = b.title || formatHostLabel(b.url);
                pill.title = `${{label}}\n${{b.url}} (Middle-click for new tab Â· Right-click to remove)`;

                const iconBadge = document.createElement('span');
                iconBadge.className = 'fnx-bm-icon';
                iconBadge.textContent = extractMonogram(b.url, label);
                iconBadge.style.color = monogramAccentColor(b.url);

                const textSpan = document.createElement('span');
                textSpan.className = 'fnx-bm-label';
                textSpan.textContent = label;

                pill.appendChild(iconBadge);
                pill.appendChild(textSpan);

                pill.onclick = (e) => {{
                    flushScrollSync();
                    triggerProgress();
                    if (e.ctrlKey || e.metaKey) {{
                        sendIpc({{ type: 'NEW_TAB', url: b.url }});
                    }} else {{
                        sendIpc({{ type: 'NAVIGATE', url: b.url }});
                    }}
                }};
                pill.onauxclick = (e) => {{
                    if (e.button === 1) {{
                        e.preventDefault();
                        triggerProgress();
                        sendIpc({{ type: 'NEW_TAB', url: b.url }});
                    }}
                }};
                pill.oncontextmenu = (e) => {{
                    e.preventDefault();
                    e.stopPropagation();
                    sendIpc({{ type: 'TOGGLE_BOOKMARK', title: label, url: b.url }});
                    showHudToast(`Removed "${{label}}" from shortcuts`);
                }};

                frag.appendChild(pill);
            }});
        }}

        bmList.appendChild(frag);
    }}

    function syncAllStyles() {{
        applyPageRules();
        applySiteRules();
        applyShadowStyles();
        renderTabs(false);
        renderBookmarksBar();
        updateAddressLabels();
    }}

    function showHudToast(text) {{
        if (!shadowRoot || !text) return;
        const toastEl = shadowRoot.getElementById('__fynexo_hud_toast__');
        const msgEl = shadowRoot.getElementById('__fynexo_hud_toast_msg__');
        if (!toastEl || !msgEl) return;
        msgEl.textContent = text;
        toastEl.style.display = 'inline-flex';
        if (hudToastTimer) clearTimeout(hudToastTimer);
        hudToastTimer = setTimeout(() => {{
            toastEl.style.display = 'none';
        }}, 1900);
    }}

    function flushScrollSync() {{
        if (!isInternalAny()) {{
            const y = (document.body ? document.body.scrollTop : 0) || window.scrollY || window.pageYOffset || (document.documentElement ? document.documentElement.scrollTop : 0) || 0;
            if (y > 24) {{
                sendIpc({{ type: 'SAVE_SCROLL', scroll_y: y }});
            }}
        }}
    }}

    function updateProtocolIcon(containerEl, rawUrl) {{
        if (!containerEl) return;
        const isSet = isInternalSettings(rawUrl);
        const isNt = isInternalNewtab(rawUrl);
        const isHttps = rawUrl && rawUrl.startsWith('https://');
        const kind = isSet ? 'settings' : (isNt ? 'search' : (isHttps ? 'lock' : 'search'));
        if (containerEl.dataset.iconKind === kind) return;
        containerEl.dataset.iconKind = kind;
        containerEl.textContent = '';
        containerEl.appendChild(createIcon(kind));
        containerEl.style.color = isSet ? 'var(--fnx-accent)' : 'var(--fnx-muted)';
    }}

    function syncBookmarkButtons() {{
        if (!shadowRoot) return;
        const curUrl = canonicalCurrentUrl();
        const internal = isInternalAny(curUrl) || !curUrl || curUrl === 'about:blank';
        const starred = !internal && isUrlBookmarked(curUrl);

        const deskStar = shadowRoot.getElementById('__fynexo_desk_star__');
        if (deskStar) {{
            deskStar.style.display = internal ? 'none' : 'flex';
            const nextKind = starred ? 'starFilled' : 'star';
            if (deskStar.dataset.starKind !== nextKind) {{
                deskStar.dataset.starKind = nextKind;
                deskStar.textContent = '';
                deskStar.appendChild(createIcon(nextKind));
            }}
            deskStar.classList.toggle('active-route', starred);
            deskStar.title = starred
                ? `Remove shortcut (${{keybinds.bookmark_page || 'Ctrl+D'}})`
                : `Save shortcut (${{keybinds.bookmark_page || 'Ctrl+D'}})`;
        }}

        const deskReader = shadowRoot.getElementById('__fynexo_desk_reader__');
        if (deskReader) {{
            deskReader.style.display = internal ? 'none' : 'flex';
            deskReader.classList.toggle('active-route', isReaderOpen);
            deskReader.title = `Reader mode (${{keybinds.reader_mode || 'Alt+R'}})`;
        }}

        const deskDark = shadowRoot.getElementById('__fynexo_desk_dark__');
        if (deskDark) {{
            deskDark.classList.toggle('active-route', !!config.force_dark);
            deskDark.title = `Smart Force Dark: ${{config.force_dark ? 'On' : 'Off'}} (${{keybinds.force_dark || 'Alt+Shift+D'}})`;
        }}

        const bmAddBtn = shadowRoot.getElementById('__fynexo_bm_add__');
        if (bmAddBtn) {{
            if (internal) {{
                bmAddBtn.textContent = 'Manage';
                bmAddBtn.title = 'Manage shortcuts in Studio Preferences';
            }} else {{
                bmAddBtn.textContent = starred ? 'â˜… Saved' : '+ Save page';
                bmAddBtn.title = starred ? 'Remove current page from shortcuts' : 'Pin current page to shortcuts bar';
            }}
        }}

        const incogBadge = shadowRoot.getElementById('__fynexo_incog_badge__');
        if (incogBadge) {{
            incogBadge.style.display = config.incognito ? 'inline-flex' : 'none';
        }}

        const sheetStar = shadowRoot.getElementById('__fynexo_sheet_star__');
        if (sheetStar) {{
            sheetStar.style.display = internal ? 'none' : 'flex';
            const nextKind = starred ? 'starFilled' : 'star';
            if (sheetStar.dataset.starKind !== nextKind) {{
                sheetStar.dataset.starKind = nextKind;
                sheetStar.textContent = '';
                sheetStar.appendChild(createIcon(nextKind));
            }}
            sheetStar.style.color = starred ? 'var(--fnx-accent)' : 'var(--fnx-secondary)';
            sheetStar.style.borderColor = starred ? 'var(--fnx-accent)' : 'var(--fnx-border)';
        }}
    }}

    function toggleCurrentBookmark() {{
        const curUrl = canonicalCurrentUrl();
        if (!curUrl || isInternalAny(curUrl) || curUrl === 'about:blank') return;
        const curTitle = document.title || formatHostLabel(curUrl);
        sendIpc({{ type: 'TOGGLE_BOOKMARK', title: curTitle, url: curUrl }});
    }}

    function updateAddressLabels() {{
        if (!shadowRoot) return;
        const href = canonicalCurrentUrl();
        const label = formatHostLabel(href);
        const isSet = isInternalSettings(href);
        const isNt = isInternalNewtab(href);
        const engName = currentEngineObj().name || 'Google';

        const schemeEl = shadowRoot.getElementById('__fynexo_addr_scheme__');
        const domEl = shadowRoot.getElementById('__fynexo_addr_domain__');
        const pathEl = shadowRoot.getElementById('__fynexo_addr_path__');
        if (domEl && pathEl) {{
            if (isNt || !href || href === 'about:blank') {{
                if (schemeEl) schemeEl.textContent = '';
                domEl.textContent = '';
                pathEl.textContent = `Search ${{engName}} or type a URL...`;
                pathEl.style.color = 'var(--fnx-muted)';
            }} else if (isSet) {{
                if (schemeEl) schemeEl.textContent = 'fynexo://';
                domEl.textContent = 'settings';
                domEl.style.color = 'var(--fnx-fg)';
                pathEl.textContent = ' Â· Studio Preferences';
                pathEl.style.color = 'var(--fnx-muted)';
            }} else {{
                try {{
                    const u = new URL(href);
                    if (schemeEl) schemeEl.textContent = u.protocol + '//';
                    domEl.textContent = u.host || label;
                    domEl.style.color = 'var(--fnx-fg)';
                    const tail = (u.pathname && u.pathname !== '/' ? u.pathname : '') + (u.search || '') + (u.hash || '');
                    pathEl.textContent = tail;
                    pathEl.style.color = 'var(--fnx-muted)';
                }} catch (_) {{
                    if (schemeEl) schemeEl.textContent = '';
                    domEl.textContent = label;
                    domEl.style.color = 'var(--fnx-fg)';
                    pathEl.textContent = '';
                }}
            }}
        }}

        const mobLabel = shadowRoot.getElementById('__fynexo_mb_label__');
        if (mobLabel) {{
            const mobText = (isNt || !href || href === 'about:blank')
                ? `Search ${{engName}} or type URL`
                : label;
            if (mobLabel.textContent !== mobText) mobLabel.textContent = mobText;
        }}

        const addrHint = shadowRoot.getElementById('__fynexo_addr_hint__');
        if (addrHint) {{
            addrHint.textContent = String(keybinds.omnibox || 'Ctrl+L').replace(/\+/g, ' ');
        }}

        updateProtocolIcon(shadowRoot.getElementById('__fynexo_desk_proto__'), href);
        updateProtocolIcon(shadowRoot.getElementById('__fynexo_mb_proto__'), href);

        const deskPrefs = shadowRoot.getElementById('__fynexo_desk_prefs__');
        if (deskPrefs) deskPrefs.classList.toggle('active-route', isSet);
        const mobPrefs = shadowRoot.getElementById('__fynexo_mb_prefs__');
        if (mobPrefs) mobPrefs.classList.toggle('active-route', isSet);

        syncBookmarkButtons();

        const engineBadge = shadowRoot.getElementById('__fynexo_hud_engine__') || shadowRoot.getElementById('__fynexo_inline_engine__');
        if (engineBadge) {{
            engineBadge.textContent = engName;
        }}
    }}

    // Pure-DOM Find in Page (CSS.highlights + TreeWalker, fallback window.find)
    function clearFindHighlights() {{
        findRanges = [];
        findIdx = -1;
        try {{
            if (typeof CSS !== 'undefined' && CSS.highlights) {{
                CSS.highlights.delete('fnx-find');
                CSS.highlights.delete('fnx-find-active');
            }}
        }} catch (_) {{}}
        if (shadowRoot) {{
            const cnt = shadowRoot.getElementById('__fynexo_find_count__');
            if (cnt) cnt.textContent = '0/0';
        }}
    }}

    function scrollRangeIntoView(range) {{
        if (!range) return;
        try {{
            const rect = range.getBoundingClientRect();
            if (!rect || (rect.width === 0 && rect.height === 0)) return;
            const margin = 90;
            if (rect.top < margin || rect.bottom > window.innerHeight - margin) {{
                const targetY = Math.max(0, (window.scrollY || window.pageYOffset || 0) + rect.top - (window.innerHeight * 0.35));
                window.scrollTo({{ top: targetY, behavior: 'smooth' }});
            }}
        }} catch (_) {{}}
    }}

    function runFindQuery(rawQuery) {{
        if (!shadowRoot) return;
        const cnt = shadowRoot.getElementById('__fynexo_find_count__');
        const q = String(rawQuery || '');
        if (!q) {{
            clearFindHighlights();
            return;
        }}

        if (typeof CSS !== 'undefined' && CSS.highlights && typeof Highlight !== 'undefined' && document.body) {{
            const needle = q.toLowerCase();
            const nLen = needle.length;
            const matches = [];
            const walker = document.createTreeWalker(document.body, NodeFilter.SHOW_TEXT);
            let node;
            while ((node = walker.nextNode())) {{
                const p = node.parentElement;
                if (!p) continue;
                const tag = p.tagName;
                if (tag === 'SCRIPT' || tag === 'STYLE' || tag === 'NOSCRIPT' || tag === 'TEMPLATE') continue;
                if (p.closest && p.closest('#__fynexo_host__')) continue;
                const val = node.nodeValue;
                if (!val || val.length < nLen) continue;
                const lower = val.toLowerCase();
                let pos = 0;
                while ((pos = lower.indexOf(needle, pos)) !== -1) {{
                    const r = document.createRange();
                    r.setStart(node, pos);
                    r.setEnd(node, pos + nLen);
                    matches.push(r);
                    pos += nLen;
                    if (matches.length >= 400) break;
                }}
                if (matches.length >= 400) break;
            }}

            findRanges = matches;
            if (matches.length === 0) {{
                findIdx = -1;
                CSS.highlights.delete('fnx-find');
                CSS.highlights.delete('fnx-find-active');
                if (cnt) cnt.textContent = '0/0';
                return;
            }}

            findIdx = 0;
            CSS.highlights.set('fnx-find', new Highlight(...matches));
            CSS.highlights.set('fnx-find-active', new Highlight(matches[0]));
            if (cnt) cnt.textContent = `1/${{matches.length}}`;
            scrollRangeIntoView(matches[0]);
        }} else if (typeof window.find === 'function') {{
            const ok = window.find(q, false, false, true, false, false, false);
            if (cnt) cnt.textContent = ok ? 'Match' : '0/0';
        }}
    }}

    function stepFindMatch(delta) {{
        if (!shadowRoot) return;
        const inp = shadowRoot.getElementById('__fynexo_find_input__');
        const cnt = shadowRoot.getElementById('__fynexo_find_count__');
        const q = inp ? inp.value : '';
        if (!q) return;

        if (findRanges.length > 0 && typeof CSS !== 'undefined' && CSS.highlights && typeof Highlight !== 'undefined') {{
            findIdx = (findIdx + delta + findRanges.length) % findRanges.length;
            const active = findRanges[findIdx];
            CSS.highlights.set('fnx-find-active', new Highlight(active));
            if (cnt) cnt.textContent = `${{findIdx + 1}}/${{findRanges.length}}`;
            scrollRangeIntoView(active);
        }} else if (typeof window.find === 'function') {{
            window.find(q, false, delta < 0, true, false, false, false);
        }}
    }}

    function openFindBar() {{
        if (!shadowRoot) return;
        const bar = shadowRoot.getElementById('__fynexo_find_bar__');
        const inp = shadowRoot.getElementById('__fynexo_find_input__');
        if (!bar || !inp) return;
        isFindOpen = true;
        bar.style.display = 'flex';
        const sel = window.getSelection ? String(window.getSelection()).trim() : '';
        if (sel && sel.length < 80 && !sel.includes('\n')) {{
            inp.value = sel;
        }}
        requestAnimationFrame(() => {{
            inp.focus();
            inp.select();
            if (inp.value) runFindQuery(inp.value);
        }});
    }}

    function closeFindBar() {{
        if (!shadowRoot) return;
        isFindOpen = false;
        const bar = shadowRoot.getElementById('__fynexo_find_bar__');
        if (bar) bar.style.display = 'none';
        clearFindHighlights();
    }}

    // Pure-DOM Zero-Bloat Reader Mode
    function pickArticleContainer() {{
        const explicit = document.querySelector('article, [role="main"], main, .post-content, .article-content, .entry-content, .markdown-body');
        if (explicit && (explicit.textContent || '').trim().length > 260) {{
            return explicit;
        }}
        let best = document.body;
        let bestScore = 0;
        const candidates = document.querySelectorAll('section, div');
        const maxScan = Math.min(candidates.length, 220);
        for (let i = 0; i < maxScan; i++) {{
            const el = candidates[i];
            const cls = ((el.className || '') + ' ' + (el.id || '')).toLowerCase();
            if (/(nav|footer|header|sidebar|comment|menu|modal|banner|promo|ad-)/.test(cls)) continue;
            const pList = el.querySelectorAll('p');
            if (pList.length < 2) continue;
            let textLen = 0;
            for (let j = 0; j < pList.length; j++) {{
                textLen += (pList[j].textContent || '').trim().length;
            }}
            if (textLen > bestScore) {{
                bestScore = textLen;
                best = el;
            }}
        }}
        return best;
    }}

    function syncReaderTypography() {{
        if (!shadowRoot) return;
        const article = shadowRoot.getElementById('__fynexo_reader_article__');
        const widthBtn = shadowRoot.getElementById('__fynexo_reader_width__');
        if (article) {{
            article.style.fontSize = readerFontPx + 'px';
            article.style.maxWidth = readerWide ? '880px' : '680px';
        }}
        if (widthBtn) {{
            widthBtn.textContent = readerWide ? 'Narrow' : 'Wide';
        }}
    }}

    function closeReaderMode() {{
        if (!shadowRoot || !isReaderOpen) return;
        isReaderOpen = false;
        const overlay = shadowRoot.getElementById('__fynexo_reader__');
        const article = shadowRoot.getElementById('__fynexo_reader_article__');
        if (overlay) overlay.style.display = 'none';
        if (article) article.textContent = '';
        if (document.documentElement) document.documentElement.style.overflow = '';
        try {{ window.scrollTo(0, readerSavedScroll); }} catch (_) {{}}
        syncBookmarkButtons();
    }}

    function toggleReaderMode() {{
        if (isInternalAny()) return;
        if (isReaderOpen) {{
            closeReaderMode();
            return;
        }}
        if (!shadowRoot) return;
        const overlay = shadowRoot.getElementById('__fynexo_reader__');
        const article = shadowRoot.getElementById('__fynexo_reader_article__');
        const metaSpan = shadowRoot.getElementById('__fynexo_reader_meta__');
        if (!overlay || !article) return;

        const root = pickArticleContainer();
        if (!root) {{
            showHudToast('No readable article found on this page');
            return;
        }}

        article.textContent = '';
        const frag = document.createDocumentFragment();
        const pageTitle = (document.title || '').trim();
        let hasH1 = false;
        let charCount = 0;
        const seenText = Object.create(null);

        const blocks = root.querySelectorAll('h1, h2, h3, h4, p, pre, blockquote, ul, ol, img');
        const limit = Math.min(blocks.length, 450);

        for (let i = 0; i < limit; i++) {{
            const node = blocks[i];
            if (!node || (node.closest && node.closest('nav, header, footer, aside, form, #__fynexo_host__'))) continue;
            const tag = node.tagName.toLowerCase();
            if ((tag === 'p' || tag === 'blockquote') && node.parentElement && (node.parentElement.tagName === 'LI' || node.parentElement.tagName === 'BLOCKQUOTE')) {{
                continue;
            }}

            if (tag === 'img') {{
                const src = node.currentSrc || node.getAttribute('src') || '';
                const nw = node.naturalWidth || Number(node.getAttribute('width')) || 0;
                if (src && /^https?:\/\//i.test(src) && nw >= 200) {{
                    const img = document.createElement('img');
                    img.src = src;
                    img.className = 'fnx-rd-img';
                    img.loading = 'lazy';
                    frag.appendChild(img);
                }}
                continue;
            }}

            if (tag === 'ul' || tag === 'ol') {{
                const items = node.querySelectorAll(':scope > li');
                if (items.length === 0) continue;
                const listEl = document.createElement(tag);
                listEl.className = 'fnx-rd-list';
                let addedLi = 0;
                for (let k = 0; k < items.length && k < 60; k++) {{
                    const txt = (items[k].textContent || '').replace(/\s+/g, ' ').trim();
                    if (txt.length < 3) continue;
                    const li = document.createElement('li');
                    li.textContent = txt;
                    listEl.appendChild(li);
                    charCount += txt.length;
                    addedLi++;
                }}
                if (addedLi > 0) frag.appendChild(listEl);
                continue;
            }}

            const txt = tag === 'pre'
                ? (node.textContent || '').trim()
                : (node.textContent || '').replace(/\s+/g, ' ').trim();
            if (!txt || (tag === 'p' && txt.length < 16)) continue;
            if (seenText[txt]) continue;
            seenText[txt] = true;

            if (tag === 'h1') hasH1 = true;
            const cleanEl = document.createElement(tag);
            cleanEl.className = 'fnx-rd-' + tag;
            cleanEl.textContent = txt;
            frag.appendChild(cleanEl);
            charCount += txt.length;
        }}

        if (charCount < 120) {{
            article.textContent = '';
            showHudToast('Not enough article text for Reader mode');
            return;
        }}

        if (!hasH1 && pageTitle) {{
            const h1 = document.createElement('h1');
            h1.className = 'fnx-rd-h1';
            h1.textContent = pageTitle;
            article.appendChild(h1);
        }}
        article.appendChild(frag);

        const mins = Math.max(1, Math.round(charCount / 900));
        if (metaSpan) {{
            metaSpan.textContent = `${{formatHostLabel(window.location.href)}} Â· ~${{mins}} min read`;
        }}

        readerSavedScroll = window.scrollY || window.pageYOffset || 0;
        isReaderOpen = true;
        syncReaderTypography();
        overlay.style.display = 'flex';
        overlay.scrollTop = 0;
        if (document.documentElement) document.documentElement.style.overflow = 'hidden';
        syncBookmarkButtons();
    }}

    function computeTabsSig() {{
        let s = String(tabState.active_id) + ':' + String(tabState.closed_count || 0) + '|';
        for (let i = 0; i < tabState.tabs.length; i++) {{
            const t = tabState.tabs[i];
            s += t.id + ':' + (t.pinned ? '1' : '0') + ':' + (t.title || '') + ':' + (t.url || '') + ';';
        }}
        return s;
    }}

    function renderDrawerCards() {{
        if (!shadowRoot) return;
        const reopenBtn = shadowRoot.getElementById('__fynexo_drawer_reopen__');
        if (reopenBtn) {{
            const cc = Number(tabState.closed_count) || 0;
            reopenBtn.style.display = cc > 0 ? 'inline-flex' : 'none';
            reopenBtn.textContent = cc > 1 ? `â†º Reopen (${{cc}})` : 'â†º Reopen';
        }}

        const sig = computeTabsSig();
        if (sig === lastDrawerTabsSig) return;
        lastDrawerTabsSig = sig;

        const drawerList = shadowRoot.getElementById('__fynexo_drawer_list__');
        if (!drawerList) return;
        drawerList.textContent = '';

        const frag = document.createDocumentFragment();
        tabState.tabs.forEach(tab => {{
            const isActive = tab.id === tabState.active_id;
            const isSet = isInternalSettings(tab.url);
            const isNt = isInternalNewtab(tab.url);
            const card = document.createElement('div');
            card.className = isActive ? 'fnx-drawer-card active' : 'fnx-drawer-card';

            const badge = document.createElement('div');
            badge.className = 'fnx-drawer-monogram';
            badge.textContent = extractMonogram(tab.url, tab.title);

            const meta = document.createElement('div');
            meta.className = 'fnx-drawer-meta';

            const t = document.createElement('div');
            t.className = 'fnx-drawer-title';
            const cleanTitle = isSet ? 'Preferences' : (isNt ? 'Start page' : (tab.title || 'New tab'));
            t.textContent = tab.pinned ? `ðŸ“Œ ${{cleanTitle}}` : cleanTitle;

            const u = document.createElement('div');
            u.className = 'fnx-drawer-url';
            u.textContent = isSet ? 'fynexo://settings' : (isNt ? 'fynexo://newtab' : formatHostLabel(tab.url || ''));

            meta.appendChild(t);
            meta.appendChild(u);

            const pinBtn = document.createElement('button');
            pinBtn.className = 'fnx-drawer-close';
            pinBtn.title = tab.pinned ? 'Unpin tab' : 'Pin tab';
            pinBtn.appendChild(createIcon('pin'));
            if (tab.pinned) {{
                pinBtn.style.color = 'var(--fnx-accent)';
                pinBtn.style.borderColor = 'var(--fnx-accent)';
            }}
            pinBtn.onclick = (e) => {{
                e.stopPropagation();
                sendIpc({{ type: 'TOGGLE_PIN_TAB', id: tab.id }});
            }};

            card.appendChild(badge);
            card.appendChild(meta);
            card.appendChild(pinBtn);

            if (!tab.pinned) {{
                const cBtn = document.createElement('button');
                cBtn.className = 'fnx-drawer-close';
                cBtn.setAttribute('aria-label', 'Close tab');
                cBtn.appendChild(createIcon('closeSmall'));
                cBtn.onclick = (e) => {{
                    e.stopPropagation();
                    sendIpc({{ type: 'CLOSE_TAB', id: tab.id }});
                }};
                card.appendChild(cBtn);
            }}

            card.onclick = () => {{
                closeMobileDrawer();
                if (!isActive) {{
                    flushScrollSync();
                    triggerProgress();
                    sendIpc({{ type: 'SWITCH_TAB', id: tab.id }});
                }}
            }};

            frag.appendChild(card);
        }});
        drawerList.appendChild(frag);
    }}

    let tabContextMenuEl = null;

    function hideTabContextMenu() {{
        if (tabContextMenuEl) {{
            tabContextMenuEl.style.display = 'none';
        }}
    }}

    function showTabContextMenu(e, tab, idx) {{
        if (!shadowRoot) return;
        if (!tabContextMenuEl) {{
            tabContextMenuEl = document.createElement('div');
            tabContextMenuEl.id = '__fynexo_tab_context_menu__';
        }}
        const targetParent = shadowRoot.getElementById('__fynexo_shell__') || shadowRoot;
        targetParent.appendChild(tabContextMenuEl);
        tabContextMenuEl.textContent = '';

        function addItem(label, shortcut, iconName, onClick, isDanger) {{
            const item = document.createElement('div');
            item.style.cssText = `display: flex; align-items: center; justify-content: space-between; padding: 6px 10px; border-radius: 6px; color: ${{isDanger ? '#f87171' : 'var(--fnx-fg)'}}; cursor: pointer; transition: background 0.08s ease; gap: 12px;`;

            const leftWrap = document.createElement('div');
            leftWrap.style.cssText = 'display: flex; align-items: center; gap: 8px;';
            if (iconName) {{
                const icon = createIcon(iconName);
                icon.style.cssText = 'width: 13.5px; height: 13.5px; opacity: 0.85; flex-shrink: 0;';
                leftWrap.appendChild(icon);
            }}
            const textSpan = document.createElement('span');
            textSpan.textContent = label;
            leftWrap.appendChild(textSpan);
            item.appendChild(leftWrap);

            if (shortcut) {{
                const sc = document.createElement('span');
                sc.style.cssText = 'font-size: 10.5px; color: var(--fnx-muted);';
                sc.textContent = shortcut;
                item.appendChild(sc);
            }}

            item.onmouseenter = () => {{
                item.style.background = isDanger ? 'rgba(239, 68, 68, 0.18)' : 'var(--fnx-hover)';
            }};
            item.onmouseleave = () => {{
                item.style.background = 'transparent';
            }};
            item.onclick = (ev) => {{
                ev.stopPropagation();
                hideTabContextMenu();
                onClick();
            }};
            tabContextMenuEl.appendChild(item);
        }}

        function addSep() {{
            const sep = document.createElement('div');
            sep.style.cssText = 'height: 1px; background: var(--fnx-border); margin: 3px 6px;';
            tabContextMenuEl.appendChild(sep);
        }}

        // 1. Reload
        addItem('Reload tab', 'Ctrl+R', 'reload', () => {{
            if (tab.id === tabState.active_id) {{
                sendIpc({{ type: 'RELOAD' }});
            }} else {{
                flushScrollSync();
                sendIpc({{ type: 'SWITCH_TAB', id: tab.id }});
                setTimeout(() => sendIpc({{ type: 'RELOAD' }}), 60);
            }}
        }});

        // 2. Duplicate
        addItem('Duplicate tab', '', 'plus', () => {{
            sendIpc({{ type: 'DUPLICATE_TAB', id: tab.id }});
        }});

        // 3. Pin / Unpin
        if (tab.pinned) {{
            addItem('Unpin tab', '', 'pin', () => {{
                sendIpc({{ type: 'TOGGLE_PIN_TAB', id: tab.id }});
            }});
        }} else {{
            addItem('Pin tab', '', 'pin', () => {{
                sendIpc({{ type: 'TOGGLE_PIN_TAB', id: tab.id }});
            }});
        }}

        // 3.5 Background Playback (Keep alive / continuous audio)
        if (tab.background_play) {{
            addItem('Disable background running', '', 'media', () => {{
                sendIpc({{ type: 'TOGGLE_BACKGROUND_PLAY', id: tab.id }});
            }});
        }} else {{
            addItem('Allow running in background', '', 'media', () => {{
                sendIpc({{ type: 'TOGGLE_BACKGROUND_PLAY', id: tab.id }});
            }});
        }}

        // 3.6 Mute / Unmute Tab Audio
        addItem(tab.muted ? 'Unmute tab' : 'Mute tab', '', tab.muted ? 'volumeUp' : 'volumeMute', () => {{
            sendIpc({{ type: 'TOGGLE_MUTE_TAB', id: tab.id }});
        }});

        addSep();

        // 4. Close Tab
        addItem('Close tab', 'Ctrl+W', 'closeSmall', () => {{
            sendIpc({{ type: 'CLOSE_TAB', id: tab.id, force: true }});
        }});

        // 5. Close Other Tabs
        if (tabState.tabs.length > 1) {{
            addItem('Close other tabs', '', '', () => {{
                sendIpc({{ type: 'CLOSE_OTHER_TABS', id: tab.id }});
            }});
        }}

        // 6. Close Tabs to the Right
        if (idx < tabState.tabs.length - 1) {{
            addItem('Close tabs to the right', '', '', () => {{
                sendIpc({{ type: 'CLOSE_TABS_TO_RIGHT', id: tab.id }});
            }});
        }}

        // 7. Close All Tabs
        addItem('Close all tabs', '', '', () => {{
            sendIpc({{ type: 'CLOSE_ALL_TABS' }});
        }}, true);

        addSep();

        // 8. Reopen Closed Tab
        addItem('Reopen closed tab', 'Ctrl+Shift+T', 'undo', () => {{
            sendIpc({{ type: 'REOPEN_CLOSED_TAB' }});
        }});

        // 9. New Tab
        addItem('New tab', 'Ctrl+T', 'plus', () => {{
            triggerProgress();
            sendIpc({{ type: 'NEW_TAB' }});
        }});

        addSep();

        // 10. Workspace session snapshot
        addItem('Save current workspace...', '', 'workspace', () => {{
            const name = window.prompt('Enter workspace name (e.g. Work, Music, Dev):');
            if (name && name.trim()) {{
                sendIpc({{ type: 'SAVE_WORKSPACE', name: name.trim() }});
            }}
        }});

        const menuW = 205;
        const menuH = tabContextMenuEl.children.length * 30 + 16;
        let posX = e.clientX;
        let posY = e.clientY + 4;
        if (posX + menuW > window.innerWidth - 8) {{
            posX = window.innerWidth - menuW - 8;
        }}
        if (posY + menuH > window.innerHeight - 8) {{
            posY = e.clientY - menuH - 4;
        }}
        tabContextMenuEl.style.left = Math.max(8, posX) + 'px';
        tabContextMenuEl.style.top = Math.max(8, posY) + 'px';
        tabContextMenuEl.style.display = 'flex';
    }}

    function renderTabs(force) {{
        if (!shadowRoot) return;

        const tabCount = Math.max(1, tabState.tabs.length);
        const mobileTabBadge = shadowRoot.getElementById('__fynexo_mobile_tab_count__');
        if (mobileTabBadge) {{
            const countStr = String(tabCount);
            if (mobileTabBadge.textContent !== countStr) mobileTabBadge.textContent = countStr;
        }}

        const drawerCount = shadowRoot.getElementById('__fynexo_drawer_count__');
        if (drawerCount) {{
            drawerCount.textContent = `${{tabState.tabs.length}} ${{tabState.tabs.length === 1 ? 'tab' : 'tabs'}}`;
        }}

        const isDesktopBarShown = window.innerWidth > 768
            ? (styles.topbar.visible_on_desktop !== false)
            : (styles.topbar.visible_on_mobile === true);

        if (isDesktopBarShown) {{
            const sig = computeTabsSig();
            if (force || sig !== lastDeskTabsSig) {{
                lastDeskTabsSig = sig;
                const container = shadowRoot.getElementById('__fynexo_tabs_container__');
                if (container) {{
                    container.textContent = '';
                    const frag = document.createDocumentFragment();
                    let activeTabEl = null;

                    tabState.tabs.forEach((tab, idx) => {{
                        const isActive = tab.id === tabState.active_id;
                        const isSet = isInternalSettings(tab.url);
                        const isNt = isInternalNewtab(tab.url);
                        const cleanTitle = isSet ? 'Preferences' : (isNt ? 'Start page' : (tab.title || 'New tab'));

                        const el = document.createElement('div');
                        let cls = isActive ? 'fnx-tab-item active' : 'fnx-tab-item';
                        if (tab.pinned) cls += ' pinned';
                        el.className = cls;
                        if (isActive) activeTabEl = el;

                        el.title = cleanTitle;

                        el.onclick = (e) => {{
                            if (e.target.closest && (e.target.closest('.fnx-tab-close') || e.target.closest('.fnx-tab-audio-btn'))) return;
                            if (!isActive) {{
                                flushScrollSync();
                                triggerProgress();
                                sendIpc({{ type: 'SWITCH_TAB', id: tab.id }});
                            }}
                        }};
                        el.oncontextmenu = (e) => {{
                            e.preventDefault();
                            e.stopPropagation();
                            showTabContextMenu(e, tab, idx);
                        }};
                        el.onmousedown = (e) => {{
                            if (e.button === 1) {{
                                e.preventDefault();
                                e.stopPropagation();
                                sendIpc({{ type: 'CLOSE_TAB', id: tab.id, force: true }});
                            }}
                        }};
                        el.onauxclick = (e) => {{
                            if (e.button === 1) {{
                                e.preventDefault();
                                e.stopPropagation();
                                sendIpc({{ type: 'CLOSE_TAB', id: tab.id, force: true }});
                            }}
                        }};

                        const mono = document.createElement('span');
                        mono.className = 'fnx-tab-monogram';
                        mono.textContent = extractMonogram(tab.url, tab.title);
                        mono.style.color = monogramAccentColor(tab.url);
                        el.appendChild(mono);

                        if (tab.background_play || tab.muted) {{
                            const audioBtn = document.createElement('button');
                            audioBtn.className = 'fnx-tab-audio-btn';
                            const isMuted = !!tab.muted;
                            const aIcon = createIcon(isMuted ? 'volumeMute' : 'volumeUp');
                            aIcon.style.cssText = 'width: 12px; height: 12px; pointer-events: none;';
                            audioBtn.appendChild(aIcon);
                            audioBtn.title = isMuted ? 'Unmute tab (Click to unmute)' : 'Mute tab (Click to mute)';
                            audioBtn.style.cssText = `
                                display: inline-flex;
                                align-items: center;
                                justify-content: center;
                                width: 18px;
                                height: 18px;
                                border-radius: 4px;
                                background: ${{isMuted ? 'rgba(239, 68, 68, 0.16)' : 'rgba(255, 255, 255, 0.08)'}};
                                color: ${{isMuted ? '#f87171' : 'var(--fnx-secondary)'}};
                                margin: 0 2px;
                                flex-shrink: 0;
                                border: none;
                                cursor: pointer;
                                padding: 0;
                                transition: background 0.15s ease, color 0.15s ease;
                            `;
                            audioBtn.onmouseenter = () => {{
                                audioBtn.style.background = isMuted ? 'rgba(239, 68, 68, 0.28)' : 'rgba(255, 255, 255, 0.18)';
                                audioBtn.style.color = 'var(--fnx-fg)';
                            }};
                            audioBtn.onmouseleave = () => {{
                                audioBtn.style.background = isMuted ? 'rgba(239, 68, 68, 0.16)' : 'rgba(255, 255, 255, 0.08)';
                                audioBtn.style.color = isMuted ? '#f87171' : 'var(--fnx-secondary)';
                            }};
                            audioBtn.onclick = (e) => {{
                                e.preventDefault();
                                e.stopPropagation();
                                sendIpc({{ type: 'TOGGLE_MUTE_TAB', id: tab.id }});
                            }};
                            el.appendChild(audioBtn);
                        }}

                        if (!tab.pinned) {{
                            const titleSpan = document.createElement('span');
                            titleSpan.className = 'fnx-tab-title';
                            titleSpan.textContent = cleanTitle;

                            const closeBtn = document.createElement('button');
                            closeBtn.className = 'fnx-tab-close';
                            closeBtn.appendChild(createIcon('closeSmall'));
                            closeBtn.title = `Close tab (${{idx + 1}})`;
                            closeBtn.onclick = (e) => {{
                                e.stopPropagation();
                                sendIpc({{ type: 'CLOSE_TAB', id: tab.id }});
                            }};

                            el.appendChild(titleSpan);
                            el.appendChild(closeBtn);
                        }}

                        frag.appendChild(el);
                    }});

                    container.appendChild(frag);
                    if (activeTabEl && container) {{
                        try {{
                            const cRect = container.getBoundingClientRect();
                            const tRect = activeTabEl.getBoundingClientRect();
                            if (tRect.left < cRect.left) {{
                                container.scrollLeft -= (cRect.left - tRect.left + 8);
                            }} else if (tRect.right > cRect.right) {{
                                container.scrollLeft += (tRect.right - cRect.right + 8);
                            }}
                        }} catch (_) {{}}
                    }}
                }}
            }}
        }}

        if (isDrawerOpen) {{
            renderDrawerCards();
        }}
    }}

    function renderOmniboxSuggestions(rawQuery, listEl, inputEl, closeFn) {{
        if (!shadowRoot || !listEl) return;
        listEl.textContent = '';

        const trimmed = (rawQuery || '').trim();
        const q = trimmed.toLowerCase();
        const results = [];
        const seen = Object.create(null);

        function pushCandidate(item) {{
            const key = item.kind === 'tab'
                ? ('tab:' + item.tabId)
                : (item.kind === 'bang' ? ('bang:' + item.trigger) : normalizeUrlKey(item.url));
            if (!key || seen[key]) return;
            seen[key] = true;
            results.push(item);
        }}

        // 0. Bang Shortcuts when starting with '!'
        if (q.startsWith('!')) {{
            const bangs = getBangs();
            const tok = q.split(' ')[0];
            for (let i = 0; i < bangs.length; i++) {{
                const b = bangs[i];
                if (!b || !b.trigger) continue;
                const trig = (b.trigger.startsWith('!') ? b.trigger : '!' + b.trigger).toLowerCase();
                if (trig.startsWith(tok) || (b.name || '').toLowerCase().includes(tok.slice(1))) {{
                    pushCandidate({{
                        kind: 'bang',
                        trigger: trig,
                        title: `${{trig}} â€” Search ${{b.name || trig}}`,
                        url: b.url || '',
                        badge: '!Bang'
                    }});
                }}
            }}
        }}

        // 1. Open Tabs
        for (let i = 0; i < tabState.tabs.length; i++) {{
            const t = tabState.tabs[i];
            if (t.id === tabState.active_id || isInternalAny(t.url)) continue;
            const title = t.title || formatHostLabel(t.url);
            const url = t.url || '';
            if (!q || title.toLowerCase().includes(q) || url.toLowerCase().includes(q)) {{
                pushCandidate({{ kind: 'tab', tabId: t.id, title: title, url: url, badge: 'Tab' }});
            }}
        }}

        // 2. Bookmarks
        for (let i = 0; i < bookmarks.length; i++) {{
            const b = bookmarks[i];
            const title = b.title || formatHostLabel(b.url);
            const url = b.url || '';
            if (!q || title.toLowerCase().includes(q) || url.toLowerCase().includes(q)) {{
                pushCandidate({{ kind: 'bookmark', title: title, url: url, badge: 'Saved' }});
            }}
        }}

        // 3. Recent History
        for (let i = 0; i < historyItems.length; i++) {{
            const h = historyItems[i];
            const title = h.title || formatHostLabel(h.url);
            const url = h.url || '';
            if (q && (title.toLowerCase().includes(q) || url.toLowerCase().includes(q))) {{
                pushCandidate({{ kind: 'history', title: title, url: url, badge: 'History' }});
            }} else if (!q && results.length < 5) {{
                pushCandidate({{ kind: 'history', title: title, url: url, badge: 'Recent' }});
            }}
        }}

        suggestItems = results.slice(0, 6);
        suggestIndex = -1;

        if (suggestItems.length === 0) {{
            listEl.style.display = 'none';
            return;
        }}

        listEl.style.display = 'flex';
        const frag = document.createDocumentFragment();
        suggestItems.forEach((item, idx) => {{
            const row = document.createElement('button');
            row.type = 'button';
            row.className = 'fnx-suggest-row';
            row.dataset.idx = String(idx);
            row.onmousedown = (e) => e.preventDefault();

            const iconWrap = document.createElement('span');
            iconWrap.className = 'fnx-suggest-icon';
            const iconName = item.kind === 'bookmark'
                ? 'starFilled'
                : (item.kind === 'history' ? 'history' : (item.kind === 'bang' ? 'search' : 'go'));
            iconWrap.appendChild(createIcon(iconName));
            if (item.kind === 'bookmark' || item.kind === 'bang') iconWrap.style.color = 'var(--fnx-accent)';

            const textWrap = document.createElement('span');
            textWrap.className = 'fnx-suggest-main';
            const tSpan = document.createElement('span');
            tSpan.className = 'fnx-suggest-title';
            tSpan.textContent = item.title;
            const uSpan = document.createElement('span');
            uSpan.className = 'fnx-suggest-url';
            uSpan.textContent = item.kind === 'bang' ? item.url : formatHostLabel(item.url);
            textWrap.appendChild(tSpan);
            textWrap.appendChild(uSpan);

            const tag = document.createElement('span');
            tag.className = 'fnx-suggest-badge';
            tag.textContent = item.badge;

            row.appendChild(iconWrap);
            row.appendChild(textWrap);
            row.appendChild(tag);

            row.onclick = () => {{
                if (item.kind === 'bang') {{
                    const cur = inputEl.value.trim();
                    if (!cur.includes(' ')) {{
                        inputEl.value = item.trigger + ' ';
                        inputEl.focus();
                        renderOmniboxSuggestions(inputEl.value, listEl, inputEl, closeFn);
                        return;
                    }}
                    const resolved = resolveUrl(cur, config.search_engine);
                    if (closeFn) closeFn();
                    flushScrollSync();
                    triggerProgress();
                    sendIpc({{ type: 'NAVIGATE', url: resolved }});
                    return;
                }}
                if (closeFn) closeFn();
                triggerProgress();
                if (item.kind === 'tab' && item.tabId) {{
                    flushScrollSync();
                    sendIpc({{ type: 'SWITCH_TAB', id: item.tabId }});
                }} else {{
                    flushScrollSync();
                    sendIpc({{ type: 'NAVIGATE', url: item.url }});
                }}
            }};
            frag.appendChild(row);
        }});
        listEl.appendChild(frag);
    }}

    function updateSuggestHighlight(listEl) {{
        if (!listEl) return;
        const rows = listEl.querySelectorAll('.fnx-suggest-row');
        rows.forEach((r, idx) => {{
            r.classList.toggle('selected', idx === suggestIndex);
        }});
    }}

    function renderHudQuickBar(quickRow, closeFn) {{
        if (!shadowRoot || !quickRow) return;
        quickRow.textContent = '';

        const leftGroup = document.createElement('div');
        leftGroup.className = 'fnx-quick-tabs';

        function makeChip(label, active, onClick) {{
            const c = document.createElement('button');
            c.type = 'button';
            c.className = active ? 'fnx-quick-chip active' : 'fnx-quick-chip';
            c.textContent = label;
            c.onmousedown = (e) => e.preventDefault();
            c.onclick = onClick;
            return c;
        }}

        leftGroup.appendChild(makeChip('Start', false, () => {{
            if (closeFn) closeFn();
            triggerProgress();
            sendIpc({{ type: 'NAVIGATE', url: 'fynexo://newtab' }});
        }}));

        if ((Number(tabState.closed_count) || 0) > 0) {{
            leftGroup.appendChild(makeChip('â†º Reopen', false, () => {{
                if (closeFn) closeFn();
                triggerProgress();
                sendIpc({{ type: 'REOPEN_CLOSED_TAB' }});
            }}));
        }}

        if (!isInternalAny()) {{
            leftGroup.appendChild(makeChip('Find', isFindOpen, () => {{
                if (closeFn) closeFn();
                openFindBar();
            }}));
            leftGroup.appendChild(makeChip('Reader', isReaderOpen, () => {{
                if (closeFn) closeFn();
                toggleReaderMode();
            }}));
        }}

        leftGroup.appendChild(makeChip(config.force_dark ? 'Dark: On' : 'Dark: Off', !!config.force_dark, () => {{
            config.force_dark = !config.force_dark;
            sendIpc({{ type: 'SAVE_CONFIG', config: config }});
            applyPageRules();
            renderHudQuickBar(quickRow, closeFn);
            if (config.force_dark && isPageAlreadyDark()) {{
                showHudToast('Smart Dark: Page is already dark (skipped)');
            }} else {{
                showHudToast(config.force_dark ? 'Smart Force Dark enabled' : 'Smart Force Dark off');
            }}
        }}));

        leftGroup.appendChild(makeChip(config.incognito ? 'Private: On' : 'Private', !!config.incognito, () => {{
            config.incognito = !config.incognito;
            sendIpc({{ type: 'SAVE_CONFIG', config: config }});
            syncBookmarkButtons();
            renderHudQuickBar(quickRow, closeFn);
            showHudToast(config.incognito ? 'Private mode on (history paused)' : 'Private mode off');
        }}));

        const rightGroup = document.createElement('div');
        rightGroup.className = 'fnx-quick-actions';

        const zoomPct = Math.round((Number(config.default_zoom) || 1) * 100);
        rightGroup.appendChild(makeChip('âˆ’', false, () => {{
            sendIpc({{ type: 'SET_ZOOM', delta: -0.1 }});
        }}));
        rightGroup.appendChild(makeChip(`${{zoomPct}}%`, zoomPct !== 100, () => {{
            sendIpc({{ type: 'SET_ZOOM', zoom: 1.0 }});
        }}));
        rightGroup.appendChild(makeChip('+', false, () => {{
            sendIpc({{ type: 'SET_ZOOM', delta: 0.1 }});
        }}));

        rightGroup.appendChild(makeChip('Prefs', false, () => {{
            if (closeFn) closeFn();
            togglePreferences();
        }}));

        quickRow.appendChild(leftGroup);
        quickRow.appendChild(rightGroup);
    }}

    function bindOmniboxKeydown(inputEl, listEl, submitFn, closeFn) {{
        inputEl.addEventListener('keydown', (e) => {{
            if (e.key === 'ArrowDown' && suggestItems.length > 0) {{
                e.preventDefault();
                suggestIndex = (suggestIndex + 1) % suggestItems.length;
                updateSuggestHighlight(listEl);
                return;
            }}
            if (e.key === 'ArrowUp' && suggestItems.length > 0) {{
                e.preventDefault();
                suggestIndex = (suggestIndex - 1 + suggestItems.length) % suggestItems.length;
                updateSuggestHighlight(listEl);
                return;
            }}
            if (e.key === 'Escape') {{
                e.preventDefault();
                e.stopPropagation();
                if (closeFn) closeFn();
                return;
            }}
            if (e.key === 'Enter') {{
                e.preventDefault();
                if (suggestIndex >= 0 && suggestItems[suggestIndex]) {{
                    const item = suggestItems[suggestIndex];
                    if (item.kind === 'bang') {{
                        const cur = inputEl.value.trim();
                        if (!cur.includes(' ')) {{
                            inputEl.value = item.trigger + ' ';
                            renderOmniboxSuggestions(inputEl.value, listEl, inputEl, closeFn);
                            return;
                        }}
                        submitFn();
                        return;
                    }}
                    if (closeFn) closeFn();
                    flushScrollSync();
                    triggerProgress();
                    if (item.kind === 'tab' && item.tabId) {{
                        sendIpc({{ type: 'SWITCH_TAB', id: item.tabId }});
                    }} else {{
                        sendIpc({{ type: 'NAVIGATE', url: item.url }});
                    }}
                }} else {{
                    submitFn();
                }}
            }}
        }});
    }}

    function fillQueryTemplate(tpl, encodedQuery) {{
        const s = String(tpl || 'https://www.google.com/search?q={{query}}');
        if (s.includes('{{query}}')) return s.split('{{query}}').join(encodedQuery);
        if (s.includes('%s')) return s.split('%s').join(encodedQuery);
        return s + encodedQuery;
    }}

    function resolveUrl(raw, engine) {{
        raw = String(raw || '').trim();
        if (!raw) return '';

        if (/^fynexo:\/\//i.test(raw)) return raw;
        if (/^(settings|preferences|fynexo:settings|about:settings|about:config)$/i.test(raw)) {{
            return 'fynexo://settings';
        }}
        if (/^(newtab|start|home|fynexo:newtab|about:newtab|about:home)$/i.test(raw)) {{
            return 'fynexo://newtab';
        }}

        // Bang shortcut resolution (!gh, !yt, !b, !w, etc.)
        if (raw.startsWith('!')) {{
            const spIdx = raw.indexOf(' ');
            const tok = (spIdx === -1 ? raw : raw.slice(0, spIdx)).toLowerCase();
            const rest = spIdx === -1 ? '' : raw.slice(spIdx + 1).trim();
            const bangs = getBangs();
            for (let i = 0; i < bangs.length; i++) {{
                const b = bangs[i];
                if (!b || !b.trigger) continue;
                const trig = (b.trigger.startsWith('!') ? b.trigger : '!' + b.trigger).toLowerCase();
                if (trig === tok) {{
                    if (!rest) {{
                        try {{
                            return new URL(fillQueryTemplate(b.url, '')).origin;
                        }} catch (_) {{
                            return fillQueryTemplate(b.url, '');
                        }}
                    }}
                    return fillQueryTemplate(b.url, encodeURIComponent(rest));
                }}
            }}
        }}

        if (/^https?:\/\//i.test(raw)) return raw;
        if (/^(localhost|127\.0\.0\.1)(:\d+)?/i.test(raw)) return 'http://' + raw;
        if (raw.includes('.') && !raw.includes(' ')) return 'https://' + raw;

        const q = encodeURIComponent(raw);
        const engObj = currentEngineObj(engine);
        return fillQueryTemplate(engObj && engObj.url, q);
    }}

    function triggerProgress() {{
        if (!shadowRoot) return;
        const pb = shadowRoot.getElementById('__fynexo_progress__');
        if (!pb) return;
        pb.style.transition = 'none';
        pb.style.width = '0%';
        pb.style.opacity = '1';
        setTimeout(() => {{
            pb.style.transition = 'width 0.28s cubic-bezier(0.22, 1, 0.36, 1), opacity 0.2s ease';
            pb.style.width = '60%';
        }}, 16);
    }}

    function finishProgress() {{
        if (!shadowRoot) return;
        const pb = shadowRoot.getElementById('__fynexo_progress__');
        if (!pb) return;
        pb.style.width = '100%';
        setTimeout(() => {{
            pb.style.opacity = '0';
            setTimeout(() => {{ pb.style.width = '0%'; }}, 220);
        }}, 120);
    }}

    function openMobileDrawer() {{
        if (!shadowRoot) return;
        isDrawerOpen = true;
        renderDrawerCards();
        const drawer = shadowRoot.getElementById('__fynexo_mobile_drawer__');
        if (drawer) drawer.style.display = 'flex';
    }}

    function closeMobileDrawer() {{
        if (!shadowRoot) return;
        isDrawerOpen = false;
        const drawer = shadowRoot.getElementById('__fynexo_mobile_drawer__');
        if (drawer) drawer.style.display = 'none';
    }}

    function togglePreferences() {{
        if (isInternalSettings()) {{
            sendIpc({{ type: 'GO_BACK' }});
        }} else {{
            triggerProgress();
            sendIpc({{ type: 'NAVIGATE', url: 'fynexo://settings' }});
        }}
    }}

    // Keybinding Parser & Matcher
    function matchesKeybind(e, spec) {{
        if (!spec || typeof spec !== 'string') return false;
        const raw = spec.trim();
        if (!raw) return false;

        const tokens = [];
        let buf = '';
        for (let i = 0; i < raw.length; i++) {{
            if (raw[i] === '+' && buf.length > 0) {{
                tokens.push(buf.toLowerCase());
                buf = '';
            }} else {{
                buf += raw[i];
            }}
        }}
        if (buf.length > 0) tokens.push(buf.toLowerCase());
        if (tokens.length === 0) return false;

        let wantCtrl = false;
        let wantAlt = false;
        let wantShift = false;
        let wantKey = '';

        for (let i = 0; i < tokens.length; i++) {{
            const t = tokens[i];
            if (t === 'ctrl' || t === 'cmd' || t === 'meta' || t === 'control') wantCtrl = true;
            else if (t === 'alt' || t === 'option') wantAlt = true;
            else if (t === 'shift') wantShift = true;
            else wantKey = t;
        }}
        if (!wantKey) return false;

        const hasCtrl = !!(e.ctrlKey || e.metaKey);
        if (hasCtrl !== wantCtrl || !!e.altKey !== wantAlt) return false;

        const evKey = (e.key || '').toLowerCase();
        if (wantKey === '=' && (evKey === '=' || evKey === '+')) {{
            return true;
        }}
        if (!!e.shiftKey !== wantShift) return false;
        if (wantKey === 'space') return evKey === ' ';
        if (wantKey === 'esc') return evKey === 'escape';
        return evKey === wantKey;
    }}

    // Combined Single-Eval Boot Sync
    window.__fynexo_boot_sync__ = function(rtPayload, tabsPayload) {{
        if (rtPayload) {{
            if (rtPayload.config) config = Object.assign(config, rtPayload.config);
            if (rtPayload.styles) styles = mergeStyles(rtPayload.styles);
            if (rtPayload.active_theme) activeTheme = Object.assign(activeTheme, rtPayload.active_theme);
            if (rtPayload.keybinds) keybinds = Object.assign({{}}, DEFAULT_KEYBINDS, rtPayload.keybinds);
            if (Array.isArray(rtPayload.site_rules)) {{
                enabledRules = rtPayload.site_rules.filter(r => r && r.enabled);
            }}
            if (Array.isArray(rtPayload.bookmarks)) bookmarks = rtPayload.bookmarks;
            if (Array.isArray(rtPayload.history)) historyItems = rtPayload.history;
            cacheSlimRuntime();
            if (typeof window.__fynexo_on_settings_runtime__ === 'function') {{
                window.__fynexo_on_settings_runtime__(rtPayload);
            }}
        }}
        if (tabsPayload) {{
            tabState = tabsPayload;
            writeSessionCache('__fnx_tabs__', tabsPayload);
        }}
        syncAllStyles();
    }};

    window.__fynexo_set_runtime__ = function(payload) {{
        if (!payload) return;
        if (payload.config) config = Object.assign(config, payload.config);
        if (payload.styles) styles = mergeStyles(payload.styles);
        if (payload.active_theme) activeTheme = Object.assign(activeTheme, payload.active_theme);
        if (payload.keybinds) keybinds = Object.assign({{}}, DEFAULT_KEYBINDS, payload.keybinds);
        if (Array.isArray(payload.site_rules)) {{
            enabledRules = payload.site_rules.filter(r => r && r.enabled);
        }}
        if (Array.isArray(payload.bookmarks)) bookmarks = payload.bookmarks;
        if (Array.isArray(payload.history)) historyItems = payload.history;
        cacheSlimRuntime();
        syncAllStyles();
        const input = shadowRoot ? (shadowRoot.getElementById('__fynexo_inline_input__') || shadowRoot.getElementById('__fynexo_input__')) : null;
        if (input) {{
            input.placeholder = `Search ${{currentEngineObj().name}} or enter !bang / URL...`;
        }}
        if (shadowRoot) {{
            const inlineQuick = shadowRoot.getElementById('__fynexo_inline_quick__');
            if (inlineQuick) renderHudQuickBar(inlineQuick, closeOmniboxRef);
            const modalQuick = shadowRoot.getElementById('__fynexo_hud_quick__');
            if (isModalActive && modalQuick) renderHudQuickBar(modalQuick, closeOmniboxRef);
        }}
        if (typeof window.__fynexo_on_settings_runtime__ === 'function') {{
            window.__fynexo_on_settings_runtime__(payload);
        }}
    }};

    window.__fynexo_set_fullscreen__ = function(full) {{
        isFullscreen = !!full;
        if (!isFullscreen) isTopbarPeek = false;
        syncAllStyles();
    }};

    window.__fynexo_toast__ = function(msg) {{
        showHudToast(msg);
        if (typeof handleDownloadToast === 'function') {{
            handleDownloadToast(msg);
        }}
    }};

    window.__fynexo_restore_scroll__ = function(targetY) {{
        const y = Number(targetY) || 0;
        if (y <= 24 || isInternalAny()) return;
        const attemptScroll = () => {{
            try {{
                if (document.body) document.body.scrollTop = y;
                if (document.documentElement) document.documentElement.scrollTop = y;
                window.scrollTo(0, y);
            }} catch (_) {{}}
        }};
        attemptScroll();
        if (document.readyState !== 'complete') {{
            window.addEventListener('load', () => setTimeout(attemptScroll, 80), {{ once: true }});
        }} else {{
            setTimeout(attemptScroll, 80);
        }}
    }};

    // Android Hardware / Gesture Back Hook: returns true if Fynexo consumed the back action
    window.__fynexo_handle_back__ = function() {{
        if (isReaderOpen) {{
            closeReaderMode();
            return true;
        }}
        if (isFindOpen) {{
            closeFindBar();
            return true;
        }}
        if (isDrawerOpen) {{
            closeMobileDrawer();
            return true;
        }}
        if (isInlineActive || isModalActive) {{
            if (closeOmniboxRef) closeOmniboxRef();
            return true;
        }}
        if (document.fullscreenElement && typeof document.exitFullscreen === 'function') {{
            try {{ document.exitFullscreen(); }} catch (_) {{}}
            return true;
        }}
        return false;
    }};

    window.__fynexo_set_config__ = function(newCfg) {{
        if (!newCfg) return;
        config = Object.assign(config, newCfg);
        syncAllStyles();
    }};

    window.__fnx_set_tab_muted__ = function(isMuted) {{
        window.__fnx_muted = !!isMuted;
        try {{
            document.querySelectorAll('video, audio').forEach(el => {{
                try {{ el.muted = !!isMuted; }} catch(_) {{}}
            }});
        }} catch(_) {{}}
    }};

    try {{
        const origMediaPlay = HTMLMediaElement.prototype.play;
        HTMLMediaElement.prototype.play = function() {{
            if (window.__fnx_muted) {{
                try {{ this.muted = true; }} catch(_) {{}}
            }}
            return origMediaPlay.apply(this, arguments);
        }};
    }} catch(_) {{}}

    window.__fynexo_set_tabs__ = function(state) {{
        if (!state) return;
        tabState = state;
        writeSessionCache('__fnx_tabs__', state);
        renderTabs(false);
        updateAddressLabels();
        try {{
            const currentTab = state.tabs && state.tabs.find(t => t.id === state.active_id);
            if (currentTab && typeof currentTab.muted === 'boolean') {{
                window.__fnx_set_tab_muted__(currentTab.muted);
            }}
        }} catch(_) {{}}
    }};

    function setKeyboardHiddenState(hidden) {{
        if (styles.mobile_bar.auto_hide_on_keyboard === false) hidden = false;
        if (isPageKeyboardOpen === hidden) return;
        isPageKeyboardOpen = hidden;
        if (document.body) {{
            if (hidden) document.body.classList.add('fnx-kb-active');
            else document.body.classList.remove('fnx-kb-active');
        }}
        applyShadowStyles();
    }}

    function isEditableWebElement(el) {{
        if (!el || el.id === '__fynexo_host__') return false;
        const tag = (el.tagName || '').toLowerCase();
        if (tag === 'textarea') return true;
        if (tag === 'input') {{
            const t = (el.type || 'text').toLowerCase();
            return !['button', 'checkbox', 'radio', 'submit', 'reset', 'range', 'file', 'image', 'hidden'].includes(t);
        }}
        return !!el.isContentEditable;
    }}

    function initInterface() {{
        try {{
            if (document.getElementById('__fynexo_host__')) return;

            try {{
                if (!window.__fynexo_scroll_proxied__) {{
                    window.__fynexo_scroll_proxied__ = true;
                    Object.defineProperty(window, 'scrollY', {{
                        get: () => (document.body ? document.body.scrollTop : 0) || (document.documentElement ? document.documentElement.scrollTop : 0) || 0,
                        configurable: true
                    }});
                    Object.defineProperty(window, 'pageYOffset', {{
                        get: () => (document.body ? document.body.scrollTop : 0) || (document.documentElement ? document.documentElement.scrollTop : 0) || 0,
                        configurable: true
                    }});
                }}
            }} catch (_) {{}}

            // Feature 3: Background Audio Keep-Alive & Anti-Pause (Prevent auto-pause when minimized/occluded/screen locked)
            try {{
                if (!window.__fynexo_bg_audio_hooked__) {{
                    window.__fynexo_bg_audio_hooked__ = true;
                    try {{
                        Object.defineProperty(Document.prototype, 'hidden', {{
                            get: () => false,
                            configurable: true
                        }});
                        Object.defineProperty(Document.prototype, 'visibilityState', {{
                            get: () => 'visible',
                            configurable: true
                        }});
                    }} catch (_) {{}}
                    try {{
                        Object.defineProperty(document, 'hidden', {{
                            get: () => false,
                            configurable: true
                        }});
                        Object.defineProperty(document, 'visibilityState', {{
                            get: () => 'visible',
                            configurable: true
                        }});
                    }} catch (_) {{}}

                    window.addEventListener('visibilitychange', (e) => {{
                        e.stopImmediatePropagation();
                    }}, true);

                    window.addEventListener('blur', () => {{
                        try {{
                            const mediaEls = document.querySelectorAll('video, audio');
                            mediaEls.forEach(m => {{
                                if (!m.paused) {{
                                    setTimeout(() => {{
                                        if (m.paused) {{
                                            try {{ m.play(); }} catch (_) {{}}
                                        }}
                                    }}, 60);
                                }}
                            }});
                        }} catch (_) {{}}
                    }}, true);
                }}
            }} catch (_) {{}}

            const rootContainer = document.documentElement || document.body;
            if (!rootContainer) return;

            const host = document.createElement('div');
            host.id = '__fynexo_host__';
            host.style.cssText = `
                all: initial !important;
                position: fixed !important;
                top: 0 !important;
                left: 0 !important;
                width: 100% !important;
                height: 0 !important;
                overflow: visible !important;
                z-index: 2147483647 !important;
            `;
            rootContainer.appendChild(host);

            shadowRoot = host.attachShadow({{ mode: 'open' }});

            const baseStyle = document.createElement('style');

            baseStyle.textContent = {base_styles_css};

            shadowRoot.appendChild(baseStyle);

            const shell = document.createElement('div');
            shell.id = '__fynexo_shell__';
            shadowRoot.appendChild(shell);

            const progressBar = document.createElement('div');
            progressBar.id = '__fynexo_progress__';
            shell.appendChild(progressBar);

            const hudToast = document.createElement('div');
            hudToast.id = '__fynexo_hud_toast__';
            const hudToastDot = document.createElement('span');
            hudToastDot.style.cssText = 'width: 6px; height: 6px; border-radius: 50%; background: var(--fnx-accent); flex-shrink: 0;';
            const hudToastMsg = document.createElement('span');
            hudToastMsg.id = '__fynexo_hud_toast_msg__';
            hudToast.appendChild(hudToastDot);
            hudToast.appendChild(hudToastMsg);
            shell.appendChild(hudToast);

            // 1. Desktop Topbar (3-Tier Layered Studio Deck + Single-Row Compact Mode)
            const topbar = document.createElement('div');
            topbar.id = '__fynexo_topbar__';
            shell.appendChild(topbar);

            topbar.addEventListener('mouseleave', () => {{
                if (isFullscreen && isTopbarPeek) {{
                    isTopbarPeek = false;
                    applyShadowStyles();
                }}
            }});

            function makeDesktopIconBtn(iconName, tooltip, onClick) {{
                const b = document.createElement('button');
                b.type = 'button';
                b.className = 'fnx-icon-btn';
                b.title = tooltip;
                b.appendChild(createIcon(iconName));
                b.onclick = onClick;
                return b;
            }}

            function makeWinBtn(iconName, tooltip, extraClass, onClick) {{
                const b = document.createElement('button');
                b.type = 'button';
                b.className = extraClass ? `fnx-win-btn ${{extraClass}}` : 'fnx-win-btn';
                b.title = tooltip;
                b.appendChild(createIcon(iconName));
                b.onclick = onClick;
                return b;
            }}

            // Tier 1: Recessed Tab Deck (#__fynexo_tab_row__)
            const tabRow = document.createElement('div');
            tabRow.id = '__fynexo_tab_row__';

            const btnBmToggle = makeDesktopIconBtn('chevronDown', 'Toggle shortcuts bar', () => {{
                const cur = (styles.topbar.bookmarks_bar || 'always').toLowerCase();
                styles.topbar.bookmarks_bar = cur === 'hidden' ? 'always' : 'hidden';
                syncAllStyles();
                sendIpc({{ type: 'SAVE_STYLES', styles: styles }});
                showHudToast(styles.topbar.bookmarks_bar === 'hidden' ? 'Shortcuts bar hidden' : 'Shortcuts bar visible');
            }});
            btnBmToggle.id = '__fynexo_btn_bm_toggle__';
            tabRow.appendChild(btnBmToggle);

            const tabsDeckCluster = document.createElement('div');
            tabsDeckCluster.id = '__fynexo_tabs_deck_cluster__';

            const tabsContainer = document.createElement('div');
            tabsContainer.id = '__fynexo_tabs_container__';
            tabsContainer.style.cssText = 'display: flex; overflow-x: auto; flex: 0 1 auto; max-width: calc(100% - 64px); height: 100%; scrollbar-width: none;';

            const handleTabsWheel = (e) => {{
                const delta = Math.abs(e.deltaY) > Math.abs(e.deltaX) ? e.deltaY : e.deltaX;
                if (delta !== 0) {{
                    e.preventDefault();
                    tabsContainer.scrollLeft += delta;
                }}
            }};
            tabsContainer.addEventListener('wheel', handleTabsWheel, {{ passive: false }});
            tabRow.addEventListener('wheel', (e) => {{
                if (!e.target.closest || !e.target.closest('#__fynexo_win_controls__')) {{
                    handleTabsWheel(e);
                }}
            }}, {{ passive: false }});

            const btnAddTab = makeDesktopIconBtn('plus', `New tab (${{keybinds.new_tab || 'Ctrl+T'}})`, () => {{
                triggerProgress();
                sendIpc({{ type: 'NEW_TAB' }});
            }});
            btnAddTab.id = '__fynexo_btn_newtab__';
            btnAddTab.style.cssText = 'width: 28px; height: 28px; margin: 0 4px 3px 6px; border-radius: 8px; align-self: flex-end; flex-shrink: 0;';

            const dragRegion = document.createElement('div');
            dragRegion.style.cssText = 'flex: 1; min-width: 24px; height: 100%;';

            let lastMaxTime = 0;
            const triggerMaximize = () => {{
                const now = Date.now();
                if (now - lastMaxTime < 350) return;
                lastMaxTime = now;
                sendIpc({{ type: 'WINDOW_MAXIMIZE' }});
            }};

            const isDragTarget = (t) => (t === tabRow || t === tabsDeckCluster || t === tabsContainer || t === dragRegion);
            tabRow.onmousedown = (e) => {{
                if (e.button === 0 && isDragTarget(e.target)) {{
                    if (e.detail === 2) {{
                        triggerMaximize();
                        return;
                    }}
                    sendIpc({{ type: 'WINDOW_DRAG' }});
                }}
            }};

            tabsDeckCluster.appendChild(tabsContainer);
            tabsDeckCluster.appendChild(btnAddTab);
            tabsDeckCluster.appendChild(dragRegion);
            tabRow.appendChild(tabsDeckCluster);

            const winControls = document.createElement('div');
            winControls.id = '__fynexo_win_controls__';
            winControls.appendChild(makeWinBtn('fullscreen', 'Immersive Fullscreen (F11)', '', () => sendIpc({{ type: 'TOGGLE_FULLSCREEN' }})));
            winControls.appendChild(makeWinBtn('winMin', 'Minimize', '', () => sendIpc({{ type: 'WINDOW_MINIMIZE' }})));
            winControls.appendChild(makeWinBtn('winMax', 'Maximize', '', () => sendIpc({{ type: 'WINDOW_MAXIMIZE' }})));
            winControls.appendChild(makeWinBtn('winClose', 'Close', 'close', () => sendIpc({{ type: 'WINDOW_CLOSE' }})));
            tabRow.appendChild(winControls);
            topbar.appendChild(tabRow);

            // Tier 2: Navigation & Recessed Omnibox Deck (#__fynexo_nav_row__)
            const navRow = document.createElement('div');
            navRow.id = '__fynexo_nav_row__';

            const leftZone = document.createElement('div');
            leftZone.id = '__fynexo_left_zone__';
            leftZone.style.cssText = 'display: flex; align-items: center; gap: 2px; height: 100%; flex-shrink: 0;';

            const btnBack = makeDesktopIconBtn('back', 'Back (Alt+Left)', () => {{ triggerProgress(); sendIpc({{ type: 'GO_BACK' }}); }});
            btnBack.id = '__fynexo_btn_back__';
            leftZone.appendChild(btnBack);

            const btnForward = makeDesktopIconBtn('forward', 'Forward (Alt+Right)', () => {{ triggerProgress(); sendIpc({{ type: 'GO_FORWARD' }}); }});
            btnForward.id = '__fynexo_btn_forward__';
            leftZone.appendChild(btnForward);

            const btnReload = makeDesktopIconBtn('reload', 'Reload (Ctrl+R)', () => {{ triggerProgress(); sendIpc({{ type: 'RELOAD' }}); }});
            btnReload.id = '__fynexo_btn_reload__';
            leftZone.appendChild(btnReload);

            const btnHome = makeDesktopIconBtn('home', 'Start page', () => {{
                flushScrollSync();
                triggerProgress();
                sendIpc({{ type: 'NAVIGATE', url: 'fynexo://newtab' }});
            }});
            btnHome.id = '__fynexo_btn_home__';
            leftZone.appendChild(btnHome);
            navRow.appendChild(leftZone);

            // Center Zone (hosts Recessed Capsule Omnibox in Deck mode, or Tabs in Compact mode)
            const centerZone = document.createElement('div');
            centerZone.id = '__fynexo_center_zone__';
            centerZone.style.cssText = 'display: flex; align-items: center; flex: 1; height: 100%; min-width: 0; overflow: hidden;';
            centerZone.onmousedown = (e) => {{
                if (e.button === 0 && (e.target === centerZone || isDragTarget(e.target))) {{
                    if (e.detail === 2) {{
                        triggerMaximize();
                        return;
                    }}
                    sendIpc({{ type: 'WINDOW_DRAG' }});
                }}
            }};

            const addrSlot = document.createElement('div');
            addrSlot.id = '__fynexo_addr_slot__';

            const addrBtn = document.createElement('button');
            addrBtn.type = 'button';
            addrBtn.id = '__fynexo_address_btn__';
            addrBtn.title = 'Click to edit URL or search (!bangs supported)';
            const addrIcon = document.createElement('span');
            addrIcon.id = '__fynexo_desk_proto__';
            addrIcon.style.cssText = 'display: flex; align-items: center; flex-shrink: 0;';

            const hostSpan = document.createElement('span');
            hostSpan.id = '__fynexo_cmd_label__';
            const addrScheme = document.createElement('span');
            addrScheme.id = '__fynexo_addr_scheme__';
            const addrDomain = document.createElement('span');
            addrDomain.id = '__fynexo_addr_domain__';
            const addrPath = document.createElement('span');
            addrPath.id = '__fynexo_addr_path__';
            hostSpan.appendChild(addrScheme);
            hostSpan.appendChild(addrDomain);
            hostSpan.appendChild(addrPath);

            addrBtn.appendChild(addrIcon);
            addrBtn.appendChild(hostSpan);
            addrBtn.onclick = () => openHud();
            addrSlot.appendChild(addrBtn);

            // Trailing inline actions inside the right edge of the recessed address bar capsule
            const addrActions = document.createElement('div');
            addrActions.id = '__fynexo_addr_actions__';

            const addrHint = document.createElement('span');
            addrHint.id = '__fynexo_addr_hint__';
            addrHint.className = 'fnx-addr-hint';
            addrHint.textContent = 'Ctrl L';
            addrActions.appendChild(addrHint);

            const btnDeskReader = document.createElement('button');
            btnDeskReader.type = 'button';
            btnDeskReader.id = '__fynexo_desk_reader__';
            btnDeskReader.className = 'fnx-addr-action-btn';
            btnDeskReader.title = 'Reader mode (Alt+R)';
            btnDeskReader.appendChild(createIcon('reader'));
            btnDeskReader.onclick = (e) => {{
                e.stopPropagation();
                toggleReaderMode();
            }};
            addrActions.appendChild(btnDeskReader);

            const btnDeskStar = document.createElement('button');
            btnDeskStar.type = 'button';
            btnDeskStar.id = '__fynexo_desk_star__';
            btnDeskStar.className = 'fnx-addr-action-btn';
            btnDeskStar.dataset.starKind = 'star';
            btnDeskStar.title = 'Bookmark this page (Ctrl+D)';
            btnDeskStar.appendChild(createIcon('star'));
            btnDeskStar.onclick = (e) => {{
                e.stopPropagation();
                toggleCurrentBookmark();
            }};
            addrActions.appendChild(btnDeskStar);

            addrSlot.appendChild(addrActions);
            centerZone.appendChild(addrSlot);
            navRow.appendChild(centerZone);

            // Right Studio Utility Zone
            const rightZone = document.createElement('div');
            rightZone.id = '__fynexo_right_zone__';
            rightZone.style.cssText = 'display: flex; align-items: center; height: 100%; gap: 3px; flex-shrink: 0;';

            // 1. Incognito / Private Badge (Conditional: only shown in private mode)
            const incogBadge = document.createElement('button');
            incogBadge.type = 'button';
            incogBadge.id = '__fynexo_incog_badge__';
            incogBadge.className = 'fnx-incog-pill';
            incogBadge.title = 'Private mode active â€” click to exit Private mode';
            incogBadge.textContent = 'Private';
            incogBadge.style.display = config.incognito ? 'inline-flex' : 'none';
            incogBadge.onclick = () => {{
                config.incognito = false;
                sendIpc({{ type: 'SAVE_CONFIG', config: config }});
                incogBadge.style.display = 'none';
                syncBookmarkButtons();
                showHudToast('Private mode turned off');
            }};
            rightZone.appendChild(incogBadge);

            // 2. Media Control Button (Conditional: only shown when audio/video is playing)
            const btnMedia = makeDesktopIconBtn('media', 'Media Control & Volume Booster', (e) => {{
                if (e && e.stopPropagation) e.stopPropagation();
                toggleMediaPopover();
            }});
            btnMedia.id = '__fynexo_btn_media__';
            btnMedia.classList.add('fnx-popover-trigger');
            btnMedia.style.display = 'none';
            btnMedia.style.position = 'relative';
            const mediaPulse = document.createElement('span');
            mediaPulse.className = 'fnx-pulse-dot';
            mediaPulse.style.cssText = 'position: absolute; top: 3px; right: 3px;';
            btnMedia.appendChild(mediaPulse);
            rightZone.appendChild(btnMedia);

            // 3. Downloads Button (Conditional: only shown when downloading or recent download)
            const btnDownloads = makeDesktopIconBtn('download', 'Downloads', (e) => {{
                if (e && e.stopPropagation) e.stopPropagation();
                toggleDownloadsPopover();
            }});
            btnDownloads.id = '__fynexo_btn_downloads__';
            btnDownloads.classList.add('fnx-popover-trigger');
            btnDownloads.style.display = 'none';
            btnDownloads.style.position = 'relative';
            const dlBadge = document.createElement('span');
            dlBadge.id = '__fynexo_dl_badge__';
            dlBadge.className = 'fnx-btn-badge';
            dlBadge.style.display = 'none';
            btnDownloads.appendChild(dlBadge);
            rightZone.appendChild(btnDownloads);

            // 4. Adblock & Privacy Shield (Always visible)
            const btnShield = makeDesktopIconBtn('shield', 'Adblock & Privacy Shield', (e) => {{
                if (e && e.stopPropagation) e.stopPropagation();
                toggleShieldPopover();
            }});
            btnShield.id = '__fynexo_btn_shield__';
            btnShield.classList.add('fnx-popover-trigger');
            btnShield.style.position = 'relative';
            const shieldBadge = document.createElement('span');
            shieldBadge.id = '__fynexo_shield_badge__';
            shieldBadge.className = 'fnx-btn-badge';
            shieldBadge.textContent = '0';
            shieldBadge.style.display = 'none';
            btnShield.appendChild(shieldBadge);
            rightZone.appendChild(btnShield);

            // 4.5 Side Panel Toggle (Alt+S)
            const btnSidePanel = makeDesktopIconBtn('sidepanel', 'Side Panel (Alt+S)', (e) => {{
                if (e && e.stopPropagation) e.stopPropagation();
                toggleSidePanel();
            }});
            btnSidePanel.id = '__fynexo_btn_sidepanel__';
            rightZone.appendChild(btnSidePanel);

            // 5. Main Menu 3-Dots Button (Always visible)
            const btnMore = makeDesktopIconBtn('more', 'Menu', (e) => {{
                if (e && e.stopPropagation) e.stopPropagation();
                toggleMainMenu();
            }});
            btnMore.id = '__fynexo_btn_more__';
            btnMore.classList.add('fnx-popover-trigger');
            rightZone.appendChild(btnMore);

            navRow.appendChild(rightZone);
            topbar.appendChild(navRow);

            // --- Nav Popovers System (Menu, Media Booster, Shield, Downloads, Task Manager, Side Panel) ---
            let activeNavPopoverEl = null;
            let mainMenuEl = null;
            let mediaPopoverEl = null;
            let shieldPopoverEl = null;
            let downloadsPopoverEl = null;
            let taskMgrPopoverEl = null;
            let sidePanelEl = null;
            let isSidePanelOpen = false;

            function closeAllNavPopovers() {{
                if (mainMenuEl) mainMenuEl.style.display = 'none';
                if (mediaPopoverEl) mediaPopoverEl.style.display = 'none';
                if (shieldPopoverEl) shieldPopoverEl.style.display = 'none';
                if (downloadsPopoverEl) downloadsPopoverEl.style.display = 'none';
                if (taskMgrPopoverEl) taskMgrPopoverEl.style.display = 'none';
                activeNavPopoverEl = null;
            }}

            function positionNavPopover(btn, popoverEl) {{
                if (!btn || !popoverEl) return;
                const rect = btn.getBoundingClientRect();
                const right = window.innerWidth - rect.right;
                const top = rect.bottom + 6;
                popoverEl.style.top = top + 'px';
                popoverEl.style.right = Math.max(8, right) + 'px';
                popoverEl.style.left = 'auto';
            }}

            // Popover 1: Main Menu (Three dots)
            mainMenuEl = document.createElement('div');
            mainMenuEl.id = '__fynexo_main_menu__';
            mainMenuEl.className = 'fnx-nav-popover';
            mainMenuEl.style.minWidth = '225px';

            function addMenuItem(iconName, label, shortcut, onClick) {{
                const item = document.createElement('div');
                item.className = 'fnx-menu-item';
                const leftWrap = document.createElement('div');
                leftWrap.style.cssText = 'display: flex; align-items: center; gap: 8px;';
                if (iconName) {{
                    const icon = createIcon(iconName);
                    icon.style.cssText = 'width: 14px; height: 14px; opacity: 0.85; flex-shrink: 0;';
                    leftWrap.appendChild(icon);
                }}
                const labelSpan = document.createElement('span');
                labelSpan.textContent = label;
                leftWrap.appendChild(labelSpan);
                item.appendChild(leftWrap);
                if (shortcut) {{
                    const scSpan = document.createElement('span');
                    scSpan.style.cssText = 'font-size: 10.5px; color: var(--fnx-muted);';
                    scSpan.textContent = shortcut;
                    item.appendChild(scSpan);
                }}
                item.onclick = (e) => {{
                    e.stopPropagation();
                    closeAllNavPopovers();
                    onClick();
                }};
                mainMenuEl.appendChild(item);
                return item;
            }}

            function addMenuSep() {{
                const sep = document.createElement('div');
                sep.className = 'fnx-menu-sep';
                mainMenuEl.appendChild(sep);
            }}

            // Studio & Preferences
            addMenuItem('settings', 'Studio & Preferences', keybinds.preferences || 'Ctrl+,', () => {{
                togglePreferences();
            }});

            // Task Manager / Resource Monitor
            addMenuItem('cpu', 'Task Manager / Memory', '', () => {{
                openTaskMgrPopover();
            }});

            // Smart Force Dark (live toggle switch)
            const darkItem = document.createElement('div');
            darkItem.className = 'fnx-menu-item';
            const darkLeft = document.createElement('div');
            darkLeft.style.cssText = 'display: flex; align-items: center; gap: 8px;';
            const darkIcon = createIcon('moon');
            darkIcon.style.cssText = 'width: 14px; height: 14px; opacity: 0.85; flex-shrink: 0;';
            darkLeft.appendChild(darkIcon);
            const darkLabel = document.createElement('span');
            darkLabel.textContent = 'Smart Force Dark';
            darkLeft.appendChild(darkLabel);
            darkItem.appendChild(darkLeft);
            const darkBadge = document.createElement('span');
            darkBadge.className = 'fnx-chip';
            darkBadge.style.cssText = 'font-size: 10px; padding: 1px 6px;';
            darkBadge.textContent = config.force_dark ? 'ON' : 'OFF';
            if (config.force_dark) darkBadge.classList.add('active');
            darkItem.appendChild(darkBadge);
            darkItem.onclick = (e) => {{
                e.stopPropagation();
                config.force_dark = !config.force_dark;
                sendIpc({{ type: 'SAVE_CONFIG', config: config }});
                applyPageRules();
                syncBookmarkButtons();
                darkBadge.textContent = config.force_dark ? 'ON' : 'OFF';
                darkBadge.classList.toggle('active', !!config.force_dark);
                if (config.force_dark && isPageAlreadyDark()) {{
                    showHudToast('Smart Dark: Page is already dark (skipped)');
                }} else {{
                    showHudToast(config.force_dark ? 'Smart Force Dark enabled' : 'Smart Force Dark off');
                }}
            }};
            mainMenuEl.appendChild(darkItem);

            addMenuSep();

            // New Tab
            addMenuItem('plus', 'New tab', keybinds.new_tab || 'Ctrl+T', () => {{
                triggerProgress();
                sendIpc({{ type: 'NEW_TAB' }});
            }});

            // History Drawer
            addMenuItem('history', 'History drawer', '', () => {{
                openHistoryDrawer();
            }});

            // Shortcuts Bar
            addMenuItem('bookmarksBar', 'Shortcuts bar', '', () => {{
                const cur = (styles.topbar.bookmarks_bar || 'always').toLowerCase();
                styles.topbar.bookmarks_bar = cur === 'hidden' ? 'always' : 'hidden';
                syncAllStyles();
                sendIpc({{ type: 'SAVE_STYLES', styles: styles }});
                showHudToast(styles.topbar.bookmarks_bar === 'hidden' ? 'Shortcuts bar hidden' : 'Shortcuts bar visible');
            }});

            addMenuSep();

            // Zoom controls
            const zoomRow = document.createElement('div');
            zoomRow.style.cssText = 'display: flex; align-items: center; justify-content: space-between; padding: 5px 10px;';
            const zoomLabel = document.createElement('span');
            zoomLabel.style.cssText = 'font-size: 12px; color: var(--fnx-secondary);';
            zoomLabel.textContent = 'Zoom';
            zoomRow.appendChild(zoomLabel);
            const zoomBtns = document.createElement('div');
            zoomBtns.style.cssText = 'display: flex; align-items: center; gap: 4px;';
            const btnZoomOut = makeDesktopIconBtn('minus', `Zoom out (${{keybinds.zoom_out || 'Ctrl+-'}})`, (e) => {{
                e.stopPropagation();
                const cur = Number(config.default_zoom) || 1.0;
                const next = Math.max(0.5, Number((cur - 0.1).toFixed(1)));
                config.default_zoom = next;
                sendIpc({{ type: 'SET_ZOOM', zoom: next }});
                zoomValSpan.textContent = Math.round(next * 100) + '%';
            }});
            btnZoomOut.style.cssText = 'width: 24px; height: 24px; border-radius: 5px;';
            const zoomValSpan = document.createElement('span');
            zoomValSpan.style.cssText = 'font-size: 11.5px; min-width: 40px; text-align: center; font-weight: 500; cursor: pointer;';
            zoomValSpan.textContent = Math.round((Number(config.default_zoom) || 1.0) * 100) + '%';
            zoomValSpan.title = 'Click to reset zoom (100%)';
            zoomValSpan.onclick = (e) => {{
                e.stopPropagation();
                config.default_zoom = 1.0;
                sendIpc({{ type: 'SET_ZOOM', zoom: 1.0 }});
                zoomValSpan.textContent = '100%';
            }};
            const btnZoomIn = makeDesktopIconBtn('plus', `Zoom in (${{keybinds.zoom_in || 'Ctrl+='}})`, (e) => {{
                e.stopPropagation();
                const cur = Number(config.default_zoom) || 1.0;
                const next = Math.min(2.5, Number((cur + 0.1).toFixed(1)));
                config.default_zoom = next;
                sendIpc({{ type: 'SET_ZOOM', zoom: next }});
                zoomValSpan.textContent = Math.round(next * 100) + '%';
            }});
            btnZoomIn.style.cssText = 'width: 24px; height: 24px; border-radius: 5px;';
            const btnZoomFull = makeDesktopIconBtn('fullscreen', `Fullscreen (${{keybinds.fullscreen || 'F11'}})`, (e) => {{
                e.stopPropagation();
                closeAllNavPopovers();
                sendIpc({{ type: 'TOGGLE_FULLSCREEN' }});
            }});
            btnZoomFull.style.cssText = 'width: 24px; height: 24px; border-radius: 5px;';
            zoomBtns.appendChild(btnZoomOut);
            zoomBtns.appendChild(zoomValSpan);
            zoomBtns.appendChild(btnZoomIn);
            zoomBtns.appendChild(btnZoomFull);
            zoomRow.appendChild(zoomBtns);
            mainMenuEl.appendChild(zoomRow);

            addMenuSep();

            // Find in page
            addMenuItem('search', 'Find in page', keybinds.find_in_page || 'Ctrl+F', () => {{
                if (!isInternalSettings()) openFindBar();
            }});

            // Reader Mode
            addMenuItem('reader', 'Reader mode', keybinds.reader_mode || 'Alt+R', () => {{
                toggleReaderMode();
            }});

            // Downloads Folder
            addMenuItem('download', 'Open downloads folder', '', () => {{
                sendIpc({{ type: 'OPEN_DOWNLOADS_DIR' }});
            }});

            // Popover 2: Media Control & Audio Booster (>100% Volume)
            mediaPopoverEl = document.createElement('div');
            mediaPopoverEl.id = '__fynexo_media_popover__';
            mediaPopoverEl.className = 'fnx-nav-popover';
            mediaPopoverEl.style.cssText = 'min-width: 285px; max-width: 310px; padding: 12px; gap: 10px;';

            let currentMediaEl = null;
            let currentMediaTitle = '';
            let mediaGainValue = 100;
            let mediaAudioCtx = null;
            let mediaGainNode = null;
            let mediaCompressor = null;
            let mediaSourceNode = null;
            let mediaIsMuted = false;

            function attachMediaGain(mediaEl) {{
                if (!mediaEl || mediaEl.__fnx_gain_node) return mediaEl.__fnx_gain_node || null;
                try {{
                    const AudioCtxClass = window.AudioContext || window.webkitAudioContext;
                    if (!AudioCtxClass) return null;
                    if (!mediaAudioCtx) {{
                        mediaAudioCtx = new AudioCtxClass();
                    }}
                    if (mediaAudioCtx.state === 'suspended') {{
                        mediaAudioCtx.resume();
                    }}
                    mediaSourceNode = mediaAudioCtx.createMediaElementSource(mediaEl);
                    mediaGainNode = mediaAudioCtx.createGain();
                    mediaCompressor = mediaAudioCtx.createDynamicsCompressor();
                    mediaCompressor.threshold.setValueAtTime(-3, mediaAudioCtx.currentTime);
                    mediaCompressor.knee.setValueAtTime(24, mediaAudioCtx.currentTime);
                    mediaCompressor.ratio.setValueAtTime(12, mediaAudioCtx.currentTime);
                    mediaCompressor.attack.setValueAtTime(0.003, mediaAudioCtx.currentTime);
                    mediaCompressor.release.setValueAtTime(0.25, mediaAudioCtx.currentTime);

                    mediaSourceNode.connect(mediaGainNode);
                    mediaGainNode.connect(mediaCompressor);
                    mediaCompressor.connect(mediaAudioCtx.destination);
                    mediaEl.__fnx_gain_node = mediaGainNode;
                    return mediaGainNode;
                }} catch (_) {{
                    return null;
                }}
            }}

            function applyMediaVolume(val) {{
                mediaGainValue = Math.max(0, Math.min(300, Number(val) || 100));
                if (!currentMediaEl) return;
                if (mediaIsMuted) {{
                    currentMediaEl.muted = true;
                    if (mediaGainNode && mediaAudioCtx) {{
                        try {{ mediaGainNode.gain.setValueAtTime(0, mediaAudioCtx.currentTime); }} catch (_) {{}}
                    }}
                    return;
                }}
                currentMediaEl.muted = false;
                if (mediaGainValue <= 100) {{
                    currentMediaEl.volume = mediaGainValue / 100;
                    if (mediaGainNode && mediaAudioCtx) {{
                        try {{ mediaGainNode.gain.setValueAtTime(1.0, mediaAudioCtx.currentTime); }} catch (_) {{}}
                    }}
                }} else {{
                    currentMediaEl.volume = 1.0;
                    const g = attachMediaGain(currentMediaEl);
                    if (g && mediaAudioCtx) {{
                        try {{ g.gain.setValueAtTime(mediaGainValue / 100, mediaAudioCtx.currentTime); }} catch (_) {{}}
                    }}
                }}
            }}

            const mediaHeader = document.createElement('div');
            mediaHeader.style.cssText = 'display: flex; align-items: center; justify-content: space-between;';
            const mediaHeaderLeft = document.createElement('div');
            mediaHeaderLeft.style.cssText = 'display: flex; align-items: center; gap: 6px; font-weight: 600; font-size: 13px;';
            const mediaIconSpan = document.createElement('span');
            mediaIconSpan.textContent = 'ðŸŽµ';
            const mediaHeaderTxt = document.createElement('span');
            mediaHeaderTxt.textContent = 'Media Control';
            mediaHeaderLeft.appendChild(mediaIconSpan);
            mediaHeaderLeft.appendChild(mediaHeaderTxt);
            mediaHeader.appendChild(mediaHeaderLeft);
            const mediaStatusPill = document.createElement('span');
            mediaStatusPill.className = 'fnx-chip';
            mediaStatusPill.style.cssText = 'font-size: 10.5px; padding: 1px 7px; color: #34d399; border-color: rgba(52, 211, 153, 0.3);';
            mediaStatusPill.textContent = 'Playing';
            mediaHeader.appendChild(mediaStatusPill);
            mediaPopoverEl.appendChild(mediaHeader);

            const mediaCard = document.createElement('div');
            mediaCard.style.cssText = 'background: var(--fnx-surface); border: 1px solid var(--fnx-border); border-radius: 8px; padding: 8px 10px;';
            const mediaTitleEl = document.createElement('div');
            mediaTitleEl.style.cssText = 'font-weight: 500; font-size: 12.5px; white-space: nowrap; overflow: hidden; text-overflow: ellipsis; color: var(--fnx-fg);';
            mediaTitleEl.textContent = 'Media Player';
            const mediaDomainEl = document.createElement('div');
            mediaDomainEl.style.cssText = 'font-size: 11px; color: var(--fnx-muted); margin-top: 2px;';
            mediaDomainEl.textContent = window.location.hostname || 'fynexo';
            mediaCard.appendChild(mediaTitleEl);
            mediaCard.appendChild(mediaDomainEl);
            mediaPopoverEl.appendChild(mediaCard);

            const mediaControlsRow = document.createElement('div');
            mediaControlsRow.style.cssText = 'display: flex; align-items: center; justify-content: center; gap: 12px; padding: 2px 0;';
            const btnPlayPause = document.createElement('button');
            btnPlayPause.type = 'button';
            btnPlayPause.className = 'fnx-icon-btn';
            btnPlayPause.style.cssText = 'width: 38px; height: 38px; border-radius: 50%; background: var(--fnx-accent); color: #fff; display: flex; align-items: center; justify-content: center; cursor: pointer;';
            btnPlayPause.title = 'Play / Pause';
            btnPlayPause.appendChild(createIcon('pause'));
            btnPlayPause.onclick = (e) => {{
                e.stopPropagation();
                if (currentMediaEl) {{
                    if (currentMediaEl.paused) {{
                        currentMediaEl.play();
                    }} else {{
                        currentMediaEl.pause();
                    }}
                    syncMediaPopoverUI();
                }}
            }};
            mediaControlsRow.appendChild(btnPlayPause);
            mediaPopoverEl.appendChild(mediaControlsRow);

            const volSection = document.createElement('div');
            volSection.style.cssText = 'display: flex; flex-direction: column; gap: 6px; background: rgba(0, 0, 0, 0.15); padding: 8px 10px; border-radius: 8px;';
            const volHeader = document.createElement('div');
            volHeader.style.cssText = 'display: flex; align-items: center; justify-content: space-between; font-size: 12px;';
            const volHeaderLeft = document.createElement('div');
            volHeaderLeft.style.cssText = 'display: flex; align-items: center; gap: 6px;';
            const btnMute = makeDesktopIconBtn('volumeUp', 'Toggle Mute', (e) => {{
                e.stopPropagation();
                mediaIsMuted = !mediaIsMuted;
                applyMediaVolume(mediaGainValue);
                syncMediaPopoverUI();
            }});
            btnMute.style.cssText = 'width: 22px; height: 22px;';
            const volTextSpan = document.createElement('span');
            volTextSpan.textContent = 'Volume & Boost';
            volHeaderLeft.appendChild(btnMute);
            volHeaderLeft.appendChild(volTextSpan);
            volHeader.appendChild(volHeaderLeft);
            const volPercentLabel = document.createElement('span');
            volPercentLabel.style.cssText = 'font-weight: 700; color: var(--fnx-accent); font-size: 12px;';
            volPercentLabel.textContent = '100%';
            volHeader.appendChild(volPercentLabel);
            volSection.appendChild(volHeader);

            const volSlider = document.createElement('input');
            volSlider.type = 'range';
            volSlider.min = '0';
            volSlider.max = '300';
            volSlider.step = '5';
            volSlider.value = '100';
            volSlider.style.cssText = 'width: 100%; height: 5px; accent-color: var(--fnx-accent); cursor: pointer; margin: 4px 0;';
            volSlider.oninput = (e) => {{
                e.stopPropagation();
                mediaIsMuted = false;
                applyMediaVolume(volSlider.value);
                syncMediaPopoverUI();
            }};
            volSection.appendChild(volSlider);

            const presetsRow = document.createElement('div');
            presetsRow.style.cssText = 'display: flex; align-items: center; justify-content: space-between; gap: 3px;';
            const volPresets = [
                {{ val: 50, label: '50%' }},
                {{ val: 100, label: '100%' }},
                {{ val: 150, label: '150% ðŸ”¥' }},
                {{ val: 200, label: '200% ðŸ”¥' }},
                {{ val: 300, label: '300% âš¡' }}
            ];
            volPresets.forEach(p => {{
                const chip = document.createElement('button');
                chip.type = 'button';
                chip.className = 'fnx-chip';
                chip.style.cssText = 'font-size: 10.5px; padding: 2px 4px; flex: 1; justify-content: center; text-align: center;';
                chip.textContent = p.label;
                chip.onclick = (e) => {{
                    e.stopPropagation();
                    mediaIsMuted = false;
                    volSlider.value = String(p.val);
                    applyMediaVolume(p.val);
                    syncMediaPopoverUI();
                }};
                presetsRow.appendChild(chip);
            }});
            volSection.appendChild(presetsRow);
            mediaPopoverEl.appendChild(volSection);

            function syncMediaPopoverUI() {{
                const isPlaying = currentMediaEl && !currentMediaEl.paused && !currentMediaEl.ended;
                mediaStatusPill.textContent = isPlaying ? 'Playing' : 'Paused';
                mediaStatusPill.style.color = isPlaying ? '#34d399' : 'var(--fnx-muted)';
                mediaStatusPill.style.borderColor = isPlaying ? 'rgba(52, 211, 153, 0.3)' : 'var(--fnx-border)';

                btnPlayPause.textContent = '';
                btnPlayPause.appendChild(createIcon(isPlaying ? 'pause' : 'play'));

                btnMute.textContent = '';
                btnMute.appendChild(createIcon(mediaIsMuted ? 'volumeMute' : 'volumeUp'));

                if (mediaIsMuted) {{
                    volPercentLabel.textContent = 'Muted';
                    volPercentLabel.style.color = 'var(--fnx-muted)';
                }} else if (mediaGainValue > 100) {{
                    volPercentLabel.textContent = mediaGainValue + '% ðŸ”¥';
                    volPercentLabel.style.color = '#fb923c';
                }} else {{
                    volPercentLabel.textContent = mediaGainValue + '%';
                    volPercentLabel.style.color = 'var(--fnx-accent)';
                }}

                const pageH1 = document.querySelector('h1.ytd-watch-metadata, h1.title, h1');
                const t = (pageH1 && pageH1.textContent.trim()) || document.title || 'Media Player';
                mediaTitleEl.textContent = t;
                mediaDomainEl.textContent = window.location.hostname || 'fynexo';
            }}

            function findActiveMedia() {{
                const mediaEls = Array.from(document.querySelectorAll('video, audio'));
                for (let i = 0; i < mediaEls.length; i++) {{
                    const m = mediaEls[i];
                    if (!m.paused && !m.ended && m.currentTime > 0) {{
                        return m;
                    }}
                }}
                if (currentMediaEl && mediaEls.includes(currentMediaEl)) {{
                    return currentMediaEl;
                }}
                return mediaEls[0] || null;
            }}

            function updateMediaButtonState() {{
                if (isInternalAny()) {{
                    btnMedia.style.display = 'none';
                    return;
                }}
                const m = findActiveMedia();
                const isPlaying = m && !m.paused && !m.ended;
                if (m && (isPlaying || m.currentTime > 0)) {{
                    currentMediaEl = m;
                    btnMedia.style.display = 'flex';
                    mediaPulse.style.display = isPlaying ? 'inline-block' : 'none';
                }} else {{
                    btnMedia.style.display = 'none';
                }}
            }}

            document.addEventListener('play', () => requestAnimationFrame(updateMediaButtonState), true);
            document.addEventListener('pause', () => requestAnimationFrame(updateMediaButtonState), true);
            document.addEventListener('ended', () => requestAnimationFrame(updateMediaButtonState), true);
            document.addEventListener('timeupdate', () => {{
                if (btnMedia.style.display === 'none') updateMediaButtonState();
            }}, {{ passive: true, capture: true }});

            // Popover 3: Adblock & Privacy Shield
            shieldPopoverEl = document.createElement('div');
            shieldPopoverEl.id = '__fynexo_shield_popover__';
            shieldPopoverEl.className = 'fnx-nav-popover';
            shieldPopoverEl.style.cssText = 'min-width: 250px; max-width: 275px; padding: 12px; gap: 10px;';

            const shieldHeader = document.createElement('div');
            shieldHeader.style.cssText = 'display: flex; align-items: center; justify-content: space-between;';
            const shieldHeaderLeft = document.createElement('div');
            shieldHeaderLeft.style.cssText = 'display: flex; align-items: center; gap: 6px; font-weight: 600; font-size: 13px;';
            const shieldHIcon = document.createElement('span');
            shieldHIcon.textContent = 'ðŸ›¡ï¸';
            const shieldHTxt = document.createElement('span');
            shieldHTxt.textContent = 'Adblock Shield';
            shieldHeaderLeft.appendChild(shieldHIcon);
            shieldHeaderLeft.appendChild(shieldHTxt);
            shieldHeader.appendChild(shieldHeaderLeft);
            const shieldStatusPill = document.createElement('span');
            shieldStatusPill.className = 'fnx-chip';
            shieldStatusPill.style.cssText = 'font-size: 10.5px; padding: 1px 7px; color: #34d399; border-color: rgba(52, 211, 153, 0.3);';
            shieldStatusPill.textContent = config.adblock !== false ? 'Active' : 'Disabled';
            shieldHeader.appendChild(shieldStatusPill);
            shieldPopoverEl.appendChild(shieldHeader);

            const shieldCard = document.createElement('div');
            shieldCard.style.cssText = 'background: var(--fnx-surface); border: 1px solid var(--fnx-border); border-radius: 8px; padding: 10px; text-align: center;';
            const shieldCountNum = document.createElement('div');
            shieldCountNum.id = '__fynexo_shield_pop_count__';
            shieldCountNum.style.cssText = 'font-size: 26px; font-weight: 700; color: var(--fnx-accent);';
            shieldCountNum.textContent = '0';
            const shieldCountDesc = document.createElement('div');
            shieldCountDesc.style.cssText = 'font-size: 11.5px; color: var(--fnx-secondary); margin-top: 2px;';
            shieldCountDesc.textContent = 'Ads & Trackers Blocked';
            shieldCard.appendChild(shieldCountNum);
            shieldCard.appendChild(shieldCountDesc);
            shieldPopoverEl.appendChild(shieldCard);

            const shieldToggleRow = document.createElement('div');
            shieldToggleRow.style.cssText = 'display: flex; align-items: center; justify-content: space-between; padding: 2px 2px; font-size: 12.5px;';
            const shieldToggleLabel = document.createElement('span');
            shieldToggleLabel.textContent = 'Shield on this site';
            const shieldSwitch = document.createElement('input');
            shieldSwitch.type = 'checkbox';
            shieldSwitch.checked = config.adblock !== false;
            shieldSwitch.style.cssText = 'accent-color: var(--fnx-accent); cursor: pointer; width: 16px; height: 16px;';
            shieldSwitch.onchange = () => {{
                config.adblock = shieldSwitch.checked;
                sendIpc({{ type: 'SAVE_CONFIG', config: config }});
                applyPageRules();
                shieldStatusPill.textContent = config.adblock ? 'Active' : 'Disabled';
                shieldStatusPill.style.color = config.adblock ? '#34d399' : 'var(--fnx-muted)';
                showHudToast(config.adblock ? 'Adblock shield enabled' : 'Adblock shield paused');
            }};
            shieldToggleRow.appendChild(shieldToggleLabel);
            shieldToggleRow.appendChild(shieldSwitch);
            shieldPopoverEl.appendChild(shieldToggleRow);

            const btnOpenAdbStudio = document.createElement('button');
            btnOpenAdbStudio.type = 'button';
            btnOpenAdbStudio.className = 'fnx-chip';
            btnOpenAdbStudio.style.cssText = 'width: 100%; justify-content: center; padding: 6px 0; font-size: 11.5px;';
            btnOpenAdbStudio.textContent = 'Manage Rules & Whitelist';
            btnOpenAdbStudio.onclick = () => {{
                closeAllNavPopovers();
                sendIpc({{ type: 'NAVIGATE', url: 'fynexo://settings#adblock' }});
            }};
            shieldPopoverEl.appendChild(btnOpenAdbStudio);

            function updateShieldState() {{
                if (isInternalAny()) {{
                    shieldBadge.style.display = 'none';
                    return;
                }}
                const ads = document.querySelectorAll('ins.adsbygoogle, [id^="div-gpt-ad"], [id^="google_ads_iframe"], iframe[src*="doubleclick.net"], iframe[src*="googlesyndication.com"]');
                const count = ads.length;
                if (count > 0 && config.adblock !== false) {{
                    shieldBadge.textContent = count > 99 ? '99+' : String(count);
                    shieldBadge.style.display = 'flex';
                }} else {{
                    shieldBadge.style.display = 'none';
                }}
            }}

            // Popover 4: Downloads Tray
            downloadsPopoverEl = document.createElement('div');
            downloadsPopoverEl.id = '__fynexo_downloads_popover__';
            downloadsPopoverEl.className = 'fnx-nav-popover';
            downloadsPopoverEl.style.cssText = 'min-width: 280px; max-width: 320px; padding: 12px; gap: 8px;';

            let recentDownloads = [];

            const dlHeader = document.createElement('div');
            dlHeader.style.cssText = 'display: flex; align-items: center; justify-content: space-between;';
            const dlHeaderLeft = document.createElement('div');
            dlHeaderLeft.style.cssText = 'display: flex; align-items: center; gap: 6px; font-weight: 600; font-size: 13px;';
            const dlHIcon = document.createElement('span');
            dlHIcon.textContent = 'ðŸ“¥';
            const dlHTxt = document.createElement('span');
            dlHTxt.textContent = 'Downloads';
            dlHeaderLeft.appendChild(dlHIcon);
            dlHeaderLeft.appendChild(dlHTxt);
            dlHeader.appendChild(dlHeaderLeft);
            const btnOpenDlFolder = document.createElement('button');
            btnOpenDlFolder.type = 'button';
            btnOpenDlFolder.className = 'fnx-chip';
            btnOpenDlFolder.style.cssText = 'font-size: 11px; padding: 2px 7px;';
            btnOpenDlFolder.textContent = 'Open folder';
            btnOpenDlFolder.onclick = () => {{
                sendIpc({{ type: 'OPEN_DOWNLOADS_DIR' }});
            }};
            dlHeader.appendChild(btnOpenDlFolder);
            downloadsPopoverEl.appendChild(dlHeader);

            const dlListContainer = document.createElement('div');
            dlListContainer.id = '__fynexo_dl_list__';
            dlListContainer.style.cssText = 'display: flex; flex-direction: column; gap: 5px; max-height: 220px; overflow-y: auto;';
            downloadsPopoverEl.appendChild(dlListContainer);

            function renderDownloadsList() {{
                dlListContainer.textContent = '';
                if (recentDownloads.length === 0) {{
                    const emptyTip = document.createElement('div');
                    emptyTip.style.cssText = 'padding: 16px 8px; text-align: center; color: var(--fnx-muted); font-size: 12px;';
                    emptyTip.textContent = 'No downloads in this session';
                    dlListContainer.appendChild(emptyTip);
                    return;
                }}
                recentDownloads.forEach(dl => {{
                    const row = document.createElement('div');
                    row.className = 'fnx-menu-item';
                    row.style.cssText = 'padding: 6px 8px; border-radius: 6px; background: var(--fnx-surface); border: 1px solid var(--fnx-border);';
                    const rowLeft = document.createElement('div');
                    rowLeft.style.cssText = 'display: flex; align-items: center; gap: 8px; min-width: 0;';
                    const fileIcon = createIcon('download');
                    fileIcon.style.cssText = 'width: 14px; height: 14px; color: var(--fnx-accent); flex-shrink: 0;';
                    rowLeft.appendChild(fileIcon);
                    const nameWrap = document.createElement('div');
                    nameWrap.style.cssText = 'min-width: 0; display: flex; flex-direction: column;';
                    const nameSpan = document.createElement('span');
                    nameSpan.style.cssText = 'font-size: 12px; font-weight: 500; white-space: nowrap; overflow: hidden; text-overflow: ellipsis;';
                    nameSpan.textContent = dl.name;
                    const timeSpan = document.createElement('span');
                    timeSpan.style.cssText = 'font-size: 10px; color: var(--fnx-muted);';
                    timeSpan.textContent = dl.time + ' Â· ' + dl.status;
                    nameWrap.appendChild(nameSpan);
                    nameWrap.appendChild(timeSpan);
                    rowLeft.appendChild(nameWrap);
                    row.appendChild(rowLeft);
                    row.onclick = () => {{
                        sendIpc({{ type: 'OPEN_DOWNLOADS_DIR' }});
                    }};
                    dlListContainer.appendChild(row);
                }});
            }}
            renderDownloadsList();

            function handleDownloadToast(msg) {{
                if (!msg) return;
                const lower = msg.toLowerCase();
                if (lower.includes('download') || lower.includes('saved') || lower.includes('exported')) {{
                    const match = msg.match(/(?:Downloading|Saved|Exported)\s+(.+?)(?:\s+to\s+Downloads|$)/i);
                    const fname = match ? match[1] : 'File';
                    const now = new Date();
                    const timeStr = now.toLocaleTimeString([], {{ hour: '2-digit', minute: '2-digit' }});
                    recentDownloads.unshift({{
                        name: fname,
                        time: timeStr,
                        status: lower.includes('downloading') ? 'Downloading...' : 'Completed'
                    }});
                    if (recentDownloads.length > 8) recentDownloads.pop();
                    btnDownloads.style.display = 'flex';
                    dlBadge.textContent = String(recentDownloads.length);
                    dlBadge.style.display = 'flex';
                    renderDownloadsList();
                }}
            }}

            function toggleMainMenu() {{
                const isOpen = (mainMenuEl && mainMenuEl.style.display === 'flex') || (taskMgrPopoverEl && taskMgrPopoverEl.style.display === 'flex');
                if (isOpen) {{
                    closeAllNavPopovers();
                }} else {{
                    closeAllNavPopovers();
                    positionNavPopover(btnMore, mainMenuEl);
                    mainMenuEl.style.display = 'flex';
                    activeNavPopoverEl = mainMenuEl;
                }}
            }}

            function toggleMediaPopover() {{
                const isOpen = mediaPopoverEl && (mediaPopoverEl.style.display === 'flex' || activeNavPopoverEl === mediaPopoverEl);
                if (isOpen) {{
                    closeAllNavPopovers();
                }} else {{
                    closeAllNavPopovers();
                    syncMediaPopoverUI();
                    positionNavPopover(btnMedia, mediaPopoverEl);
                    mediaPopoverEl.style.display = 'flex';
                    activeNavPopoverEl = mediaPopoverEl;
                }}
            }}

            function toggleShieldPopover() {{
                const isOpen = shieldPopoverEl && (shieldPopoverEl.style.display === 'flex' || activeNavPopoverEl === shieldPopoverEl);
                if (isOpen) {{
                    closeAllNavPopovers();
                }} else {{
                    closeAllNavPopovers();
                    const ads = document.querySelectorAll('ins.adsbygoogle, [id^="div-gpt-ad"], [id^="google_ads_iframe"], iframe[src*="doubleclick.net"], iframe[src*="googlesyndication.com"]');
                    const c = shieldPopoverEl.querySelector('#__fynexo_shield_pop_count__');
                    if (c) c.textContent = String(ads.length);
                    positionNavPopover(btnShield, shieldPopoverEl);
                    shieldPopoverEl.style.display = 'flex';
                    activeNavPopoverEl = shieldPopoverEl;
                }}
            }}

            function toggleDownloadsPopover() {{
                const isOpen = downloadsPopoverEl && (downloadsPopoverEl.style.display === 'flex' || activeNavPopoverEl === downloadsPopoverEl);
                if (isOpen) {{
                    closeAllNavPopovers();
                }} else {{
                    closeAllNavPopovers();
                    renderDownloadsList();
                    positionNavPopover(btnDownloads, downloadsPopoverEl);
                    downloadsPopoverEl.style.display = 'flex';
                    activeNavPopoverEl = downloadsPopoverEl;
                }}
            }}

            shell.appendChild(mainMenuEl);
            shell.appendChild(mediaPopoverEl);
            shell.appendChild(shieldPopoverEl);
            shell.appendChild(downloadsPopoverEl);

            // Popover 5: Task Manager / Performance
            taskMgrPopoverEl = document.createElement('div');
            taskMgrPopoverEl.id = '__fynexo_taskmgr_popover__';
            taskMgrPopoverEl.className = 'fnx-nav-popover';
            taskMgrPopoverEl.style.cssText = 'min-width: 260px; padding: 14px; display: none; flex-direction: column; gap: 10px; font-family: inherit;';

            const tmHeader = document.createElement('div');
            tmHeader.style.cssText = 'display: flex; align-items: center; justify-content: space-between; border-bottom: 1px solid var(--fnx-border); padding-bottom: 8px;';
            const tmTitle = document.createElement('span');
            tmTitle.style.cssText = 'font-size: 13px; font-weight: 600; display: flex; align-items: center; gap: 6px;';
            const tmIcon = createIcon('cpu');
            tmIcon.style.cssText = 'width: 15px; height: 15px; color: var(--fnx-accent);';
            tmTitle.appendChild(tmIcon);
            tmTitle.appendChild(document.createTextNode('Resource Monitor'));
            tmHeader.appendChild(tmTitle);
            taskMgrPopoverEl.appendChild(tmHeader);

            const tmStatsWrap = document.createElement('div');
            tmStatsWrap.style.cssText = 'display: flex; flex-direction: column; gap: 7px; font-size: 12px;';
            function makeStatRow(labelText, valueText, valueId, valueStyle, valueTag) {{
                const row = document.createElement('div');
                row.style.cssText = 'display: flex; justify-content: space-between; color: var(--fnx-secondary);';
                const l = document.createElement('span');
                l.textContent = labelText;
                const v = document.createElement(valueTag || 'strong');
                if (valueId) v.id = valueId;
                v.style.cssText = valueStyle;
                v.textContent = valueText;
                row.appendChild(l);
                row.appendChild(v);
                return row;
            }}
            const tmHostRow = makeStatRow('Working Set RAM:', '-- MB', '__fynexo_tm_host__', 'color:var(--fnx-fg);');
            const tmCommitRow = makeStatRow('Commit Memory:', '-- MB', '__fynexo_tm_commit__', 'color:var(--fnx-fg);');
            const tmModeRow = makeStatRow('Renderer Model:', 'Single (Max 256MB)', '', 'color:#10b981;font-weight:500;', 'span');
            tmStatsWrap.appendChild(tmHostRow);
            tmStatsWrap.appendChild(tmCommitRow);
            tmStatsWrap.appendChild(tmModeRow);
            taskMgrPopoverEl.appendChild(tmStatsWrap);

            const btnTrimMem = document.createElement('button');
            btnTrimMem.type = 'button';
            btnTrimMem.className = 'fnx-chip';
            btnTrimMem.style.cssText = 'width: 100%; justify-content: center; padding: 7px 0; font-size: 11.5px; font-weight: 500; background: var(--fnx-accent); color: #000; cursor: pointer; border-radius: 6px; margin-top: 4px;';
            btnTrimMem.textContent = 'âš¡ Flush Working Set / Trim';
            btnTrimMem.onclick = () => {{
                sendIpc({{ type: 'FORCE_TRIM_MEMORY' }});
            }};
            taskMgrPopoverEl.appendChild(btnTrimMem);

            function openTaskMgrPopover() {{
                closeAllNavPopovers();
                positionNavPopover(btnMore, taskMgrPopoverEl);
                taskMgrPopoverEl.style.display = 'flex';
                activeNavPopoverEl = taskMgrPopoverEl;
                sendIpc({{ type: 'GET_RESOURCE_USAGE' }});
            }}

            window.__fynexo_on_resource_stats__ = function(stats) {{
                if (!stats) return;
                const h = taskMgrPopoverEl.querySelector('#__fynexo_tm_host__');
                const c = taskMgrPopoverEl.querySelector('#__fynexo_tm_commit__');
                if (h && stats.host_mb !== undefined) h.textContent = stats.host_mb + ' MB';
                if (c && stats.commit_mb !== undefined) c.textContent = stats.commit_mb + ' MB';
            }};

            shell.appendChild(taskMgrPopoverEl);

            // Tier 3: Slim Shortcuts / Bookmarks Bar (#__fynexo_bm_row__)
            const bmRow = document.createElement('div');
            bmRow.id = '__fynexo_bm_row__';

            const bmStartBtn = document.createElement('button');
            bmStartBtn.type = 'button';
            bmStartBtn.id = '__fynexo_bm_start__';
            bmStartBtn.className = 'fnx-bm-pill';
            bmStartBtn.title = 'Open Start page (fynexo://newtab)';
            const bmStartIcon = document.createElement('span');
            bmStartIcon.style.cssText = 'display: inline-flex; align-items: center; color: var(--fnx-secondary);';
            bmStartIcon.appendChild(createIcon('grid'));
            const bmStartText = document.createElement('span');
            bmStartText.textContent = 'Start';
            bmStartBtn.appendChild(bmStartIcon);
            bmStartBtn.appendChild(bmStartText);
            bmStartBtn.onclick = () => {{
                flushScrollSync();
                triggerProgress();
                sendIpc({{ type: 'NAVIGATE', url: 'fynexo://newtab' }});
            }};
            bmRow.appendChild(bmStartBtn);

            const bmSep = document.createElement('span');
            bmSep.style.cssText = 'width: 1px; height: 13px; background: var(--fnx-border); margin: 0 3px; flex-shrink: 0;';
            bmRow.appendChild(bmSep);

            const bmList = document.createElement('div');
            bmList.id = '__fynexo_bm_list__';
            bmRow.appendChild(bmList);

            const bmSepRight = document.createElement('span');
            bmSepRight.style.cssText = 'width: 1px; height: 13px; background: var(--fnx-border); margin: 0 3px; flex-shrink: 0;';
            bmRow.appendChild(bmSepRight);

            const bmAddBtn = document.createElement('button');
            bmAddBtn.type = 'button';
            bmAddBtn.id = '__fynexo_bm_add__';
            bmAddBtn.className = 'fnx-bm-pill';
            bmAddBtn.style.color = 'var(--fnx-muted)';
            bmAddBtn.textContent = '+ Save page';
            bmAddBtn.onclick = () => {{
                if (isInternalAny()) {{
                    togglePreferences();
                }} else {{
                    toggleCurrentBookmark();
                }}
            }};
            bmRow.appendChild(bmAddBtn);

            topbar.appendChild(bmRow);
            shell.appendChild(topbar);

            // 1B. Find in Page Bar
            const findBar = document.createElement('div');
            findBar.id = '__fynexo_find_bar__';
            const findIcon = document.createElement('span');
            findIcon.style.cssText = 'display:flex;align-items:center;color:var(--fnx-muted);flex-shrink:0;';
            findIcon.appendChild(createIcon('search'));
            const findInput = document.createElement('input');
            findInput.id = '__fynexo_find_input__';
            findInput.type = 'text';
            findInput.placeholder = 'Find in page...';
            findInput.spellcheck = false;
            findInput.autocomplete = 'off';
            const findCount = document.createElement('span');
            findCount.id = '__fynexo_find_count__';
            findCount.textContent = '0/0';

            const btnFindPrev = document.createElement('button');
            btnFindPrev.type = 'button';
            btnFindPrev.className = 'fnx-find-btn';
            btnFindPrev.title = 'Previous match (Shift+Enter)';
            btnFindPrev.appendChild(createIcon('chevUp'));
            btnFindPrev.onclick = () => stepFindMatch(-1);

            const btnFindNext = document.createElement('button');
            btnFindNext.type = 'button';
            btnFindNext.className = 'fnx-find-btn';
            btnFindNext.title = 'Next match (Enter)';
            btnFindNext.appendChild(createIcon('chevDown'));
            btnFindNext.onclick = () => stepFindMatch(1);

            const btnFindClose = document.createElement('button');
            btnFindClose.type = 'button';
            btnFindClose.className = 'fnx-find-btn';
            btnFindClose.title = 'Close (Esc)';
            btnFindClose.appendChild(createIcon('closeSmall'));
            btnFindClose.onclick = () => closeFindBar();

            findInput.addEventListener('input', () => runFindQuery(findInput.value));
            findInput.addEventListener('keydown', (e) => {{
                if (e.key === 'Enter') {{
                    e.preventDefault();
                    stepFindMatch(e.shiftKey ? -1 : 1);
                }} else if (e.key === 'Escape') {{
                    e.preventDefault();
                    closeFindBar();
                }}
            }});

            findBar.appendChild(findIcon);
            findBar.appendChild(findInput);
            findBar.appendChild(findCount);
            findBar.appendChild(btnFindPrev);
            findBar.appendChild(btnFindNext);
            findBar.appendChild(btnFindClose);
            shell.appendChild(findBar);

            // 1C. Pure-DOM Reader Mode Overlay
            const readerOverlay = document.createElement('div');
            readerOverlay.id = '__fynexo_reader__';
            const readerBar = document.createElement('div');
            readerBar.className = 'fnx-rd-toolbar';
            const readerMeta = document.createElement('span');
            readerMeta.id = '__fynexo_reader_meta__';
            readerMeta.className = 'fnx-rd-meta';
            readerMeta.textContent = 'Reader mode';

            const readerCtrls = document.createElement('div');
            readerCtrls.className = 'fnx-rd-controls';

            const btnRdSmaller = document.createElement('button');
            btnRdSmaller.type = 'button';
            btnRdSmaller.className = 'fnx-rd-btn';
            btnRdSmaller.textContent = 'Aâˆ’';
            btnRdSmaller.onclick = () => {{
                readerFontPx = Math.max(14, readerFontPx - 2);
                syncReaderTypography();
            }};

            const btnRdLarger = document.createElement('button');
            btnRdLarger.type = 'button';
            btnRdLarger.className = 'fnx-rd-btn';
            btnRdLarger.textContent = 'A+';
            btnRdLarger.onclick = () => {{
                readerFontPx = Math.min(26, readerFontPx + 2);
                syncReaderTypography();
            }};

            const btnRdWidth = document.createElement('button');
            btnRdWidth.type = 'button';
            btnRdWidth.id = '__fynexo_reader_width__';
            btnRdWidth.className = 'fnx-rd-btn';
            btnRdWidth.textContent = 'Wide';
            btnRdWidth.onclick = () => {{
                readerWide = !readerWide;
                syncReaderTypography();
            }};

            const btnRdExit = document.createElement('button');
            btnRdExit.type = 'button';
            btnRdExit.className = 'fnx-rd-btn';
            btnRdExit.textContent = 'Exit Reader';
            btnRdExit.onclick = () => closeReaderMode();

            readerCtrls.appendChild(btnRdSmaller);
            readerCtrls.appendChild(btnRdLarger);
            readerCtrls.appendChild(btnRdWidth);
            readerCtrls.appendChild(btnRdExit);

            readerBar.appendChild(readerMeta);
            readerBar.appendChild(readerCtrls);

            const readerArticle = document.createElement('article');
            readerArticle.id = '__fynexo_reader_article__';

            readerOverlay.appendChild(readerBar);
            readerOverlay.appendChild(readerArticle);
            shell.appendChild(readerOverlay);

            // 2. Studio Mobile Bottom Dock
            const mobileBar = document.createElement('div');
            mobileBar.id = '__fynexo_bottom_bar__';

            const mbBack = document.createElement('button');
            mbBack.className = 'fnx-mb-btn';
            mbBack.setAttribute('aria-label', 'Back');
            mbBack.appendChild(createIcon('back'));
            mbBack.onclick = () => {{ triggerProgress(); sendIpc({{ type: 'GO_BACK' }}); }};

            const mbPill = document.createElement('div');
            mbPill.id = '__fynexo_mb_pill__';

            const mbPillTrigger = document.createElement('button');
            mbPillTrigger.id = '__fynexo_mb_pill_trigger__';
            const mbProtoIcon = document.createElement('span');
            mbProtoIcon.id = '__fynexo_mb_proto__';
            mbProtoIcon.style.cssText = 'display: flex; align-items: center; flex-shrink: 0;';
            const mbLabel = document.createElement('span');
            mbLabel.id = '__fynexo_mb_label__';
            mbLabel.textContent = formatHostLabel(window.location.href);
            mbPillTrigger.appendChild(mbProtoIcon);
            mbPillTrigger.appendChild(mbLabel);
            mbPillTrigger.onclick = () => openHud();

            const mbReload = document.createElement('button');
            mbReload.id = '__fynexo_mb_reload__';
            mbReload.setAttribute('aria-label', 'Reload');
            mbReload.appendChild(createIcon('reload'));
            mbReload.onclick = (e) => {{
                e.stopPropagation();
                triggerProgress();
                sendIpc({{ type: 'RELOAD' }});
            }};

            mbPill.appendChild(mbPillTrigger);
            mbPill.appendChild(mbReload);

            const mbTabs = document.createElement('button');
            mbTabs.className = 'fnx-mb-btn';
            mbTabs.setAttribute('aria-label', 'Tabs');
            const tabBadge = document.createElement('div');
            tabBadge.className = 'fnx-tab-badge';
            const tabCountSpan = document.createElement('span');
            tabCountSpan.id = '__fynexo_mobile_tab_count__';
            tabCountSpan.textContent = '1';
            tabBadge.appendChild(tabCountSpan);
            mbTabs.appendChild(tabBadge);
            mbTabs.onclick = () => openMobileDrawer();

            const mbPrefs = document.createElement('button');
            mbPrefs.id = '__fynexo_mb_prefs__';
            mbPrefs.className = 'fnx-mb-btn';
            mbPrefs.setAttribute('aria-label', 'Preferences');
            mbPrefs.appendChild(createIcon('settings'));
            mbPrefs.onclick = () => togglePreferences();

            mobileBar.appendChild(mbBack);
            mobileBar.appendChild(mbPill);
            mobileBar.appendChild(mbTabs);
            mobileBar.appendChild(mbPrefs);
            shell.appendChild(mobileBar);

            // 3. Thumb-First Mobile Tab Switcher Sheet
            const mobileDrawer = document.createElement('div');
            mobileDrawer.id = '__fynexo_mobile_drawer__';

            const drawerSheet = document.createElement('div');
            drawerSheet.id = '__fynexo_drawer_sheet__';

            const sheetHandle = document.createElement('div');
            sheetHandle.className = 'fnx-sheet-handle';
            drawerSheet.appendChild(sheetHandle);

            const drawerHeader = document.createElement('div');
            drawerHeader.style.cssText = 'display: flex; align-items: center; justify-content: space-between; padding-bottom: 10px; margin-bottom: 8px; gap: 8px;';
            const drawerTitle = document.createElement('span');
            drawerTitle.style.cssText = 'font-weight: 600; font-size: 15px; letter-spacing: -0.015em; color: var(--fnx-fg);';
            drawerTitle.textContent = 'Open tabs';

            const drawerHeadRight = document.createElement('div');
            drawerHeadRight.style.cssText = 'display: flex; align-items: center; gap: 8px;';

            const btnDrawerReopen = document.createElement('button');
            btnDrawerReopen.type = 'button';
            btnDrawerReopen.id = '__fynexo_drawer_reopen__';
            btnDrawerReopen.style.cssText = 'display: none; padding: 3px 9px; font-size: 11.5px; font-weight: 500; border-radius: 6px; border: 1px solid var(--fnx-border); background: var(--fnx-surface); color: var(--fnx-fg); cursor: pointer;';
            btnDrawerReopen.textContent = 'â†º Reopen';
            btnDrawerReopen.onclick = () => {{
                closeMobileDrawer();
                triggerProgress();
                sendIpc({{ type: 'REOPEN_CLOSED_TAB' }});
            }};

            const drawerCount = document.createElement('span');
            drawerCount.id = '__fynexo_drawer_count__';
            drawerCount.style.cssText = 'font-size: 12px; color: var(--fnx-muted);';
            drawerCount.textContent = '1 tab';

            drawerHeadRight.appendChild(btnDrawerReopen);
            drawerHeadRight.appendChild(drawerCount);
            drawerHeader.appendChild(drawerTitle);
            drawerHeader.appendChild(drawerHeadRight);

            const drawerList = document.createElement('div');
            drawerList.id = '__fynexo_drawer_list__';
            drawerList.style.cssText = 'overflow-y: auto; flex: 1; max-height: 56vh;';

            const sheetBottomBar = document.createElement('div');
            sheetBottomBar.className = 'fnx-sheet-bottom-bar';

            const btnSheetFwd = document.createElement('button');
            btnSheetFwd.style.cssText = 'height: 38px; width: 42px; border: 1px solid var(--fnx-border); background: var(--fnx-surface); color: var(--fnx-secondary); border-radius: 9px; display: flex; align-items: center; justify-content: center; cursor: pointer; flex-shrink: 0;';
            btnSheetFwd.title = 'Forward';
            btnSheetFwd.appendChild(createIcon('forward'));
            btnSheetFwd.onclick = () => {{
                closeMobileDrawer();
                triggerProgress();
                sendIpc({{ type: 'GO_FORWARD' }});
            }};

            const btnSheetStar = document.createElement('button');
            btnSheetStar.id = '__fynexo_sheet_star__';
            btnSheetStar.dataset.starKind = 'star';
            btnSheetStar.style.cssText = 'height: 38px; width: 42px; border: 1px solid var(--fnx-border); background: var(--fnx-surface); color: var(--fnx-secondary); border-radius: 9px; display: flex; align-items: center; justify-content: center; cursor: pointer; flex-shrink: 0;';
            btnSheetStar.title = 'Bookmark page';
            btnSheetStar.appendChild(createIcon('star'));
            btnSheetStar.onclick = () => toggleCurrentBookmark();

            const btnDrawerNew = document.createElement('button');
            btnDrawerNew.style.cssText = 'flex: 1; height: 38px; padding: 0 14px; font-size: 13.5px; font-weight: 500; background: var(--fnx-accent); border: none; color: #ffffff; border-radius: 9px; cursor: pointer; display: flex; align-items: center; justify-content: center; gap: 6px;';
            btnDrawerNew.appendChild(createIcon('plus'));
            btnDrawerNew.appendChild(document.createTextNode('New tab'));
            btnDrawerNew.onclick = () => {{
                closeMobileDrawer();
                flushScrollSync();
                triggerProgress();
                sendIpc({{ type: 'NEW_TAB' }});
            }};

            const btnDrawerDone = document.createElement('button');
            btnDrawerDone.textContent = 'Done';
            btnDrawerDone.style.cssText = 'height: 38px; padding: 0 15px; font-size: 13px; font-weight: 500; background: var(--fnx-surface); border: 1px solid var(--fnx-border); color: var(--fnx-fg); border-radius: 9px; cursor: pointer; flex-shrink: 0;';
            btnDrawerDone.onclick = () => closeMobileDrawer();

            sheetBottomBar.appendChild(btnSheetFwd);
            sheetBottomBar.appendChild(btnSheetStar);
            sheetBottomBar.appendChild(btnDrawerNew);
            sheetBottomBar.appendChild(btnDrawerDone);

            drawerSheet.appendChild(drawerHeader);
            drawerSheet.appendChild(drawerList);
            drawerSheet.appendChild(sheetBottomBar);
            mobileDrawer.appendChild(drawerSheet);
            mobileDrawer.onclick = (e) => {{
                if (e.target === mobileDrawer) closeMobileDrawer();
            }};
            shell.appendChild(mobileDrawer);

            // 3.5 Side Drawer / Web Panel (Alt+S)
            isSidePanelOpen = false;
            sidePanelEl = document.createElement('div');
            sidePanelEl.id = '__fynexo_side_panel__';
            sidePanelEl.style.cssText = `
                position: fixed;
                right: 0;
                top: var(--fnx-host-top-offset, 38px);
                width: 380px;
                max-width: 90vw;
                height: calc(100vh - var(--fnx-host-top-offset, 38px));
                background: var(--fnx-bg);
                border-left: 1px solid var(--fnx-border);
                box-shadow: -6px 0 24px rgba(0, 0, 0, 0.45);
                z-index: 2147483640;
                display: flex;
                flex-direction: column;
                transform: translateX(100%);
                transition: transform 0.22s cubic-bezier(0.16, 1, 0.3, 1);
                font-family: inherit;
            `;

            const spHeader = document.createElement('div');
            spHeader.style.cssText = 'display: flex; align-items: center; justify-content: space-between; padding: 8px 12px; border-bottom: 1px solid var(--fnx-border); background: var(--fnx-surface); gap: 8px; flex-shrink: 0;';

            const spTabsWrap = document.createElement('div');
            spTabsWrap.style.cssText = 'display: flex; align-items: center; gap: 4px;';
            const btnSpWeb = document.createElement('button');
            btnSpWeb.className = 'fnx-chip active';
            btnSpWeb.textContent = 'ðŸŒ Web Panel';
            btnSpWeb.style.cssText = 'font-size: 11px; padding: 3px 8px; cursor: pointer;';
            const btnSpNotes = document.createElement('button');
            btnSpNotes.className = 'fnx-chip';
            btnSpNotes.textContent = 'ðŸ“ Quick Notes';
            btnSpNotes.style.cssText = 'font-size: 11px; padding: 3px 8px; cursor: pointer;';
            spTabsWrap.appendChild(btnSpWeb);
            spTabsWrap.appendChild(btnSpNotes);
            spHeader.appendChild(spTabsWrap);

            const btnSpClose = document.createElement('button');
            btnSpClose.appendChild(createIcon('closeSmall'));
            btnSpClose.style.cssText = 'background: transparent; border: none; color: var(--fnx-muted); cursor: pointer; padding: 4px; display: flex; align-items: center; justify-content: center; border-radius: 4px;';
            btnSpClose.onclick = () => toggleSidePanel(false);
            spHeader.appendChild(btnSpClose);
            sidePanelEl.appendChild(spHeader);

            const spBodyWeb = document.createElement('div');
            spBodyWeb.style.cssText = 'display: flex; flex-direction: column; flex: 1; min-height: 0;';
            const spUrlBar = document.createElement('div');
            spUrlBar.style.cssText = 'display: flex; align-items: center; gap: 6px; padding: 6px 10px; border-bottom: 1px solid var(--fnx-border); background: var(--fnx-panel);';
            const spInput = document.createElement('input');
            spInput.type = 'text';
            spInput.placeholder = 'Enter URL for side panel...';
            spInput.style.cssText = 'flex: 1; height: 26px; font-size: 11.5px; border-radius: 5px; border: 1px solid var(--fnx-border); background: var(--fnx-bg); color: var(--fnx-fg); padding: 0 8px; outline: none;';
            const spGoBtn = document.createElement('button');
            spGoBtn.className = 'fnx-chip';
            spGoBtn.style.cssText = 'font-size: 11px; padding: 3px 8px; cursor: pointer;';
            spGoBtn.textContent = 'Go';
            spUrlBar.appendChild(spInput);
            spUrlBar.appendChild(spGoBtn);
            spBodyWeb.appendChild(spUrlBar);

            const spIframe = document.createElement('iframe');
            spIframe.style.cssText = 'flex: 1; width: 100%; border: none; background: #000;';
            spIframe.src = 'about:blank';
            spBodyWeb.appendChild(spIframe);

            function loadSpUrl(url) {{
                if (!url) return;
                let target = url.trim();
                if (!target.startsWith('http://') && !target.startsWith('https://')) {{
                    target = 'https://' + target;
                }}
                spInput.value = target;
                spIframe.src = target;
            }}
            spGoBtn.onclick = () => loadSpUrl(spInput.value);
            spInput.onkeydown = (e) => {{
                if (e.key === 'Enter') loadSpUrl(spInput.value);
            }};

            const spBodyNotes = document.createElement('div');
            spBodyNotes.style.cssText = 'display: none; flex-direction: column; flex: 1; padding: 10px; gap: 8px;';
            const spTextarea = document.createElement('textarea');
            spTextarea.placeholder = 'Type quick notes, scratchpad, code snippets here... Auto-saved locally.';
            spTextarea.style.cssText = 'flex: 1; width: 100%; border: 1px solid var(--fnx-border); border-radius: 6px; background: var(--fnx-panel); color: var(--fnx-fg); font-family: "JetBrains Mono", Consolas, monospace; font-size: 12px; padding: 8px; resize: none; outline: none; box-sizing: border-box;';
            try {{
                spTextarea.value = window.localStorage.getItem('__fnx_side_notes__') || '';
            }} catch (_) {{}}
            spTextarea.oninput = () => {{
                try {{
                    window.localStorage.setItem('__fnx_side_notes__', spTextarea.value);
                }} catch (_) {{}}
            }};
            spBodyNotes.appendChild(spTextarea);

            btnSpWeb.onclick = () => {{
                btnSpWeb.classList.add('active');
                btnSpNotes.classList.remove('active');
                spBodyWeb.style.display = 'flex';
                spBodyNotes.style.display = 'none';
            }};
            btnSpNotes.onclick = () => {{
                btnSpNotes.classList.add('active');
                btnSpWeb.classList.remove('active');
                spBodyWeb.style.display = 'none';
                spBodyNotes.style.display = 'flex';
            }};

            sidePanelEl.appendChild(spBodyWeb);
            sidePanelEl.appendChild(spBodyNotes);
            shell.appendChild(sidePanelEl);

            function toggleSidePanel(force) {{
                const next = force !== undefined ? !!force : !isSidePanelOpen;
                isSidePanelOpen = next;
                sidePanelEl.style.transform = next ? 'translateX(0)' : 'translateX(100%)';
                if (typeof btnSidePanel !== 'undefined' && btnSidePanel) {{
                    btnSidePanel.style.color = next ? 'var(--fnx-accent)' : 'inherit';
                }}
                if (next && spBodyWeb.style.display !== 'none' && spIframe.src === 'about:blank') {{
                    loadSpUrl('https://translate.google.com');
                }}
            }}

            // 4. Lazy-Input Omnibox System (0 <input> mounted at idle; swaps button -> <input> ONLY on click/Ctrl+L)
            const hudRoot = document.createElement('div');
            hudRoot.id = '__fynexo_hud_root__';

            const hudCard = document.createElement('div');
            hudCard.id = '__fynexo_hud_card__';

            const modalBar = document.createElement('div');
            modalBar.style.cssText = 'display: flex; align-items: center; padding: 10px 11px; gap: 9px;';

            const hudSuggest = document.createElement('div');
            hudSuggest.id = '__fynexo_hud_suggest__';

            const hudQuick = document.createElement('div');
            hudQuick.id = '__fynexo_hud_quick__';

            hudCard.appendChild(modalBar);
            hudCard.appendChild(hudSuggest);
            hudCard.appendChild(hudQuick);
            hudRoot.appendChild(hudCard);
            shell.appendChild(hudRoot);
            shadowRoot.appendChild(shell);

            function positionInlineDropdown(dropdownEl) {{
                if (!dropdownEl) return;
                const rect = addrSlot.getBoundingClientRect();
                const vw = window.innerWidth || 1024;
                const isDeck = ((styles.topbar && styles.topbar.layout) || 'deck').toLowerCase() !== 'compact';
                const dropW = isDeck
                    ? Math.min(Math.max(rect.width, 440), vw - 16)
                    : Math.min(Math.max(rect.width, 440), vw - 16);
                let leftPx = rect.left;
                if (!isDeck) {{
                    const pos = (styles.address_bar.position || 'right').toLowerCase();
                    if (pos === 'right') {{
                        leftPx = rect.right - dropW;
                    }} else if (pos === 'center') {{
                        leftPx = rect.left + (rect.width - dropW) / 2;
                    }}
                }}
                leftPx = Math.max(8, Math.min(leftPx, vw - dropW - 8));
                dropdownEl.style.width = Math.round(dropW) + 'px';
                dropdownEl.style.left = Math.round(leftPx) + 'px';
                dropdownEl.style.top = Math.round(rect.bottom + 4) + 'px';
            }}

            function deactivateInlineAddressBar() {{
                if (!isInlineActive) return;
                isInlineActive = false;
                const box = shadowRoot.getElementById('__fynexo_inline_box__');
                if (box) box.remove();
                const drop = shadowRoot.getElementById('__fynexo_inline_dropdown__');
                if (drop) drop.remove();
                addrSlot.classList.remove('editing');
                addrBtn.style.display = 'flex';
                suggestItems = [];
                suggestIndex = -1;
            }}

            function deactivateModalHud() {{
                if (!isModalActive && hudRoot.style.display !== 'flex') return;
                isModalActive = false;
                hudRoot.style.display = 'none';
                modalBar.textContent = '';
                hudSuggest.textContent = '';
                hudSuggest.style.display = 'none';
                hudQuick.textContent = '';
                suggestItems = [];
                suggestIndex = -1;
            }}

            function closeAnyOmnibox() {{
                deactivateInlineAddressBar();
                deactivateModalHud();
            }}
            closeOmniboxRef = closeAnyOmnibox;

            hudRoot.onclick = (e) => {{
                if (e.target === hudRoot) deactivateModalHud();
            }};

            function activateInlineAddressBar() {{
                if (isModalActive) deactivateModalHud();
                const existingInp = shadowRoot.getElementById('__fynexo_inline_input__');
                if (isInlineActive && existingInp) {{
                    existingInp.focus();
                    if (existingInp.value) existingInp.select();
                    return;
                }}

                isInlineActive = true;
                addrSlot.classList.add('editing');
                addrBtn.style.display = 'none';

                const box = document.createElement('div');
                box.id = '__fynexo_inline_box__';

                const engBtn = document.createElement('button');
                engBtn.type = 'button';
                engBtn.id = '__fynexo_inline_engine__';
                engBtn.title = 'Click to cycle search engine';
                engBtn.textContent = currentEngineObj().name || 'Google';
                engBtn.onmousedown = (e) => e.preventDefault();

                const inp = document.createElement('input');
                inp.id = '__fynexo_inline_input__';
                inp.type = 'text';
                inp.spellcheck = false;
                inp.autocomplete = 'off';
                inp.setAttribute('autocapitalize', 'off');
                inp.setAttribute('enterkeyhint', 'go');
                inp.placeholder = `Search ${{currentEngineObj().name}} or enter !bang / URL...`;

                const btnClear = document.createElement('button');
                btnClear.type = 'button';
                btnClear.className = 'fnx-inline-clear';
                btnClear.title = 'Clear';
                btnClear.appendChild(createIcon('clear'));
                btnClear.onmousedown = (e) => e.preventDefault();

                const href = canonicalCurrentUrl();
                if (!isInternalAny(href) && !href.startsWith('data:') && href !== 'about:blank') {{
                    inp.value = href;
                }}

                const syncClear = () => {{
                    btnClear.style.display = inp.value.length > 0 ? 'flex' : 'none';
                }};
                syncClear();

                box.appendChild(engBtn);
                box.appendChild(inp);
                box.appendChild(btnClear);
                addrSlot.insertBefore(box, addrActions);

                const dropdown = document.createElement('div');
                dropdown.id = '__fynexo_inline_dropdown__';
                const inlineSuggest = document.createElement('div');
                inlineSuggest.id = '__fynexo_inline_suggest__';
                const inlineQuick = document.createElement('div');
                inlineQuick.id = '__fynexo_inline_quick__';
                dropdown.appendChild(inlineSuggest);
                dropdown.appendChild(inlineQuick);
                shell.appendChild(dropdown);

                positionInlineDropdown(dropdown);
                renderHudQuickBar(inlineQuick, deactivateInlineAddressBar);
                renderOmniboxSuggestions('', inlineSuggest, inp, deactivateInlineAddressBar);

                const submitInline = () => {{
                    const target = resolveUrl(inp.value, config.search_engine);
                    deactivateInlineAddressBar();
                    if (target) {{
                        flushScrollSync();
                        triggerProgress();
                        sendIpc({{ type: 'NAVIGATE', url: target }});
                    }}
                }};

                inp.addEventListener('input', () => {{
                    syncClear();
                    renderOmniboxSuggestions(inp.value, inlineSuggest, inp, deactivateInlineAddressBar);
                }});

                btnClear.onclick = () => {{
                    inp.value = '';
                    syncClear();
                    renderOmniboxSuggestions('', inlineSuggest, inp, deactivateInlineAddressBar);
                    inp.focus();
                }};

                engBtn.onclick = () => {{
                    const list = getEngines();
                    const curIdx = list.findIndex(e => e.id === config.search_engine);
                    const next = list[(curIdx + 1) % list.length];
                    if (next) {{
                        config.search_engine = next.id;
                        engBtn.textContent = next.name;
                        inp.placeholder = `Search ${{next.name}} or enter !bang / URL...`;
                        updateAddressLabels();
                        sendIpc({{ type: 'SAVE_CONFIG', config: config }});
                    }}
                    inp.focus();
                }};

                bindOmniboxKeydown(inp, inlineSuggest, submitInline, deactivateInlineAddressBar);

                requestAnimationFrame(() => {{
                    positionInlineDropdown(dropdown);
                    inp.focus();
                    if (inp.value) inp.select();
                }});
            }}

            function openModalHud() {{
                if (isInlineActive) deactivateInlineAddressBar();
                const existingInp = shadowRoot.getElementById('__fynexo_input__');
                if (isModalActive && existingInp) {{
                    existingInp.focus();
                    if (existingInp.value) existingInp.select();
                    return;
                }}

                isModalActive = true;
                modalBar.textContent = '';

                const engineBadge = document.createElement('button');
                engineBadge.type = 'button';
                engineBadge.id = '__fynexo_hud_engine__';
                engineBadge.title = 'Click to switch search engine';
                engineBadge.textContent = currentEngineObj().name || 'Google';
                engineBadge.style.cssText = 'font-size: 11.5px; font-weight: 500; color: var(--fnx-fg); background: var(--fnx-surface); border: 1px solid var(--fnx-border-hover); padding: 5px 9px; border-radius: 8px; cursor: pointer; white-space: nowrap; flex-shrink: 0;';
                engineBadge.onmousedown = (e) => e.preventDefault();

                const input = document.createElement('input');
                input.id = '__fynexo_input__';
                input.type = 'text';
                input.spellcheck = false;
                input.autocomplete = 'off';
                input.setAttribute('autocapitalize', 'off');
                input.setAttribute('enterkeyhint', 'go');
                input.placeholder = `Search ${{currentEngineObj().name}} or enter !bang / URL...`;
                input.style.cssText = 'flex: 1; min-width: 0; background: transparent; border: none; outline: none; color: var(--fnx-fg); letter-spacing: -0.01em; user-select: auto;';

                const btnClear = document.createElement('button');
                btnClear.type = 'button';
                btnClear.title = 'Clear';
                btnClear.style.cssText = 'width: 26px; height: 26px; border: none; background: transparent; display: none; align-items: center; justify-content: center; cursor: pointer; flex-shrink: 0; opacity: 0.85;';
                btnClear.appendChild(createIcon('clear'));
                btnClear.onmousedown = (e) => e.preventDefault();

                const btnGo = document.createElement('button');
                btnGo.type = 'button';
                btnGo.title = 'Go';
                btnGo.style.cssText = 'width: 32px; height: 30px; border: none; background: var(--fnx-accent); color: #ffffff; border-radius: 8px; display: flex; align-items: center; justify-content: center; cursor: pointer; flex-shrink: 0;';
                btnGo.appendChild(createIcon('go'));
                btnGo.onmousedown = (e) => e.preventDefault();

                const href = canonicalCurrentUrl();
                if (!isInternalAny(href) && !href.startsWith('data:') && href !== 'about:blank') {{
                    input.value = href;
                }}

                const syncClearVisibility = () => {{
                    btnClear.style.display = input.value.length > 0 ? 'flex' : 'none';
                }};
                syncClearVisibility();

                const submitModal = () => {{
                    const target = resolveUrl(input.value, config.search_engine);
                    deactivateModalHud();
                    if (target) {{
                        flushScrollSync();
                        triggerProgress();
                        sendIpc({{ type: 'NAVIGATE', url: target }});
                    }}
                }};

                input.addEventListener('input', () => {{
                    syncClearVisibility();
                    renderOmniboxSuggestions(input.value, hudSuggest, input, deactivateModalHud);
                }});

                btnClear.onclick = () => {{
                    input.value = '';
                    syncClearVisibility();
                    renderOmniboxSuggestions('', hudSuggest, input, deactivateModalHud);
                    input.focus();
                }};

                engineBadge.onclick = () => {{
                    const list = getEngines();
                    const curIdx = list.findIndex(e => e.id === config.search_engine);
                    const next = list[(curIdx + 1) % list.length];
                    if (next) {{
                        config.search_engine = next.id;
                        engineBadge.textContent = next.name;
                        input.placeholder = `Search ${{next.name}} or enter !bang / URL...`;
                        updateAddressLabels();
                        sendIpc({{ type: 'SAVE_CONFIG', config: config }});
                    }}
                    input.focus();
                }};

                btnGo.onclick = submitModal;
                bindOmniboxKeydown(input, hudSuggest, submitModal, deactivateModalHud);

                modalBar.appendChild(engineBadge);
                modalBar.appendChild(input);
                modalBar.appendChild(btnClear);
                modalBar.appendChild(btnGo);

                renderHudQuickBar(hudQuick, deactivateModalHud);
                renderOmniboxSuggestions('', hudSuggest, input, deactivateModalHud);
                hudRoot.style.display = 'flex';
                updateAddressLabels();

                requestAnimationFrame(() => {{
                    input.focus();
                    if (input.value) input.select();
                }});
            }}

            function openHud() {{
                const useInline = window.innerWidth > 768 && (styles.address_bar.mode || 'inline').toLowerCase() !== 'modal';
                if (useInline) {{
                    activateInlineAddressBar();
                }} else {{
                    openModalHud();
                }}
            }}

            // Close inline <input> & restore <button> when clicking outside
            window.addEventListener('pointerdown', (e) => {{
                if (!isInlineActive) return;
                const path = typeof e.composedPath === 'function' ? e.composedPath() : [];
                const dropEl = shadowRoot.getElementById('__fynexo_inline_dropdown__');
                if (!path.includes(addrSlot) && (!dropEl || !path.includes(dropEl))) {{
                    deactivateInlineAddressBar();
                }}
            }}, true);

            window.addEventListener('resize', () => {{
                if (isInlineActive) {{
                    if (window.innerWidth <= 768) {{
                        deactivateInlineAddressBar();
                    }} else {{
                        positionInlineDropdown(shadowRoot.getElementById('__fynexo_inline_dropdown__'));
                    }}
                }}
            }}, {{ passive: true }});

            syncAllStyles();

            if (document.readyState === 'complete') {{
                finishProgress();
            }} else {{
                window.addEventListener('load', () => {{
                    finishProgress();
                    if (config.force_dark) applyPageRules();
                }}, {{ once: true }});
            }}

            // Top-edge hover peek in Immersive Fullscreen
            window.addEventListener('mousemove', (e) => {{
                if (!isFullscreen) return;
                if (e.clientY <= 6 && !isTopbarPeek) {{
                    isTopbarPeek = true;
                    applyShadowStyles();
                }}
            }}, {{ passive: true }});

            // Mobile Webpage Input Auto-Hide
            document.addEventListener('focusin', (e) => {{
                if (window.innerWidth <= 768 && hudRoot.style.display !== 'flex') {{
                    if (isEditableWebElement(e.target)) {{
                        setKeyboardHiddenState(true);
                    }}
                }}
            }}, true);

            document.addEventListener('focusout', () => {{
                if (window.innerWidth <= 768) {{
                    setTimeout(() => {{
                        if (!isEditableWebElement(document.activeElement)) {{
                            setKeyboardHiddenState(false);
                        }}
                    }}, 120);
                }}
            }}, true);

            if (window.visualViewport) {{
                window.visualViewport.addEventListener('resize', () => {{
                    if (window.innerWidth <= 768 && hudRoot.style.display !== 'flex') {{
                        const ratio = window.visualViewport.height / Math.max(window.screen.height, window.innerHeight, 1);
                        if (ratio < 0.68 || isEditableWebElement(document.activeElement)) {{
                            setKeyboardHiddenState(true);
                        }} else {{
                            setKeyboardHiddenState(false);
                        }}
                    }}
                }}, {{ passive: true }});
            }}

            window.addEventListener('keydown', (e) => {{
                if (window.__fynexo_recording_keybind__) return;

                // Protect plain single-key shortcuts when typing inside webpage inputs
                if (!e.ctrlKey && !e.metaKey && !e.altKey && !/^f\d+$/i.test(e.key || '') && e.key !== 'Escape') {{
                    if (isEditableWebElement(e.target)) return;
                }}

                if (matchesKeybind(e, keybinds.omnibox) || (e.altKey && !e.ctrlKey && !e.shiftKey && (e.key || '').toLowerCase() === 'd')) {{
                    e.preventDefault();
                    e.stopPropagation();
                    openHud();
                    return;
                }}
                if (matchesKeybind(e, keybinds.reopen_closed_tab)) {{
                    e.preventDefault();
                    e.stopPropagation();
                    triggerProgress();
                    sendIpc({{ type: 'REOPEN_CLOSED_TAB' }});
                    return;
                }}
                if (matchesKeybind(e, keybinds.new_tab)) {{
                    e.preventDefault();
                    e.stopPropagation();
                    flushScrollSync();
                    triggerProgress();
                    sendIpc({{ type: 'NEW_TAB' }});
                    return;
                }}
                if (matchesKeybind(e, keybinds.close_tab)) {{
                    e.preventDefault();
                    e.stopPropagation();
                    if (tabState.active_id) {{
                        sendIpc({{ type: 'CLOSE_TAB', id: tabState.active_id }});
                    }}
                    return;
                }}
                if (matchesKeybind(e, keybinds.next_tab) || (String(keybinds.next_tab || '').toLowerCase() === 'ctrl+tab' && e.ctrlKey && e.shiftKey && e.key === 'Tab')) {{
                    e.preventDefault();
                    e.stopPropagation();
                    if (tabState.tabs.length > 1) {{
                        const curIdx = tabState.tabs.findIndex(t => t.id === tabState.active_id);
                        const nextIdx = (curIdx + (e.shiftKey ? -1 : 1) + tabState.tabs.length) % tabState.tabs.length;
                        flushScrollSync();
                        triggerProgress();
                        sendIpc({{ type: 'SWITCH_TAB', id: tabState.tabs[nextIdx].id }});
                    }}
                    return;
                }}
                if (matchesKeybind(e, keybinds.bookmark_page)) {{
                    e.preventDefault();
                    e.stopPropagation();
                    toggleCurrentBookmark();
                    return;
                }}
                if (matchesKeybind(e, keybinds.find_in_page)) {{
                    if (!isInternalSettings()) {{
                        e.preventDefault();
                        e.stopPropagation();
                        openFindBar();
                        return;
                    }}
                }}
                if (matchesKeybind(e, keybinds.reader_mode)) {{
                    if (!isInternalAny()) {{
                        e.preventDefault();
                        e.stopPropagation();
                        toggleReaderMode();
                        return;
                    }}
                }}
                if (matchesKeybind(e, keybinds.force_dark)) {{
                    e.preventDefault();
                    e.stopPropagation();
                    config.force_dark = !config.force_dark;
                    sendIpc({{ type: 'SAVE_CONFIG', config: config }});
                    applyPageRules();
                    syncBookmarkButtons();
                    if (config.force_dark && isPageAlreadyDark()) {{
                        showHudToast('Smart Dark: Page is already dark (skipped)');
                    }} else {{
                        showHudToast(config.force_dark ? 'Smart Force Dark enabled' : 'Smart Force Dark off');
                    }}
                    return;
                }}
                if (matchesKeybind(e, keybinds.fullscreen)) {{
                    e.preventDefault();
                    e.stopPropagation();
                    sendIpc({{ type: 'TOGGLE_FULLSCREEN' }});
                    return;
                }}
                if (matchesKeybind(e, keybinds.preferences)) {{
                    e.preventDefault();
                    e.stopPropagation();
                    closeAnyOmnibox();
                    flushScrollSync();
                    togglePreferences();
                    return;
                }}
                if (matchesKeybind(e, keybinds.zoom_in)) {{
                    e.preventDefault();
                    e.stopPropagation();
                    sendIpc({{ type: 'SET_ZOOM', delta: 0.1 }});
                    return;
                }}
                if (matchesKeybind(e, keybinds.zoom_out)) {{
                    e.preventDefault();
                    e.stopPropagation();
                    sendIpc({{ type: 'SET_ZOOM', delta: -0.1 }});
                    return;
                }}
                if (matchesKeybind(e, keybinds.zoom_reset)) {{
                    e.preventDefault();
                    e.stopPropagation();
                    sendIpc({{ type: 'SET_ZOOM', zoom: 1.0 }});
                    return;
                }}
                if (e.altKey && !e.ctrlKey && (e.key || '').toLowerCase() === 's') {{
                    e.preventDefault();
                    e.stopPropagation();
                    if (typeof toggleSidePanel === 'function') toggleSidePanel();
                    return;
                }}
                if (e.key === 'Escape') {{
                    if (typeof isSidePanelOpen !== 'undefined' && isSidePanelOpen) {{
                        toggleSidePanel(false);
                        return;
                    }}
                    if (tabContextMenuEl && tabContextMenuEl.style.display !== 'none') {{
                        hideTabContextMenu();
                        return;
                    }}
                    if (typeof closeAllNavPopovers === 'function' && activeNavPopoverEl && activeNavPopoverEl.style.display !== 'none') {{
                        closeAllNavPopovers();
                        return;
                    }}
                    if (isInlineActive || isModalActive) {{
                        closeAnyOmnibox();
                        return;
                    }}
                    if (isReaderOpen) {{
                        closeReaderMode();
                        return;
                    }}
                    if (isFindOpen) {{
                        closeFindBar();
                        return;
                    }}
                    if (isDrawerOpen) {{
                        closeMobileDrawer();
                        return;
                    }}
                    if (isFullscreen) {{
                        sendIpc({{ type: 'TOGGLE_FULLSCREEN', force: false }});
                        return;
                    }}
                }}
            }}, true);

            let scrollSaveTimer = null;
            window.addEventListener('scroll', () => {{
                if (isInternalAny() || isReaderOpen) return;
                if (scrollSaveTimer) clearTimeout(scrollSaveTimer);
                scrollSaveTimer = setTimeout(flushScrollSync, 350);
            }}, {{ passive: true, capture: true }});

            window.addEventListener('click', (e) => {{
                const host = document.getElementById('__fynexo_hud_host__');
                const path = typeof e.composedPath === 'function' ? e.composedPath() : [];
                const isInsideHud = host && (
                    e.target === host ||
                    (host.contains && host.contains(e.target)) ||
                    path.includes(host)
                );
                if (isInsideHud) {{
                    return;
                }}
                hideTabContextMenu();
                if (typeof closeAllNavPopovers === 'function') closeAllNavPopovers();
            }}, true);
            window.addEventListener('contextmenu', (e) => {{
                const host = document.getElementById('__fynexo_hud_host__');
                const path = typeof e.composedPath === 'function' ? e.composedPath() : [];
                const isInsideHud = host && (
                    e.target === host ||
                    (host.contains && host.contains(e.target)) ||
                    path.includes(host)
                );
                if (isInsideHud) {{
                    return;
                }}
                hideTabContextMenu();
            }}, true);
            if (shadowRoot) {{
                shadowRoot.addEventListener('click', (e) => {{
                    if (tabContextMenuEl && !tabContextMenuEl.contains(e.target)) {{
                        hideTabContextMenu();
                    }}
                    const targetEl = (e.target && e.target.nodeType === 1) ? e.target : (e.target && e.target.parentElement);
                    const inNavPop = targetEl && targetEl.closest && targetEl.closest('.fnx-nav-popover');
                    const inNavBtn = targetEl && targetEl.closest && (
                        targetEl.closest('.fnx-popover-trigger') ||
                        targetEl.closest('#__fynexo_btn_more__') ||
                        targetEl.closest('#__fynexo_btn_shield__') ||
                        targetEl.closest('#__fynexo_btn_media__') ||
                        targetEl.closest('#__fynexo_btn_downloads__') ||
                        targetEl.closest('#__fynexo_btn_sidepanel__') ||
                        targetEl.closest('#__fynexo_desk_reader__') ||
                        targetEl.closest('#__fynexo_btn_reader_pill__')
                    );
                    if (!inNavPop && !inNavBtn && typeof closeAllNavPopovers === 'function') {{
                        closeAllNavPopovers();
                    }}
                }});
                shadowRoot.addEventListener('contextmenu', (e) => {{
                    const targetEl = (e.target && e.target.nodeType === 1) ? e.target : (e.target && e.target.parentElement);
                    if (!targetEl || !targetEl.closest || !targetEl.closest('.fnx-tab-item')) {{
                        hideTabContextMenu();
                    }}
                }});
            }}

            document.addEventListener('click', (e) => {{
                const a = e.target.closest ? e.target.closest('a') : null;
                if (a && a.href) {{
                    const rawHref = a.getAttribute('href') || '';
                    if (!rawHref.startsWith('#') && !rawHref.startsWith('javascript:')) {{
                        const isBlank = a.target === '_blank' || a.target === 'blank';
                        const isCtrlOrMiddle = (e.ctrlKey && e.button === 0) || e.button === 1;
                        if (isBlank || isCtrlOrMiddle) {{
                            e.preventDefault();
                            e.stopPropagation();
                            triggerProgress();
                            sendIpc({{ type: 'NEW_TAB', url: a.href }});
                        }}
                    }}
                }}
            }}, true);

            try {{
                window.open = function(url) {{
                    if (url) {{
                        triggerProgress();
                        sendIpc({{ type: 'NEW_TAB', url: url }});
                    }}
                    return null;
                }};
            }} catch (_) {{}}

            // Viewport & Dimensions Patch for Web Games, Canvas & IO Games
            try {{
                const getActiveTopH = () => {{
                    if (isFullscreen) return 0;
                    if (!document.documentElement || !document.documentElement.classList.contains('fnx-game-mode')) return 0;
                    const isDeck = ((styles && styles.topbar && styles.topbar.layout) || 'deck').toLowerCase() !== 'compact';
                    const bmShown = (styles && styles.topbar && (styles.topbar.bookmarks_bar || 'always').toLowerCase()) === 'always';
                    return isDeck ? (40 + 38 + (bmShown ? 28 : 0)) : (38 + (bmShown ? 28 : 0));
                }};

                const protoWin = Object.getPrototypeOf(window) || window;
                const origInnerH = Object.getOwnPropertyDescriptor(protoWin, 'innerHeight') ||
                                   Object.getOwnPropertyDescriptor(window, 'innerHeight');
                if (origInnerH && origInnerH.get) {{
                    Object.defineProperty(window, 'innerHeight', {{
                        get: function() {{
                            const raw = origInnerH.get.call(this);
                            return Math.max(0, raw - getActiveTopH());
                        }},
                        configurable: true,
                        enumerable: true
                    }});
                }}

                const origClientH = Object.getOwnPropertyDescriptor(Element.prototype, 'clientHeight');
                if (origClientH && origClientH.get) {{
                    Object.defineProperty(document.documentElement, 'clientHeight', {{
                        get: function() {{
                            const raw = origClientH.get.call(this);
                            return Math.max(0, raw - getActiveTopH());
                        }},
                        configurable: true,
                        enumerable: true
                    }});
                }}

                if (window.visualViewport) {{
                    const protoVv = Object.getPrototypeOf(window.visualViewport);
                    const origVvH = Object.getOwnPropertyDescriptor(protoVv, 'height');
                    if (origVvH && origVvH.get) {{
                        Object.defineProperty(window.visualViewport, 'height', {{
                            get: function() {{
                                const raw = origVvH.get.call(this);
                                return Math.max(0, raw - getActiveTopH());
                            }},
                            configurable: true,
                            enumerable: true
                        }});
                    }}
                }}
            }} catch (_) {{}}

            // HTML5 Fullscreen API Synchronization for Web Games & Video Players
            document.addEventListener('fullscreenchange', () => {{
                const isDomFull = !!document.fullscreenElement;
                if (isDomFull) {{
                    if (!isFullscreen) sendIpc({{ type: 'TOGGLE_FULLSCREEN', force: true }});
                }} else {{
                    if (isFullscreen) sendIpc({{ type: 'TOGGLE_FULLSCREEN', force: false }});
                }}
            }});

            // Game & Viewport Isolation Engine (Zero Scroll, No Scrollbars for Games & Canvas)
            function checkIsGameMode() {{
                try {{
                    if (isInternalAny()) return false;
                    const host = (window.location.hostname || '').toLowerCase();
                    // Never flag video/media or major web platforms as games
                    if (host.includes('youtube') || host.includes('bilibili') || host.includes('netflix') ||
                        host.includes('twitch') || host.includes('vimeo') || host.includes('google') ||
                        host.includes('github') || host.includes('twitter') || host.includes('x.com') ||
                        host.includes('reddit') || host.includes('wikipedia')) {{
                        return false;
                    }}

                    // 1. Well-known game platforms / IO game domains
                    if (host.endsWith('.io') || host.includes('game') || host.includes('poki.') ||
                        host.includes('crazygames.') || host.includes('itch.') || host.includes('1v1.lol') ||
                        host.includes('y8.') || host.includes('krunker')) {{
                        return true;
                    }}

                    // 2. Pointer lock active (FPS / 3D Canvas Games)
                    if (document.pointerLockElement) return true;

                    // 3. Fullscreen element is a canvas (e.g. web game fullscreen)
                    if (document.fullscreenElement && document.fullscreenElement.tagName === 'CANVAS') {{
                        return true;
                    }}
                }} catch (_) {{}}
                return false;
            }}

            function syncGameMode() {{
                const isGame = checkIsGameMode();
                const docEl = document.documentElement;
                if (!docEl) return;
                if (isGame) {{
                    if (!docEl.classList.contains('fnx-game-mode')) {{
                        docEl.classList.add('fnx-game-mode');
                        if (document.body) {{
                            document.body.classList.add('fnx-game-mode');
                            document.body.scrollTop = 0;
                            document.body.scrollLeft = 0;
                        }}
                        docEl.scrollTop = 0;
                        docEl.scrollLeft = 0;
                    }}
                }} else {{
                    if (docEl.classList.contains('fnx-game-mode')) {{
                        docEl.classList.remove('fnx-game-mode');
                        if (document.body) document.body.classList.remove('fnx-game-mode');
                    }}
                }}
            }}

            syncGameMode();
            document.addEventListener('DOMContentLoaded', syncGameMode, {{ once: true }});
            window.addEventListener('load', syncGameMode, {{ once: true }});
            window.addEventListener('resize', syncGameMode, {{ passive: true }});
            document.addEventListener('fullscreenchange', syncGameMode, {{ passive: true }});
            document.addEventListener('pointerlockchange', syncGameMode, {{ passive: true }});
            document.addEventListener('mozpointerlockchange', syncGameMode, {{ passive: true }});

            // YouTube Auto Ad Skipper & Video Ad Cleaner (Zero-Blackscreen / Fluid Transition)
            function initYouTubeAdSkipper() {{
                try {{
                    const host = (window.location.hostname || '').toLowerCase();
                    if (!host.includes('youtube.com')) return;

                    let savedRate = 1.0;

                    const skipAds = () => {{
                        if (config.adblock === false || document.hidden) return;
                        const player = document.querySelector('.html5-video-player');
                        if (!player) return;

                        const isAd = player.classList.contains('ad-showing') || player.classList.contains('ad-interrupting');
                        const v = player.querySelector('video');

                        if (v) {{
                            if (isAd) {{
                                // Mute ad and fast-forward at 16x without seeking currentTime to avoid MSE pipeline crash/black screen
                                v.muted = true;
                                v.playbackRate = 16.0;
                            }} else {{
                                // Restore original playback speed and unmute as soon as ad finishes
                                if (v.playbackRate > 2.0) {{
                                    v.playbackRate = savedRate || 1.0;
                                    v.muted = false;
                                }} else {{
                                    savedRate = v.playbackRate;
                                }}
                            }}
                        }}

                        if (isAd) {{
                            // Only target clickable button elements to prevent container click delegation issues
                            const skipBtn = document.querySelector(
                                'button.ytp-ad-skip-button-modern, button.ytp-ad-skip-button, .ytp-skip-ad-button, .ytp-ad-skip-button-container button'
                            );
                            if (skipBtn && typeof skipBtn.click === 'function') {{
                                try {{ skipBtn.click(); }} catch (_) {{}}
                            }}
                            const overlayBtn = document.querySelector('.ytp-ad-overlay-close-button');
                            if (overlayBtn && typeof overlayBtn.click === 'function') {{
                                try {{ overlayBtn.click(); }} catch (_) {{}}
                            }}
                        }}
                    }};

                    setInterval(skipAds, 250);
                }} catch (_) {{}}
            }}
            initYouTubeAdSkipper();

            window.addEventListener('wheel', () => {{
                if (document.documentElement && document.documentElement.classList.contains('fnx-game-mode')) {{
                    if (document.body && (document.body.scrollTop !== 0 || document.body.scrollLeft !== 0)) {{
                        document.body.scrollTop = 0;
                        document.body.scrollLeft = 0;
                    }}
                    if (document.documentElement.scrollTop !== 0 || document.documentElement.scrollLeft !== 0) {{
                        document.documentElement.scrollTop = 0;
                        document.documentElement.scrollLeft = 0;
                    }}
                }}
            }}, {{ passive: true }});

            let mutLastTitle = '';
            let mutLastUrl = '';
            let titleObserved = false;

            function syncTitle() {{
                const isSet = isInternalSettings();
                const isNt = isInternalNewtab();
                const t = isSet ? 'Preferences' : (isNt ? 'Start page' : (document.title || window.location.hostname || 'New tab'));
                const u = isSet ? 'fynexo://settings' : (isNt ? 'fynexo://newtab' : window.location.href);
                if (t === mutLastTitle && u === mutLastUrl) return;
                mutLastTitle = t;
                mutLastUrl = u;
                sendIpc({{
                    type: 'UPDATE_TAB_TITLE',
                    title: t,
                    url: u
                }});
                updateAddressLabels();
            }}

            function bindTitleObserver() {{
                syncTitle();
                if (titleObserved) return;
                const titleEl = document.querySelector('title');
                if (titleEl) {{
                    titleObserved = true;
                    new MutationObserver(syncTitle).observe(titleEl, {{
                        subtree: true,
                        characterData: true,
                        childList: true
                    }});
                }}
            }}

            bindTitleObserver();
            document.addEventListener('DOMContentLoaded', bindTitleObserver, {{ once: true }});
            window.addEventListener('load', bindTitleObserver, {{ once: true }});
            window.addEventListener('popstate', () => requestAnimationFrame(syncTitle), {{ passive: true }});
            window.addEventListener('hashchange', () => requestAnimationFrame(syncTitle), {{ passive: true }});

            // Frame-0 Immediate Style & Layout Synchronization (0ms paint, zero overlap)
            syncAllStyles();

            // Single Combined IPC Sync on Boot (with resilient retry if window.ipc is delayed)
            function sendBootSyncWhenReady(retries) {{
                if (window.ipc && typeof window.ipc.postMessage === 'function') {{
                    sendIpc({{ type: 'BOOT_SYNC' }});
                }} else if (retries < 30) {{
                    setTimeout(() => sendBootSyncWhenReady(retries + 1), 40);
                }}
            }}
            sendBootSyncWhenReady(0);

            // O(1) Pointer-Flag Anti-Detachment Observer
            try {{
                const antiStripObserver = new MutationObserver(() => {{
                    if (!host.isConnected) {{
                        const target = document.documentElement || document.body;
                        if (target) target.appendChild(host);
                    }}
                    if (!pageRulesEl || !pageRulesEl.isConnected) {{
                        applyPageRules();
                    }}
                    if (siteRulesEl && !siteRulesEl.isConnected) {{
                        applySiteRules();
                    }}
                    syncGameMode();
                }});
                antiStripObserver.observe(document.documentElement, {{ childList: true }});
            }} catch (_) {{}}

        }} catch (err) {{
            console.error('[Fynexo HUD Exception]:', err);
            try {{
                const h = document.getElementById('__fynexo_host__');
                if (h && (!shadowRoot || !shadowRoot.getElementById('__fynexo_topbar__'))) {{
                    h.remove();
                }}
            }} catch (_) {{}}
        }}
    }}

    function ensureMounted() {{
        const target = document.documentElement || document.body;
        if (!target) {{
            if (window.requestAnimationFrame) window.requestAnimationFrame(ensureMounted);
            else setTimeout(ensureMounted, 5);
            return;
        }}
        const existingHost = document.getElementById('__fynexo_host__');
        if (!existingHost || !shadowRoot || !shadowRoot.getElementById('__fynexo_topbar__')) {{
            if (existingHost && (!shadowRoot || !shadowRoot.getElementById('__fynexo_topbar__'))) {{
                try {{ existingHost.remove(); }} catch (_) {{}}
            }}
            initInterface();
        }}
    }}

    ensureMounted();
    document.addEventListener('DOMContentLoaded', ensureMounted, {{ once: true }});
}})();
"##,
        initial_json = initial_json,
        icons_script = ICONS_SCRIPT,
        defaults_script = DEFAULTS_SCRIPT,
        base_styles_css = base_css_json
    )
}
