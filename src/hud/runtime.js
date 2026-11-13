(function () {
   // 1. Iframe & Blank-Park Isolation: Only top-level real pages host Fynexo Chrome
   if (window.top !== window.self) return;
   if (window.location && window.location.href === 'about:blank') return;

   // Capture pristine built-ins & IPC reference at document_start before any page script runs.
   const __FNX_IPC_TOKEN = '__FNX_IPC_TOKEN__';
   const __FNX_BRIDGE_TOKEN = '__FNX_BRIDGE_TOKEN__';
   const __fnxJsonStringify = JSON.stringify;
   const __fnxJsonParse = JSON.parse;
   const __fnxDefineProperty = Object.defineProperty;
   const __fnxStringSlice = String.prototype.slice;
   const __fnxArraySlice = Array.prototype.slice;
   let __fnxIpcPost =
      window.ipc && typeof window.ipc.postMessage === 'function'
         ? window.ipc.postMessage.bind(window.ipc)
         : null;

   function exposeReadOnly(name, fn) {
      try {
         __fnxDefineProperty(window, name, {
            value: fn,
            writable: false,
            configurable: false,
            enumerable: false,
         });
      } catch (_) {
         window[name] = fn;
      }
   }

   function exposeBridge(name, fn) {
      function guarded() {
         'use strict';
         const len = arguments.length;
         if (len < 1 || arguments[len - 1] !== __FNX_BRIDGE_TOKEN) return;
         return fn.apply(null, __fnxArraySlice.call(arguments, 0, len - 1));
      }
      exposeReadOnly(name, guarded);
   }

   __FNX_ICONS__;

   __FNX_DEFAULTS__;

   function mergeStyles(raw) {
      const s = raw || {};
      return {
         colors: Object.assign({}, DEFAULT_STYLES.colors, s.colors || {}),
         topbar: Object.assign({}, DEFAULT_STYLES.topbar, s.topbar || {}),
         tabs: Object.assign({}, DEFAULT_STYLES.tabs, s.tabs || {}),
         address_bar: Object.assign(
            {},
            DEFAULT_STYLES.address_bar,
            s.address_bar || {},
         ),
         search_modal: Object.assign(
            {},
            DEFAULT_STYLES.search_modal,
            s.search_modal || {},
         ),
         mobile_bar: Object.assign(
            {},
            DEFAULT_STYLES.mobile_bar,
            s.mobile_bar || {},
         ),
         tab_drawer: Object.assign(
            {},
            DEFAULT_STYLES.tab_drawer,
            s.tab_drawer || {},
         ),
         settings_page: Object.assign(
            {},
            DEFAULT_STYLES.settings_page,
            s.settings_page || {},
         ),
         newtab: Object.assign({}, DEFAULT_STYLES.newtab, s.newtab || {}),
      };
   }

   // Frame-0 Cache Hydration (internal pages only; never expose/read sessionStorage on external sites)
   function readSessionCache(key) {
      if (!isInternalAny()) return null;
      try {
         const v = window.sessionStorage && window.sessionStorage.getItem(key);
         return v ? __fnxJsonParse(v) : null;
      } catch (_) {
         return null;
      }
   }

   function writeSessionCache(key, obj) {
      if (!isInternalAny()) return;
      try {
         if (window.sessionStorage)
            window.sessionStorage.setItem(key, __fnxJsonStringify(obj));
      } catch (_) {}
   }

   const cachedRt = readSessionCache('__fnx_rt__');
   const cachedTabs = readSessionCache('__fnx_tabs__');
   const bootPayload = cachedRt || __FNX_INITIAL_JSON__ || {};

   let config = Object.assign(
      {
         theme: 'dark',
         enforce_perf: true,
         adblock: true,
         restore_session: true,
         force_dark: false,
         incognito: false,
         default_zoom: 1.0,
         user_agent_mode: 'default',
         custom_user_agent: '',
         tab_cache_mode: 'balanced',
         search_engine: 'google',
         homepage: 'fynexo://newtab',
         search_engines: DEFAULT_ENGINES,
         bangs: DEFAULT_BANGS,
      },
      bootPayload.config || {},
   );
   let styles = mergeStyles(bootPayload.styles);
   let activeTheme = Object.assign(
      {},
      DEFAULT_STYLES.colors,
      bootPayload.active_theme || {},
   );
   let keybinds = Object.assign(
      {},
      DEFAULT_KEYBINDS,
      bootPayload.keybinds || {},
   );
   let enabledRules = Array.isArray(bootPayload.enabled_rules)
      ? bootPayload.enabled_rules
      : Array.isArray(bootPayload.site_rules)
        ? bootPayload.site_rules.filter((r) => r && r.enabled)
        : [];
   let bookmarks = Array.isArray(bootPayload.bookmarks)
      ? bootPayload.bookmarks
      : [];
   let historyItems = Array.isArray(bootPayload.history)
      ? bootPayload.history
      : [];

   function getEngines() {
      return Array.isArray(config.search_engines) &&
         config.search_engines.length > 0
         ? config.search_engines
         : DEFAULT_ENGINES;
   }

   function getBangs() {
      return Array.isArray(config.bangs) ? config.bangs : DEFAULT_BANGS;
   }

   function currentEngineObj(engineId) {
      const list = getEngines();
      const targetId = engineId || config.search_engine;
      for (let i = 0; i < list.length; i++) {
         if (list[i] && list[i].id === targetId) return list[i];
      }
      return list[0] || DEFAULT_ENGINES[0];
   }

   function cacheSlimRuntime() {
      writeSessionCache('__fnx_rt__', {
         config: config,
         styles: styles,
         active_theme: activeTheme,
         keybinds: keybinds,
         enabled_rules: enabledRules,
         bookmarks: bookmarks,
         history: historyItems.slice(0, 25),
      });
   }

   let tabState = cachedTabs ||
      (bootPayload.tabs && Array.isArray(bootPayload.tabs.tabs)
         ? bootPayload.tabs
         : null) || {
         tabs: [],
         active_id: 1,
         closed_count: 0,
      };

   let shadowRoot = null;
   let pageRulesEl = null;
   let siteRulesEl = null;
   let isPageKeyboardOpen = false;
   let isDrawerOpen = false;
   let isFullscreen = false;
   let isTopbarPeek = false;
   let bootSyncDone = false;
   let onBootSyncDoneCb = null;
   let isFindOpen = false;
   let isReaderOpen = false;
   let navState = { canBack: false, canForward: false };
   let isInlineActive = false;
   let isModalActive = false;
   let closeOmniboxRef = null;
   let closeAllNavPopoversRef = null;
   let closeSidePanelRef = null;
   let isAnyNavPopoverOpenRef = null;
   let isSidePanelOpenRef = null;
   let openHistoryDrawerRef = null;
   let closeHistoryDrawerRef = null;
   let isHistoryOpenRef = null;
   let openPromptModalRef = null;
   let closePromptModalRef = null;
   let isPromptOpenRef = null;
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
   let progressTimer = null;
   let progressFadeTimer = null;
   let progressResetTimer = null;
   let onResourceStatsCb = null;
   let onForegroundResumeCb = null;
   let onTabsRenderedCb = null;
   let draggingTabId = null;
   let isTabMutedState = false;
   let isBackgroundParked = false;

   function restorePageFocus() {
      try {
         if (
            shadowRoot &&
            shadowRoot.activeElement &&
            typeof shadowRoot.activeElement.blur === 'function'
         ) {
            shadowRoot.activeElement.blur();
         }
         if (
            document.activeElement &&
            typeof document.activeElement.blur === 'function'
         ) {
            document.activeElement.blur();
         }
         if (document.body && typeof document.body.focus === 'function') {
            if (!document.body.hasAttribute('tabindex')) {
               document.body.setAttribute('tabindex', '-1');
            }
            document.body.focus({ preventScroll: true });
         }
      } catch (_) {}
   }

   function copyTextToClipboard(text, toastMsg) {
      const value = String(text || '');
      if (!value) return;
      const done = () => {
         if (toastMsg) showHudToast(toastMsg);
      };
      const execFallback = () => {
         try {
            const ta = document.createElement('textarea');
            ta.value = value;
            ta.style.cssText = 'position:fixed;left:-9999px;opacity:0;';
            (document.body || document.documentElement).appendChild(ta);
            ta.select();
            document.execCommand('copy');
            ta.remove();
            done();
         } catch (_) {}
      };
      if (
         navigator.clipboard &&
         typeof navigator.clipboard.writeText === 'function'
      ) {
         navigator.clipboard.writeText(value).then(done).catch(execFallback);
      } else {
         execFallback();
      }
   }

   function copyCurrentUrl() {
      const u = canonicalCurrentUrl();
      if (!u) return;
      copyTextToClipboard(u, 'URL copied');
   }

   function sendIpc(data) {
      'use strict';
      let raw;
      try {
         raw = __fnxJsonStringify(data);
      } catch (_) {
         return;
      }
      if (!raw || raw[0] !== '{') return;
      const payload =
         '{"__t":"' +
         __FNX_IPC_TOKEN +
         '"' +
         (raw.length > 2 ? ',' + __fnxStringSlice.call(raw, 1) : '}');
      if (
         !__fnxIpcPost &&
         window.ipc &&
         typeof window.ipc.postMessage === 'function'
      ) {
         __fnxIpcPost = window.ipc.postMessage.bind(window.ipc);
      }
      if (__fnxIpcPost) {
         __fnxIpcPost(payload);
      } else if (window.__TAURI_INTERNALS__ && window.__TAURI_INTERNALS__.ipc) {
         window.__TAURI_INTERNALS__.ipc.postMessage(payload);
      }
   }

   function normalizeUrlKey(u) {
      if (!u) return '';
      const t = String(u).trim();
      return t.length > 8 &&
         t.endsWith('/') &&
         (t.match(/\//g) || []).length === 3
         ? t.slice(0, -1).toLowerCase()
         : t.toLowerCase();
   }

   function isUrlBookmarked(u) {
      if (!u || isInternalAny(u)) return false;
      const key = normalizeUrlKey(u);
      for (let i = 0; i < bookmarks.length; i++) {
         if (normalizeUrlKey(bookmarks[i].url) === key) return true;
      }
      return false;
   }

   function parseInternalRoute(rawUrl) {
      const u = rawUrl || (window.location && window.location.href) || '';
      if (!u) return '';
      try {
         const base =
            (window.location && window.location.href) || 'about:blank';
         const parsed = new URL(u, base);
         const proto = parsed.protocol.toLowerCase();
         const host = parsed.hostname.toLowerCase();
         const path =
            parsed.pathname.toLowerCase().replace(/^\/+/, '').split('/')[0] ||
            '';
         if (proto === 'fynexo:') {
            if (
               host === 'settings' ||
               (host === 'localhost' && path === 'settings')
            )
               return 'settings';
            if (
               host === 'newtab' ||
               (host === 'localhost' && (path === '' || path === 'newtab'))
            )
               return 'newtab';
            return '';
         }
         if (
            (proto === 'http:' || proto === 'https:') &&
            host === 'fynexo.localhost'
         ) {
            if (path === 'settings') return 'settings';
            if (path === '' || path === 'newtab') return 'newtab';
         }
      } catch (_) {}
      return '';
   }

   function isInternalSettings(rawUrl) {
      return parseInternalRoute(rawUrl) === 'settings';
   }

   function isInternalNewtab(rawUrl) {
      return parseInternalRoute(rawUrl) === 'newtab';
   }

   function isInternalAny(rawUrl) {
      return isInternalSettings(rawUrl) || isInternalNewtab(rawUrl);
   }

   function canonicalCurrentUrl() {
      if (isInternalSettings()) return 'fynexo://settings';
      if (isInternalNewtab()) return 'fynexo://newtab';
      return window.location.href || '';
   }

   function formatHostLabel(rawUrl) {
      if (!rawUrl && isInternalNewtab()) return 'Search or enter URL';
      if (!rawUrl && isInternalSettings()) return 'Preferences';
      if (isInternalNewtab(rawUrl)) return 'Search or enter URL';
      if (isInternalSettings(rawUrl)) return 'Preferences';
      if (!rawUrl || rawUrl === 'about:blank') return 'Search or enter URL';
      try {
         const parsed = new URL(rawUrl);
         return parsed.hostname.replace(/^www\./, '') || rawUrl;
      } catch (_) {
         return rawUrl;
      }
   }

   function extractMonogram(rawUrl, title) {
      if (isInternalNewtab(rawUrl)) return '✦';
      if (!rawUrl || isInternalSettings(rawUrl)) return '⚙';
      try {
         const host = new URL(rawUrl).hostname.replace(/^(www|m)\./, '');
         if (host) return host.charAt(0).toUpperCase();
      } catch (_) {}
      const clean = (title || '').trim();
      return clean ? clean.charAt(0).toUpperCase() : '•';
   }

   function monogramAccentColor(rawUrl) {
      if (isInternalNewtab(rawUrl)) return 'var(--fnx-accent)';
      if (!rawUrl || isInternalSettings(rawUrl)) return 'var(--fnx-secondary)';
      let host = '';
      try {
         host = new URL(rawUrl).hostname
            .replace(/^(www|m)\./, '')
            .toLowerCase();
      } catch (_) {
         host = String(rawUrl).toLowerCase();
      }
      if (
         host.includes('youtube') ||
         host.includes('gmail') ||
         host.includes('mail.google') ||
         host.includes('netflix')
      )
         return '#f87171';
      if (host.includes('bilibili')) return '#fb7185';
      if (
         host.includes('github') ||
         host.includes('discord') ||
         host.includes('gemini')
      )
         return '#818cf8';
      if (
         host.includes('translate') ||
         host.includes('wikipedia') ||
         host.includes('bing') ||
         host.includes('zhihu')
      )
         return '#38bdf8';
      if (
         host.includes('maps') ||
         host.includes('spotify') ||
         host.includes('duckduckgo') ||
         host.includes(' cooling')
      )
         return '#34d399';
      if (
         host.includes('reddit') ||
         host.includes('stackoverflow') ||
         host.includes('amazon') ||
         host.includes('taobao')
      )
         return '#fb923c';
      const palette = [
         '#7a88f7',
         '#38bdf8',
         '#34d399',
         '#f472b6',
         '#fb923c',
         '#a78bfa',
      ];
      let h = 0;
      for (let i = 0; i < host.length; i++) {
         h = (h << 5) - h + host.charCodeAt(i);
         h |= 0;
      }
      return palette[Math.abs(h) % palette.length];
   }

   function siteBrandIconName(rawUrl) {
      if (isInternalNewtab(rawUrl)) return 'siteNewtab';
      if (!rawUrl || isInternalSettings(rawUrl)) return 'siteSettings';
      let host = '';
      try {
         host = new URL(rawUrl).hostname
            .replace(/^(www|m)\./, '')
            .toLowerCase();
      } catch (_) {
         host = String(rawUrl).toLowerCase();
      }
      if (host.includes('youtube.com') || host === 'youtu.be')
         return 'siteYouTube';
      if (host.includes('github.com') || host.includes('githubusercontent.com'))
         return 'siteGitHub';
      if (host.includes('wikipedia.org')) return 'siteWikipedia';
      if (
         host === 'google.com' ||
         host.startsWith('google.') ||
         host.includes('.google.')
      )
         return 'siteGoogle';
      if (host.includes('bilibili.com')) return 'siteBilibili';
      if (host.includes('reddit.com')) return 'siteReddit';
      if (host.includes('discord.com') || host.includes('discord.gg'))
         return 'siteDiscord';
      if (host === 'x.com' || host.endsWith('.x.com') || host.includes('twitter.com'))
         return 'siteX';
      return '';
   }

   function renderSiteEmblem(targetEl, rawUrl, title) {
      if (!targetEl) return;
      const brandName = siteBrandIconName(rawUrl);
      if (brandName) {
         targetEl.textContent = '';
         const icon = createIcon(brandName);
         if (brandName === 'siteNewtab') {
            targetEl.style.color = 'var(--fnx-accent)';
         } else if (brandName === 'siteSettings') {
            targetEl.style.color = 'var(--fnx-secondary)';
         } else {
            targetEl.style.color = '';
         }
         targetEl.appendChild(icon);
         return;
      }
      targetEl.textContent = extractMonogram(rawUrl, title);
      targetEl.style.color = monogramAccentColor(rawUrl);
   }

   function resolvePalette() {
      const src =
         styles.colors && styles.colors.use_custom_colors
            ? styles.colors
            : activeTheme;
      return {
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
         tabInactive: src.tab_inactive || src.tabInactive || 'transparent',
      };
   }

   function isBookmarksBarShown() {
      const mode = (
         (styles.topbar && styles.topbar.bookmarks_bar) ||
         'always'
      ).toLowerCase();
      if (mode === 'hidden') return false;
      if (mode === 'newtab') return isInternalNewtab();
      return true;
   }

   function isMobilePlatform() {
      return /Android|iPhone|iPad|iPod|Mobile/i.test(navigator.userAgent);
   }

   function computeDesktopTopHeight() {
      const tb = styles.topbar || {};
      const isDeck = (tb.layout || 'deck').toLowerCase() !== 'compact';
      const navH = tb.height || '40px';
      const titlebarH = tb.titlebar_height || '42px';
      const bmShown = isBookmarksBarShown();
      if (isDeck) {
         return bmShown
            ? `calc(${titlebarH} + ${navH} + 29px)`
            : `calc(${titlebarH} + ${navH} + 1px)`;
      }
      return bmShown ? `calc(${navH} + 29px)` : `calc(${navH} + 1px)`;
   }

   function isPageAlreadyDark() {
      try {
         const meta = document.querySelector('meta[name="color-scheme"]');
         if (meta) {
            const c = (meta.getAttribute('content') || '').toLowerCase();
            if (c.includes('dark') && !c.includes('light')) return true;
         }

         const pr = document.getElementById('__fynexo_page_rules__');
         const wasDisabled = pr ? pr.disabled : false;
         if (pr) pr.disabled = true;

         const checkEl = (el) => {
            if (!el || !window.getComputedStyle) return null;
            const bg = window.getComputedStyle(el).backgroundColor || '';
            const m = bg.match(
               /rgba?\(\s*(\d+)\s*,\s*(\d+)\s*,\s*(\d+)(?:\s*,\s*([\d.]+))?\s*\)/i,
            );
            if (!m) return null;
            const a = m[4] !== undefined ? parseFloat(m[4]) : 1;
            if (a < 0.12) return null;
            return (
               0.299 * Number(m[1]) +
               0.587 * Number(m[2]) +
               0.114 * Number(m[3])
            );
         };

         let dark = false;
         const lumBody = checkEl(document.body);
         if (lumBody !== null) {
            dark = lumBody < 112;
         } else {
            const lumHtml = checkEl(document.documentElement);
            if (lumHtml !== null) {
               dark = lumHtml < 112;
            }
         }

         if (pr) pr.disabled = wasDisabled;
         return dark;
      } catch (_) {}
      return false;
   }

   function domainMatchesRule(hostname, pattern) {
      const p = String(pattern || '*')
         .trim()
         .toLowerCase();
      if (!p || p === '*') return true;
      const clean = p.startsWith('*.') ? p.slice(2) : p.replace(/^www\./, '');
      const h = String(hostname || '').toLowerCase();
      return h === clean || h.endsWith('.' + clean);
   }

   function applySiteRules() {
      if (isInternalAny()) {
         if (siteRulesEl && siteRulesEl.isConnected) siteRulesEl.remove();
         return;
      }
      const host = window.location.hostname || '';
      let combinedCss = '';
      for (let i = 0; i < enabledRules.length; i++) {
         const r = enabledRules[i];
         if (!r || !r.enabled) continue;
         if (!domainMatchesRule(host, r.domain)) continue;
         if (r.css && r.css.trim()) {
            combinedCss += '\n/* Rule: ' + (r.name || r.id) + ' */\n' + r.css;
         }
         if (r.js && r.js.trim() && !appliedRuleJsIds[r.id]) {
            appliedRuleJsIds[r.id] = true;
            const code = r.js;
            const runRuleJs = () => {
               try {
                  new Function(code)();
               } catch (err) {
                  console.warn(
                     '[Fynexo Site Rule JS Error]:',
                     r.name || r.id,
                     err,
                  );
               }
            };
            if (document.readyState === 'loading') {
               document.addEventListener('DOMContentLoaded', runRuleJs, {
                  once: true,
               });
            } else {
               runRuleJs();
            }
         }
      }

      if (!combinedCss) {
         if (siteRulesEl && siteRulesEl.isConnected) siteRulesEl.remove();
         siteRulesEl = null;
         return;
      }

      if (!siteRulesEl || !siteRulesEl.isConnected) {
         siteRulesEl = document.getElementById('__fynexo_site_rules__');
         if (!siteRulesEl) {
            siteRulesEl = document.createElement('style');
            siteRulesEl.id = '__fynexo_site_rules__';
            (
               document.documentElement ||
               document.head ||
               document.body
            ).appendChild(siteRulesEl);
         }
      }
      if (siteRulesEl.textContent !== combinedCss) {
         siteRulesEl.textContent = combinedCss;
      }
   }

   function applyPageRules() {
      if (!pageRulesEl || !pageRulesEl.isConnected) {
         pageRulesEl = document.getElementById('__fynexo_page_rules__');
         if (!pageRulesEl) {
            pageRulesEl = document.createElement('style');
            pageRulesEl.id = '__fynexo_page_rules__';
            (
               document.documentElement ||
               document.head ||
               document.body
            ).appendChild(pageRulesEl);
         }
      }

      const topH = computeDesktopTopHeight();
      const mobH = styles.mobile_bar.height || '54px';
      const deskTopVisible =
         styles.topbar.visible_on_desktop !== false && !isFullscreen;
      const mobTopVisible =
         styles.topbar.visible_on_mobile === true && !isFullscreen;
      const mobBarVisible =
         styles.mobile_bar.visible !== false && !isFullscreen;
      const mobPos = (styles.mobile_bar.position || 'bottom').toLowerCase();

      const highlightCss = `
            ::highlight(fnx-find) {
                background-color: rgba(250, 204, 21, 0.38);
                color: inherit;
            }
            ::highlight(fnx-find-active) {
                background-color: #f59e0b;
                color: #09090b;
            }
        `;

      const perfCss = '';

      const isDarkAlready = isPageAlreadyDark();
      const forceDarkCss =
         config.force_dark && !isInternalAny() && !isDarkAlready
            ? `
            html:not([data-fynexo-internal]) {
                background-color: #111113 !important;
            }
            html:not([data-fynexo-internal]) > body {
                filter: invert(0.92) hue-rotate(180deg) !important;
                background-color: #ffffff;
            }
            html:not([data-fynexo-internal]) > body img,
            html:not([data-fynexo-internal]) > body video,
            html:not([data-fynexo-internal]) > body picture,
            html:not([data-fynexo-internal]) > body canvas,
            html:not([data-fynexo-internal]) > body iframe,
            html:not([data-fynexo-internal]) > body svg:not(:root),
            html:not([data-fynexo-internal]) > body [style*="background-image"] {
                filter: invert(1) hue-rotate(180deg) !important;
            }
        `
            : '';

      const adblockCss =
         config.adblock !== false && !isInternalAny()
            ? `
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
            ytd-compact-promoted-video-renderer {
                display: none !important;
                height: 0 !important;
                min-height: 0 !important;
                pointer-events: none !important;
            }
        `
            : '';

      const isMob = isMobilePlatform();
      const deskOffsetCss =
         deskTopVisible && !isMob
            ? `
            :root {
                --fnx-host-top-offset: ${topH};
            }
            ::-webkit-scrollbar {
                width: 10px !important;
                height: 10px !important;
            }
            ::-webkit-scrollbar-button {
                display: none !important;
                width: 0 !important;
                height: 0 !important;
            }
            ::-webkit-scrollbar-track {
                background: transparent !important;
            }
            ::-webkit-scrollbar-thumb {
                background: rgba(128, 128, 128, 0.38) !important;
                border-radius: 5px !important;
                border: 2px solid transparent !important;
                background-clip: content-box !important;
            }
            ::-webkit-scrollbar-thumb:hover {
                background: rgba(128, 128, 128, 0.65) !important;
                border: 2px solid transparent !important;
                background-clip: content-box !important;
            }
            ::-webkit-scrollbar-corner {
                background: transparent !important;
            }
            html:not([data-fynexo-internal]) {
                overflow: hidden !important;
                height: 100vh !important;
                max-height: 100vh !important;
            }
            html:not([data-fynexo-internal]) > body {
                position: fixed !important;
                top: ${topH} !important;
                left: 0 !important;
                right: 0 !important;
                bottom: 0 !important;
                overflow-y: auto !important;
                overflow-x: auto !important;
            }

            /* Game & Interactive Canvas Viewport Isolation (Zero-Scroll / Lock Viewport) */
            html.fnx-game-mode,
            html.fnx-game-mode > body,
            html > body.fnx-game-mode,
            html:not([data-fynexo-internal]) > body.fnx-game-mode,
            html.fnx-game-mode:not([data-fynexo-internal]) > body {
                overflow: hidden !important;
                overflow-x: hidden !important;
                overflow-y: hidden !important;
            }
            html[data-fynexo-internal] {
                overflow: hidden !important;
                height: 100vh !important;
                max-height: 100vh !important;
                --fnx-host-top-offset: 0px !important;
            }
            html[data-fynexo-internal] > body {
                position: fixed !important;
                top: ${topH} !important;
                left: 0 !important;
                right: 0 !important;
                bottom: 0 !important;
                width: 100% !important;
                max-width: 100% !important;
                height: calc(100vh - ${topH}) !important;
                max-height: calc(100vh - ${topH}) !important;
                overflow-y: auto !important;
                overflow-x: hidden !important;
                box-sizing: border-box !important;
            }
            /* YouTube Desktop Layout Harmonization & Fixes */
            ytd-app {
                --ytd-masthead-height: 56px !important;
            }
            /* Compensate fixed body top offset for absolute-positioned inline hover preview */
            ytd-video-preview {
                margin-top: calc(var(--fnx-host-top-offset, ${topH}) * -1) !important;
            }
            #masthead-container {
                top: var(--fnx-host-top-offset, ${topH}) !important;
                background-color: var(--yt-spec-base-background, #0f0f0f);
            }
            ytd-mini-guide-renderer,
            ytd-app[guide-persistent-and-visible] #guide {
                top: calc(var(--fnx-host-top-offset, ${topH}) + 56px) !important;
                bottom: 0 !important;
                height: calc(100vh - var(--fnx-host-top-offset, ${topH}) - 56px) !important;
                max-height: calc(100vh - var(--fnx-host-top-offset, ${topH}) - 56px) !important;
            }
            ytd-app[guide-persistent-and-visible] #guide-wrapper {
                top: 0 !important;
                height: 100% !important;
                max-height: 100% !important;
            }
            /* Drawer mode: starts at topH and overlays masthead with 0 duplicate headers */
            tp-yt-app-drawer#guide,
            ytd-app:not([guide-persistent-and-visible]) #guide,
            tp-yt-app-drawer {
                top: var(--fnx-host-top-offset, ${topH}) !important;
                bottom: 0 !important;
                height: calc(100vh - var(--fnx-host-top-offset, ${topH})) !important;
                max-height: calc(100vh - var(--fnx-host-top-offset, ${topH})) !important;
                --app-drawer-content-padding: 0px !important;
                z-index: 2200 !important;
            }
            ytd-app:not([guide-persistent-and-visible]) #guide-wrapper {
                top: 0 !important;
                height: 100% !important;
                max-height: 100% !important;
            }
            ytd-app #guide-content {
                flex: 1 1 auto !important;
                min-height: 0 !important;
                max-height: 100% !important;
            }
            ytd-watch-grid #fixed-secondary,
            ytd-watch-grid #panels.ytd-watch-grid,
            ytd-watch-grid ytd-watch-engagement-panels,
            ytd-watch-flexy #panels.ytd-watch-flexy,
            ytd-watch-flexy #playlist.ytd-watch-flexy {
                top: calc(var(--fnx-host-top-offset, ${topH}) + 56px) !important;
                max-height: calc(100vh - var(--fnx-host-top-offset, ${topH}) - 56px) !important;
            }
            /* GitHub Primer Global Navigation Drawer & SidePanels Harmonization */
            dialog#dialog-global-nav,
            modal-dialog[open] > dialog,
            dialog.Overlay--drawer,
            dialog.Overlay--placement-left,
            dialog.Overlay--placement-right,
            dialog.SidePanel,
            .Overlay.SidePanel,
            .Overlay--drawer,
            .Overlay--placement-left,
            .Overlay--placement-right,
            [data-component="SidePanel"],
            [data-testid="global-navigation-modal"],
            div[role="dialog"][aria-label*="navigation" i],
            div[role="dialog"]:has([data-testid="global-navigation-modal"]),
            [class*="prc-Dialog-Dialog"]:has([data-testid="global-navigation-modal"]),
            aside.SidePanel {
                top: var(--fnx-host-top-offset, ${topH}) !important;
                inset-block-start: var(--fnx-host-top-offset, ${topH}) !important;
                bottom: 0 !important;
                inset-block-end: 0 !important;
                height: calc(100vh - var(--fnx-host-top-offset, ${topH})) !important;
                max-height: calc(100vh - var(--fnx-host-top-offset, ${topH})) !important;
                margin-top: 0 !important;
                margin-bottom: 0 !important;
                margin-block-start: 0 !important;
                margin-block-end: 0 !important;
            }
            dialog#dialog-global-nav::backdrop,
            dialog.SidePanel::backdrop,
            dialog.Overlay--placement-left::backdrop,
            dialog.Overlay--placement-right::backdrop,
            .Overlay-backdrop,
            .Overlay-backdrop--side,
            [class*="prc-Dialog-Backdrop"],
            [class*="Overlay-backdrop"] {
                top: var(--fnx-host-top-offset, ${topH}) !important;
                inset-block-start: var(--fnx-host-top-offset, ${topH}) !important;
                bottom: 0 !important;
                inset-block-end: 0 !important;
                height: calc(100vh - var(--fnx-host-top-offset, ${topH})) !important;
            }
        `
            : `
            :root { --fnx-host-top-offset: 0px; }
        `;

      let mobPaddingRules = '';
      if (mobTopVisible) {
         mobPaddingRules += `padding-top: ${topH} !important;`;
      }
      if (mobBarVisible) {
         if (mobPos === 'top') {
            mobPaddingRules += `padding-top: calc(${mobH} + env(safe-area-inset-top, 0px)) !important;`;
         } else {
            mobPaddingRules += `padding-bottom: calc(${mobH} + max(env(safe-area-inset-bottom, 0px), 12px)) !important;`;
         }
      }

      const mobOffsetCss = isMob
         ? `
            :root {
                --fnx-host-top-offset: ${mobTopVisible ? topH : mobBarVisible && mobPos === 'top' ? mobH : '0px'};
                --fnx-host-bottom-offset: ${mobBarVisible && mobPos !== 'top' ? mobH : '0px'};
            }
            html:not([data-fynexo-internal]) body:not(.fnx-kb-active) {
                ${mobPaddingRules}
            }
            html:not([data-fynexo-internal]) body.fnx-kb-active {
                padding-bottom: 0px !important;
            }
        `
         : '';

      const nextCss =
         highlightCss +
         perfCss +
         forceDarkCss +
         adblockCss +
         deskOffsetCss +
         mobOffsetCss;
      if (pageRulesEl.textContent !== nextCss) {
         pageRulesEl.textContent = nextCss;
      }
   }

   function applyShadowStyles() {
      if (!shadowRoot) return;
      const p = resolvePalette();

      let dynStyle = shadowRoot.getElementById('__fynexo_dynamic_styles__');
      if (!dynStyle) {
         dynStyle = document.createElement('style');
         dynStyle.id = '__fynexo_dynamic_styles__';
         shadowRoot.appendChild(dynStyle);
      }

      const tb = styles.topbar;
      const ts = styles.tabs;
      const ab = styles.address_bar;
      const sm = styles.search_modal;
      const mb = styles.mobile_bar;
      const td = styles.tab_drawer;

      const isDeck = (tb.layout || 'deck').toLowerCase() !== 'compact';
      const bmShown = isBookmarksBarShown();
      const mobIsTop = (mb.position || 'bottom').toLowerCase() === 'top';
      const showDeskTop =
         tb.visible_on_desktop !== false && (!isFullscreen || isTopbarPeek);
      const liveTopOffset = showDeskTop ? computeDesktopTopHeight() : '0px';
      const addrWidth = ab.width || '340px';
      const tabRadius = ts.border_radius || '6px';

      dynStyle.textContent = `
            :host, #__fynexo_shell__ {
                --fnx-bg: ${p.bg};
                --fnx-panel: ${p.panel};
                --fnx-surface: ${p.surface};
                --fnx-elevated: ${p.elevated};
                --fnx-border: ${p.border};
                --fnx-border-hover: ${p.borderHover};
                --fnx-fg: ${p.fg};
                --fnx-secondary: ${p.secondary};
                --fnx-muted: ${p.muted};
                --fnx-accent: ${p.accent};
                --fnx-accent-bg: ${p.accentBg};
                --fnx-hover: ${p.hover};
                --fnx-tab-active: ${p.tabActive};
                --fnx-tab-inactive: ${p.tabInactive};
                --fnx-topbar-live: ${liveTopOffset};
            }

            #__fynexo_topbar__ {
                background: ${tb.background || 'var(--fnx-panel)'};
                border-bottom: ${tb.border_bottom && tb.border_bottom !== '1px solid var(--fnx-border)' ? tb.border_bottom : '1px solid color-mix(in srgb, var(--fnx-fg) 10%, var(--fnx-panel))'};
                ${tb.custom_css || ''}
            }

            #__fynexo_tab_row__ {
                display: ${isDeck ? 'flex' : 'none'};
                height: ${tb.titlebar_height || '42px'};
                padding: 0 0 0 12px;
                background: var(--fnx-bg);
                border-bottom: 1px solid rgba(128, 128, 128, 0.14);
            }

            #__fynexo_win_controls__ {
                height: ${isDeck ? tb.titlebar_height || '42px' : '100%'};
            }

            #__fynexo_nav_row__ {
                height: ${tb.height || '40px'};
                padding: 0 ${isDeck ? tb.padding_x || '8px' : '0'} 0 ${isDeck ? '16px' : '0'};
                background: ${tb.background || 'var(--fnx-panel)'};
            }

            #__fynexo_bm_row__ {
                display: ${bmShown ? 'flex' : 'none'};
                height: 28px;
                padding: 0 ${tb.padding_x || '8px'} 0 12px;
                background: ${tb.background || 'var(--fnx-panel)'};
                border-top: 1px solid rgba(128, 128, 128, 0.14);
            }

            .fnx-icon-btn {
                width: ${tb.button_size || '28px'};
                height: ${tb.button_size || '28px'};
                border-radius: ${tb.button_radius || '7px'};
            }

            #__fynexo_tabs_container__ {
                gap: ${typeof ts.gap === 'string' && ts.gap !== '' ? ts.gap : '4px'};
                align-items: center;
                height: 100%;
                padding: 0;
            }

            .fnx-tab-item {
                height: ${ts.height || (isDeck ? '30px' : '28px')};
                width: ${ts.width || '204px'};
                flex: 0 1 ${ts.width || '204px'};
                max-width: ${ts.max_width || ts.width || '204px'};
                min-width: ${ts.min_width || '88px'};
                border-radius: ${tabRadius};
                font-size: ${ts.font_size || '12.5px'};
                margin: 0;
                box-sizing: border-box;
                ${ts.custom_css || ''}
            }

            .fnx-tab-item:not(.active) {
                height: ${ts.height || (isDeck ? '30px' : '28px')};
                background: var(--fnx-tab-inactive);
                border: 1px solid transparent;
                color: color-mix(in srgb, var(--fnx-fg) 68%, var(--fnx-bg));
            }

            .fnx-tab-item.active {
                height: ${ts.height || (isDeck ? '30px' : '28px')};
                border: 1px solid var(--fnx-border);
                border-top-color: var(--fnx-border-hover);
                background: var(--fnx-elevated);
                color: var(--fnx-fg);
                z-index: 2;
            }

            .fnx-tab-item.pinned {
                width: 32px !important;
                min-width: 32px !important;
                max-width: 32px !important;
                flex: 0 0 32px !important;
                padding: 0 !important;
                justify-content: center !important;
            }

            .fnx-tab-monogram {
                display: ${ts.show_indicator_dot !== false ? 'inline-flex' : 'none'};
                width: ${ts.icon_size || '16px'};
                height: ${ts.icon_size || '16px'};
                font-size: ${ts.icon_font_size || '12px'};
            }
            .fnx-tab-item.pinned .fnx-tab-monogram {
                display: inline-flex !important;
            }

            .fnx-tab-item:not(.active) .fnx-tab-close {
                ${
                   ts.close_on_hover_only !== false
                      ? 'width: 0; opacity: 0; margin-left: -4px; overflow: hidden; pointer-events: none;'
                      : 'opacity: 0.62; pointer-events: auto;'
                }
            }
            .fnx-tab-item:not(.active):hover .fnx-tab-close {
                width: 16px;
                opacity: 0.85;
                margin-left: 0;
                pointer-events: auto;
            }

            #__fynexo_addr_slot__ {
                height: ${ab.height || '29px'};
                width: 100%;
                max-width: ${isDeck ? 'clamp(320px, 46vw, 560px)' : '38vw'};
                border-radius: ${ab.border_radius || '8px'};
                background: ${ab.background || 'var(--fnx-surface)'};
                border: ${ab.border || '1px solid var(--fnx-border)'};
                font-size: ${ab.font_size || '12.5px'};
                box-sizing: border-box;
                ${ab.custom_css || ''}
            }
            #__fynexo_addr_slot__.editing {
                width: 100%;
                max-width: ${isDeck ? 'clamp(360px, 54vw, 640px)' : '46vw'};
                border-color: var(--fnx-accent);
                background: var(--fnx-bg);
            }

            #__fynexo_address_btn__,
            #__fynexo_inline_box__ {
                height: 100%;
                font-size: ${ab.font_size || '12.5px'};
            }

            #__fynexo_hud_root__ {
                background: ${sm.backdrop_color || 'rgba(0, 0, 0, 0.56)'};
            }

            #__fynexo_hud_card__ {
                width: ${sm.width || '580px'};
                margin-top: ${sm.top_offset || '66px'};
                border-radius: ${sm.border_radius || '12px'};
                background: ${sm.background || 'var(--fnx-panel)'};
                border: ${sm.border || '1px solid var(--fnx-border-hover)'};
                ${sm.custom_css || ''}
            }

            #__fynexo_input__ {
                font-size: ${sm.input_font_size || '14.5px'};
            }

            #__fynexo_bottom_bar__ {
                height: calc(${mb.height || '54px'} + ${mobIsTop ? 'env(safe-area-inset-top, 0px)' : 'max(env(safe-area-inset-bottom, 0px), 12px)'});
                padding-left: max(env(safe-area-inset-left, 0px), ${mb.padding_x || '8px'});
                padding-right: max(env(safe-area-inset-right, 0px), ${mb.padding_x || '8px'});
                padding-bottom: ${mobIsTop ? '0px' : 'max(env(safe-area-inset-bottom, 0px), 12px)'};
                padding-top: ${mobIsTop ? 'env(safe-area-inset-top, 0px)' : '0px'};
                top: ${mobIsTop ? '0' : 'auto'};
                bottom: ${mobIsTop ? 'auto' : '0'};
                background: ${mb.background || 'var(--fnx-panel)'};
                border-top: ${mobIsTop ? 'none' : mb.border || '1px solid var(--fnx-border)'};
                border-bottom: ${mobIsTop ? mb.border || '1px solid var(--fnx-border)' : 'none'};
                ${mb.custom_css || ''}
            }

            .fnx-mb-btn {
                width: ${mb.button_size || '38px'};
                height: ${mb.button_size || '38px'};
            }

            #__fynexo_mb_pill__ {
                height: ${mb.pill_height || '38px'};
                border-radius: ${mb.pill_radius || '11px'};
            }

            #__fynexo_drawer_sheet__ {
                max-height: ${td.max_height || '80vh'};
                border-radius: ${td.border_radius || '16px'} ${td.border_radius || '16px'} 0 0;
            }

            .fnx-drawer-card {
                border-radius: ${td.card_radius || '10px'};
                padding: ${td.card_padding || '11px 13px'};
                ${td.custom_css || ''}
            }

            ${
               !isMobilePlatform()
                  ? `
                #__fynexo_topbar__ {
                    display: ${showDeskTop ? 'flex' : 'none'} !important;
                }
                #__fynexo_bottom_bar__ {
                    display: none !important;
                }
            `
                  : `
                #__fynexo_topbar__ {
                    display: ${tb.visible_on_mobile === true && !isFullscreen ? 'flex' : 'none'} !important;
                }
                #__fynexo_bottom_bar__ {
                    display: ${mb.visible !== false && !isPageKeyboardOpen && !isFullscreen ? 'flex' : 'none'} !important;
                }
                #__fynexo_hud_card__ {
                    width: calc(100vw - 20px) !important;
                    max-width: calc(100vw - 20px) !important;
                    margin-top: 10px !important;
                }
            `
            }
        `;

      // Sync layout between 2-Row Layered Deck ("deck") and Single-Row ("compact")
      const tabRow = shadowRoot.getElementById('__fynexo_tab_row__');
      const navRow = shadowRoot.getElementById('__fynexo_nav_row__');
      const tabsDeckCluster = shadowRoot.getElementById(
         '__fynexo_tabs_deck_cluster__',
      );
      const winControls = shadowRoot.getElementById('__fynexo_win_controls__');
      const addrSlot = shadowRoot.getElementById('__fynexo_addr_slot__');
      const leftZone = shadowRoot.getElementById('__fynexo_left_zone__');
      const centerZone = shadowRoot.getElementById('__fynexo_center_zone__');
      const rightZone = shadowRoot.getElementById('__fynexo_right_zone__');

      if (
         tabRow &&
         navRow &&
         tabsDeckCluster &&
         winControls &&
         addrSlot &&
         leftZone &&
         centerZone &&
         rightZone
      ) {
         if (isDeck) {
            if (tabsDeckCluster.parentNode !== tabRow) {
               tabRow.insertBefore(tabsDeckCluster, winControls);
            }
            if (winControls.parentNode !== tabRow) {
               tabRow.appendChild(winControls);
            }
            if (addrSlot.parentNode !== centerZone) {
               centerZone.appendChild(addrSlot);
            }
         } else {
            if (tabsDeckCluster.parentNode !== centerZone) {
               centerZone.insertBefore(tabsDeckCluster, centerZone.firstChild);
            }
            if (winControls.parentNode !== rightZone) {
               rightZone.appendChild(winControls);
            }
            const pos = (ab.position || 'right').toLowerCase();
            const targetParent =
               pos === 'left'
                  ? leftZone
                  : pos === 'center'
                    ? centerZone
                    : rightZone;
            if (addrSlot.parentNode !== targetParent) {
               if (pos === 'left') leftZone.appendChild(addrSlot);
               else
                  targetParent.insertBefore(addrSlot, targetParent.firstChild);
            }
         }
      }
   }

   function renderBookmarksBar() {
      if (!shadowRoot) return;
      const bmList = shadowRoot.getElementById('__fynexo_bm_list__');
      if (!bmList) return;
      bmList.textContent = '';

      const frag = document.createDocumentFragment();
      const items = Array.isArray(bookmarks) ? bookmarks : [];

      if (items.length === 0) {
         const emptyHint = document.createElement('span');
         emptyHint.style.cssText =
            'font-size: 11.5px; color: var(--fnx-muted); padding: 0 6px;';
         emptyHint.textContent =
            'No shortcuts saved — click the star icon in the address bar to pin sites here';
         frag.appendChild(emptyHint);
      } else {
         items.forEach((b) => {
            if (!b || !b.url) return;
            const pill = document.createElement('button');
            pill.type = 'button';
            pill.className = 'fnx-bm-pill';
            const label = b.title || formatHostLabel(b.url);
            pill.title = `${label}\n${b.url} (Middle-click for new tab · Right-click to remove)`;

            const iconBadge = document.createElement('span');
            iconBadge.className = 'fnx-bm-icon';
            renderSiteEmblem(iconBadge, b.url, label);

            const textSpan = document.createElement('span');
            textSpan.className = 'fnx-bm-label';
            textSpan.textContent = label;

            pill.appendChild(iconBadge);
            pill.appendChild(textSpan);

            pill.onclick = (e) => {
               flushScrollSync();
               triggerProgress();
               if (e.ctrlKey || e.metaKey) {
                  sendIpc({ type: 'NEW_TAB', url: b.url, title: label });
               } else {
                  sendIpc({ type: 'NAVIGATE', url: b.url });
               }
            };
            pill.onmousedown = (e) => {
               if (e.button === 1) e.preventDefault();
            };
            pill.onauxclick = (e) => {
               if (e.button === 1) {
                  e.preventDefault();
                  triggerProgress();
                  sendIpc({ type: 'NEW_TAB', url: b.url, title: label });
               }
            };
            pill.oncontextmenu = (e) => {
               e.preventDefault();
               e.stopPropagation();
               sendIpc({ type: 'TOGGLE_BOOKMARK', title: label, url: b.url });
               showHudToast(`Removed "${label}" from shortcuts`);
            };

            frag.appendChild(pill);
         });
      }

      bmList.appendChild(frag);
   }

   function syncAllStyles() {
      applyPageRules();
      applySiteRules();
      applyShadowStyles();
      renderTabs(false);
      renderBookmarksBar();
      updateAddressLabels();
   }

   function showHudToast(text) {
      if (!shadowRoot || !text) return;
      const toastEl = shadowRoot.getElementById('__fynexo_hud_toast__');
      const msgEl = shadowRoot.getElementById('__fynexo_hud_toast_msg__');
      if (!toastEl || !msgEl) return;
      msgEl.textContent = text;
      toastEl.style.display = 'inline-flex';
      if (hudToastTimer) clearTimeout(hudToastTimer);
      hudToastTimer = setTimeout(() => {
         toastEl.style.display = 'none';
      }, 1900);
   }

   function flushScrollSync() {
      if (!isBackgroundParked && !isInternalAny()) {
         const y =
            (document.body ? document.body.scrollTop : 0) ||
            window.scrollY ||
            window.pageYOffset ||
            (document.documentElement
               ? document.documentElement.scrollTop
               : 0) ||
            0;
         if (y > 24) {
            sendIpc({ type: 'SAVE_SCROLL', scroll_y: y });
         }
      }
   }

   function updateProtocolIcon(containerEl, rawUrl) {
      if (!containerEl) return;
      const isSet = isInternalSettings(rawUrl);
      const isNt = isInternalNewtab(rawUrl);
      const isHttps = rawUrl && rawUrl.startsWith('https://');
      const kind = isSet
         ? 'settings'
         : isNt
           ? 'search'
           : isHttps
             ? 'lock'
             : 'search';
      if (containerEl.dataset.iconKind === kind) return;
      containerEl.dataset.iconKind = kind;
      containerEl.textContent = '';
      containerEl.appendChild(createIcon(kind));
      containerEl.style.color = isSet
         ? 'var(--fnx-accent)'
         : 'var(--fnx-muted)';
   }

   function syncBookmarkButtons() {
      if (!shadowRoot) return;
      const curUrl = canonicalCurrentUrl();
      const internal =
         isInternalAny(curUrl) || !curUrl || curUrl === 'about:blank';
      const starred = !internal && isUrlBookmarked(curUrl);

      const deskCopy = shadowRoot.getElementById('__fynexo_desk_copy__');
      if (deskCopy) {
         deskCopy.style.display = internal ? 'none' : 'flex';
      }

      const addrActions = shadowRoot.getElementById('__fynexo_addr_actions__');
      if (addrActions) {
         addrActions.style.display = internal ? 'none' : 'flex';
      }

      const deskStar = shadowRoot.getElementById('__fynexo_desk_star__');
      if (deskStar) {
         deskStar.style.display = internal ? 'none' : 'flex';
         const nextKind = starred ? 'starFilled' : 'star';
         if (deskStar.dataset.starKind !== nextKind) {
            deskStar.dataset.starKind = nextKind;
            deskStar.textContent = '';
            deskStar.appendChild(createIcon(nextKind));
         }
         deskStar.classList.toggle('active-route', starred);
         deskStar.title = starred
            ? `Remove shortcut (${keybinds.bookmark_page || 'Ctrl+D'})`
            : `Save shortcut (${keybinds.bookmark_page || 'Ctrl+D'})`;
      }

      cachedReaderable = !internal && isPageReaderable();
      const deskReader = shadowRoot.getElementById('__fynexo_desk_reader__');
      if (deskReader) {
         deskReader.style.display =
            !internal && (cachedReaderable || isReaderOpen) ? 'flex' : 'none';
         deskReader.classList.toggle('active-route', isReaderOpen);
         deskReader.title = `Reader mode (${keybinds.reader_mode || 'Alt+R'})`;
      }

      const deskDark = shadowRoot.getElementById('__fynexo_desk_dark__');
      if (deskDark) {
         deskDark.classList.toggle('active-route', !!config.force_dark);
         deskDark.title = `Smart Force Dark: ${config.force_dark ? 'On' : 'Off'} (${keybinds.force_dark || 'Alt+Shift+D'})`;
      }

      const bmAddBtn = shadowRoot.getElementById('__fynexo_bm_add__');
      if (bmAddBtn) {
         bmAddBtn.textContent = '';
         if (internal) {
            bmAddBtn.appendChild(createIcon('settings'));
            bmAddBtn.appendChild(document.createTextNode('Manage'));
            bmAddBtn.title = 'Manage shortcuts in Studio Preferences';
         } else {
            bmAddBtn.appendChild(createIcon(starred ? 'starFilled' : 'plus'));
            bmAddBtn.appendChild(
               document.createTextNode(starred ? 'Saved' : 'Save page'),
            );
            bmAddBtn.title = starred
               ? 'Remove current page from shortcuts'
               : 'Pin current page to shortcuts bar';
         }
      }

      const incogBadge = shadowRoot.getElementById('__fynexo_incog_badge__');
      if (incogBadge) {
         incogBadge.style.display = config.incognito ? 'inline-flex' : 'none';
      }

      const sheetStar = shadowRoot.getElementById('__fynexo_sheet_star__');
      if (sheetStar) {
         sheetStar.style.display = internal ? 'none' : 'flex';
         const nextKind = starred ? 'starFilled' : 'star';
         if (sheetStar.dataset.starKind !== nextKind) {
            sheetStar.dataset.starKind = nextKind;
            sheetStar.textContent = '';
            sheetStar.appendChild(createIcon(nextKind));
         }
         sheetStar.style.color = starred
            ? 'var(--fnx-accent)'
            : 'var(--fnx-secondary)';
         sheetStar.style.borderColor = starred
            ? 'var(--fnx-accent)'
            : 'var(--fnx-border)';
      }
   }

   function toggleCurrentBookmark() {
      const curUrl = canonicalCurrentUrl();
      if (!curUrl || isInternalAny(curUrl) || curUrl === 'about:blank') return;
      const curTitle = document.title || formatHostLabel(curUrl);
      sendIpc({ type: 'TOGGLE_BOOKMARK', title: curTitle, url: curUrl });
   }

   function updateAddressLabels() {
      if (!shadowRoot) return;
      const href = canonicalCurrentUrl();
      const label = formatHostLabel(href);
      const isSet = isInternalSettings(href);
      const isNt = isInternalNewtab(href);
      const engName = currentEngineObj().name || 'Google';

      const schemeEl = shadowRoot.getElementById('__fynexo_addr_scheme__');
      const domEl = shadowRoot.getElementById('__fynexo_addr_domain__');
      const pathEl = shadowRoot.getElementById('__fynexo_addr_path__');
      if (domEl && pathEl) {
         if (isNt || !href || href === 'about:blank') {
            if (schemeEl) schemeEl.textContent = '';
            domEl.textContent = '';
            pathEl.textContent = `Search ${engName} or type a URL...`;
            pathEl.style.color = 'var(--fnx-muted)';
         } else if (isSet) {
            if (schemeEl) schemeEl.textContent = 'fynexo://';
            domEl.textContent = 'settings';
            domEl.style.color = 'var(--fnx-fg)';
            pathEl.textContent = ' · Studio Preferences';
            pathEl.style.color = 'var(--fnx-muted)';
         } else {
            try {
               const u = new URL(href);
               if (schemeEl) {
                  schemeEl.textContent =
                     u.protocol === 'https:' ? '' : u.protocol + '//';
               }
               const rawHost = u.host || label;
               domEl.textContent = rawHost.replace(/^www\./i, '');
               domEl.style.color = 'var(--fnx-fg)';
               const tail =
                  (u.pathname && u.pathname !== '/' ? u.pathname : '') +
                  (u.search || '') +
                  (u.hash || '');
               pathEl.textContent = tail;
               pathEl.style.color = 'var(--fnx-muted)';
            } catch (_) {
               if (schemeEl) schemeEl.textContent = '';
               domEl.textContent = label;
               domEl.style.color = 'var(--fnx-fg)';
               pathEl.textContent = '';
            }
         }
      }

      const mobLabel = shadowRoot.getElementById('__fynexo_mb_label__');
      if (mobLabel) {
         const mobText =
            isNt || !href || href === 'about:blank'
               ? `Search ${engName} or type URL`
               : label;
         if (mobLabel.textContent !== mobText) mobLabel.textContent = mobText;
      }

      const addrHint = shadowRoot.getElementById('__fynexo_addr_hint__');
      if (addrHint) {
         addrHint.textContent = String(keybinds.omnibox || 'Ctrl+L').replace(
            /\+/g,
            ' ',
         );
      }

      updateProtocolIcon(
         shadowRoot.getElementById('__fynexo_desk_proto__'),
         href,
      );
      updateProtocolIcon(
         shadowRoot.getElementById('__fynexo_mb_proto__'),
         href,
      );

      const deskPrefs = shadowRoot.getElementById('__fynexo_desk_prefs__');
      if (deskPrefs) deskPrefs.classList.toggle('active-route', isSet);
      const mobPrefs = shadowRoot.getElementById('__fynexo_drawer_prefs__');
      if (mobPrefs) mobPrefs.classList.toggle('active-route', isSet);

      syncBookmarkButtons();

      const engineBadge =
         shadowRoot.getElementById('__fynexo_hud_engine__') ||
         shadowRoot.getElementById('__fynexo_inline_engine__');
      if (engineBadge) {
         engineBadge.textContent = engName;
      }
   }

   // Pure-DOM Find in Page (CSS.highlights + TreeWalker, fallback window.find)
   function clearFindHighlights() {
      findRanges = [];
      findIdx = -1;
      try {
         if (typeof CSS !== 'undefined' && CSS.highlights) {
            CSS.highlights.delete('fnx-find');
            CSS.highlights.delete('fnx-find-active');
         }
      } catch (_) {}
      if (shadowRoot) {
         const cnt = shadowRoot.getElementById('__fynexo_find_count__');
         if (cnt) cnt.textContent = '0/0';
      }
   }

   function scrollRangeIntoView(range) {
      if (!range) return;
      try {
         const rect = range.getBoundingClientRect();
         if (!rect || (rect.width === 0 && rect.height === 0)) return;
         const margin = 90;
         if (rect.top < margin || rect.bottom > window.innerHeight - margin) {
            const targetY = Math.max(
               0,
               (window.scrollY || window.pageYOffset || 0) +
                  rect.top -
                  window.innerHeight * 0.35,
            );
            window.scrollTo({ top: targetY, behavior: 'smooth' });
         }
      } catch (_) {}
   }

   function runFindQuery(rawQuery) {
      if (!shadowRoot) return;
      const cnt = shadowRoot.getElementById('__fynexo_find_count__');
      const q = String(rawQuery || '');
      if (!q) {
         clearFindHighlights();
         return;
      }

      if (
         typeof CSS !== 'undefined' &&
         CSS.highlights &&
         typeof Highlight !== 'undefined' &&
         document.body
      ) {
         const needle = q.toLowerCase();
         const nLen = needle.length;
         const matches = [];
         const walker = document.createTreeWalker(
            document.body,
            NodeFilter.SHOW_TEXT,
         );
         let node;
         while ((node = walker.nextNode())) {
            const p = node.parentElement;
            if (!p) continue;
            const tag = p.tagName;
            if (
               tag === 'SCRIPT' ||
               tag === 'STYLE' ||
               tag === 'NOSCRIPT' ||
               tag === 'TEMPLATE'
            )
               continue;
            if (p.closest && p.closest('#__fynexo_host__')) continue;
            const val = node.nodeValue;
            if (!val || val.length < nLen) continue;
            const lower = val.toLowerCase();
            let pos = 0;
            while ((pos = lower.indexOf(needle, pos)) !== -1) {
               const r = document.createRange();
               r.setStart(node, pos);
               r.setEnd(node, pos + nLen);
               matches.push(r);
               pos += nLen;
               if (matches.length >= 400) break;
            }
            if (matches.length >= 400) break;
         }

         findRanges = matches;
         if (matches.length === 0) {
            findIdx = -1;
            CSS.highlights.delete('fnx-find');
            CSS.highlights.delete('fnx-find-active');
            if (cnt) cnt.textContent = '0/0';
            return;
         }

         findIdx = 0;
         CSS.highlights.set('fnx-find', new Highlight(...matches));
         CSS.highlights.set('fnx-find-active', new Highlight(matches[0]));
         if (cnt) cnt.textContent = `1/${matches.length}`;
         scrollRangeIntoView(matches[0]);
      } else if (typeof window.find === 'function') {
         const ok = window.find(q, false, false, true, false, false, false);
         if (cnt) cnt.textContent = ok ? 'Match' : '0/0';
      }
   }

   function stepFindMatch(delta) {
      if (!shadowRoot) return;
      const inp = shadowRoot.getElementById('__fynexo_find_input__');
      const cnt = shadowRoot.getElementById('__fynexo_find_count__');
      const q = inp ? inp.value : '';
      if (!q) return;

      if (
         findRanges.length > 0 &&
         typeof CSS !== 'undefined' &&
         CSS.highlights &&
         typeof Highlight !== 'undefined'
      ) {
         findIdx = (findIdx + delta + findRanges.length) % findRanges.length;
         const active = findRanges[findIdx];
         CSS.highlights.set('fnx-find-active', new Highlight(active));
         if (cnt) cnt.textContent = `${findIdx + 1}/${findRanges.length}`;
         scrollRangeIntoView(active);
      } else if (typeof window.find === 'function') {
         window.find(q, false, delta < 0, true, false, false, false);
      }
   }

   function openFindBar() {
      if (!shadowRoot) return;
      const bar = shadowRoot.getElementById('__fynexo_find_bar__');
      const inp = shadowRoot.getElementById('__fynexo_find_input__');
      if (!bar || !inp) return;
      isFindOpen = true;
      bar.style.display = 'flex';
      const sel = window.getSelection
         ? String(window.getSelection()).trim()
         : '';
      if (sel && sel.length < 80 && !sel.includes('\n')) {
         inp.value = sel;
      }
      requestAnimationFrame(() => {
         inp.focus();
         inp.select();
         if (inp.value) runFindQuery(inp.value);
      });
   }

   function closeFindBar() {
      if (!shadowRoot) return;
      isFindOpen = false;
      const bar = shadowRoot.getElementById('__fynexo_find_bar__');
      if (bar) bar.style.display = 'none';
      clearFindHighlights();
      restorePageFocus();
   }

   // Pure-DOM Zero-Bloat Reader Mode
   let cachedReaderable = false;
   let readerCheckTimer = null;

   function isPageReaderable() {
      if (isInternalAny() || !document.body) return false;
      if (isReaderOpen) return true;
      const host = (window.location.hostname || '').toLowerCase();
      if (
         host === 'youtube.com' ||
         host.endsWith('.youtube.com') ||
         host === 'youtu.be'
      ) {
         return false;
      }

      const root =
         document.querySelector(
            '.markdown-body, .post-content, .article-content, .entry-content, article',
         ) ||
         document.querySelector('main, [role="main"]') ||
         document.body;
      if (!root) return false;

      const pList = root.querySelectorAll('p');
      if (pList.length < 3) return false;

      const maxScan = Math.min(pList.length, 160);
      let longPCount = 0;
      let weightedChars = 0;
      const cjkRe = /[\u4e00-\u9fff\u3040-\u30ff\uac00-\ud7af]/g;

      for (let i = 0; i < maxScan; i++) {
         const p = pList[i];
         if (
            !p ||
            p.hidden ||
            p.closest(
               'nav, header, footer, aside, form, [role="navigation"], [role="menu"], [role="dialog"], [aria-hidden="true"], #__fynexo_host__',
            )
         ) {
            continue;
         }
         const txt = (p.textContent || '').replace(/\s+/g, ' ').trim();
         if (!txt || txt.length < 25) continue;
         const cjkMatches = txt.match(cjkRe);
         const cjkCount = cjkMatches ? cjkMatches.length : 0;
         const weighted = txt.length - cjkCount + cjkCount * 2.2;
         if (weighted < 35) continue;
         weightedChars += weighted;
         if (weighted >= 65) longPCount += 1;
         if (weightedChars >= 900 && longPCount >= 3) {
            return true;
         }
      }
      return false;
   }

   function refreshReaderableState() {
      const internal = isInternalAny(canonicalCurrentUrl());
      const next = !internal && isPageReaderable();
      cachedReaderable = next;
      const deskReader =
         shadowRoot && shadowRoot.getElementById('__fynexo_desk_reader__');
      if (deskReader) {
         deskReader.style.display =
            !internal && (cachedReaderable || isReaderOpen) ? 'flex' : 'none';
      }
   }

   function scheduleReaderableCheck() {
      if (readerCheckTimer) clearTimeout(readerCheckTimer);
      readerCheckTimer = setTimeout(() => {
         readerCheckTimer = null;
         refreshReaderableState();
      }, 280);
   }

   function pickArticleContainer() {
      const explicit = document.querySelector(
         'article, [role="main"], main, .post-content, .article-content, .entry-content, .markdown-body',
      );
      if (explicit && (explicit.textContent || '').trim().length > 260) {
         return explicit;
      }
      let best = document.body;
      let bestScore = 0;
      const candidates = document.querySelectorAll('section, div');
      const maxScan = Math.min(candidates.length, 220);
      for (let i = 0; i < maxScan; i++) {
         const el = candidates[i];
         const cls = ((el.className || '') + ' ' + (el.id || '')).toLowerCase();
         if (
            /(nav|footer|header|sidebar|comment|menu|modal|banner|promo|ad-)/.test(
               cls,
            )
         )
            continue;
         const pList = el.querySelectorAll('p');
         if (pList.length < 2) continue;
         let textLen = 0;
         for (let j = 0; j < pList.length; j++) {
            textLen += (pList[j].textContent || '').trim().length;
         }
         if (textLen > bestScore) {
            bestScore = textLen;
            best = el;
         }
      }
      return best;
   }

   function syncReaderTypography() {
      if (!shadowRoot) return;
      const article = shadowRoot.getElementById('__fynexo_reader_article__');
      const widthBtn = shadowRoot.getElementById('__fynexo_reader_width__');
      const sizeEl = shadowRoot.getElementById('__fynexo_reader_size__');
      if (article) {
         article.style.fontSize = readerFontPx + 'px';
         article.style.maxWidth = readerWide ? '880px' : '680px';
      }
      if (sizeEl) {
         sizeEl.textContent = readerFontPx + 'px';
      }
      if (widthBtn) {
         widthBtn.classList.toggle('active', !!readerWide);
      }
   }

   function closeReaderMode() {
      if (!shadowRoot || !isReaderOpen) return;
      isReaderOpen = false;
      const overlay = shadowRoot.getElementById('__fynexo_reader__');
      const article = shadowRoot.getElementById('__fynexo_reader_article__');
      if (overlay) overlay.style.display = 'none';
      if (article) article.textContent = '';
      if (document.documentElement)
         document.documentElement.style.overflow = '';
      try {
         window.scrollTo(0, readerSavedScroll);
      } catch (_) {}
      syncBookmarkButtons();
   }

   function toggleReaderMode() {
      if (isInternalAny()) return;
      if (isReaderOpen) {
         closeReaderMode();
         return;
      }
      if (!shadowRoot) return;
      const overlay = shadowRoot.getElementById('__fynexo_reader__');
      const article = shadowRoot.getElementById('__fynexo_reader_article__');
      const metaSpan = shadowRoot.getElementById('__fynexo_reader_meta__');
      if (!overlay || !article) return;

      const root = pickArticleContainer();
      if (!root) {
         showHudToast('No readable article found on this page');
         return;
      }

      article.textContent = '';
      const frag = document.createDocumentFragment();
      const pageTitle = (document.title || '').trim();
      let hasH1 = false;
      let charCount = 0;
      const seenText = Object.create(null);

      const blocks = root.querySelectorAll(
         'h1, h2, h3, h4, p, pre, blockquote, ul, ol, img',
      );
      const limit = Math.min(blocks.length, 450);

      for (let i = 0; i < limit; i++) {
         const node = blocks[i];
         if (
            !node ||
            (node.closest &&
               node.closest(
                  'nav, header, footer, aside, form, #__fynexo_host__',
               ))
         )
            continue;
         const tag = node.tagName.toLowerCase();
         if (
            (tag === 'p' || tag === 'blockquote') &&
            node.parentElement &&
            (node.parentElement.tagName === 'LI' ||
               node.parentElement.tagName === 'BLOCKQUOTE')
         ) {
            continue;
         }

         if (tag === 'img') {
            const src = node.currentSrc || node.getAttribute('src') || '';
            const nw =
               node.naturalWidth || Number(node.getAttribute('width')) || 0;
            if (src && /^https?:\/\//i.test(src) && nw >= 200) {
               const img = document.createElement('img');
               img.src = src;
               img.className = 'fnx-rd-img';
               img.loading = 'lazy';
               frag.appendChild(img);
            }
            continue;
         }

         if (tag === 'ul' || tag === 'ol') {
            const items = node.querySelectorAll(':scope > li');
            if (items.length === 0) continue;
            const listEl = document.createElement(tag);
            listEl.className = 'fnx-rd-list';
            let addedLi = 0;
            for (let k = 0; k < items.length && k < 60; k++) {
               const txt = (items[k].textContent || '')
                  .replace(/\s+/g, ' ')
                  .trim();
               if (txt.length < 3) continue;
               const li = document.createElement('li');
               li.textContent = txt;
               listEl.appendChild(li);
               charCount += txt.length;
               addedLi++;
            }
            if (addedLi > 0) frag.appendChild(listEl);
            continue;
         }

         const txt =
            tag === 'pre'
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
      }

      if (charCount < 120) {
         article.textContent = '';
         showHudToast('Not enough article text for Reader mode');
         return;
      }

      if (!hasH1 && pageTitle) {
         const h1 = document.createElement('h1');
         h1.className = 'fnx-rd-h1';
         h1.textContent = pageTitle;
         article.appendChild(h1);
      }
      article.appendChild(frag);

      const mins = Math.max(1, Math.round(charCount / 900));
      if (metaSpan) {
         metaSpan.textContent = `${formatHostLabel(window.location.href)} · ~${mins} min read`;
      }

      readerSavedScroll = window.scrollY || window.pageYOffset || 0;
      isReaderOpen = true;
      syncReaderTypography();
      overlay.style.display = 'flex';
      overlay.scrollTop = 0;
      if (document.documentElement)
         document.documentElement.style.overflow = 'hidden';
      syncBookmarkButtons();
   }

   function computeTabsSig() {
      let s =
         String(tabState.active_id) +
         ':' +
         String(tabState.closed_count || 0) +
         '|';
      for (let i = 0; i < tabState.tabs.length; i++) {
         const t = tabState.tabs[i];
         s +=
            t.id +
            ':' +
            (t.pinned ? '1' : '0') +
            ':' +
            (t.background_play ? '1' : '0') +
            ':' +
            (t.muted ? '1' : '0') +
            ':' +
            (t.playing_audio ? '1' : '0') +
            ':' +
            (t.suspended ? '1' : '0') +
            ':' +
            (t.title || '') +
            ':' +
            (t.url || '') +
            ';';
      }
      return s;
   }

   function renderDrawerCards() {
      if (!shadowRoot) return;
      const reopenBtn = shadowRoot.getElementById('__fynexo_drawer_reopen__');
      if (reopenBtn) {
         const cc = Number(tabState.closed_count) || 0;
         reopenBtn.style.display = cc > 0 ? 'inline-flex' : 'none';
         const reopenTxt = reopenBtn.querySelector('.fnx-drawer-reopen-txt');
         if (reopenTxt) {
            reopenTxt.textContent = cc > 1 ? `Reopen (${cc})` : 'Reopen';
         }
      }

      const sig = computeTabsSig();
      if (sig === lastDrawerTabsSig) return;
      lastDrawerTabsSig = sig;

      const drawerList = shadowRoot.getElementById('__fynexo_drawer_list__');
      if (!drawerList) return;
      drawerList.textContent = '';

      const frag = document.createDocumentFragment();
      tabState.tabs.forEach((tab, idx) => {
         const isActive = tab.id === tabState.active_id;
         const isSet = isInternalSettings(tab.url);
         const isNt = isInternalNewtab(tab.url);
         const card = document.createElement('div');
         card.className = isActive
            ? 'fnx-drawer-card active'
            : 'fnx-drawer-card';

         const badge = document.createElement('div');
         badge.className = 'fnx-drawer-monogram';
         renderSiteEmblem(badge, tab.url, tab.title);

         const meta = document.createElement('div');
         meta.className = 'fnx-drawer-meta';

         const t = document.createElement('div');
         t.className = 'fnx-drawer-title';
         const cleanTitle = isSet
            ? 'Preferences'
            : isNt
              ? 'Start page'
              : tab.title || 'New tab';
         if (tab.pinned) {
            const pinBadge = document.createElement('span');
            pinBadge.className = 'fnx-drawer-pin-icon';
            pinBadge.appendChild(createIcon('pin'));
            t.appendChild(pinBadge);
         }
         const tTxt = document.createElement('span');
         tTxt.className = 'fnx-drawer-title-text';
         tTxt.textContent = cleanTitle;
         t.appendChild(tTxt);

         const u = document.createElement('div');
         u.className = 'fnx-drawer-url';
         u.textContent = isSet
            ? 'fynexo://settings'
            : isNt
              ? 'fynexo://newtab'
              : formatHostLabel(tab.url || '');

         meta.appendChild(t);
         meta.appendChild(u);

         const pinBtn = document.createElement('button');
         pinBtn.type = 'button';
         pinBtn.className = 'fnx-drawer-close';
         pinBtn.title = tab.pinned ? 'Unpin tab' : 'Pin tab';
         pinBtn.appendChild(createIcon('pin'));
         if (tab.pinned) {
            pinBtn.style.color = 'var(--fnx-accent)';
            pinBtn.style.borderColor = 'var(--fnx-accent)';
         }
         pinBtn.onclick = (e) => {
            e.stopPropagation();
            sendIpc({ type: 'TOGGLE_PIN_TAB', id: tab.id });
         };

         card.appendChild(badge);
         card.appendChild(meta);
         card.appendChild(pinBtn);

         if (!tab.pinned) {
            const cBtn = document.createElement('button');
            cBtn.type = 'button';
            cBtn.className = 'fnx-drawer-close';
            cBtn.setAttribute('aria-label', 'Close tab');
            cBtn.appendChild(createIcon('close'));
            cBtn.onclick = (e) => {
               e.stopPropagation();
               sendIpc({ type: 'CLOSE_TAB', id: tab.id });
            };
            card.appendChild(cBtn);
         }

         let lpTimer = null;
         let lpTriggered = false;
         let lpStartX = 0;
         let lpStartY = 0;
         const clearLp = () => {
            if (lpTimer) {
               clearTimeout(lpTimer);
               lpTimer = null;
            }
         };
         card.addEventListener(
            'pointerdown',
            (e) => {
               if (e.button !== 0 || e.target.closest('button')) return;
               lpTriggered = false;
               lpStartX = e.clientX;
               lpStartY = e.clientY;
               clearLp();
               lpTimer = setTimeout(() => {
                  lpTriggered = true;
                  showTabContextMenu(
                     { clientX: lpStartX, clientY: lpStartY },
                     tab,
                     idx,
                  );
               }, 460);
            },
            { passive: true },
         );
         card.addEventListener(
            'pointermove',
            (e) => {
               if (
                  lpTimer &&
                  (Math.abs(e.clientX - lpStartX) > 8 ||
                     Math.abs(e.clientY - lpStartY) > 8)
               ) {
                  clearLp();
               }
            },
            { passive: true },
         );
         card.addEventListener('pointerup', clearLp, { passive: true });
         card.addEventListener('pointercancel', clearLp, { passive: true });
         card.oncontextmenu = (e) => {
            e.preventDefault();
            e.stopPropagation();
            clearLp();
            showTabContextMenu(e, tab, idx);
         };

         card.onclick = () => {
            if (lpTriggered) {
               lpTriggered = false;
               return;
            }
            closeMobileDrawer();
            if (!isActive) {
               flushScrollSync();
               triggerProgress();
               sendIpc({ type: 'SWITCH_TAB', id: tab.id });
            }
         };

         frag.appendChild(card);
      });
      drawerList.appendChild(frag);
   }

   let tabContextMenuEl = null;

   function hideTabContextMenu() {
      if (tabContextMenuEl) {
         tabContextMenuEl.style.display = 'none';
         tabContextMenuEl.textContent = '';
      }
   }

   function renderContextMenu(clientX, clientY, entries) {
      if (!shadowRoot || !entries || entries.length === 0) return;
      if (closeAllNavPopoversRef) closeAllNavPopoversRef();
      if (!tabContextMenuEl) {
         tabContextMenuEl = document.createElement('div');
         tabContextMenuEl.id = '__fynexo_tab_context_menu__';
      }
      const targetParent =
         shadowRoot.getElementById('__fynexo_shell__') || shadowRoot;
      targetParent.appendChild(tabContextMenuEl);
      tabContextMenuEl.textContent = '';

      // Collapse leading/trailing or duplicate separators cleanly
      const cleaned = [];
      for (let i = 0; i < entries.length; i++) {
         const entry = entries[i];
         if (!entry) continue;
         if (entry.sep) {
            if (cleaned.length > 0 && !cleaned[cleaned.length - 1].sep) {
               cleaned.push(entry);
            }
         } else {
            cleaned.push(entry);
         }
      }
      while (cleaned.length > 0 && cleaned[cleaned.length - 1].sep) {
         cleaned.pop();
      }
      if (cleaned.length === 0) return;

      for (let i = 0; i < cleaned.length; i++) {
         const entry = cleaned[i];
         if (entry.sep) {
            const sep = document.createElement('div');
            sep.className = 'fnx-menu-sep';
            tabContextMenuEl.appendChild(sep);
            continue;
         }

         const item = document.createElement('div');
         let cls = 'fnx-menu-item';
         if (entry.danger) cls += ' danger';
         if (entry.disabled) cls += ' disabled';
         item.className = cls;

         const leftWrap = document.createElement('div');
         leftWrap.style.cssText =
            'display: flex; align-items: center; gap: 9px; min-width: 0; flex: 1;';
         const iconSlot = document.createElement('span');
         iconSlot.className = 'fnx-menu-icon';
         if (entry.icon) {
            iconSlot.appendChild(createIcon(entry.icon));
         }
         leftWrap.appendChild(iconSlot);
         const textSpan = document.createElement('span');
         textSpan.className = 'fnx-menu-label';
         textSpan.textContent = entry.label;
         leftWrap.appendChild(textSpan);
         item.appendChild(leftWrap);

         if (entry.shortcut) {
            const sc = document.createElement('span');
            sc.className = 'fnx-menu-kbd';
            sc.textContent = entry.shortcut;
            item.appendChild(sc);
         }

         // Prevent stealing selection/focus before click handler reads/acts on page selection
         item.onmousedown = (ev) => ev.preventDefault();
         item.onclick = (ev) => {
            ev.stopPropagation();
            hideTabContextMenu();
            if (!entry.disabled && typeof entry.onClick === 'function') {
               entry.onClick();
            }
         };
         tabContextMenuEl.appendChild(item);
      }

      tabContextMenuEl.style.visibility = 'hidden';
      tabContextMenuEl.style.display = 'flex';
      const rect = tabContextMenuEl.getBoundingClientRect();
      const menuW = rect.width || 218;
      const menuH = rect.height || tabContextMenuEl.children.length * 28 + 10;
      const vw = window.innerWidth || 1024;
      const vh = window.innerHeight || 768;
      const rawX = Number.isFinite(clientX) ? clientX : vw / 2;
      const rawY = Number.isFinite(clientY) ? clientY : vh / 2;
      let posX = rawX;
      let posY = rawY + 4;
      if (posX + menuW > vw - 8) {
         posX = vw - menuW - 8;
      }
      if (posY + menuH > vh - 8) {
         posY = rawY - menuH - 4;
      }
      posX = Math.max(8, Math.min(posX, vw - menuW - 8));
      posY = Math.max(8, Math.min(posY, vh - menuH - 8));
      tabContextMenuEl.style.left = Math.round(posX) + 'px';
      tabContextMenuEl.style.top = Math.round(posY) + 'px';
      tabContextMenuEl.style.visibility = 'visible';
   }

   function showTabContextMenu(e, tab, idx) {
      const entries = [
         {
            label: 'Reload tab',
            shortcut: 'Ctrl+R',
            icon: 'reload',
            onClick: () => {
               if (tab.id === tabState.active_id) {
                  sendIpc({ type: 'RELOAD' });
               } else {
                  flushScrollSync();
                  sendIpc({ type: 'RELOAD_TAB', id: tab.id });
               }
            },
         },
         {
            label: 'Duplicate tab',
            icon: 'copy',
            onClick: () => sendIpc({ type: 'DUPLICATE_TAB', id: tab.id }),
         },
         {
            label: tab.pinned ? 'Unpin tab' : 'Pin tab',
            icon: 'pin',
            onClick: () => sendIpc({ type: 'TOGGLE_PIN_TAB', id: tab.id }),
         },
         {
            label: tab.background_play
               ? 'Disable background running'
               : 'Allow running in background',
            icon: 'media',
            onClick: () =>
               sendIpc({ type: 'TOGGLE_BACKGROUND_PLAY', id: tab.id }),
         },
         {
            label: tab.muted ? 'Unmute tab' : 'Mute tab',
            icon: tab.muted ? 'volumeUp' : 'volumeMute',
            onClick: () => sendIpc({ type: 'TOGGLE_MUTE_TAB', id: tab.id }),
         },
         { sep: true },
         {
            label: 'Close tab',
            shortcut: 'Ctrl+W',
            icon: 'close',
            onClick: () =>
               sendIpc({ type: 'CLOSE_TAB', id: tab.id, force: true }),
         },
         tabState.tabs.length > 1
            ? {
                 label: 'Close other tabs',
                 onClick: () => {
                    const otherCount = tabState.tabs.filter(
                       (t) => t.id !== tab.id && !t.pinned,
                    ).length;
                    if (otherCount <= 1 || !openPromptModalRef) {
                       sendIpc({ type: 'CLOSE_OTHER_TABS', id: tab.id });
                       return;
                    }
                    openPromptModalRef(
                       {
                          title: 'Close other tabs',
                          badge: `${otherCount} tabs`,
                          description:
                             'Close all other unpinned tabs in the current workspace?',
                          confirmText: 'Close tabs',
                          hideInput: true,
                       },
                       () => sendIpc({ type: 'CLOSE_OTHER_TABS', id: tab.id }),
                    );
                 },
              }
            : null,
         idx < tabState.tabs.length - 1
            ? {
                 label: 'Close tabs to the right',
                 onClick: () =>
                    sendIpc({ type: 'CLOSE_TABS_TO_RIGHT', id: tab.id }),
              }
            : null,
         {
            label: 'Close all tabs',
            danger: true,
            onClick: () => {
               const unpinned = tabState.tabs.filter((t) => !t.pinned).length;
               if (unpinned <= 1 || !openPromptModalRef) {
                  sendIpc({ type: 'CLOSE_ALL_TABS' });
                  return;
               }
               openPromptModalRef(
                  {
                     title: 'Close all tabs',
                     badge: `${unpinned} tabs`,
                     description:
                        'Close all unpinned tabs and reset to Start Page?',
                     confirmText: 'Close all tabs',
                     hideInput: true,
                  },
                  () => sendIpc({ type: 'CLOSE_ALL_TABS' }),
               );
            },
         },
         { sep: true },
         {
            label: 'Reopen closed tab',
            shortcut: 'Ctrl+Shift+T',
            icon: 'undo',
            onClick: () => sendIpc({ type: 'REOPEN_CLOSED_TAB' }),
         },
         {
            label: 'New tab',
            shortcut: 'Ctrl+T',
            icon: 'plus',
            onClick: () => {
               triggerProgress();
               sendIpc({ type: 'NEW_TAB' });
            },
         },
         { sep: true },
         {
            label: 'Save current workspace...',
            icon: 'workspace',
            onClick: () => {
               if (!openPromptModalRef) return;
               openPromptModalRef(
                  {
                     title: 'Save workspace',
                     badge: 'Session',
                     description:
                        'Save all open tabs into a named workspace (e.g. Work, Music, Dev).',
                     placeholder: 'Workspace name...',
                     confirmText: 'Save workspace',
                  },
                  (name) => {
                     if (name && name.trim()) {
                        sendIpc({ type: 'SAVE_WORKSPACE', name: name.trim() });
                     }
                  },
               );
            },
         },
      ];
      renderContextMenu(e && e.clientX, e && e.clientY, entries);
   }

   function showPageContextMenu(e) {
      if (!shadowRoot) return;
      const path = typeof e.composedPath === 'function' ? e.composedPath() : [];
      const rawTarget = (path && path[0]) || e.target;
      const targetEl =
         rawTarget && rawTarget.nodeType === 1
            ? rawTarget
            : rawTarget && rawTarget.parentElement;

      const linkEl =
         targetEl && targetEl.closest ? targetEl.closest('a[href]') : null;
      const imgEl =
         targetEl && targetEl.closest ? targetEl.closest('img[src]') : null;
      const tag = targetEl && targetEl.tagName ? targetEl.tagName.toLowerCase() : '';
      const isTextControl = tag === 'input' || tag === 'textarea';
      const inputType =
         tag === 'input'
            ? String(targetEl.type || 'text').toLowerCase()
            : '';
      const isPassword = isTextControl && inputType === 'password';
      const isEditable =
         isEditableWebElement(targetEl) &&
         !targetEl.readOnly &&
         !targetEl.disabled;

      let selStart = 0;
      let selEnd = 0;
      let selText = '';
      if (isTextControl) {
         selStart = Number(targetEl.selectionStart) || 0;
         selEnd = Number(targetEl.selectionEnd) || 0;
         if (!isPassword && selEnd > selStart) {
            selText = String(targetEl.value || '').slice(selStart, selEnd);
         }
      } else {
         try {
            selText = String(
               window.getSelection ? window.getSelection() : '',
            );
         } catch (_) {}
      }
      const trimmedSel = selText.trim();

      const entries = [];

      // 1. Link actions
      if (linkEl && linkEl.href) {
         const linkHref = linkEl.href;
         const canOpenTab = /^(https?:|fynexo:)/i.test(linkHref);
         if (canOpenTab) {
            entries.push({
               label: 'Open link in new tab',
               icon: 'externalLink',
               onClick: () => {
                  sendIpc({
                     type: 'NEW_TAB',
                     url: linkHref,
                     title: (linkEl.textContent || '').trim(),
                     background: true,
                  });
               },
            });
         }
         entries.push({
            label: 'Copy link address',
            icon: 'link',
            onClick: () => copyTextToClipboard(linkHref, 'Link copied'),
         });
         entries.push({ sep: true });
      }

      // 2. Image actions
      if (imgEl) {
         const imgSrc = imgEl.currentSrc || imgEl.src || '';
         if (imgSrc) {
            if (/^https?:/i.test(imgSrc)) {
               entries.push({
                  label: 'Open image in new tab',
                  icon: 'externalLink',
                  onClick: () =>
                     sendIpc({
                        type: 'NEW_TAB',
                        url: imgSrc,
                        background: true,
                     }),
               });
            }
            entries.push({
               label: 'Copy image URL',
               icon: 'copy',
               onClick: () =>
                  copyTextToClipboard(imgSrc, 'Image URL copied'),
            });
            entries.push({ sep: true });
         }
      }

      // 3. Editable input or text selection actions
      if (isEditable) {
         const hasEditableSel = !isPassword && selText.length > 0;
         entries.push({
            label: 'Cut',
            shortcut: 'Ctrl+X',
            icon: 'scissors',
            disabled: !hasEditableSel,
            onClick: () => {
               try {
                  targetEl.focus();
                  if (isTextControl && typeof targetEl.setRangeText === 'function') {
                     copyTextToClipboard(selText);
                     targetEl.setRangeText('', selStart, selEnd, 'start');
                     targetEl.dispatchEvent(new Event('input', { bubbles: true }));
                  } else {
                     document.execCommand('cut');
                  }
               } catch (_) {}
            },
         });
         entries.push({
            label: 'Copy',
            shortcut: 'Ctrl+C',
            icon: 'copy',
            disabled: !hasEditableSel,
            onClick: () => copyTextToClipboard(selText, 'Copied'),
         });
         entries.push({
            label: 'Paste',
            shortcut: 'Ctrl+V',
            icon: 'clipboard',
            onClick: () => {
               try {
                  targetEl.focus();
                  if (
                     navigator.clipboard &&
                     typeof navigator.clipboard.readText === 'function'
                  ) {
                     navigator.clipboard
                        .readText()
                        .then((clip) => {
                           if (!clip) return;
                           if (
                              isTextControl &&
                              typeof targetEl.setRangeText === 'function'
                           ) {
                              targetEl.setRangeText(clip, selStart, selEnd, 'end');
                              targetEl.dispatchEvent(
                                 new Event('input', { bubbles: true }),
                              );
                           } else {
                              document.execCommand('insertText', false, clip);
                           }
                        })
                        .catch(() => showHudToast('Press Ctrl+V to paste'));
                  } else {
                     showHudToast('Press Ctrl+V to paste');
                  }
               } catch (_) {
                  showHudToast('Press Ctrl+V to paste');
               }
            },
         });
         entries.push({
            label: 'Select all',
            shortcut: 'Ctrl+A',
            onClick: () => {
               try {
                  targetEl.focus();
                  if (isTextControl && typeof targetEl.select === 'function') {
                     targetEl.select();
                  } else if (window.getSelection) {
                     const range = document.createRange();
                     range.selectNodeContents(targetEl);
                     const sel = window.getSelection();
                     sel.removeAllRanges();
                     sel.addRange(range);
                  }
               } catch (_) {}
            },
         });
         entries.push({ sep: true });
      } else if (trimmedSel && !isPassword) {
         const eng = currentEngineObj();
         const shortSel =
            trimmedSel.length > 18
               ? trimmedSel.slice(0, 18) + '…'
               : trimmedSel;
         entries.push({
            label: 'Copy',
            shortcut: 'Ctrl+C',
            icon: 'copy',
            onClick: () => copyTextToClipboard(selText, 'Copied'),
         });
         entries.push({
            label: `Search ${eng.name || 'web'} for "${shortSel}"`,
            icon: 'search',
            onClick: () => {
               const searchUrl = fillQueryTemplate(
                  eng.url,
                  encodeURIComponent(trimmedSel),
               );
               triggerProgress();
               sendIpc({ type: 'NEW_TAB', url: searchUrl });
            },
         });
         entries.push({ sep: true });
      }

      // 4. Core Navigation & Page Tools
      entries.push({
         label: 'Back',
         shortcut: 'Alt+Left',
         icon: 'back',
         disabled: !navState.canBack,
         onClick: () => requestHistoryNav('back'),
      });
      entries.push({
         label: 'Forward',
         shortcut: 'Alt+Right',
         icon: 'forward',
         disabled: !navState.canForward,
         onClick: () => requestHistoryNav('forward'),
      });
      entries.push({
         label: 'Reload',
         shortcut: 'Ctrl+R',
         icon: 'reload',
         onClick: () => {
            triggerProgress();
            sendIpc({ type: 'RELOAD' });
         },
      });
      entries.push({ sep: true });

      const curUrl = canonicalCurrentUrl();
      const internal = isInternalAny(curUrl);
      const isSet = isInternalSettings(curUrl);
      if (!internal && curUrl && curUrl !== 'about:blank') {
         const starred = isUrlBookmarked(curUrl);
         entries.push({
            label: 'Copy page URL',
            icon: 'link',
            onClick: () => copyCurrentUrl(),
         });
         entries.push({
            label: starred ? 'Remove bookmark' : 'Bookmark page',
            icon: starred ? 'starFilled' : 'star',
            onClick: () => toggleCurrentBookmark(),
         });
         entries.push({
            label: isReaderOpen ? 'Exit reader mode' : 'Reader mode',
            shortcut: 'Alt+R',
            icon: 'reader',
            onClick: () => toggleReaderMode(),
         });
      }
      if (!isSet) {
         entries.push({
            label: 'Find in page',
            shortcut: keybinds.find_in_page || 'Ctrl+F',
            icon: 'search',
            onClick: () => openFindBar(),
         });
      }
      entries.push({ sep: true });

      entries.push({
         label: 'Print...',
         shortcut: 'Ctrl+P',
         icon: 'printer',
         onClick: () => {
            setTimeout(() => {
               try {
                  window.print();
               } catch (_) {}
            }, 30);
         },
      });
      if (!isMobilePlatform()) {
         entries.push({
            label: 'Inspect',
            shortcut: 'F12',
            icon: 'code',
            onClick: () => sendIpc({ type: 'OPEN_DEVTOOLS' }),
         });
      }

      renderContextMenu(e && e.clientX, e && e.clientY, entries);
   }

   let pendingMidTab = null;

   function finishMiddleCloseTab(tabId) {
      if (pendingMidTab && pendingMidTab.id === tabId && pendingMidTab.done) {
         return;
      }
      pendingMidTab = { id: tabId, done: true };
      sendIpc({ type: 'CLOSE_TAB', id: tabId, force: true });
   }

   function renderTabs(force) {
      if (!shadowRoot) return;

      const tabCount = Math.max(1, tabState.tabs.length);
      const mobileTabBadge = shadowRoot.getElementById(
         '__fynexo_mobile_tab_count__',
      );
      if (mobileTabBadge) {
         const countStr = String(tabCount);
         if (mobileTabBadge.textContent !== countStr)
            mobileTabBadge.textContent = countStr;
      }

      const drawerCount = shadowRoot.getElementById('__fynexo_drawer_count__');
      if (drawerCount) {
         drawerCount.textContent = `${tabState.tabs.length} ${tabState.tabs.length === 1 ? 'tab' : 'tabs'}`;
      }

      const isDesktopBarShown =
         window.innerWidth > 768
            ? styles.topbar.visible_on_desktop !== false
            : styles.topbar.visible_on_mobile === true;

      if (isDesktopBarShown) {
         const sig = computeTabsSig();
         if (force || sig !== lastDeskTabsSig) {
            lastDeskTabsSig = sig;
            const container = shadowRoot.getElementById(
               '__fynexo_tabs_container__',
            );
            if (container) {
               container.classList.toggle(
                  'fnx-tabs-crowded',
                  tabState.tabs.length >= 6,
               );
               container.textContent = '';
               const frag = document.createDocumentFragment();
               let activeTabEl = null;

               tabState.tabs.forEach((tab, idx) => {
                  const isActive = tab.id === tabState.active_id;
                  const isSet = isInternalSettings(tab.url);
                  const isNt = isInternalNewtab(tab.url);
                  const cleanTitle = isSet
                     ? 'Preferences'
                     : isNt
                       ? 'Start page'
                       : tab.title || 'New tab';

                  const el = document.createElement('div');
                  let cls = isActive ? 'fnx-tab-item active' : 'fnx-tab-item';
                  if (tab.pinned) cls += ' pinned';
                  if (tab.suspended && !isActive) cls += ' suspended';
                  el.className = cls;
                  if (isActive) activeTabEl = el;

                  el.title =
                     tab.suspended && !isActive
                        ? `${cleanTitle} (Suspended · click to load)`
                        : cleanTitle;
                  el.draggable = true;

                  el.ondragstart = (e) => {
                     draggingTabId = tab.id;
                     el.classList.add('dragging');
                     if (e.dataTransfer) {
                        e.dataTransfer.effectAllowed = 'move';
                        try {
                           e.dataTransfer.setData('text/plain', String(tab.id));
                        } catch (_) {}
                     }
                  };
                  el.ondragover = (e) => {
                     if (draggingTabId && draggingTabId !== tab.id) {
                        e.preventDefault();
                        el.classList.add('drag-over');
                     }
                  };
                  el.ondragleave = () => {
                     el.classList.remove('drag-over');
                  };
                  el.ondrop = (e) => {
                     e.preventDefault();
                     el.classList.remove('drag-over');
                     if (draggingTabId && draggingTabId !== tab.id) {
                        sendIpc({
                           type: 'REORDER_TAB',
                           from_id: draggingTabId,
                           to_id: tab.id,
                        });
                     }
                     draggingTabId = null;
                  };
                  el.ondragend = () => {
                     draggingTabId = null;
                     el.classList.remove('dragging');
                     container
                        .querySelectorAll('.drag-over')
                        .forEach((n) => n.classList.remove('drag-over'));
                  };

                  el.onclick = (e) => {
                     if (
                        e.target.closest &&
                        (e.target.closest('.fnx-tab-close') ||
                           e.target.closest('.fnx-tab-audio-btn'))
                     )
                        return;
                     if (!isActive) {
                        flushScrollSync();
                        triggerProgress();
                        sendIpc({ type: 'SWITCH_TAB', id: tab.id });
                     }
                  };
                  el.oncontextmenu = (e) => {
                     e.preventDefault();
                     e.stopPropagation();
                     showTabContextMenu(e, tab, idx);
                  };
                  el.onmousedown = (e) => {
                     if (e.button === 1) {
                        e.preventDefault();
                        e.stopPropagation();
                        pendingMidTab = { id: tab.id, done: false };
                     }
                  };
                  el.onauxclick = (e) => {
                     if (e.button === 1) {
                        e.preventDefault();
                        e.stopPropagation();
                        finishMiddleCloseTab(tab.id);
                     }
                  };

                  const mono = document.createElement('span');
                  mono.className = 'fnx-tab-monogram';
                  renderSiteEmblem(mono, tab.url, tab.title);
                  el.appendChild(mono);

                  if (tab.playing_audio || tab.muted) {
                     const audioBtn = document.createElement('button');
                     audioBtn.className = 'fnx-tab-audio-btn';
                     const isMuted = !!tab.muted;
                     const aIcon = createIcon(
                        isMuted ? 'volumeMute' : 'volumeUp',
                     );
                     aIcon.style.cssText =
                        'width: 13.5px; height: 13.5px; pointer-events: none; flex-shrink: 0; overflow: visible;';
                     audioBtn.appendChild(aIcon);
                     audioBtn.title = isMuted
                        ? 'Unmute tab (Click to unmute)'
                        : 'Mute tab (Click to mute)';
                     audioBtn.style.cssText = `
                                display: inline-flex;
                                align-items: center;
                                justify-content: center;
                                width: 16px;
                                height: 16px;
                                background: transparent;
                                color: var(--fnx-secondary);
                                margin: ${tab.pinned ? '0' : '0 2px 0 -2px'};
                                flex-shrink: 0;
                                border: none;
                                cursor: pointer;
                                padding: 0;
                                overflow: visible;
                                transition: color 0.15s ease, transform 0.12s ease;
                            `;
                     audioBtn.onmouseenter = () => {
                        audioBtn.style.background = 'transparent';
                        audioBtn.style.color = 'var(--fnx-fg)';
                        audioBtn.style.transform = 'scale(1.1)';
                     };
                     audioBtn.onmouseleave = () => {
                        audioBtn.style.background = 'transparent';
                        audioBtn.style.color = 'var(--fnx-secondary)';
                        audioBtn.style.transform = 'scale(1)';
                     };
                     audioBtn.onclick = (e) => {
                        e.preventDefault();
                        e.stopPropagation();
                        sendIpc({ type: 'TOGGLE_MUTE_TAB', id: tab.id });
                     };
                     el.appendChild(audioBtn);
                  }

                  if (!tab.pinned) {
                     const titleSpan = document.createElement('span');
                     titleSpan.className = 'fnx-tab-title';
                     titleSpan.textContent = cleanTitle;

                     const closeBtn = document.createElement('button');
                     closeBtn.className = 'fnx-tab-close';
                     closeBtn.appendChild(createIcon('closeSmall'));
                     closeBtn.title = `Close tab (${idx + 1})`;
                     closeBtn.onclick = (e) => {
                        e.stopPropagation();
                        sendIpc({ type: 'CLOSE_TAB', id: tab.id });
                     };

                     el.appendChild(titleSpan);
                     el.appendChild(closeBtn);
                  }

                  frag.appendChild(el);
               });

               container.appendChild(frag);
               if (activeTabEl && container) {
                  try {
                     const cRect = container.getBoundingClientRect();
                     const tRect = activeTabEl.getBoundingClientRect();
                     if (tRect.left < cRect.left) {
                        container.scrollLeft -= cRect.left - tRect.left + 8;
                     } else if (tRect.right > cRect.right) {
                        container.scrollLeft += tRect.right - cRect.right + 8;
                     }
                  } catch (_) {}
               }
            }
         }
      }

      if (isDrawerOpen) {
         renderDrawerCards();
      }
      if (typeof onTabsRenderedCb === 'function') {
         onTabsRenderedCb();
      }
   }

   function renderOmniboxSuggestions(rawQuery, listEl, inputEl, closeFn) {
      if (!shadowRoot || !listEl) return;
      listEl.textContent = '';

      const trimmed = (rawQuery || '').trim();
      const q = trimmed.toLowerCase();
      const results = [];
      const seen = Object.create(null);

      function pushCandidate(item) {
         const key =
            item.kind === 'tab'
               ? 'tab:' + item.tabId
               : item.kind === 'bang'
                 ? 'bang:' + item.trigger
                 : normalizeUrlKey(item.url);
         if (!key || seen[key]) return;
         seen[key] = true;
         results.push(item);
      }

      // 0. Bang Shortcuts when starting with '!'
      if (q.startsWith('!')) {
         const bangs = getBangs();
         const tok = q.split(' ')[0];
         for (let i = 0; i < bangs.length; i++) {
            const b = bangs[i];
            if (!b || !b.trigger) continue;
            const trig = (
               b.trigger.startsWith('!') ? b.trigger : '!' + b.trigger
            ).toLowerCase();
            if (
               trig.startsWith(tok) ||
               (b.name || '').toLowerCase().includes(tok.slice(1))
            ) {
               pushCandidate({
                  kind: 'bang',
                  trigger: trig,
                  title: `${trig} — Search ${b.name || trig}`,
                  url: b.url || '',
                  badge: '!Bang',
               });
            }
         }
      }

      // 1. Open Tabs
      for (let i = 0; i < tabState.tabs.length; i++) {
         const t = tabState.tabs[i];
         if (t.id === tabState.active_id || isInternalAny(t.url)) continue;
         const title = t.title || formatHostLabel(t.url);
         const url = t.url || '';
         if (
            !q ||
            title.toLowerCase().includes(q) ||
            url.toLowerCase().includes(q)
         ) {
            pushCandidate({
               kind: 'tab',
               tabId: t.id,
               title: title,
               url: url,
               badge: 'Tab',
            });
         }
      }

      // 2. Bookmarks
      for (let i = 0; i < bookmarks.length; i++) {
         const b = bookmarks[i];
         const title = b.title || formatHostLabel(b.url);
         const url = b.url || '';
         if (
            !q ||
            title.toLowerCase().includes(q) ||
            url.toLowerCase().includes(q)
         ) {
            pushCandidate({
               kind: 'bookmark',
               title: title,
               url: url,
               badge: 'Saved',
            });
         }
      }

      // 3. Recent History
      for (let i = 0; i < historyItems.length; i++) {
         const h = historyItems[i];
         const title = h.title || formatHostLabel(h.url);
         const url = h.url || '';
         if (
            q &&
            (title.toLowerCase().includes(q) || url.toLowerCase().includes(q))
         ) {
            pushCandidate({
               kind: 'history',
               title: title,
               url: url,
               badge: 'History',
            });
         } else if (!q && results.length < 5) {
            pushCandidate({
               kind: 'history',
               title: title,
               url: url,
               badge: 'Recent',
            });
         }
      }

      suggestItems = results.slice(0, 6);
      suggestIndex = -1;

      if (suggestItems.length === 0) {
         listEl.style.display = 'none';
         return;
      }

      listEl.style.display = 'flex';
      const frag = document.createDocumentFragment();
      suggestItems.forEach((item, idx) => {
         const row = document.createElement('button');
         row.type = 'button';
         row.className = 'fnx-suggest-row';
         row.dataset.idx = String(idx);
         row.onmousedown = (e) => e.preventDefault();

         const iconWrap = document.createElement('span');
         iconWrap.className = 'fnx-suggest-icon';
         const iconName =
            item.kind === 'bookmark'
               ? 'starFilled'
               : item.kind === 'history'
                 ? 'history'
                 : item.kind === 'bang'
                   ? 'search'
                   : 'go';
         iconWrap.appendChild(createIcon(iconName));
         if (item.kind === 'bookmark' || item.kind === 'bang')
            iconWrap.style.color = 'var(--fnx-accent)';

         const textWrap = document.createElement('span');
         textWrap.className = 'fnx-suggest-main';
         const tSpan = document.createElement('span');
         tSpan.className = 'fnx-suggest-title';
         tSpan.textContent = item.title;
         const uSpan = document.createElement('span');
         uSpan.className = 'fnx-suggest-url';
         uSpan.textContent =
            item.kind === 'bang' ? item.url : formatHostLabel(item.url);
         textWrap.appendChild(tSpan);
         textWrap.appendChild(uSpan);

         const tag = document.createElement('span');
         tag.className = 'fnx-suggest-badge';
         tag.textContent = item.badge;

         row.appendChild(iconWrap);
         row.appendChild(textWrap);
         row.appendChild(tag);

         row.onclick = () => {
            if (item.kind === 'bang') {
               const cur = inputEl.value.trim();
               if (!cur.includes(' ')) {
                  inputEl.value = item.trigger + ' ';
                  inputEl.focus();
                  renderOmniboxSuggestions(
                     inputEl.value,
                     listEl,
                     inputEl,
                     closeFn,
                  );
                  return;
               }
               const resolved = resolveUrl(cur, config.search_engine);
               if (closeFn) closeFn();
               flushScrollSync();
               triggerProgress();
               sendIpc({ type: 'NAVIGATE', url: resolved });
               return;
            }
            if (closeFn) closeFn();
            triggerProgress();
            if (item.kind === 'tab' && item.tabId) {
               flushScrollSync();
               sendIpc({ type: 'SWITCH_TAB', id: item.tabId });
            } else {
               flushScrollSync();
               sendIpc({ type: 'NAVIGATE', url: item.url });
            }
         };
         frag.appendChild(row);
      });
      listEl.appendChild(frag);
   }

   function updateSuggestHighlight(listEl) {
      if (!listEl) return;
      const rows = listEl.querySelectorAll('.fnx-suggest-row');
      rows.forEach((r, idx) => {
         r.classList.toggle('selected', idx === suggestIndex);
      });
   }

   function renderHudQuickBar(quickRow, closeFn) {
      if (!shadowRoot || !quickRow) return;
      quickRow.textContent = '';

      const leftGroup = document.createElement('div');
      leftGroup.className = 'fnx-quick-tabs';

      function makeChip(label, active, onClick, iconName) {
         const c = document.createElement('button');
         c.type = 'button';
         c.className = active ? 'fnx-quick-chip active' : 'fnx-quick-chip';
         if (iconName) {
            c.appendChild(createIcon(iconName));
         }
         c.appendChild(document.createTextNode(label));
         c.onmousedown = (e) => e.preventDefault();
         c.onclick = onClick;
         return c;
      }

      leftGroup.appendChild(
         makeChip('Start', false, () => {
            if (closeFn) closeFn();
            triggerProgress();
            sendIpc({ type: 'NAVIGATE', url: 'fynexo://newtab' });
         }),
      );

      if ((Number(tabState.closed_count) || 0) > 0) {
         leftGroup.appendChild(
            makeChip(
               'Reopen',
               false,
               () => {
                  if (closeFn) closeFn();
                  triggerProgress();
                  sendIpc({ type: 'REOPEN_CLOSED_TAB' });
               },
               'undo',
            ),
         );
      }

      if (!isInternalAny()) {
         leftGroup.appendChild(
            makeChip('Find', isFindOpen, () => {
               if (closeFn) closeFn();
               openFindBar();
            }),
         );
         leftGroup.appendChild(
            makeChip('Reader', isReaderOpen, () => {
               if (closeFn) closeFn();
               toggleReaderMode();
            }),
         );
      }

      leftGroup.appendChild(
         makeChip(
            config.force_dark ? 'Dark: On' : 'Dark: Off',
            !!config.force_dark,
            () => {
               config.force_dark = !config.force_dark;
               sendIpc({ type: 'PATCH_HUD_PREFS', force_dark: config.force_dark });
               applyPageRules();
               renderHudQuickBar(quickRow, closeFn);
               if (config.force_dark && isPageAlreadyDark()) {
                  showHudToast('Smart Dark: Page is already dark (skipped)');
               } else {
                  showHudToast(
                     config.force_dark
                        ? 'Smart Force Dark enabled'
                        : 'Smart Force Dark off',
                  );
               }
            },
         ),
      );

      leftGroup.appendChild(
         makeChip(
            config.incognito ? 'Private: On' : 'Private',
            !!config.incognito,
            () => {
               config.incognito = !config.incognito;
               sendIpc({ type: 'PATCH_HUD_PREFS', incognito: config.incognito });
               syncBookmarkButtons();
               renderHudQuickBar(quickRow, closeFn);
               showHudToast(
                  config.incognito
                     ? 'Private mode on (history paused)'
                     : 'Private mode off',
               );
            },
         ),
      );

      const rightGroup = document.createElement('div');
      rightGroup.className = 'fnx-quick-actions';

      const zoomPct = Math.round((Number(config.default_zoom) || 1) * 100);
      rightGroup.appendChild(
         makeChip('−', false, () => {
            sendIpc({ type: 'SET_ZOOM', delta: -0.1 });
         }),
      );
      rightGroup.appendChild(
         makeChip(`${zoomPct}%`, zoomPct !== 100, () => {
            sendIpc({ type: 'SET_ZOOM', zoom: 1.0 });
         }),
      );
      rightGroup.appendChild(
         makeChip('+', false, () => {
            sendIpc({ type: 'SET_ZOOM', delta: 0.1 });
         }),
      );

      rightGroup.appendChild(
         makeChip('Prefs', false, () => {
            if (closeFn) closeFn();
            togglePreferences();
         }),
      );

      quickRow.appendChild(leftGroup);
      quickRow.appendChild(rightGroup);
   }

   function bindOmniboxKeydown(inputEl, listEl, submitFn, closeFn) {
      inputEl.addEventListener('keydown', (e) => {
         if (e.key === 'ArrowDown' && suggestItems.length > 0) {
            e.preventDefault();
            suggestIndex = (suggestIndex + 1) % suggestItems.length;
            updateSuggestHighlight(listEl);
            return;
         }
         if (e.key === 'ArrowUp' && suggestItems.length > 0) {
            e.preventDefault();
            suggestIndex =
               (suggestIndex - 1 + suggestItems.length) % suggestItems.length;
            updateSuggestHighlight(listEl);
            return;
         }
         if (e.key === 'Escape') {
            e.preventDefault();
            e.stopPropagation();
            if (closeFn) closeFn();
            return;
         }
         if (e.key === 'Enter') {
            e.preventDefault();
            if (suggestIndex >= 0 && suggestItems[suggestIndex]) {
               const item = suggestItems[suggestIndex];
               if (item.kind === 'bang') {
                  const cur = inputEl.value.trim();
                  if (!cur.includes(' ')) {
                     inputEl.value = item.trigger + ' ';
                     renderOmniboxSuggestions(
                        inputEl.value,
                        listEl,
                        inputEl,
                        closeFn,
                     );
                     return;
                  }
                  submitFn();
                  return;
               }
               if (closeFn) closeFn();
               flushScrollSync();
               triggerProgress();
               if (item.kind === 'tab' && item.tabId) {
                  sendIpc({ type: 'SWITCH_TAB', id: item.tabId });
               } else {
                  sendIpc({ type: 'NAVIGATE', url: item.url });
               }
            } else {
               submitFn();
            }
         }
      });
   }

   function fillQueryTemplate(tpl, encodedQuery) {
      const s = String(tpl || 'https://www.google.com/search?q={query}');
      if (s.includes('{query}')) return s.split('{query}').join(encodedQuery);
      if (s.includes('%s')) return s.split('%s').join(encodedQuery);
      return s + encodedQuery;
   }

   function resolveUrl(raw, engine) {
      raw = String(raw || '').trim();
      if (!raw) return '';

      if (/^fynexo:\/\//i.test(raw)) return raw;
      if (
         /^(settings|preferences|fynexo:settings|about:settings|about:config)$/i.test(
            raw,
         )
      ) {
         return 'fynexo://settings';
      }
      if (
         /^(newtab|start|home|fynexo:newtab|about:newtab|about:home)$/i.test(
            raw,
         )
      ) {
         return 'fynexo://newtab';
      }

      // Bang shortcut resolution (!gh, !yt, !b, !w, etc.)
      if (raw.startsWith('!')) {
         const spIdx = raw.indexOf(' ');
         const tok = (spIdx === -1 ? raw : raw.slice(0, spIdx)).toLowerCase();
         const rest = spIdx === -1 ? '' : raw.slice(spIdx + 1).trim();
         const bangs = getBangs();
         for (let i = 0; i < bangs.length; i++) {
            const b = bangs[i];
            if (!b || !b.trigger) continue;
            const trig = (
               b.trigger.startsWith('!') ? b.trigger : '!' + b.trigger
            ).toLowerCase();
            if (trig === tok) {
               if (!rest) {
                  try {
                     return new URL(fillQueryTemplate(b.url, '')).origin;
                  } catch (_) {
                     return fillQueryTemplate(b.url, '');
                  }
               }
               return fillQueryTemplate(b.url, encodeURIComponent(rest));
            }
         }
      }

      if (/^https?:\/\//i.test(raw)) return raw;
      if (/^(localhost|127\.0\.0\.1)(:\d+)?/i.test(raw)) return 'http://' + raw;
      if (raw.includes('.') && !raw.includes(' ')) return 'https://' + raw;

      const q = encodeURIComponent(raw);
      const engObj = currentEngineObj(engine);
      return fillQueryTemplate(engObj && engObj.url, q);
   }

   function clearProgressTimers() {
      if (progressTimer) {
         clearTimeout(progressTimer);
         progressTimer = null;
      }
      if (progressFadeTimer) {
         clearTimeout(progressFadeTimer);
         progressFadeTimer = null;
      }
      if (progressResetTimer) {
         clearTimeout(progressResetTimer);
         progressResetTimer = null;
      }
   }

   function triggerProgress() {
      if (!shadowRoot) return;
      const pb = shadowRoot.getElementById('__fynexo_progress__');
      if (!pb) return;
      clearProgressTimers();
      pb.style.transition = 'none';
      pb.style.width = '0%';
      pb.style.opacity = '1';
      progressTimer = setTimeout(() => {
         progressTimer = null;
         pb.style.transition =
            'width 0.28s cubic-bezier(0.22, 1, 0.36, 1), opacity 0.2s ease';
         pb.style.width = '60%';
      }, 16);
   }

   function finishProgress(immediate) {
      if (!shadowRoot) return;
      const pb = shadowRoot.getElementById('__fynexo_progress__');
      if (!pb) return;
      clearProgressTimers();
      if (immediate === true) {
         pb.style.transition = 'none';
         pb.style.opacity = '0';
         pb.style.width = '0%';
         return;
      }
      pb.style.width = '100%';
      progressFadeTimer = setTimeout(() => {
         progressFadeTimer = null;
         pb.style.opacity = '0';
         progressResetTimer = setTimeout(() => {
            progressResetTimer = null;
            pb.style.width = '0%';
         }, 220);
      }, 120);
   }

   function openMobileDrawer() {
      if (!shadowRoot) return;
      isDrawerOpen = true;
      renderDrawerCards();
      const drawer = shadowRoot.getElementById('__fynexo_mobile_drawer__');
      if (drawer) drawer.style.display = 'flex';
   }

   function closeMobileDrawer() {
      if (!shadowRoot) return;
      isDrawerOpen = false;
      const drawer = shadowRoot.getElementById('__fynexo_mobile_drawer__');
      if (drawer) drawer.style.display = 'none';
   }

   function syncNavButtons() {
      if (!shadowRoot) return;
      const canBack = !!navState.canBack;
      const canFwd = !!navState.canForward;
      const backIds = ['__fynexo_btn_back__', '__fynexo_mb_back__'];
      for (let i = 0; i < backIds.length; i++) {
         const el = shadowRoot.getElementById(backIds[i]);
         if (!el) continue;
         el.disabled = !canBack;
         el.setAttribute('aria-disabled', canBack ? 'false' : 'true');
         el.title = canBack
            ? 'Back (Alt+Left)'
            : 'Back (Alt+Left) — No back history';
      }
      const fwdIds = ['__fynexo_btn_forward__', '__fynexo_sheet_fwd__'];
      for (let i = 0; i < fwdIds.length; i++) {
         const el = shadowRoot.getElementById(fwdIds[i]);
         if (!el) continue;
         el.disabled = !canFwd;
         el.setAttribute('aria-disabled', canFwd ? 'false' : 'true');
         el.title = canFwd
            ? 'Forward (Alt+Right)'
            : 'Forward (Alt+Right) — No forward history';
         if (el.id === '__fynexo_sheet_fwd__') {
            el.style.opacity = canFwd ? '1' : '0.34';
            el.style.cursor = canFwd ? 'pointer' : 'default';
         }
      }
   }

   function requestHistoryNav(direction) {
      const isFwd = direction === 'forward';
      if (isFwd ? !navState.canForward : !navState.canBack) return;
      triggerProgress();
      sendIpc({ type: isFwd ? 'GO_FORWARD' : 'GO_BACK' });
   }

   function togglePreferences() {
      if (isInternalSettings()) {
         if (navState.canBack) {
            requestHistoryNav('back');
         } else {
            triggerProgress();
            sendIpc({ type: 'NAVIGATE', url: 'fynexo://newtab' });
         }
      } else {
         triggerProgress();
         sendIpc({ type: 'NAVIGATE', url: 'fynexo://settings' });
      }
   }

   // Keybinding Parser & Matcher
   function matchesKeybind(e, spec) {
      if (!spec || typeof spec !== 'string') return false;
      const raw = spec.trim();
      if (!raw) return false;

      const tokens = [];
      let buf = '';
      for (let i = 0; i < raw.length; i++) {
         if (raw[i] === '+' && buf.length > 0) {
            tokens.push(buf.toLowerCase());
            buf = '';
         } else {
            buf += raw[i];
         }
      }
      if (buf.length > 0) tokens.push(buf.toLowerCase());
      if (tokens.length === 0) return false;

      let wantCtrl = false;
      let wantAlt = false;
      let wantShift = false;
      let wantKey = '';

      for (let i = 0; i < tokens.length; i++) {
         const t = tokens[i];
         if (t === 'ctrl' || t === 'cmd' || t === 'meta' || t === 'control')
            wantCtrl = true;
         else if (t === 'alt' || t === 'option') wantAlt = true;
         else if (t === 'shift') wantShift = true;
         else wantKey = t;
      }
      if (!wantKey) return false;

      const hasCtrl = !!(e.ctrlKey || e.metaKey);
      if (hasCtrl !== wantCtrl || !!e.altKey !== wantAlt) return false;

      const evKey = (e.key || '').toLowerCase();
      if (wantKey === '=' && (evKey === '=' || evKey === '+')) {
         return true;
      }
      if (!!e.shiftKey !== wantShift) return false;
      if (wantKey === 'space') return evKey === ' ';
      if (wantKey === 'esc') return evKey === 'escape';
      return evKey === wantKey;
   }

   // Combined Single-Eval Boot Sync (token-authenticated & locked against page tampering)
   exposeBridge('__fynexo_boot_sync__', function (rtPayload, tabsPayload) {
      if (rtPayload) {
         if (rtPayload.config) config = Object.assign(config, rtPayload.config);
         if (rtPayload.styles) styles = mergeStyles(rtPayload.styles);
         if (rtPayload.active_theme)
            activeTheme = Object.assign(activeTheme, rtPayload.active_theme);
         if (rtPayload.keybinds)
            keybinds = Object.assign({}, DEFAULT_KEYBINDS, rtPayload.keybinds);
         if (Array.isArray(rtPayload.enabled_rules)) {
            enabledRules = rtPayload.enabled_rules;
         } else if (Array.isArray(rtPayload.site_rules)) {
            enabledRules = rtPayload.site_rules.filter((r) => r && r.enabled);
         }
         if (Array.isArray(rtPayload.bookmarks))
            bookmarks = rtPayload.bookmarks;
         if (Array.isArray(rtPayload.history)) historyItems = rtPayload.history;
         cacheSlimRuntime();
         if (
            isInternalAny() &&
            typeof window.__fynexo_on_settings_runtime__ === 'function'
         ) {
            window.__fynexo_on_settings_runtime__(rtPayload);
         }
      }
      if (tabsPayload) {
         tabState = tabsPayload;
         writeSessionCache('__fnx_tabs__', tabsPayload);
      }
      syncAllStyles();
      bootSyncDone = true;
      if (typeof onBootSyncDoneCb === 'function') onBootSyncDoneCb();
   });

   exposeBridge('__fynexo_set_runtime__', function (payload) {
      if (!payload) return;
      if (payload.config) config = Object.assign(config, payload.config);
      if (payload.styles) styles = mergeStyles(payload.styles);
      if (payload.active_theme)
         activeTheme = Object.assign(activeTheme, payload.active_theme);
      if (payload.keybinds)
         keybinds = Object.assign({}, DEFAULT_KEYBINDS, payload.keybinds);
      if (Array.isArray(payload.enabled_rules)) {
         enabledRules = payload.enabled_rules;
      } else if (Array.isArray(payload.site_rules)) {
         enabledRules = payload.site_rules.filter((r) => r && r.enabled);
      }
      if (Array.isArray(payload.bookmarks)) bookmarks = payload.bookmarks;
      if (Array.isArray(payload.history)) historyItems = payload.history;
      cacheSlimRuntime();
      syncAllStyles();
      const input = shadowRoot
         ? shadowRoot.getElementById('__fynexo_inline_input__') ||
           shadowRoot.getElementById('__fynexo_input__')
         : null;
      if (input) {
         input.placeholder = `Search ${currentEngineObj().name} or enter !bang / URL...`;
      }
      if (shadowRoot) {
         const inlineQuick = shadowRoot.getElementById(
            '__fynexo_inline_quick__',
         );
         if (inlineQuick) renderHudQuickBar(inlineQuick, closeOmniboxRef);
         const modalQuick = shadowRoot.getElementById('__fynexo_hud_quick__');
         if (isModalActive && modalQuick)
            renderHudQuickBar(modalQuick, closeOmniboxRef);
      }
      if (
         isInternalAny() &&
         typeof window.__fynexo_on_settings_runtime__ === 'function'
      ) {
         window.__fynexo_on_settings_runtime__(payload);
      }
   });

   exposeBridge('__fynexo_set_fullscreen__', function (full) {
      isFullscreen = !!full;
      if (!isFullscreen) isTopbarPeek = false;
      syncAllStyles();
   });

   exposeBridge('__fynexo_toast__', function (msg) {
      showHudToast(msg);
      if (typeof handleDownloadToast === 'function') {
         handleDownloadToast(msg);
      }
   });

   exposeBridge('__fynexo_restore_scroll__', function (targetY) {
      const y = Number(targetY) || 0;
      if (y <= 24 || isInternalAny()) return;
      const attemptScroll = () => {
         try {
            if (document.body) document.body.scrollTop = y;
            if (document.documentElement)
               document.documentElement.scrollTop = y;
            window.scrollTo(0, y);
         } catch (_) {}
      };
      attemptScroll();
      if (document.readyState !== 'complete') {
         window.addEventListener('load', () => setTimeout(attemptScroll, 80), {
            once: true,
         });
      } else {
         setTimeout(attemptScroll, 80);
      }
   });

   // Android Hardware / Gesture Back Hook (called from Kotlin MainActivity with 0 args)
   exposeReadOnly('__fynexo_handle_back__', function () {
      if (isPromptOpenRef && isPromptOpenRef()) {
         if (closePromptModalRef) closePromptModalRef();
         return true;
      }
      if (isHistoryOpenRef && isHistoryOpenRef()) {
         if (closeHistoryDrawerRef) closeHistoryDrawerRef();
         return true;
      }
      if (
         tabContextMenuEl &&
         tabContextMenuEl.style.display === 'flex'
      ) {
         hideTabContextMenu();
         return true;
      }
      if (isAnyNavPopoverOpenRef && isAnyNavPopoverOpenRef()) {
         if (closeAllNavPopoversRef) closeAllNavPopoversRef();
         return true;
      }
      if (isSidePanelOpenRef && isSidePanelOpenRef()) {
         if (closeSidePanelRef) closeSidePanelRef();
         return true;
      }
      if (isReaderOpen) {
         closeReaderMode();
         return true;
      }
      if (isFindOpen) {
         closeFindBar();
         return true;
      }
      if (isDrawerOpen) {
         closeMobileDrawer();
         return true;
      }
      if (isInlineActive || isModalActive) {
         if (closeOmniboxRef) closeOmniboxRef();
         return true;
      }
      if (
         document.fullscreenElement &&
         typeof document.exitFullscreen === 'function'
      ) {
         try {
            document.exitFullscreen();
         } catch (_) {}
         return true;
      }
      return false;
   });

   exposeBridge('__fynexo_set_config__', function (newCfg) {
      if (!newCfg) return;
      config = Object.assign(config, newCfg);
      syncAllStyles();
   });

   function applyTabMuted(isMuted) {
      isTabMutedState = !!isMuted;
      try {
         document.querySelectorAll('video, audio').forEach((el) => {
            try {
               el.muted = isTabMutedState;
            } catch (_) {}
         });
      } catch (_) {}
   }

   exposeBridge('__fnx_set_tab_muted__', applyTabMuted);

   try {
      const origMediaPlay = HTMLMediaElement.prototype.play;
      HTMLMediaElement.prototype.play = function () {
         if (isTabMutedState) {
            try {
               this.muted = true;
            } catch (_) {}
         }
         return origMediaPlay.apply(this, arguments);
      };
   } catch (_) {}

   exposeBridge('__fynexo_set_tabs__', function (state) {
      if (!state) return;
      tabState = state;
      writeSessionCache('__fnx_tabs__', state);
      renderTabs(false);
      updateAddressLabels();
      try {
         const currentTab =
            state.tabs && state.tabs.find((t) => t.id === state.active_id);
         if (currentTab && typeof currentTab.muted === 'boolean') {
            applyTabMuted(currentTab.muted);
         }
      } catch (_) {}
   });

   exposeBridge('__fynexo_on_resource_stats__', function (stats) {
      if (typeof onResourceStatsCb === 'function') {
         onResourceStatsCb(stats);
      }
   });

   exposeBridge('__fynexo_trigger_progress__', function () {
      triggerProgress();
   });

   exposeBridge('__fynexo_finish_progress__', function (immediate) {
      finishProgress(immediate);
   });

   exposeBridge('__fynexo_set_nav_state__', function (canBack, canForward) {
      navState.canBack = !!canBack;
      navState.canForward = !!canForward;
      syncNavButtons();
   });

   exposeBridge('__fnx_set_background__', function (isBg) {
      isBackgroundParked = !!isBg;
      if (!isBackgroundParked && typeof onForegroundResumeCb === 'function') {
         onForegroundResumeCb();
      }
   });

   exposeBridge('__fynexo_request_reveal_frame__', function () {
      const done = () => sendIpc({ type: 'TAB_PAINT_READY' });
      if (typeof window.requestAnimationFrame === 'function') {
         window.requestAnimationFrame(() => {
            window.requestAnimationFrame(done);
         });
      } else {
         done();
      }
   });

   function setKeyboardHiddenState(hidden) {
      if (styles.mobile_bar.auto_hide_on_keyboard === false) hidden = false;
      if (isPageKeyboardOpen === hidden) return;
      isPageKeyboardOpen = hidden;
      if (document.body) {
         if (hidden) document.body.classList.add('fnx-kb-active');
         else document.body.classList.remove('fnx-kb-active');
      }
      applyShadowStyles();
   }

   function getDeepActiveElement() {
      let el = document.activeElement;
      while (el && el.shadowRoot && el.shadowRoot.activeElement) {
         el = el.shadowRoot.activeElement;
      }
      return el;
   }

   function isEditableWebElement(el) {
      if (!el || el.id === '__fynexo_host__') return false;
      while (el && el.shadowRoot && el.shadowRoot.activeElement) {
         el = el.shadowRoot.activeElement;
      }
      const tag = (el.tagName || '').toLowerCase();
      if (tag === 'textarea') return true;
      if (tag === 'input') {
         const t = (el.type || 'text').toLowerCase();
         return ![
            'button',
            'checkbox',
            'radio',
            'submit',
            'reset',
            'range',
            'file',
            'image',
            'hidden',
         ].includes(t);
      }
      return !!el.isContentEditable;
   }

   function initInterface() {
      try {
         if (document.getElementById('__fynexo_host__')) return;

         if (isMobilePlatform()) {
            try {
               let vp = document.querySelector('meta[name="viewport"]');
               if (!vp && (document.head || document.documentElement)) {
                  vp = document.createElement('meta');
                  vp.name = 'viewport';
                  vp.content =
                     'width=device-width, initial-scale=1.0, maximum-scale=5.0';
                  (document.head || document.documentElement).appendChild(vp);
               }
            } catch (_) {}
         }

         try {
            if (!window.__fynexo_scroll_proxied__) {
               window.__fynexo_scroll_proxied__ = true;
               Object.defineProperty(window, 'scrollY', {
                  get: () =>
                     (document.body ? document.body.scrollTop : 0) ||
                     (document.documentElement
                        ? document.documentElement.scrollTop
                        : 0) ||
                     0,
                  configurable: true,
               });
               Object.defineProperty(window, 'pageYOffset', {
                  get: () =>
                     (document.body ? document.body.scrollTop : 0) ||
                     (document.documentElement
                        ? document.documentElement.scrollTop
                        : 0) ||
                     0,
                  configurable: true,
               });
            }
         } catch (_) {}

         // Feature 3: Background Audio Keep-Alive & Anti-Pause (Prevent auto-pause when minimized/occluded/screen locked)
         try {
            if (!window.__fynexo_bg_audio_hooked__) {
               window.__fynexo_bg_audio_hooked__ = true;
               try {
                  Object.defineProperty(Document.prototype, 'hidden', {
                     get: () => false,
                     configurable: true,
                  });
                  Object.defineProperty(Document.prototype, 'visibilityState', {
                     get: () => 'visible',
                     configurable: true,
                  });
               } catch (_) {}
               try {
                  Object.defineProperty(document, 'hidden', {
                     get: () => false,
                     configurable: true,
                  });
                  Object.defineProperty(document, 'visibilityState', {
                     get: () => 'visible',
                     configurable: true,
                  });
               } catch (_) {}

               window.addEventListener(
                  'visibilitychange',
                  (e) => {
                     e.stopImmediatePropagation();
                  },
                  true,
               );

               window.addEventListener(
                  'blur',
                  () => {
                     try {
                        const mediaEls =
                           document.querySelectorAll('video, audio');
                        mediaEls.forEach((m) => {
                           if (!m.paused) {
                              setTimeout(() => {
                                 if (m.paused) {
                                    try {
                                       m.play();
                                    } catch (_) {}
                                 }
                              }, 60);
                           }
                        });
                     } catch (_) {}
                  },
                  true,
               );
            }
         } catch (_) {}

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

         shadowRoot = host.attachShadow({ mode: 'closed' });

         const baseStyle = document.createElement('style');

         baseStyle.textContent = __FNX_BASE_CSS__;

         shadowRoot.appendChild(baseStyle);

         const shell = document.createElement('div');
         shell.id = '__fynexo_shell__';
         if (isMobilePlatform()) {
            shell.classList.add('fnx-is-mobile');
         }
         shadowRoot.appendChild(shell);

         const progressBar = document.createElement('div');
         progressBar.id = '__fynexo_progress__';
         shell.appendChild(progressBar);

         const hudToast = document.createElement('div');
         hudToast.id = '__fynexo_hud_toast__';
         const hudToastDot = document.createElement('span');
         hudToastDot.style.cssText =
            'width: 6px; height: 6px; border-radius: 50%; background: var(--fnx-accent); flex-shrink: 0;';
         const hudToastMsg = document.createElement('span');
         hudToastMsg.id = '__fynexo_hud_toast_msg__';
         hudToast.appendChild(hudToastDot);
         hudToast.appendChild(hudToastMsg);
         shell.appendChild(hudToast);

         // 1. Desktop Topbar (3-Tier Layered Studio Deck + Single-Row Compact Mode)
         const topbar = document.createElement('div');
         topbar.id = '__fynexo_topbar__';
         shell.appendChild(topbar);

         topbar.addEventListener('mouseleave', () => {
            if (isFullscreen && isTopbarPeek) {
               isTopbarPeek = false;
               applyShadowStyles();
            }
         });

         function makeDesktopIconBtn(iconName, tooltip, onClick) {
            const b = document.createElement('button');
            b.type = 'button';
            b.className = 'fnx-icon-btn';
            b.title = tooltip;
            b.appendChild(createIcon(iconName));
            b.onclick = onClick;
            return b;
         }

         function makeWinBtn(iconName, tooltip, extraClass, onClick) {
            const b = document.createElement('button');
            b.type = 'button';
            b.className = extraClass
               ? `fnx-win-btn ${extraClass}`
               : 'fnx-win-btn';
            b.title = tooltip;
            b.appendChild(createIcon(iconName));
            b.onclick = onClick;
            return b;
         }

         // Tier 1: Recessed Tab Deck (#__fynexo_tab_row__)
         const tabRow = document.createElement('div');
         tabRow.id = '__fynexo_tab_row__';

         const tabsDeckCluster = document.createElement('div');
         tabsDeckCluster.id = '__fynexo_tabs_deck_cluster__';

         const tabsContainer = document.createElement('div');
         tabsContainer.id = '__fynexo_tabs_container__';
         tabsContainer.style.cssText =
            'display: flex; overflow-x: auto; flex: 0 1 auto; max-width: calc(100% - 64px); height: 100%; scrollbar-width: none;';

         const handleTabsWheel = (e) => {
            const delta =
               Math.abs(e.deltaY) > Math.abs(e.deltaX) ? e.deltaY : e.deltaX;
            if (delta !== 0) {
               e.preventDefault();
               tabsContainer.scrollLeft += delta;
            }
         };
         tabsContainer.addEventListener('wheel', handleTabsWheel, {
            passive: false,
         });
         tabRow.addEventListener(
            'wheel',
            (e) => {
               if (
                  !e.target.closest ||
                  !e.target.closest('#__fynexo_win_controls__')
               ) {
                  handleTabsWheel(e);
               }
            },
            { passive: false },
         );

         const btnAddTab = makeDesktopIconBtn(
            'plus',
            `New tab (${keybinds.new_tab || 'Ctrl+T'})`,
            () => {
               triggerProgress();
               sendIpc({ type: 'NEW_TAB' });
            },
         );
         btnAddTab.id = '__fynexo_btn_newtab__';
         btnAddTab.style.cssText =
            'width: 28px; height: 28px; margin: 0 4px 0 2px; border-radius: 7px; align-self: center; flex-shrink: 0;';

         const dragRegion = document.createElement('div');
         dragRegion.style.cssText = 'flex: 1; min-width: 24px; height: 100%;';

         let lastMaxTime = 0;
         const triggerMaximize = () => {
            const now = Date.now();
            if (now - lastMaxTime < 350) return;
            lastMaxTime = now;
            sendIpc({ type: 'WINDOW_MAXIMIZE' });
         };

         const isDragTarget = (t) =>
            t === tabRow ||
            t === tabsDeckCluster ||
            t === tabsContainer ||
            t === dragRegion;
         tabRow.onmousedown = (e) => {
            if (e.button === 0 && isDragTarget(e.target)) {
               if (e.detail === 2) {
                  triggerMaximize();
                  return;
               }
               sendIpc({ type: 'WINDOW_DRAG' });
            }
         };

         tabsDeckCluster.appendChild(tabsContainer);
         tabsDeckCluster.appendChild(btnAddTab);
         tabsDeckCluster.appendChild(dragRegion);
         tabRow.appendChild(tabsDeckCluster);

         const winControls = document.createElement('div');
         winControls.id = '__fynexo_win_controls__';
         winControls.appendChild(
            makeWinBtn('winMin', 'Minimize', '', () =>
               sendIpc({ type: 'WINDOW_MINIMIZE' }),
            ),
         );
         winControls.appendChild(
            makeWinBtn('winMax', 'Maximize', '', () =>
               sendIpc({ type: 'WINDOW_MAXIMIZE' }),
            ),
         );
         winControls.appendChild(
            makeWinBtn('winClose', 'Close', 'close', () =>
               sendIpc({ type: 'WINDOW_CLOSE' }),
            ),
         );
         tabRow.appendChild(winControls);
         topbar.appendChild(tabRow);

         // Tier 2: Navigation & Recessed Omnibox Deck (#__fynexo_nav_row__)
         const navRow = document.createElement('div');
         navRow.id = '__fynexo_nav_row__';

         const leftZone = document.createElement('div');
         leftZone.id = '__fynexo_left_zone__';
         leftZone.style.cssText =
            'display: flex; align-items: center; gap: 2px; height: 100%; flex-shrink: 0;';

         const btnBack = makeDesktopIconBtn('back', 'Back (Alt+Left)', () => {
            requestHistoryNav('back');
         });
         btnBack.id = '__fynexo_btn_back__';
         leftZone.appendChild(btnBack);

         const btnForward = makeDesktopIconBtn(
            'forward',
            'Forward (Alt+Right)',
            () => {
               requestHistoryNav('forward');
            },
         );
         btnForward.id = '__fynexo_btn_forward__';
         leftZone.appendChild(btnForward);

         const btnReload = makeDesktopIconBtn(
            'reload',
            'Reload (Ctrl+R)',
            () => {
               triggerProgress();
               sendIpc({ type: 'RELOAD' });
            },
         );
         btnReload.id = '__fynexo_btn_reload__';
         leftZone.appendChild(btnReload);
         navRow.appendChild(leftZone);

         // Center Zone (hosts Recessed Capsule Omnibox in Deck mode, or Tabs in Compact mode)
         const centerZone = document.createElement('div');
         centerZone.id = '__fynexo_center_zone__';
         centerZone.style.cssText =
            'display: flex; align-items: center; justify-content: center; flex: 1; height: 100%; min-width: 0; overflow: hidden; padding: 0 8px;';
         centerZone.onmousedown = (e) => {
            if (
               e.button === 0 &&
               (e.target === centerZone || isDragTarget(e.target))
            ) {
               if (e.detail === 2) {
                  triggerMaximize();
                  return;
               }
               sendIpc({ type: 'WINDOW_DRAG' });
            }
         };

         const addrSlot = document.createElement('div');
         addrSlot.id = '__fynexo_addr_slot__';

         const addrBtn = document.createElement('button');
         addrBtn.type = 'button';
         addrBtn.id = '__fynexo_address_btn__';
         addrBtn.title = 'Click to edit URL or search (!bangs supported)';
         const addrIcon = document.createElement('span');
         addrIcon.id = '__fynexo_desk_proto__';
         addrIcon.style.cssText =
            'display: flex; align-items: center; flex-shrink: 0;';

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

         const btnDeskCopy = document.createElement('button');
         btnDeskCopy.type = 'button';
         btnDeskCopy.id = '__fynexo_desk_copy__';
         btnDeskCopy.className = 'fnx-addr-action-btn';
         btnDeskCopy.title = 'Copy current URL (Ctrl+Shift+C)';
         btnDeskCopy.appendChild(createIcon('copy'));
         btnDeskCopy.onmousedown = (e) => e.preventDefault();
         btnDeskCopy.onclick = (e) => {
            e.stopPropagation();
            copyCurrentUrl();
         };
         addrActions.appendChild(btnDeskCopy);

         const btnDeskReader = document.createElement('button');
         btnDeskReader.type = 'button';
         btnDeskReader.id = '__fynexo_desk_reader__';
         btnDeskReader.className = 'fnx-addr-action-btn';
         btnDeskReader.style.display = 'none';
         btnDeskReader.title = 'Reader mode (Alt+R)';
         btnDeskReader.appendChild(createIcon('reader'));
         btnDeskReader.onmousedown = (e) => e.preventDefault();
         btnDeskReader.onclick = (e) => {
            e.stopPropagation();
            toggleReaderMode();
         };
         addrActions.appendChild(btnDeskReader);

         const btnDeskStar = document.createElement('button');
         btnDeskStar.type = 'button';
         btnDeskStar.id = '__fynexo_desk_star__';
         btnDeskStar.className = 'fnx-addr-action-btn';
         btnDeskStar.dataset.starKind = 'star';
         btnDeskStar.title = 'Bookmark this page (Ctrl+D)';
         btnDeskStar.appendChild(createIcon('star'));
         btnDeskStar.onmousedown = (e) => e.preventDefault();
         btnDeskStar.onclick = (e) => {
            e.stopPropagation();
            toggleCurrentBookmark();
         };
         addrActions.appendChild(btnDeskStar);

         addrSlot.appendChild(addrActions);
         centerZone.appendChild(addrSlot);
         navRow.appendChild(centerZone);

         // Right Studio Utility Zone
         const rightZone = document.createElement('div');
         rightZone.id = '__fynexo_right_zone__';
         rightZone.style.cssText =
            'display: flex; align-items: center; height: 100%; gap: 6px; flex-shrink: 0;';

         // 1. Incognito / Private Badge (Conditional: only shown in private mode)
         const incogBadge = document.createElement('button');
         incogBadge.type = 'button';
         incogBadge.id = '__fynexo_incog_badge__';
         incogBadge.className = 'fnx-incog-pill';
         incogBadge.title = 'Private mode active — click to exit Private mode';
         incogBadge.textContent = 'Private';
         incogBadge.style.display = config.incognito ? 'inline-flex' : 'none';
         incogBadge.onclick = () => {
            config.incognito = false;
            sendIpc({ type: 'PATCH_HUD_PREFS', incognito: false });
            incogBadge.style.display = 'none';
            syncBookmarkButtons();
            showHudToast('Private mode turned off');
         };
         rightZone.appendChild(incogBadge);

         // 1.5 Background Media Dock Pill (visible when any inactive tab is playing in background)
         const bgMediaPill = document.createElement('button');
         bgMediaPill.type = 'button';
         bgMediaPill.id = '__fynexo_bg_media_pill__';
         const bgMediaIcon = createIcon('media');
         bgMediaIcon.style.cssText =
            'width: 12px; height: 12px; color: #34d399; flex-shrink: 0;';
         const bgMediaLabel = document.createElement('span');
         bgMediaLabel.id = '__fynexo_bg_media_label__';
         const bgMediaMute = document.createElement('span');
         bgMediaMute.style.cssText =
            'display: inline-flex; align-items: center; justify-content: center; width: 16px; height: 16px; border-radius: 999px; flex-shrink: 0;';
         bgMediaPill.appendChild(bgMediaIcon);
         bgMediaPill.appendChild(bgMediaLabel);
         bgMediaPill.appendChild(bgMediaMute);
         let bgDockTargetTabId = null;
         function syncBackgroundMediaDock() {
            const bgTab = (tabState.tabs || []).find(
               (t) => t.id !== tabState.active_id && t.playing_audio,
            );
            if (!bgTab) {
               bgDockTargetTabId = null;
               bgMediaPill.style.display = 'none';
               return;
            }
            bgDockTargetTabId = bgTab.id;
            bgMediaLabel.textContent =
               bgTab.title || formatHostLabel(bgTab.url) || 'Background Media';
            bgMediaPill.title = `${bgMediaLabel.textContent} — click to jump to tab`;
            bgMediaMute.textContent = '';
            const mIcon = createIcon(bgTab.muted ? 'volumeMute' : 'volumeUp');
            mIcon.style.cssText = 'width: 11px; height: 11px; color: var(--fnx-secondary);';
            bgMediaMute.title = bgTab.muted
               ? 'Unmute background tab'
               : 'Mute background tab';
            bgMediaMute.appendChild(mIcon);
            bgMediaPill.style.display = 'inline-flex';
         }
         bgMediaMute.onclick = (e) => {
            e.stopPropagation();
            if (bgDockTargetTabId) {
               sendIpc({ type: 'TOGGLE_MUTE_TAB', id: bgDockTargetTabId });
            }
         };
         bgMediaPill.onclick = () => {
            if (bgDockTargetTabId) {
               flushScrollSync();
               triggerProgress();
               sendIpc({ type: 'SWITCH_TAB', id: bgDockTargetTabId });
            }
         };
         onTabsRenderedCb = syncBackgroundMediaDock;
         syncBackgroundMediaDock();
         rightZone.appendChild(bgMediaPill);

         // 2. Media Control Button (Conditional: only shown when audio/video is playing)
         const btnMedia = makeDesktopIconBtn(
            'media',
            'Media Control & Volume Booster',
            (e) => {
               if (e && e.stopPropagation) e.stopPropagation();
               toggleMediaPopover();
            },
         );
         btnMedia.id = '__fynexo_btn_media__';
         btnMedia.classList.add('fnx-popover-trigger');
         btnMedia.style.display = 'none';
         btnMedia.style.position = 'relative';
         const mediaPulse = document.createElement('span');
         mediaPulse.className = 'fnx-pulse-dot';
         mediaPulse.style.cssText = 'position: absolute; top: 2px; right: 2px;';
         btnMedia.appendChild(mediaPulse);
         rightZone.appendChild(btnMedia);

         // 3. Downloads Button (Conditional: only shown when downloading or recent download)
         const btnDownloads = makeDesktopIconBtn(
            'download',
            'Downloads',
            (e) => {
               if (e && e.stopPropagation) e.stopPropagation();
               toggleDownloadsPopover();
            },
         );
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
         const btnShield = makeDesktopIconBtn(
            'shield',
            'Adblock & Privacy Shield',
            (e) => {
               if (e && e.stopPropagation) e.stopPropagation();
               toggleShieldPopover();
            },
         );
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
         const btnSidePanel = makeDesktopIconBtn(
            'sidepanel',
            'Side Panel (Alt+S)',
            (e) => {
               if (e && e.stopPropagation) e.stopPropagation();
               toggleSidePanel();
            },
         );
         btnSidePanel.id = '__fynexo_btn_sidepanel__';
         rightZone.appendChild(btnSidePanel);

         // 5. Main Menu 3-Dots Button (Always visible)
         const btnMore = makeDesktopIconBtn('more', 'Menu', (e) => {
            if (e && e.stopPropagation) e.stopPropagation();
            toggleMainMenu();
         });
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

         function closeAllNavPopovers() {
            const pops = [
               mainMenuEl,
               mediaPopoverEl,
               shieldPopoverEl,
               downloadsPopoverEl,
               taskMgrPopoverEl,
            ];
            for (let i = 0; i < pops.length; i++) {
               if (pops[i]) {
                  pops[i].style.display = 'none';
                  pops[i].classList.remove('fnx-nav-popover--up');
               }
            }
            activeNavPopoverEl = null;
         }
         closeAllNavPopoversRef = closeAllNavPopovers;
         isAnyNavPopoverOpenRef = () => !!activeNavPopoverEl;

         function resolvePopoverAnchor(btn) {
            if (btn) {
               const r = btn.getBoundingClientRect();
               if (r.width > 0 && r.height > 0) return btn;
            }
            const mbMoreEl = shadowRoot
               ? shadowRoot.getElementById('__fynexo_mb_more__')
               : null;
            if (mbMoreEl) {
               const mr = mbMoreEl.getBoundingClientRect();
               if (mr.width > 0 && mr.height > 0) return mbMoreEl;
            }
            return btn;
         }

         function positionNavPopover(btn, popoverEl) {
            if (!popoverEl) return;
            const anchor = resolvePopoverAnchor(btn);
            if (!anchor) return;
            const rect = anchor.getBoundingClientRect();
            const vw = window.innerWidth || 1024;
            const vh = window.innerHeight || 768;

            popoverEl.style.visibility = 'hidden';
            popoverEl.style.display = 'flex';
            const popW = popoverEl.getBoundingClientRect().width || 240;

            const rightOffset = Math.max(
               8,
               Math.min(vw - rect.right, vw - popW - 8),
            );
            popoverEl.style.right = Math.round(rightOffset) + 'px';
            popoverEl.style.left = 'auto';

            const openUp = rect.top > vh * 0.55;
            if (openUp) {
               const bottomOffset = Math.max(8, vh - rect.top + 8);
               popoverEl.style.top = 'auto';
               popoverEl.style.bottom = Math.round(bottomOffset) + 'px';
               popoverEl.style.maxHeight =
                  Math.max(180, Math.round(rect.top - 16)) + 'px';
               popoverEl.style.overflowY = 'auto';
               popoverEl.classList.add('fnx-nav-popover--up');
            } else {
               const topOffset = Math.max(8, rect.bottom + 6);
               popoverEl.style.top = Math.round(topOffset) + 'px';
               popoverEl.style.bottom = 'auto';
               popoverEl.style.maxHeight =
                  Math.max(180, Math.round(vh - rect.bottom - 16)) + 'px';
               popoverEl.style.overflowY = 'auto';
               popoverEl.classList.remove('fnx-nav-popover--up');
            }
            popoverEl.style.visibility = 'visible';
         }

         // Popover 1: Main Menu (Three dots)
         mainMenuEl = document.createElement('div');
         mainMenuEl.id = '__fynexo_main_menu__';
         mainMenuEl.className = 'fnx-nav-popover';
         mainMenuEl.style.width = '238px';

         function addMenuItem(iconName, label, shortcut, onClick) {
            const item = document.createElement('div');
            item.className = 'fnx-menu-item';
            const leftWrap = document.createElement('div');
            leftWrap.style.cssText =
               'display: flex; align-items: center; gap: 9px; min-width: 0;';
            const iconSlot = document.createElement('span');
            iconSlot.className = 'fnx-menu-icon';
            if (iconName) {
               iconSlot.appendChild(createIcon(iconName));
            }
            leftWrap.appendChild(iconSlot);
            const labelSpan = document.createElement('span');
            labelSpan.textContent = label;
            leftWrap.appendChild(labelSpan);
            item.appendChild(leftWrap);
            if (shortcut) {
               const scSpan = document.createElement('span');
               scSpan.className = 'fnx-menu-kbd';
               scSpan.textContent = shortcut;
               item.appendChild(scSpan);
            }
            item.onclick = (e) => {
               e.stopPropagation();
               closeAllNavPopovers();
               onClick();
            };
            mainMenuEl.appendChild(item);
            return item;
         }

         function addMenuSep() {
            const sep = document.createElement('div');
            sep.className = 'fnx-menu-sep';
            mainMenuEl.appendChild(sep);
         }

         // Studio & Preferences
         addMenuItem(
            'settings',
            'Preferences',
            keybinds.preferences || 'Ctrl+,',
            () => {
               togglePreferences();
            },
         );

         // Task Manager / Resource Monitor
         addMenuItem('cpu', 'Resource Monitor', '', () => {
            openTaskMgrPopover();
         });

         // Mobile-only quick popover launchers (desktop exposes these directly on topbar)
         const mobShieldItem = addMenuItem(
            'shield',
            'Adblock Shield',
            '',
            () => {
               toggleShieldPopover();
            },
         );
         mobShieldItem.classList.add('fnx-mobile-only-row');
         const mobShieldStatus = document.createElement('span');
         mobShieldStatus.className = 'fnx-pop-sub';
         mobShieldStatus.style.fontSize = '11px';
         mobShieldItem.appendChild(mobShieldStatus);

         const mobMediaItem = addMenuItem('media', 'Media Control', '', () => {
            toggleMediaPopover();
         });
         mobMediaItem.classList.add('fnx-mobile-only-row');
         const mobMediaStatus = document.createElement('span');
         mobMediaStatus.className = 'fnx-pop-sub';
         mobMediaStatus.style.fontSize = '11px';
         mobMediaItem.appendChild(mobMediaStatus);

         const mobDownloadsItem = addMenuItem('download', 'Downloads', '', () => {
            toggleDownloadsPopover();
         });
         mobDownloadsItem.classList.add('fnx-mobile-only-row');

         const mobSidePanelItem = addMenuItem(
            'sidepanel',
            'Side Panel',
            '',
            () => {
               toggleSidePanel();
            },
         );
         mobSidePanelItem.classList.add('fnx-mobile-only-row');

         // Smart Force Dark (live toggle switch)
         const darkItem = document.createElement('div');
         darkItem.className = 'fnx-menu-item';
         const darkLeft = document.createElement('div');
         darkLeft.style.cssText =
            'display: flex; align-items: center; gap: 9px; min-width: 0;';
         const darkIconSlot = document.createElement('span');
         darkIconSlot.className = 'fnx-menu-icon';
         darkIconSlot.appendChild(createIcon('moon'));
         darkLeft.appendChild(darkIconSlot);
         const darkLabel = document.createElement('span');
         darkLabel.textContent = 'Smart Force Dark';
         darkLeft.appendChild(darkLabel);
         darkItem.appendChild(darkLeft);
         const darkSwitch = document.createElement('span');
         darkSwitch.className = config.force_dark
            ? 'fnx-pop-switch on'
            : 'fnx-pop-switch';
         darkItem.appendChild(darkSwitch);
         function syncMainMenuUI() {
            darkSwitch.classList.toggle('on', !!config.force_dark);
            zoomValSpan.textContent =
               Math.round((Number(config.default_zoom) || 1.0) * 100) + '%';
            const internal = isInternalAny();
            mobShieldStatus.textContent = internal
               ? 'Exempt'
               : config.adblock !== false
                 ? 'Active'
                 : 'Paused';
            if (currentMediaEl) {
               mobMediaItem.style.removeProperty('display');
               const playing =
                  !currentMediaEl.paused && !currentMediaEl.ended;
               mobMediaStatus.textContent = playing ? 'Playing' : 'Paused';
            } else {
               mobMediaItem.style.display = 'none';
            }
         }
         darkItem.onclick = (e) => {
            e.stopPropagation();
            config.force_dark = !config.force_dark;
            sendIpc({ type: 'PATCH_HUD_PREFS', force_dark: config.force_dark });
            applyPageRules();
            syncBookmarkButtons();
            syncMainMenuUI();
            if (config.force_dark && isPageAlreadyDark()) {
               showHudToast('Smart Dark: Page is already dark (skipped)');
            } else {
               showHudToast(
                  config.force_dark
                     ? 'Smart Force Dark enabled'
                     : 'Smart Force Dark off',
               );
            }
         };
         mainMenuEl.appendChild(darkItem);

         addMenuSep();

         // New Tab
         addMenuItem('plus', 'New tab', keybinds.new_tab || 'Ctrl+T', () => {
            triggerProgress();
            sendIpc({ type: 'NEW_TAB' });
         });

         // History Drawer
         addMenuItem('history', 'History drawer', '', () => {
            if (openHistoryDrawerRef) openHistoryDrawerRef();
         });

         // Shortcuts Bar (Desktop Topbar)
         const shortcutsMenuItem = addMenuItem(
            'bookmarksBar',
            'Shortcuts bar',
            '',
            () => {
               const cur = (styles.topbar.bookmarks_bar || 'always').toLowerCase();
               styles.topbar.bookmarks_bar =
                  cur === 'hidden' ? 'always' : 'hidden';
               syncAllStyles();
               sendIpc({
                  type: 'PATCH_HUD_PREFS',
                  bookmarks_bar: styles.topbar.bookmarks_bar,
               });
               showHudToast(
                  styles.topbar.bookmarks_bar === 'hidden'
                     ? 'Shortcuts bar hidden'
                     : 'Shortcuts bar visible',
               );
            },
         );
         shortcutsMenuItem.classList.add('fnx-desktop-only-row');

         addMenuSep();

         // Zoom controls
         const zoomRow = document.createElement('div');
         zoomRow.style.cssText =
            'display: flex; align-items: center; justify-content: space-between; padding: 4px 9px;';
         const zoomLabel = document.createElement('span');
         zoomLabel.style.cssText =
            'font-size: 12.5px; color: var(--fnx-secondary);';
         zoomLabel.textContent = 'Zoom';
         zoomRow.appendChild(zoomLabel);
         const zoomRight = document.createElement('div');
         zoomRight.style.cssText =
            'display: flex; align-items: center; gap: 6px;';
         const zoomBtns = document.createElement('div');
         zoomBtns.className = 'fnx-pop-seg';
         zoomBtns.style.cssText =
            'display: inline-flex; align-items: center; gap: 1px; padding: 2px;';
         const btnZoomOut = makeDesktopIconBtn(
            'minus',
            `Zoom out (${keybinds.zoom_out || 'Ctrl+-'})`,
            (e) => {
               e.stopPropagation();
               const cur = Number(config.default_zoom) || 1.0;
               const next = Math.max(0.5, Number((cur - 0.1).toFixed(1)));
               config.default_zoom = next;
               sendIpc({ type: 'SET_ZOOM', zoom: next });
               zoomValSpan.textContent = Math.round(next * 100) + '%';
            },
         );
         btnZoomOut.style.cssText =
            'width: 22px; height: 22px; border-radius: 5px; border: none;';
         const zoomValSpan = document.createElement('button');
         zoomValSpan.type = 'button';
         zoomValSpan.className = 'fnx-pop-link';
         zoomValSpan.style.cssText =
            'min-width: 42px; text-align: center; font-weight: 500; font-size: 11.5px; color: var(--fnx-fg);';
         zoomValSpan.textContent =
            Math.round((Number(config.default_zoom) || 1.0) * 100) + '%';
         zoomValSpan.title = 'Click to reset zoom (100%)';
         zoomValSpan.onclick = (e) => {
            e.stopPropagation();
            config.default_zoom = 1.0;
            sendIpc({ type: 'SET_ZOOM', zoom: 1.0 });
            zoomValSpan.textContent = '100%';
         };
         const btnZoomIn = makeDesktopIconBtn(
            'plus',
            `Zoom in (${keybinds.zoom_in || 'Ctrl+='})`,
            (e) => {
               e.stopPropagation();
               const cur = Number(config.default_zoom) || 1.0;
               const next = Math.min(2.5, Number((cur + 0.1).toFixed(1)));
               config.default_zoom = next;
               sendIpc({ type: 'SET_ZOOM', zoom: next });
               zoomValSpan.textContent = Math.round(next * 100) + '%';
            },
         );
         btnZoomIn.style.cssText =
            'width: 22px; height: 22px; border-radius: 5px; border: none;';
         const btnZoomFull = makeDesktopIconBtn(
            'fullscreen',
            `Fullscreen (${keybinds.fullscreen || 'F11'})`,
            (e) => {
               e.stopPropagation();
               closeAllNavPopovers();
               sendIpc({ type: 'TOGGLE_FULLSCREEN' });
            },
         );
         btnZoomFull.classList.add('fnx-desktop-only-row');
         btnZoomFull.style.cssText =
            'width: 26px; height: 26px; border-radius: 6px; border: 1px solid var(--fnx-border); background: var(--fnx-surface);';
         zoomBtns.appendChild(btnZoomOut);
         zoomBtns.appendChild(zoomValSpan);
         zoomBtns.appendChild(btnZoomIn);
         zoomRight.appendChild(zoomBtns);
         zoomRight.appendChild(btnZoomFull);
         zoomRow.appendChild(zoomRight);
         mainMenuEl.appendChild(zoomRow);

         addMenuSep();

         // Find in page
         addMenuItem(
            'search',
            'Find in page',
            keybinds.find_in_page || 'Ctrl+F',
            () => {
               if (!isInternalSettings()) openFindBar();
            },
         );

         // Reader Mode
         addMenuItem(
            'reader',
            'Reader mode',
            keybinds.reader_mode || 'Alt+R',
            () => {
               toggleReaderMode();
            },
         );

         // Downloads Folder (Desktop OS)
         const downloadsDirMenuItem = addMenuItem(
            'download',
            'Open downloads folder',
            '',
            () => {
               sendIpc({ type: 'OPEN_DOWNLOADS_DIR' });
            },
         );
         downloadsDirMenuItem.classList.add('fnx-desktop-only-row');

         // Popover 2: Media Control & Audio Booster (>100% Volume)
         mediaPopoverEl = document.createElement('div');
         mediaPopoverEl.id = '__fynexo_media_popover__';
         mediaPopoverEl.className = 'fnx-nav-popover fnx-nav-popover--panel';
         mediaPopoverEl.style.width = '288px';

         let currentMediaEl = null;
         let currentMediaTitle = '';
         let mediaGainValue = 100;
         let mediaAudioCtx = null;
         let mediaGainNode = null;
         let mediaCompressor = null;
         let mediaSourceNode = null;
         let mediaIsMuted = false;

         function attachMediaGain(mediaEl) {
            if (!mediaEl || mediaEl.__fnx_gain_node)
               return mediaEl.__fnx_gain_node || null;
            try {
               const AudioCtxClass =
                  window.AudioContext || window.webkitAudioContext;
               if (!AudioCtxClass) return null;
               if (!mediaAudioCtx) {
                  mediaAudioCtx = new AudioCtxClass();
               }
               if (mediaAudioCtx.state === 'suspended') {
                  mediaAudioCtx.resume();
               }
               mediaSourceNode =
                  mediaAudioCtx.createMediaElementSource(mediaEl);
               mediaGainNode = mediaAudioCtx.createGain();
               mediaCompressor = mediaAudioCtx.createDynamicsCompressor();
               mediaCompressor.threshold.setValueAtTime(
                  -3,
                  mediaAudioCtx.currentTime,
               );
               mediaCompressor.knee.setValueAtTime(
                  24,
                  mediaAudioCtx.currentTime,
               );
               mediaCompressor.ratio.setValueAtTime(
                  12,
                  mediaAudioCtx.currentTime,
               );
               mediaCompressor.attack.setValueAtTime(
                  0.003,
                  mediaAudioCtx.currentTime,
               );
               mediaCompressor.release.setValueAtTime(
                  0.25,
                  mediaAudioCtx.currentTime,
               );

               mediaSourceNode.connect(mediaGainNode);
               mediaGainNode.connect(mediaCompressor);
               mediaCompressor.connect(mediaAudioCtx.destination);
               mediaEl.__fnx_gain_node = mediaGainNode;
               return mediaGainNode;
            } catch (_) {
               return null;
            }
         }

         function applyMediaVolume(val) {
            mediaGainValue = Math.max(0, Math.min(300, Number(val) || 100));
            if (!currentMediaEl) return;
            if (mediaIsMuted) {
               currentMediaEl.muted = true;
               if (mediaGainNode && mediaAudioCtx) {
                  try {
                     mediaGainNode.gain.setValueAtTime(
                        0,
                        mediaAudioCtx.currentTime,
                     );
                  } catch (_) {}
               }
               return;
            }
            currentMediaEl.muted = false;
            if (mediaGainValue <= 100) {
               currentMediaEl.volume = mediaGainValue / 100;
               if (mediaGainNode && mediaAudioCtx) {
                  try {
                     mediaGainNode.gain.setValueAtTime(
                        1.0,
                        mediaAudioCtx.currentTime,
                     );
                  } catch (_) {}
               }
            } else {
               currentMediaEl.volume = 1.0;
               const g = attachMediaGain(currentMediaEl);
               if (g && mediaAudioCtx) {
                  try {
                     g.gain.setValueAtTime(
                        mediaGainValue / 100,
                        mediaAudioCtx.currentTime,
                     );
                  } catch (_) {}
               }
            }
         }

         const mediaHeader = document.createElement('div');
         mediaHeader.className = 'fnx-pop-head';
         const mediaHeaderLeft = document.createElement('div');
         mediaHeaderLeft.className = 'fnx-pop-title';
         mediaHeaderLeft.appendChild(createIcon('volumeUp'));
         const mediaHeaderTxt = document.createElement('span');
         mediaHeaderTxt.textContent = 'Media Control';
         mediaHeaderLeft.appendChild(mediaHeaderTxt);
         mediaHeader.appendChild(mediaHeaderLeft);

         const mediaStatusWrap = document.createElement('div');
         mediaStatusWrap.className = 'fnx-pop-status';
         const mediaStatusDot = document.createElement('span');
         mediaStatusDot.className = 'fnx-pop-dot';
         const mediaStatusPill = document.createElement('span');
         mediaStatusPill.textContent = 'Playing';
         mediaStatusWrap.appendChild(mediaStatusDot);
         mediaStatusWrap.appendChild(mediaStatusPill);
         mediaHeader.appendChild(mediaStatusWrap);
         mediaPopoverEl.appendChild(mediaHeader);

         const mediaTrackBody = document.createElement('div');
         mediaTrackBody.className = 'fnx-pop-body';
         const mediaTrackRow = document.createElement('div');
         mediaTrackRow.style.cssText =
            'display: flex; align-items: center; justify-content: space-between; gap: 10px;';
         const mediaMetaWrap = document.createElement('div');
         mediaMetaWrap.style.cssText =
            'min-width: 0; flex: 1; display: flex; flex-direction: column; gap: 2px;';
         const mediaTitleEl = document.createElement('div');
         mediaTitleEl.style.cssText =
            'font-weight: 500; font-size: 12.5px; white-space: nowrap; overflow: hidden; text-overflow: ellipsis; color: var(--fnx-fg);';
         mediaTitleEl.textContent = 'Media Player';
         const mediaDomainEl = document.createElement('div');
         mediaDomainEl.className = 'fnx-pop-sub';
         mediaDomainEl.textContent = window.location.hostname || 'fynexo';
         mediaMetaWrap.appendChild(mediaTitleEl);
         mediaMetaWrap.appendChild(mediaDomainEl);

         const mediaBtnGroup = document.createElement('div');
         mediaBtnGroup.style.cssText =
            'display: flex; align-items: center; gap: 5px; flex-shrink: 0;';
         const btnMute = document.createElement('button');
         btnMute.type = 'button';
         btnMute.className = 'fnx-pop-icon-circle';
         btnMute.title = 'Toggle Mute';
         btnMute.appendChild(createIcon('volumeUp'));
         btnMute.onclick = (e) => {
            e.stopPropagation();
            mediaIsMuted = !mediaIsMuted;
            applyMediaVolume(mediaGainValue);
            syncMediaPopoverUI();
         };
         const btnPlayPause = document.createElement('button');
         btnPlayPause.type = 'button';
         btnPlayPause.className = 'fnx-pop-icon-circle';
         btnPlayPause.title = 'Play / Pause';
         btnPlayPause.appendChild(createIcon('pause'));
         btnPlayPause.onclick = (e) => {
            e.stopPropagation();
            const target = isControllableMedia(currentMediaEl)
               ? currentMediaEl
               : findActiveMedia();
            if (!target) {
               currentMediaEl = null;
               updateMediaButtonState();
               return;
            }
            currentMediaEl = target;
            if (target.paused) {
               const p = target.play();
               if (p && typeof p.catch === 'function') p.catch(() => {});
            } else {
               target.pause();
            }
            requestAnimationFrame(() => {
               updateMediaButtonState();
               syncMediaPopoverUI();
            });
         };
         mediaBtnGroup.appendChild(btnMute);
         mediaBtnGroup.appendChild(btnPlayPause);
         mediaTrackRow.appendChild(mediaMetaWrap);
         mediaTrackRow.appendChild(mediaBtnGroup);
         mediaTrackBody.appendChild(mediaTrackRow);
         mediaPopoverEl.appendChild(mediaTrackBody);

         const volSection = document.createElement('div');
         volSection.className = 'fnx-pop-section';
         const volHeader = document.createElement('div');
         volHeader.className = 'fnx-pop-row';
         volHeader.style.minHeight = '18px';
         const volTextSpan = document.createElement('span');
         volTextSpan.className = 'fnx-pop-sub';
         volTextSpan.style.fontSize = '11.5px';
         volTextSpan.textContent = 'Volume & Boost';
         const volPercentLabel = document.createElement('span');
         volPercentLabel.style.cssText =
            'font-weight: 600; color: var(--fnx-fg); font-size: 11.5px;';
         volPercentLabel.textContent = '100%';
         volHeader.appendChild(volTextSpan);
         volHeader.appendChild(volPercentLabel);
         volSection.appendChild(volHeader);

         const volSlider = document.createElement('input');
         volSlider.type = 'range';
         volSlider.className = 'fnx-pop-range';
         volSlider.min = '0';
         volSlider.max = '300';
         volSlider.step = '5';
         volSlider.value = '100';
         volSlider.oninput = (e) => {
            e.stopPropagation();
            mediaIsMuted = false;
            applyMediaVolume(volSlider.value);
            syncMediaPopoverUI();
         };
         volSection.appendChild(volSlider);

         const presetsRow = document.createElement('div');
         presetsRow.className = 'fnx-pop-seg';
         presetsRow.style.gridTemplateColumns = 'repeat(5, 1fr)';
         const volPresets = [
            { val: 50, label: '50%' },
            { val: 100, label: '100%' },
            { val: 150, label: '150%' },
            { val: 200, label: '200%' },
            { val: 300, label: '300%' },
         ];
         const volPresetBtns = [];
         volPresets.forEach((p) => {
            const chip = document.createElement('button');
            chip.type = 'button';
            chip.dataset.vol = String(p.val);
            chip.className = 'fnx-pop-seg-btn';
            chip.textContent = p.label;
            chip.onclick = (e) => {
               e.stopPropagation();
               mediaIsMuted = false;
               volSlider.value = String(p.val);
               applyMediaVolume(p.val);
               syncMediaPopoverUI();
            };
            volPresetBtns.push(chip);
            presetsRow.appendChild(chip);
         });
         volSection.appendChild(presetsRow);
         mediaPopoverEl.appendChild(volSection);

         function syncMediaPopoverUI() {
            const isPlaying =
               currentMediaEl &&
               !currentMediaEl.paused &&
               !currentMediaEl.ended;
            mediaStatusPill.textContent = isPlaying ? 'Playing' : 'Paused';
            mediaStatusPill.style.color = isPlaying
               ? 'var(--fnx-fg)'
               : 'var(--fnx-muted)';
            mediaStatusDot.classList.toggle('off', !isPlaying);

            btnPlayPause.textContent = '';
            btnPlayPause.appendChild(createIcon(isPlaying ? 'pause' : 'play'));

            btnMute.textContent = '';
            btnMute.appendChild(
               createIcon(mediaIsMuted ? 'volumeMute' : 'volumeUp'),
            );

            if (mediaIsMuted) {
               volPercentLabel.textContent = 'Muted';
               volPercentLabel.style.color = 'var(--fnx-muted)';
            } else if (mediaGainValue > 100) {
               volPercentLabel.textContent = mediaGainValue + '%';
               volPercentLabel.style.color = 'var(--fnx-accent)';
            } else {
               volPercentLabel.textContent = mediaGainValue + '%';
               volPercentLabel.style.color = 'var(--fnx-fg)';
            }

            for (let i = 0; i < volPresetBtns.length; i++) {
               const b = volPresetBtns[i];
               b.classList.toggle(
                  'active',
                  !mediaIsMuted && Number(b.dataset.vol) === Number(mediaGainValue),
               );
            }

            const pageH1 = document.querySelector(
               'h1.ytd-watch-metadata, h1.title, h1',
            );
            const t =
               (pageH1 && pageH1.textContent.trim()) ||
               document.title ||
               'Media Player';
            mediaTitleEl.textContent = t;
            mediaDomainEl.textContent = isInternalAny()
               ? 'Internal Studio Page'
               : (window.location.hostname || '').replace(/^www\./, '') ||
                 'fynexo';
         }

         function isControllableMedia(m) {
            if (!m || !m.isConnected || m.ended || m.readyState === 0) {
               return false;
            }
            if (!m.currentSrc && !m.src) return false;
            const dur = m.duration;
            if (Number.isNaN(dur) ? m.paused : dur <= 0) return false;
            if (m.tagName === 'VIDEO') {
               try {
                  const r = m.getBoundingClientRect();
                  const cs = getComputedStyle(m);
                  if (
                     r.width < 2 ||
                     r.height < 2 ||
                     cs.display === 'none' ||
                     cs.visibility === 'hidden'
                  ) {
                     return false;
                  }
               } catch (_) {}
            }
            const hasAudibleTrack =
               (!m.muted && m.volume > 0) || isTabMutedState || mediaIsMuted;
            if (!m.paused) {
               return hasAudibleTrack || (m.controls && dur > 30);
            }
            return m.readyState >= 2 && m.currentTime > 0 && hasAudibleTrack;
         }

         function findActiveMedia() {
            const mediaEls = Array.from(
               document.querySelectorAll('video, audio'),
            );
            for (let i = 0; i < mediaEls.length; i++) {
               const m = mediaEls[i];
               if (!m.paused && isControllableMedia(m)) {
                  return m;
               }
            }
            if (
               currentMediaEl &&
               mediaEls.includes(currentMediaEl) &&
               isControllableMedia(currentMediaEl)
            ) {
               return currentMediaEl;
            }
            for (let i = 0; i < mediaEls.length; i++) {
               if (isControllableMedia(mediaEls[i])) {
                  return mediaEls[i];
               }
            }
            return null;
         }

         let lastReportedPlayingAudio = null;
         function reportMediaPlaybackState(force) {
            if (isInternalAny()) return;
            let playing = false;
            try {
               const mediaEls = document.querySelectorAll('video, audio');
               for (let i = 0; i < mediaEls.length; i++) {
                  const m = mediaEls[i];
                  if (
                     m &&
                     !m.paused &&
                     !m.ended &&
                     m.readyState > 0 &&
                     (isTabMutedState || (!m.muted && m.volume > 0))
                  ) {
                     playing = true;
                     break;
                  }
               }
            } catch (_) {}
            if (force || playing !== lastReportedPlayingAudio) {
               lastReportedPlayingAudio = playing;
               sendIpc({ type: 'UPDATE_TAB_MEDIA_STATE', playing: playing });
            }
         }

         function updateMediaButtonState() {
            reportMediaPlaybackState(false);
            const mbMoreDot = shadowRoot
               ? shadowRoot.getElementById('__fynexo_mb_more_dot__')
               : null;
            if (isInternalAny()) {
               currentMediaEl = null;
               btnMedia.style.display = 'none';
               if (mbMoreDot) mbMoreDot.classList.remove('active');
               return;
            }
            const m = findActiveMedia();
            const isPlaying = !!(m && !m.paused && !m.ended);
            if (m && (isPlaying || m.currentTime > 0)) {
               currentMediaEl = m;
               btnMedia.style.display = 'flex';
               mediaPulse.style.display = isPlaying ? 'inline-block' : 'none';
               if (mbMoreDot) mbMoreDot.classList.toggle('active', isPlaying);
            } else {
               currentMediaEl = null;
               btnMedia.style.display = 'none';
               if (mbMoreDot) mbMoreDot.classList.remove('active');
               if (mediaPopoverEl && mediaPopoverEl.style.display === 'flex') {
                  mediaPopoverEl.style.display = 'none';
               }
            }
         }

         document.addEventListener(
            'play',
            () => requestAnimationFrame(updateMediaButtonState),
            true,
         );
         document.addEventListener(
            'playing',
            () => requestAnimationFrame(updateMediaButtonState),
            true,
         );
         document.addEventListener(
            'pause',
            () => requestAnimationFrame(updateMediaButtonState),
            true,
         );
         document.addEventListener(
            'ended',
            () => requestAnimationFrame(updateMediaButtonState),
            true,
         );
         document.addEventListener(
            'emptied',
            () => {
               currentMediaEl = null;
               requestAnimationFrame(updateMediaButtonState);
            },
            true,
         );
         document.addEventListener(
            'volumechange',
            () => requestAnimationFrame(updateMediaButtonState),
            true,
         );
         document.addEventListener(
            'timeupdate',
            () => {
               if (
                  btnMedia.style.display === 'none' ||
                  lastReportedPlayingAudio !== true
               ) {
                  updateMediaButtonState();
               }
            },
            { passive: true, capture: true },
         );
         window.addEventListener(
            'yt-navigate-finish',
            () => setTimeout(() => updateMediaButtonState(), 250),
            { passive: true },
         );
         window.addEventListener(
            'turbo:load',
            () => setTimeout(() => updateMediaButtonState(), 250),
            { passive: true },
         );
         setTimeout(() => updateMediaPlaybackBoot(), 150);
         function updateMediaPlaybackBoot() {
            updateMediaButtonState();
         }

         // Popover 3: Adblock & Privacy Shield
         shieldPopoverEl = document.createElement('div');
         shieldPopoverEl.id = '__fynexo_shield_popover__';
         shieldPopoverEl.className = 'fnx-nav-popover fnx-nav-popover--panel';
         shieldPopoverEl.style.width = '268px';

         const shieldHeader = document.createElement('div');
         shieldHeader.className = 'fnx-pop-head';
         const shieldHeaderLeft = document.createElement('div');
         shieldHeaderLeft.className = 'fnx-pop-title';
         shieldHeaderLeft.appendChild(createIcon('shield'));
         const shieldHTxt = document.createElement('span');
         shieldHTxt.textContent = 'Adblock Shield';
         shieldHeaderLeft.appendChild(shieldHTxt);
         shieldHeader.appendChild(shieldHeaderLeft);

         const shieldStatusMeta = document.createElement('span');
         shieldStatusMeta.className = 'fnx-pop-sub';
         shieldStatusMeta.style.cssText =
            'font-size: 11px; color: var(--fnx-muted); flex-shrink: 0;';
         shieldHeader.appendChild(shieldStatusMeta);
         shieldPopoverEl.appendChild(shieldHeader);

         const shieldBody = document.createElement('div');
         shieldBody.className = 'fnx-pop-body';

         const shieldStatRow = document.createElement('div');
         shieldStatRow.className = 'fnx-pop-row';
         const shieldStatText = document.createElement('div');
         shieldStatText.style.cssText =
            'min-width: 0; display: flex; flex-direction: column; gap: 1px;';
         const shieldCountDesc = document.createElement('div');
         shieldCountDesc.className = 'fnx-pop-label';
         shieldCountDesc.textContent = 'Blocked elements';
         const shieldHostSub = document.createElement('div');
         shieldHostSub.className = 'fnx-pop-sub';
         shieldStatText.appendChild(shieldCountDesc);
         shieldStatText.appendChild(shieldHostSub);

         const shieldCountNum = document.createElement('div');
         shieldCountNum.id = '__fynexo_shield_pop_count__';
         shieldCountNum.style.cssText =
            'font-size: 20px; font-weight: 600; letter-spacing: -0.02em; color: var(--fnx-fg); flex-shrink: 0;';
         shieldCountNum.textContent = '0';
         shieldStatRow.appendChild(shieldStatText);
         shieldStatRow.appendChild(shieldCountNum);
         shieldBody.appendChild(shieldStatRow);

         const shieldToggleRow = document.createElement('div');
         shieldToggleRow.className = 'fnx-pop-row';
         shieldToggleRow.style.cssText =
            'padding-top: 8px; border-top: 1px solid var(--fnx-border);';
         const shieldToggleLabel = document.createElement('span');
         shieldToggleLabel.className = 'fnx-pop-label';
         shieldToggleLabel.textContent = 'Protection enabled';
         const shieldSwitch = document.createElement('button');
         shieldSwitch.type = 'button';
         shieldSwitch.role = 'switch';
         shieldSwitch.className = 'fnx-pop-switch';

         function syncShieldPopoverUI() {
            const internal = isInternalAny();
            const adOn = config.adblock !== false;
            shieldHostSub.textContent = internal
               ? 'System page · Exempt'
               : (window.location.hostname || '').replace(/^www\./, '') ||
                 'this page';
            shieldStatusMeta.textContent = internal
               ? 'Exempt'
               : adOn
                 ? '65+ domain rules'
                 : 'Paused';
            shieldSwitch.disabled = internal;
            shieldSwitch.classList.toggle('on', !internal && adOn);
            shieldSwitch.setAttribute(
               'aria-checked',
               !internal && adOn ? 'true' : 'false',
            );
            shieldSwitch.setAttribute(
               'aria-disabled',
               internal ? 'true' : 'false',
            );
            shieldToggleLabel.style.color = internal
               ? 'var(--fnx-muted)'
               : 'var(--fnx-fg)';
            if (internal) shieldCountNum.textContent = '0';
         }
         syncShieldPopoverUI();

         shieldSwitch.onclick = (e) => {
            e.stopPropagation();
            if (isInternalAny()) return;
            config.adblock = config.adblock === false;
            sendIpc({ type: 'PATCH_HUD_PREFS', adblock: config.adblock });
            applyPageRules();
            syncShieldPopoverUI();
            showHudToast(
               config.adblock
                  ? 'Adblock shield enabled'
                  : 'Adblock shield paused',
            );
         };
         shieldToggleRow.appendChild(shieldToggleLabel);
         shieldToggleRow.appendChild(shieldSwitch);
         shieldBody.appendChild(shieldToggleRow);
         shieldPopoverEl.appendChild(shieldBody);

         const btnOpenAdbStudio = document.createElement('button');
         btnOpenAdbStudio.type = 'button';
         btnOpenAdbStudio.className = 'fnx-pop-footer-row';
         const adbLinkLabel = document.createElement('span');
         adbLinkLabel.textContent = 'Manage rules & whitelist';
         const adbLinkArrow = document.createElement('span');
         adbLinkArrow.style.cssText = 'opacity: 0.65; font-size: 12px;';
         adbLinkArrow.textContent = '→';
         btnOpenAdbStudio.appendChild(adbLinkLabel);
         btnOpenAdbStudio.appendChild(adbLinkArrow);
         btnOpenAdbStudio.onclick = () => {
            closeAllNavPopovers();
            sendIpc({ type: 'NAVIGATE', url: 'fynexo://settings' });
         };
         shieldPopoverEl.appendChild(btnOpenAdbStudio);

         function updateShieldState() {
            if (isInternalAny()) {
               shieldBadge.style.display = 'none';
               return;
            }
            const ads = document.querySelectorAll(
               'ins.adsbygoogle, [id^="div-gpt-ad"], [id^="google_ads_iframe"], iframe[src*="doubleclick.net"], iframe[src*="googlesyndication.com"]',
            );
            const count = ads.length;
            if (count > 0 && config.adblock !== false) {
               shieldBadge.textContent = count > 99 ? '99+' : String(count);
               shieldBadge.style.display = 'flex';
            } else {
               shieldBadge.style.display = 'none';
            }
         }

         // Popover 4: Downloads Tray
         downloadsPopoverEl = document.createElement('div');
         downloadsPopoverEl.id = '__fynexo_downloads_popover__';
         downloadsPopoverEl.className = 'fnx-nav-popover fnx-nav-popover--panel';
         downloadsPopoverEl.style.width = '296px';

         let recentDownloads = [];

         const dlHeader = document.createElement('div');
         dlHeader.className = 'fnx-pop-head';
         const dlHeaderLeft = document.createElement('div');
         dlHeaderLeft.className = 'fnx-pop-title';
         dlHeaderLeft.appendChild(createIcon('download'));
         const dlHTxt = document.createElement('span');
         dlHTxt.textContent = 'Downloads';
         dlHeaderLeft.appendChild(dlHTxt);
         dlHeader.appendChild(dlHeaderLeft);
         const btnOpenDlFolder = document.createElement('button');
         btnOpenDlFolder.type = 'button';
         btnOpenDlFolder.className = 'fnx-pop-link fnx-desktop-only-row';
         btnOpenDlFolder.textContent = 'Open folder';
         btnOpenDlFolder.onclick = () => {
            sendIpc({ type: 'OPEN_DOWNLOADS_DIR' });
         };
         dlHeader.appendChild(btnOpenDlFolder);
         downloadsPopoverEl.appendChild(dlHeader);

         const dlListContainer = document.createElement('div');
         dlListContainer.id = '__fynexo_dl_list__';
         dlListContainer.style.cssText =
            'display: flex; flex-direction: column; gap: 2px; padding: 6px; max-height: 224px; overflow-y: auto;';
         downloadsPopoverEl.appendChild(dlListContainer);

         function renderDownloadsList() {
            dlListContainer.textContent = '';
            if (recentDownloads.length === 0) {
               const emptyTip = document.createElement('div');
               emptyTip.style.cssText =
                  'padding: 18px 8px; text-align: center; color: var(--fnx-muted); font-size: 12px;';
               emptyTip.textContent = 'No downloads in this session';
               dlListContainer.appendChild(emptyTip);
               return;
            }
            recentDownloads.forEach((dl) => {
               const row = document.createElement('div');
               row.className = 'fnx-pop-dl-row';
               const rowLeft = document.createElement('div');
               rowLeft.style.cssText =
                  'display: flex; align-items: center; gap: 8px; min-width: 0; flex: 1;';
               const fileIcon = createIcon('download');
               fileIcon.style.cssText =
                  'width: 13.5px; height: 13.5px; color: var(--fnx-secondary); flex-shrink: 0;';
               rowLeft.appendChild(fileIcon);
               const nameWrap = document.createElement('div');
               nameWrap.style.cssText =
                  'min-width: 0; display: flex; flex-direction: column; gap: 1px;';
               const nameSpan = document.createElement('span');
               nameSpan.style.cssText =
                  'font-size: 12px; font-weight: 500; white-space: nowrap; overflow: hidden; text-overflow: ellipsis; color: var(--fnx-fg);';
               nameSpan.textContent = dl.name;
               const timeSpan = document.createElement('span');
               timeSpan.className = 'fnx-pop-sub';
               timeSpan.style.fontSize = '10.5px';
               timeSpan.textContent = dl.time + ' · ' + dl.status;
               nameWrap.appendChild(nameSpan);
               nameWrap.appendChild(timeSpan);
               rowLeft.appendChild(nameWrap);
               row.appendChild(rowLeft);
               if (dl.status === 'Completed') {
                  const openSpan = document.createElement('span');
                  openSpan.className = 'fnx-pop-dl-open';
                  openSpan.textContent = 'Open';
                  row.appendChild(openSpan);
               }
               row.onclick = () => {
                  if (dl.status === 'Completed') {
                     sendIpc({ type: 'OPEN_DOWNLOAD_FILE', filename: dl.name });
                  } else {
                     sendIpc({ type: 'OPEN_DOWNLOADS_DIR' });
                  }
               };
               dlListContainer.appendChild(row);
            });
         }
         renderDownloadsList();

         function handleDownloadToast(msg) {
            if (!msg) return;
            const lower = msg.toLowerCase();
            if (
               lower.includes('download') ||
               lower.includes('saved') ||
               lower.includes('exported')
            ) {
               const match = msg.match(
                  /(?:Downloading|Saved|Exported)\s+(.+?)(?:\s+to\s+Downloads|$)/i,
               );
               const fname = match ? match[1] : 'File';
               const now = new Date();
               const timeStr = now.toLocaleTimeString([], {
                  hour: '2-digit',
                  minute: '2-digit',
               });
               const nextStatus = lower.includes('downloading')
                  ? 'Downloading...'
                  : 'Completed';
               const existing = recentDownloads.find((d) => d.name === fname);
               if (existing) {
                  existing.time = timeStr;
                  existing.status = nextStatus;
               } else {
                  recentDownloads.unshift({
                     name: fname,
                     time: timeStr,
                     status: nextStatus,
                  });
                  if (recentDownloads.length > 8) recentDownloads.pop();
               }
               btnDownloads.style.display = 'flex';
               dlBadge.textContent = String(recentDownloads.length);
               dlBadge.style.display = 'flex';
               renderDownloadsList();
            }
         }

         function toggleMainMenu(anchorBtn) {
            const isOpen = !!activeNavPopoverEl;
            if (isOpen) {
               closeAllNavPopovers();
            } else {
               closeAllNavPopovers();
               syncMainMenuUI();
               positionNavPopover(anchorBtn || btnMore, mainMenuEl);
               mainMenuEl.style.display = 'flex';
               activeNavPopoverEl = mainMenuEl;
            }
         }

         function toggleMediaPopover() {
            const isOpen =
               mediaPopoverEl &&
               (mediaPopoverEl.style.display === 'flex' ||
                  activeNavPopoverEl === mediaPopoverEl);
            if (isOpen) {
               closeAllNavPopovers();
            } else {
               closeAllNavPopovers();
               syncMediaPopoverUI();
               positionNavPopover(btnMedia, mediaPopoverEl);
               mediaPopoverEl.style.display = 'flex';
               activeNavPopoverEl = mediaPopoverEl;
            }
         }

         function toggleShieldPopover() {
            const isOpen =
               shieldPopoverEl &&
               (shieldPopoverEl.style.display === 'flex' ||
                  activeNavPopoverEl === shieldPopoverEl);
            if (isOpen) {
               closeAllNavPopovers();
            } else {
               closeAllNavPopovers();
               syncShieldPopoverUI();
               const ads = document.querySelectorAll(
                  'ins.adsbygoogle, [id^="div-gpt-ad"], [id^="google_ads_iframe"], iframe[src*="doubleclick.net"], iframe[src*="googlesyndication.com"]',
               );
               const c = shieldPopoverEl.querySelector(
                  '#__fynexo_shield_pop_count__',
               );
               if (c) c.textContent = String(ads.length);
               positionNavPopover(btnShield, shieldPopoverEl);
               shieldPopoverEl.style.display = 'flex';
               activeNavPopoverEl = shieldPopoverEl;
            }
         }

         function toggleDownloadsPopover() {
            const isOpen =
               downloadsPopoverEl &&
               (downloadsPopoverEl.style.display === 'flex' ||
                  activeNavPopoverEl === downloadsPopoverEl);
            if (isOpen) {
               closeAllNavPopovers();
            } else {
               closeAllNavPopovers();
               renderDownloadsList();
               positionNavPopover(btnDownloads, downloadsPopoverEl);
               downloadsPopoverEl.style.display = 'flex';
               activeNavPopoverEl = downloadsPopoverEl;
            }
         }

         shell.appendChild(mainMenuEl);
         shell.appendChild(mediaPopoverEl);
         shell.appendChild(shieldPopoverEl);
         shell.appendChild(downloadsPopoverEl);

         // Popover 5: Task Manager / Performance
         taskMgrPopoverEl = document.createElement('div');
         taskMgrPopoverEl.id = '__fynexo_taskmgr_popover__';
         taskMgrPopoverEl.className = 'fnx-nav-popover fnx-nav-popover--panel';
         taskMgrPopoverEl.style.width = '276px';

         const tmHeader = document.createElement('div');
         tmHeader.className = 'fnx-pop-head';
         const tmTitle = document.createElement('div');
         tmTitle.className = 'fnx-pop-title';
         tmTitle.appendChild(createIcon('cpu'));
         tmTitle.appendChild(document.createTextNode('Resource Monitor'));
         const tmBadge = document.createElement('span');
         tmBadge.className = 'fnx-pop-sub';
         tmBadge.textContent = 'Single-WebView';
         tmHeader.appendChild(tmTitle);
         tmHeader.appendChild(tmBadge);
         taskMgrPopoverEl.appendChild(tmHeader);

         const tmBody = document.createElement('div');
         tmBody.className = 'fnx-pop-body';

         const tmStatsWrap = document.createElement('div');
         tmStatsWrap.style.cssText =
            'display: flex; flex-direction: column; gap: 6px;';
         function makeStatRow(labelText, valueText, valueId, valueColor) {
            const row = document.createElement('div');
            row.className = 'fnx-pop-row';
            row.style.minHeight = '20px';
            const l = document.createElement('span');
            l.className = 'fnx-pop-sub';
            l.style.fontSize = '12px';
            l.textContent = labelText;
            const v = document.createElement('span');
            if (valueId) v.id = valueId;
            v.style.cssText = `font-size: 12px; font-weight: 600; color: ${valueColor || 'var(--fnx-fg)'};`;
            v.textContent = valueText;
            row.appendChild(l);
            row.appendChild(v);
            return row;
         }
         const tmHostRow = makeStatRow(
            'Working Set RAM',
            '-- MB',
            '__fynexo_tm_host__',
         );
         const tmCommitRow = makeStatRow(
            'Commit Memory',
            '-- MB',
            '__fynexo_tm_commit__',
         );
         const tmModeRow = makeStatRow(
            'Renderer Limit',
            '256 MB cap',
            '',
            '#10B981',
         );
         tmStatsWrap.appendChild(tmHostRow);
         tmStatsWrap.appendChild(tmCommitRow);
         tmStatsWrap.appendChild(tmModeRow);
         tmBody.appendChild(tmStatsWrap);

         const cacheWrap = document.createElement('div');
         cacheWrap.style.cssText =
            'display: flex; flex-direction: column; gap: 6px; border-top: 1px solid var(--fnx-border); padding-top: 9px;';
         const cacheTitle = document.createElement('div');
         cacheTitle.className = 'fnx-pop-sub';
         cacheTitle.style.fontSize = '11.5px';
         cacheTitle.textContent = 'Tab cache & freeze';
         cacheWrap.appendChild(cacheTitle);
         const cacheBtnRow = document.createElement('div');
         cacheBtnRow.className = 'fnx-pop-seg';
         cacheBtnRow.style.gridTemplateColumns = 'repeat(3, 1fr)';
         const cacheOptions = [
            { id: 'eco', label: 'Eco (2)' },
            {
               id: 'balanced',
               label: isMobilePlatform() ? 'Balanced (2)' : 'Balanced (3)',
            },
            {
               id: 'smooth',
               label: isMobilePlatform() ? 'Smooth (4)' : 'Smooth (6)',
            },
         ];
         const cacheBtns = [];
         function syncCacheModeButtons() {
            const cur = String(
               config.tab_cache_mode || 'balanced',
            ).toLowerCase();
            for (let i = 0; i < cacheBtns.length; i++) {
               const b = cacheBtns[i];
               const active =
                  b.dataset.mode === cur ||
                  (!['eco', 'balanced', 'smooth'].includes(cur) &&
                     b.dataset.mode === 'balanced');
               b.classList.toggle('active', active);
            }
         }
         for (let i = 0; i < cacheOptions.length; i++) {
            const opt = cacheOptions[i];
            const b = document.createElement('button');
            b.type = 'button';
            b.dataset.mode = opt.id;
            b.className = 'fnx-pop-seg-btn';
            b.textContent = opt.label;
            b.onclick = () => {
               config.tab_cache_mode = opt.id;
               syncCacheModeButtons();
               sendIpc({ type: 'PATCH_HUD_PREFS', tab_cache_mode: opt.id });
               showHudToast('Tab cache mode: ' + opt.label);
            };
            cacheBtns.push(b);
            cacheBtnRow.appendChild(b);
         }
         cacheWrap.appendChild(cacheBtnRow);
         tmBody.appendChild(cacheWrap);
         taskMgrPopoverEl.appendChild(tmBody);

         const tmFooter = document.createElement('div');
         tmFooter.className = 'fnx-pop-footer';
         const btnTrimMem = document.createElement('button');
         btnTrimMem.type = 'button';
         btnTrimMem.className = 'fnx-pop-btn';
         btnTrimMem.textContent = 'Flush working set';
         btnTrimMem.onclick = () => {
            sendIpc({ type: 'FORCE_TRIM_MEMORY' });
         };
         tmFooter.appendChild(btnTrimMem);
         taskMgrPopoverEl.appendChild(tmFooter);

         function openTaskMgrPopover() {
            closeAllNavPopovers();
            syncCacheModeButtons();
            positionNavPopover(btnMore, taskMgrPopoverEl);
            taskMgrPopoverEl.style.display = 'flex';
            activeNavPopoverEl = taskMgrPopoverEl;
            sendIpc({ type: 'GET_RESOURCE_USAGE' });
         }

         onResourceStatsCb = function (stats) {
            if (!stats) return;
            const h = taskMgrPopoverEl.querySelector('#__fynexo_tm_host__');
            const c = taskMgrPopoverEl.querySelector('#__fynexo_tm_commit__');
            if (h && stats.host_mb !== undefined)
               h.textContent = stats.host_mb + ' MB';
            if (c && stats.commit_mb !== undefined)
               c.textContent = stats.commit_mb + ' MB';
         };

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
         bmStartIcon.style.cssText =
            'display: inline-flex; align-items: center; color: var(--fnx-secondary);';
         bmStartIcon.appendChild(createIcon('grid'));
         const bmStartText = document.createElement('span');
         bmStartText.textContent = 'Start';
         bmStartBtn.appendChild(bmStartIcon);
         bmStartBtn.appendChild(bmStartText);
         bmStartBtn.onclick = () => {
            flushScrollSync();
            triggerProgress();
            sendIpc({ type: 'NAVIGATE', url: 'fynexo://newtab' });
         };
         bmRow.appendChild(bmStartBtn);

         const bmSep = document.createElement('span');
         bmSep.style.cssText =
            'width: 1px; height: 13px; background: var(--fnx-border); margin: 0 3px; flex-shrink: 0;';
         bmRow.appendChild(bmSep);

         const bmList = document.createElement('div');
         bmList.id = '__fynexo_bm_list__';
         bmRow.appendChild(bmList);

         const bmSepRight = document.createElement('span');
         bmSepRight.style.cssText =
            'width: 1px; height: 13px; background: var(--fnx-border); margin: 0 3px; flex-shrink: 0;';
         bmRow.appendChild(bmSepRight);

         const bmAddBtn = document.createElement('button');
         bmAddBtn.type = 'button';
         bmAddBtn.id = '__fynexo_bm_add__';
         bmAddBtn.className = 'fnx-bm-pill';
         bmAddBtn.style.color = 'var(--fnx-muted)';
         bmAddBtn.appendChild(createIcon('plus'));
         bmAddBtn.appendChild(document.createTextNode('Save page'));
         bmAddBtn.onclick = () => {
            if (isInternalAny()) {
               togglePreferences();
            } else {
               toggleCurrentBookmark();
            }
         };
         bmRow.appendChild(bmAddBtn);

         topbar.appendChild(bmRow);
         shell.appendChild(topbar);

         // 1B. Find in Page Bar
         const findBar = document.createElement('div');
         findBar.id = '__fynexo_find_bar__';
         const findIcon = document.createElement('span');
         findIcon.style.cssText =
            'display:flex;align-items:center;color:var(--fnx-muted);flex-shrink:0;';
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
         btnFindClose.appendChild(createIcon('close'));
         btnFindClose.onclick = () => closeFindBar();

         findInput.addEventListener('input', () =>
            runFindQuery(findInput.value),
         );
         findInput.addEventListener('keydown', (e) => {
            if (e.key === 'Enter') {
               e.preventDefault();
               stepFindMatch(e.shiftKey ? -1 : 1);
            } else if (e.key === 'Escape') {
               e.preventDefault();
               closeFindBar();
            }
         });

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

         const readerSizeSeg = document.createElement('div');
         readerSizeSeg.className = 'fnx-rd-seg';

         const btnRdSmaller = document.createElement('button');
         btnRdSmaller.type = 'button';
         btnRdSmaller.className = 'fnx-pop-seg-btn';
         btnRdSmaller.title = 'Decrease font size';
         btnRdSmaller.textContent = 'A−';
         btnRdSmaller.onclick = () => {
            readerFontPx = Math.max(14, readerFontPx - 2);
            syncReaderTypography();
         };

         const readerSizeVal = document.createElement('span');
         readerSizeVal.id = '__fynexo_reader_size__';
         readerSizeVal.className = 'fnx-rd-size';
         readerSizeVal.textContent = readerFontPx + 'px';

         const btnRdLarger = document.createElement('button');
         btnRdLarger.type = 'button';
         btnRdLarger.className = 'fnx-pop-seg-btn';
         btnRdLarger.title = 'Increase font size';
         btnRdLarger.textContent = 'A+';
         btnRdLarger.onclick = () => {
            readerFontPx = Math.min(26, readerFontPx + 2);
            syncReaderTypography();
         };

         readerSizeSeg.appendChild(btnRdSmaller);
         readerSizeSeg.appendChild(readerSizeVal);
         readerSizeSeg.appendChild(btnRdLarger);

         const readerWidthSeg = document.createElement('div');
         readerWidthSeg.className = 'fnx-rd-seg';

         const btnRdWidth = document.createElement('button');
         btnRdWidth.type = 'button';
         btnRdWidth.id = '__fynexo_reader_width__';
         btnRdWidth.className = 'fnx-pop-seg-btn';
         btnRdWidth.title = 'Toggle wide column';
         btnRdWidth.textContent = 'Wide';
         btnRdWidth.onclick = () => {
            readerWide = !readerWide;
            syncReaderTypography();
         };
         readerWidthSeg.appendChild(btnRdWidth);

         const btnRdExit = document.createElement('button');
         btnRdExit.type = 'button';
         btnRdExit.className = 'fnx-rd-btn';
         btnRdExit.appendChild(createIcon('close'));
         btnRdExit.appendChild(document.createTextNode('Exit Reader'));
         btnRdExit.onclick = () => closeReaderMode();

         readerCtrls.appendChild(readerSizeSeg);
         readerCtrls.appendChild(readerWidthSeg);
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
         mbBack.id = '__fynexo_mb_back__';
         mbBack.className = 'fnx-mb-btn';
         mbBack.setAttribute('aria-label', 'Back');
         mbBack.appendChild(createIcon('back'));
         mbBack.onclick = () => {
            requestHistoryNav('back');
         };

         const mbPill = document.createElement('div');
         mbPill.id = '__fynexo_mb_pill__';

         const mbPillTrigger = document.createElement('button');
         mbPillTrigger.id = '__fynexo_mb_pill_trigger__';
         const mbProtoIcon = document.createElement('span');
         mbProtoIcon.id = '__fynexo_mb_proto__';
         mbProtoIcon.style.cssText =
            'display: flex; align-items: center; flex-shrink: 0;';
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
         mbReload.onclick = (e) => {
            e.stopPropagation();
            triggerProgress();
            sendIpc({ type: 'RELOAD' });
         };

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

         const mbMore = document.createElement('button');
         mbMore.id = '__fynexo_mb_more__';
         mbMore.className = 'fnx-mb-btn fnx-popover-trigger';
         mbMore.setAttribute('aria-label', 'Menu & tools');
         mbMore.appendChild(createIcon('more'));
         const mbMoreDot = document.createElement('span');
         mbMoreDot.id = '__fynexo_mb_more_dot__';
         mbMoreDot.className = 'fnx-mb-dot';
         mbMore.appendChild(mbMoreDot);
         mbMore.onclick = (e) => {
            if (e && e.stopPropagation) e.stopPropagation();
            toggleMainMenu(mbMore);
         };

         mobileBar.appendChild(mbBack);
         mobileBar.appendChild(mbPill);
         mobileBar.appendChild(mbTabs);
         mobileBar.appendChild(mbMore);
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
         drawerHeader.style.cssText =
            'display: flex; align-items: center; justify-content: space-between; padding-bottom: 10px; margin-bottom: 8px; gap: 8px;';
         const drawerTitle = document.createElement('span');
         drawerTitle.style.cssText =
            'font-weight: 600; font-size: 15px; letter-spacing: -0.015em; color: var(--fnx-fg);';
         drawerTitle.textContent = 'Open tabs';

         const drawerHeadRight = document.createElement('div');
         drawerHeadRight.style.cssText =
            'display: flex; align-items: center; gap: 8px;';

         const btnDrawerReopen = document.createElement('button');
         btnDrawerReopen.type = 'button';
         btnDrawerReopen.id = '__fynexo_drawer_reopen__';
         btnDrawerReopen.className = 'fnx-drawer-head-btn';
         btnDrawerReopen.style.display = 'none';
         btnDrawerReopen.appendChild(createIcon('undo'));
         const drawerReopenTxt = document.createElement('span');
         drawerReopenTxt.className = 'fnx-drawer-reopen-txt';
         drawerReopenTxt.textContent = 'Reopen';
         btnDrawerReopen.appendChild(drawerReopenTxt);
         btnDrawerReopen.onclick = () => {
            closeMobileDrawer();
            triggerProgress();
            sendIpc({ type: 'REOPEN_CLOSED_TAB' });
         };

         const btnDrawerPrefs = document.createElement('button');
         btnDrawerPrefs.type = 'button';
         btnDrawerPrefs.id = '__fynexo_drawer_prefs__';
         btnDrawerPrefs.className = 'fnx-drawer-head-btn';
         btnDrawerPrefs.setAttribute('aria-label', 'Preferences');
         btnDrawerPrefs.appendChild(createIcon('settings'));
         const drawerPrefsTxt = document.createElement('span');
         drawerPrefsTxt.textContent = 'Settings';
         btnDrawerPrefs.appendChild(drawerPrefsTxt);
         btnDrawerPrefs.onclick = () => {
            closeMobileDrawer();
            togglePreferences();
         };

         const drawerCount = document.createElement('span');
         drawerCount.id = '__fynexo_drawer_count__';
         drawerCount.style.cssText =
            'font-size: 12px; color: var(--fnx-muted);';
         drawerCount.textContent = '1 tab';

         drawerHeadRight.appendChild(btnDrawerReopen);
         drawerHeadRight.appendChild(btnDrawerPrefs);
         drawerHeadRight.appendChild(drawerCount);
         drawerHeader.appendChild(drawerTitle);
         drawerHeader.appendChild(drawerHeadRight);

         const drawerList = document.createElement('div');
         drawerList.id = '__fynexo_drawer_list__';
         drawerList.style.cssText =
            'overflow-y: auto; flex: 1; max-height: 56vh;';

         const sheetBottomBar = document.createElement('div');
         sheetBottomBar.className = 'fnx-sheet-bottom-bar';

         const btnSheetFwd = document.createElement('button');
         btnSheetFwd.id = '__fynexo_sheet_fwd__';
         btnSheetFwd.style.cssText =
            'height: 38px; width: 42px; border: 1px solid var(--fnx-border); background: var(--fnx-surface); color: var(--fnx-secondary); border-radius: 9px; display: flex; align-items: center; justify-content: center; cursor: pointer; flex-shrink: 0;';
         btnSheetFwd.title = 'Forward';
         btnSheetFwd.appendChild(createIcon('forward'));
         btnSheetFwd.onclick = () => {
            closeMobileDrawer();
            requestHistoryNav('forward');
         };

         const btnSheetStar = document.createElement('button');
         btnSheetStar.id = '__fynexo_sheet_star__';
         btnSheetStar.dataset.starKind = 'star';
         btnSheetStar.style.cssText =
            'height: 38px; width: 42px; border: 1px solid var(--fnx-border); background: var(--fnx-surface); color: var(--fnx-secondary); border-radius: 9px; display: flex; align-items: center; justify-content: center; cursor: pointer; flex-shrink: 0;';
         btnSheetStar.title = 'Bookmark page';
         btnSheetStar.appendChild(createIcon('star'));
         btnSheetStar.onclick = () => toggleCurrentBookmark();

         const btnDrawerNew = document.createElement('button');
         btnDrawerNew.style.cssText =
            'flex: 1; height: 38px; padding: 0 14px; font-size: 13.5px; font-weight: 500; background: var(--fnx-accent); border: none; color: #ffffff; border-radius: 9px; cursor: pointer; display: flex; align-items: center; justify-content: center; gap: 6px;';
         btnDrawerNew.appendChild(createIcon('plus'));
         btnDrawerNew.appendChild(document.createTextNode('New tab'));
         btnDrawerNew.onclick = () => {
            closeMobileDrawer();
            flushScrollSync();
            triggerProgress();
            sendIpc({ type: 'NEW_TAB' });
         };

         const btnDrawerDone = document.createElement('button');
         btnDrawerDone.textContent = 'Done';
         btnDrawerDone.style.cssText =
            'height: 38px; padding: 0 15px; font-size: 13px; font-weight: 500; background: var(--fnx-surface); border: 1px solid var(--fnx-border); color: var(--fnx-fg); border-radius: 9px; cursor: pointer; flex-shrink: 0;';
         btnDrawerDone.onclick = () => closeMobileDrawer();

         sheetBottomBar.appendChild(btnSheetFwd);
         sheetBottomBar.appendChild(btnSheetStar);
         sheetBottomBar.appendChild(btnDrawerNew);
         sheetBottomBar.appendChild(btnDrawerDone);

         drawerSheet.appendChild(drawerHeader);
         drawerSheet.appendChild(drawerList);
         drawerSheet.appendChild(sheetBottomBar);
         mobileDrawer.appendChild(drawerSheet);
         mobileDrawer.onclick = (e) => {
            if (e.target === mobileDrawer) closeMobileDrawer();
         };
         shell.appendChild(mobileDrawer);

         // 3.5 Side Drawer / Web Panel (Alt+S)
         isSidePanelOpen = false;
         sidePanelEl = document.createElement('div');
         sidePanelEl.id = '__fynexo_side_panel__';

         const spHeader = document.createElement('div');
         spHeader.className = 'fnx-sp-header';

         const spTabsWrap = document.createElement('div');
         spTabsWrap.className = 'fnx-pop-seg fnx-sp-tabs';

         function makeSpTab(iconName, label, isActive) {
            const btn = document.createElement('button');
            btn.type = 'button';
            btn.className = isActive
               ? 'fnx-pop-seg-btn fnx-sp-tab active'
               : 'fnx-pop-seg-btn fnx-sp-tab';
            btn.appendChild(createIcon(iconName));
            btn.appendChild(document.createTextNode(label));
            return btn;
         }

         const btnSpWeb = makeSpTab('globe', 'Web Panel', true);
         const btnSpNotes = makeSpTab('fileText', 'Quick Notes', false);
         spTabsWrap.appendChild(btnSpWeb);
         spTabsWrap.appendChild(btnSpNotes);
         spHeader.appendChild(spTabsWrap);

         const btnSpClose = document.createElement('button');
         btnSpClose.type = 'button';
         btnSpClose.className = 'fnx-sp-close';
         btnSpClose.title = 'Close side panel (Esc)';
         btnSpClose.appendChild(createIcon('close'));
         btnSpClose.onclick = () => toggleSidePanel(false);
         spHeader.appendChild(btnSpClose);
         sidePanelEl.appendChild(spHeader);

         const spBodyWeb = document.createElement('div');
         spBodyWeb.className = 'fnx-sp-body';
         const spUrlBar = document.createElement('div');
         spUrlBar.className = 'fnx-sp-urlbar';
         const spInput = document.createElement('input');
         spInput.type = 'text';
         spInput.className = 'fnx-sp-input';
         spInput.placeholder = 'Enter URL for side panel...';
         const spGoBtn = document.createElement('button');
         spGoBtn.type = 'button';
         spGoBtn.className = 'fnx-sp-go';
         spGoBtn.textContent = 'Go';

         const spReloadBtn = document.createElement('button');
         spReloadBtn.type = 'button';
         spReloadBtn.className = 'fnx-sp-icon-btn';
         spReloadBtn.title = 'Reload side panel';
         spReloadBtn.appendChild(createIcon('reload'));

         const spPopBtn = document.createElement('button');
         spPopBtn.type = 'button';
         spPopBtn.className = 'fnx-sp-icon-btn';
         spPopBtn.title = 'Open in new tab (if blocked in frame)';
         spPopBtn.appendChild(createIcon('externalLink'));

         spUrlBar.appendChild(spInput);
         spUrlBar.appendChild(spGoBtn);
         spUrlBar.appendChild(spReloadBtn);
         spUrlBar.appendChild(spPopBtn);
         spBodyWeb.appendChild(spUrlBar);

         const spIframe = document.createElement('iframe');
         spIframe.className = 'fnx-sp-iframe';
         spIframe.setAttribute(
            'sandbox',
            'allow-scripts allow-same-origin allow-forms allow-popups allow-popups-to-escape-sandbox',
         );
         spIframe.src = 'about:blank';
         spBodyWeb.appendChild(spIframe);

         let sidePanelLastUrl = '';
         let sidePanelIdleTimer = null;
         function resolveSpHttpUrl(raw) {
            const trimmed = String(raw || '').trim();
            if (!trimmed) return '';
            const lower = trimmed.toLowerCase();
            if (/^[a-z][a-z0-9+.-]*:/i.test(trimmed)) {
               if (!lower.startsWith('http://') && !lower.startsWith('https://')) {
                  showHudToast('Side panel only supports http:// or https:// URLs');
                  return '';
               }
               return trimmed;
            }
            return 'https://' + trimmed;
         }
         function loadSpUrl(url) {
            const target = resolveSpHttpUrl(url);
            if (!target) return;
            sidePanelLastUrl = target;
            spInput.value = target;
            spIframe.src = target;
         }
         spGoBtn.onclick = () => loadSpUrl(spInput.value);
         spReloadBtn.onclick = () => {
            const target = resolveSpHttpUrl(spInput.value || sidePanelLastUrl);
            if (target) {
               spIframe.src = 'about:blank';
               requestAnimationFrame(() => loadSpUrl(target));
            }
         };
         spPopBtn.onclick = () => {
            const target = resolveSpHttpUrl(spInput.value || sidePanelLastUrl);
            if (!target) return;
            toggleSidePanel(false);
            triggerProgress();
            sendIpc({ type: 'NEW_TAB', url: target });
         };
         spInput.onkeydown = (e) => {
            if (e.key === 'Enter') loadSpUrl(spInput.value);
         };

         const spBodyNotes = document.createElement('div');
         spBodyNotes.className = 'fnx-sp-body fnx-sp-notes-wrap hidden';
         const spTextarea = document.createElement('textarea');
         spTextarea.className = 'fnx-sp-textarea';
         spTextarea.placeholder =
            'Type quick notes, scratchpad, code snippets here... Auto-saved locally.';
         try {
            spTextarea.value =
               window.localStorage.getItem('__fnx_side_notes__') || '';
         } catch (_) {}
         spTextarea.oninput = () => {
            try {
               window.localStorage.setItem(
                  '__fnx_side_notes__',
                  spTextarea.value,
               );
            } catch (_) {}
         };
         spBodyNotes.appendChild(spTextarea);

         btnSpWeb.onclick = () => {
            btnSpWeb.classList.add('active');
            btnSpNotes.classList.remove('active');
            spBodyWeb.classList.remove('hidden');
            spBodyNotes.classList.add('hidden');
         };
         btnSpNotes.onclick = () => {
            btnSpNotes.classList.add('active');
            btnSpWeb.classList.remove('active');
            spBodyNotes.classList.remove('hidden');
            spBodyWeb.classList.add('hidden');
         };

         sidePanelEl.appendChild(spBodyWeb);
         sidePanelEl.appendChild(spBodyNotes);
         shell.appendChild(sidePanelEl);

         function toggleSidePanel(force) {
            const next = force !== undefined ? !!force : !isSidePanelOpen;
            isSidePanelOpen = next;
            if (sidePanelIdleTimer) {
               clearTimeout(sidePanelIdleTimer);
               sidePanelIdleTimer = null;
            }
            sidePanelEl.style.transform = next
               ? 'translateX(0)'
               : 'translateX(100%)';
            if (typeof btnSidePanel !== 'undefined' && btnSidePanel) {
               btnSidePanel.style.color = next
                  ? 'var(--fnx-accent)'
                  : 'inherit';
            }
            if (next) {
               if (
                  !spBodyWeb.classList.contains('hidden') &&
                  spIframe.src === 'about:blank'
               ) {
                  loadSpUrl(sidePanelLastUrl || 'https://translate.google.com');
               }
            } else if (spIframe.src && spIframe.src !== 'about:blank') {
               sidePanelIdleTimer = setTimeout(() => {
                  sidePanelIdleTimer = null;
                  if (!isSidePanelOpen) {
                     spIframe.src = 'about:blank';
                  }
               }, 45000);
            }
         }
         closeSidePanelRef = () => toggleSidePanel(false);
         isSidePanelOpenRef = () => isSidePanelOpen;

         // 3.6 History Drawer Modal (Linear / Raycast Style)
         const historyRoot = document.createElement('div');
         historyRoot.id = '__fynexo_history_root__';

         const historyCard = document.createElement('div');
         historyCard.id = '__fynexo_history_card__';
         historyCard.onclick = (e) => e.stopPropagation();

         const histHeader = document.createElement('div');
         histHeader.className = 'fnx-pop-header';
         const histTitle = document.createElement('span');
         histTitle.className = 'fnx-pop-title';
         histTitle.textContent = 'Browsing History';
         const histCountBadge = document.createElement('span');
         histCountBadge.className = 'fnx-pop-badge';
         const histCloseBtn = document.createElement('button');
         histCloseBtn.type = 'button';
         histCloseBtn.className = 'fnx-sp-close';
         histCloseBtn.title = 'Close (Esc)';
         histCloseBtn.appendChild(createIcon('close'));
         const histHeadRight = document.createElement('div');
         histHeadRight.style.cssText =
            'display: inline-flex; align-items: center; gap: 6px;';
         histHeadRight.appendChild(histCountBadge);
         histHeadRight.appendChild(histCloseBtn);
         histHeader.appendChild(histTitle);
         histHeader.appendChild(histHeadRight);

         const histSearchWrap = document.createElement('div');
         histSearchWrap.className = 'fnx-hist-search-wrap';
         histSearchWrap.appendChild(createIcon('search'));
         const histSearchInput = document.createElement('input');
         histSearchInput.type = 'text';
         histSearchInput.className = 'fnx-hist-search-input';
         histSearchInput.placeholder = 'Search visited pages...';
         histSearchWrap.appendChild(histSearchInput);

         const histList = document.createElement('div');
         histList.className = 'fnx-hist-list';

         const histFooter = document.createElement('button');
         histFooter.type = 'button';
         histFooter.className = 'fnx-pop-footer-row';
         const histFooterLabel = document.createElement('span');
         histFooterLabel.textContent = 'Manage browsing history';
         const histFooterArrow = document.createElement('span');
         histFooterArrow.textContent = '→';
         histFooter.appendChild(histFooterLabel);
         histFooter.appendChild(histFooterArrow);

         historyCard.appendChild(histHeader);
         historyCard.appendChild(histSearchWrap);
         historyCard.appendChild(histList);
         historyCard.appendChild(histFooter);
         historyRoot.appendChild(historyCard);
         shell.appendChild(historyRoot);

         let isHistoryDrawerOpen = false;
         function closeHistoryDrawer() {
            if (!isHistoryDrawerOpen) return;
            isHistoryDrawerOpen = false;
            historyRoot.style.display = 'none';
            histSearchInput.value = '';
            histList.textContent = '';
         }
         function renderHistoryDrawerList(filterText) {
            const q = String(filterText || '')
               .trim()
               .toLowerCase();
            const allItems = Array.isArray(historyItems) ? historyItems : [];
            const filtered = q
               ? allItems.filter((h) => {
                    const t = String((h && h.title) || '').toLowerCase();
                    const u = String((h && h.url) || '').toLowerCase();
                    return t.includes(q) || u.includes(q);
                 })
               : allItems;
            histCountBadge.textContent =
               filtered.length === 1 ? '1 item' : `${filtered.length} items`;
            histList.textContent = '';

            if (filtered.length === 0) {
               const empty = document.createElement('div');
               empty.className = 'fnx-hist-empty';
               empty.textContent = q
                  ? 'No matching pages found in history.'
                  : 'No visited pages recorded yet.';
               histList.appendChild(empty);
               return;
            }

            const frag = document.createDocumentFragment();
            filtered.slice(0, 60).forEach((h) => {
               if (!h || !h.url) return;
               const row = document.createElement('div');
               row.className = 'fnx-hist-row';
               row.role = 'button';
               row.tabIndex = 0;

               const emblem = document.createElement('div');
               emblem.className = 'fnx-hist-emblem';
               renderSiteEmblem(emblem, h.url, h.title);

               const meta = document.createElement('div');
               meta.className = 'fnx-hist-meta';
               const tEl = document.createElement('div');
               tEl.className = 'fnx-hist-title';
               tEl.textContent = h.title || formatHostLabel(h.url);
               const uEl = document.createElement('div');
               uEl.className = 'fnx-hist-url';
               uEl.textContent = formatHostLabel(h.url);
               uEl.title = h.url;
               meta.appendChild(tEl);
               meta.appendChild(uEl);

               const bgBtn = document.createElement('button');
               bgBtn.type = 'button';
               bgBtn.className = 'fnx-hist-open-btn';
               bgBtn.title = 'Open in new tab';
               bgBtn.appendChild(createIcon('externalLink'));
               bgBtn.onclick = (ev) => {
                  ev.stopPropagation();
                  sendIpc({
                     type: 'NEW_TAB',
                     url: h.url,
                     title: h.title || '',
                     background: true,
                  });
               };

               const openCurrent = () => {
                  closeHistoryDrawer();
                  triggerProgress();
                  sendIpc({ type: 'NAVIGATE', url: h.url });
               };
               row.onclick = openCurrent;
               row.onkeydown = (ev) => {
                  if (ev.key === 'Enter' || ev.key === ' ') {
                     ev.preventDefault();
                     openCurrent();
                  }
               };

               row.appendChild(emblem);
               row.appendChild(meta);
               row.appendChild(bgBtn);
               frag.appendChild(row);
            });
            histList.appendChild(frag);
         }
         function openHistoryDrawer() {
            if (typeof closeAllNavPopovers === 'function')
               closeAllNavPopovers();
            hideTabContextMenu();
            isHistoryDrawerOpen = true;
            histSearchInput.value = '';
            renderHistoryDrawerList('');
            historyRoot.style.display = 'flex';
            requestAnimationFrame(() => {
               try {
                  histSearchInput.focus();
               } catch (_) {}
            });
         }
         histSearchInput.oninput = () =>
            renderHistoryDrawerList(histSearchInput.value);
         histSearchInput.onkeydown = (e) => {
            if (e.key === 'Escape') {
               e.preventDefault();
               closeHistoryDrawer();
               restorePageFocus();
            }
         };
         histCloseBtn.onclick = () => closeHistoryDrawer();
         historyRoot.onclick = () => closeHistoryDrawer();
         histFooter.onclick = () => {
            closeHistoryDrawer();
            triggerProgress();
            sendIpc({ type: 'NAVIGATE', url: 'fynexo://settings' });
         };
         openHistoryDrawerRef = openHistoryDrawer;
         closeHistoryDrawerRef = closeHistoryDrawer;
         isHistoryOpenRef = () => isHistoryDrawerOpen;

         // 3.7 Custom Prompt Dialog (Replaces native window.prompt)
         const promptRoot = document.createElement('div');
         promptRoot.id = '__fynexo_prompt_root__';

         const promptCard = document.createElement('div');
         promptCard.id = '__fynexo_prompt_card__';
         promptCard.onclick = (e) => e.stopPropagation();

         const promptHeader = document.createElement('div');
         promptHeader.className = 'fnx-pop-header';
         const promptTitle = document.createElement('span');
         promptTitle.className = 'fnx-pop-title';
         const promptBadge = document.createElement('span');
         promptBadge.className = 'fnx-pop-badge';
         promptHeader.appendChild(promptTitle);
         promptHeader.appendChild(promptBadge);

         const promptBody = document.createElement('div');
         promptBody.className = 'fnx-prompt-body';
         const promptDesc = document.createElement('div');
         promptDesc.className = 'fnx-prompt-desc';
         const promptInput = document.createElement('input');
         promptInput.type = 'text';
         promptInput.className = 'fnx-prompt-input';

         const promptActions = document.createElement('div');
         promptActions.className = 'fnx-prompt-actions';
         const promptCancelBtn = document.createElement('button');
         promptCancelBtn.type = 'button';
         promptCancelBtn.className = 'fnx-prompt-btn';
         promptCancelBtn.textContent = 'Cancel';
         const promptConfirmBtn = document.createElement('button');
         promptConfirmBtn.type = 'button';
         promptConfirmBtn.className = 'fnx-prompt-btn primary';
         promptConfirmBtn.textContent = 'Confirm';
         promptActions.appendChild(promptCancelBtn);
         promptActions.appendChild(promptConfirmBtn);

         promptBody.appendChild(promptDesc);
         promptBody.appendChild(promptInput);
         promptBody.appendChild(promptActions);
         promptCard.appendChild(promptHeader);
         promptCard.appendChild(promptBody);
         promptRoot.appendChild(promptCard);
         shell.appendChild(promptRoot);

         let isPromptDialogOpen = false;
         let isPromptConfirmOnly = false;
         let activePromptConfirm = null;
         function closePromptModal() {
            if (!isPromptDialogOpen) return;
            isPromptDialogOpen = false;
            isPromptConfirmOnly = false;
            activePromptConfirm = null;
            promptInput.value = '';
            promptRoot.style.display = 'none';
         }
         function submitPromptModal() {
            const val = isPromptConfirmOnly ? 'confirm' : promptInput.value;
            if (!isPromptConfirmOnly && (!val || !val.trim())) {
               try {
                  promptInput.focus();
               } catch (_) {}
               return;
            }
            const cb = activePromptConfirm;
            closePromptModal();
            if (typeof cb === 'function') cb(val.trim());
         }
         function openPromptModal(opts, onConfirm) {
            const cfg = opts || {};
            isPromptConfirmOnly = !!cfg.hideInput;
            promptTitle.textContent = cfg.title || 'Prompt';
            if (cfg.badge) {
               promptBadge.textContent = cfg.badge;
               promptBadge.style.display = 'inline-block';
            } else {
               promptBadge.style.display = 'none';
            }
            if (cfg.description) {
               promptDesc.textContent = cfg.description;
               promptDesc.style.display = 'block';
            } else {
               promptDesc.style.display = 'none';
            }
            promptInput.style.display = isPromptConfirmOnly ? 'none' : 'block';
            promptInput.placeholder = cfg.placeholder || '';
            promptInput.value = cfg.defaultValue || '';
            promptConfirmBtn.textContent = cfg.confirmText || 'Confirm';
            activePromptConfirm = onConfirm;
            isPromptDialogOpen = true;
            promptRoot.style.display = 'flex';
            requestAnimationFrame(() => {
               try {
                  if (isPromptConfirmOnly) {
                     promptConfirmBtn.focus();
                  } else {
                     promptInput.focus();
                     promptInput.select();
                  }
               } catch (_) {}
            });
         }
         promptCancelBtn.onclick = () => closePromptModal();
         promptConfirmBtn.onclick = () => submitPromptModal();
         promptRoot.onclick = () => closePromptModal();
         promptInput.onkeydown = (e) => {
            if (e.key === 'Enter') {
               e.preventDefault();
               submitPromptModal();
            } else if (e.key === 'Escape') {
               e.preventDefault();
               closePromptModal();
               restorePageFocus();
            }
         };
         openPromptModalRef = openPromptModal;
         closePromptModalRef = closePromptModal;
         isPromptOpenRef = () => isPromptDialogOpen;

         // 4. Lazy-Input Omnibox System (0 <input> mounted at idle; swaps button -> <input> ONLY on click/Ctrl+L)
         const hudRoot = document.createElement('div');
         hudRoot.id = '__fynexo_hud_root__';

         const hudCard = document.createElement('div');
         hudCard.id = '__fynexo_hud_card__';

         const modalBar = document.createElement('div');
         modalBar.style.cssText =
            'display: flex; align-items: center; padding: 10px 11px; gap: 9px;';

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

         function positionInlineDropdown(dropdownEl) {
            if (!dropdownEl) return;
            const rect = addrSlot.getBoundingClientRect();
            const vw = window.innerWidth || 1024;
            const isDeck =
               (
                  (styles.topbar && styles.topbar.layout) ||
                  'deck'
               ).toLowerCase() !== 'compact';
            const dropW = isDeck
               ? Math.min(Math.max(rect.width, 440), vw - 16)
               : Math.min(Math.max(rect.width, 440), vw - 16);
            let leftPx = rect.left;
            if (!isDeck) {
               const pos = (
                  styles.address_bar.position || 'right'
               ).toLowerCase();
               if (pos === 'right') {
                  leftPx = rect.right - dropW;
               } else if (pos === 'center') {
                  leftPx = rect.left + (rect.width - dropW) / 2;
               }
            }
            leftPx = Math.max(8, Math.min(leftPx, vw - dropW - 8));
            dropdownEl.style.width = Math.round(dropW) + 'px';
            dropdownEl.style.left = Math.round(leftPx) + 'px';
            dropdownEl.style.top = Math.round(rect.bottom + 4) + 'px';
         }

         function deactivateInlineAddressBar() {
            if (!isInlineActive) return;
            isInlineActive = false;
            const box = shadowRoot.getElementById('__fynexo_inline_box__');
            if (box) box.remove();
            const drop = shadowRoot.getElementById(
               '__fynexo_inline_dropdown__',
            );
            if (drop) drop.remove();
            addrSlot.classList.remove('editing');
            addrBtn.style.display = 'flex';
            suggestItems = [];
            suggestIndex = -1;
         }

         function deactivateModalHud() {
            if (!isModalActive && hudRoot.style.display !== 'flex') return;
            isModalActive = false;
            hudRoot.style.display = 'none';
            modalBar.textContent = '';
            hudSuggest.textContent = '';
            hudSuggest.style.display = 'none';
            hudQuick.textContent = '';
            suggestItems = [];
            suggestIndex = -1;
         }

         function closeAnyOmnibox() {
            deactivateInlineAddressBar();
            deactivateModalHud();
            restorePageFocus();
         }
         closeOmniboxRef = closeAnyOmnibox;

         hudRoot.onclick = (e) => {
            if (e.target === hudRoot) deactivateModalHud();
         };

         function activateInlineAddressBar() {
            if (isModalActive) deactivateModalHud();
            const existingInp = shadowRoot.getElementById(
               '__fynexo_inline_input__',
            );
            if (isInlineActive && existingInp) {
               existingInp.focus();
               if (existingInp.value) existingInp.select();
               return;
            }

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
            inp.placeholder = `Search ${currentEngineObj().name} or enter !bang / URL...`;

            const btnClear = document.createElement('button');
            btnClear.type = 'button';
            btnClear.className = 'fnx-inline-clear';
            btnClear.title = 'Clear';
            btnClear.appendChild(createIcon('clear'));
            btnClear.onmousedown = (e) => e.preventDefault();

            const href = canonicalCurrentUrl();
            if (
               !isInternalAny(href) &&
               !href.startsWith('data:') &&
               href !== 'about:blank'
            ) {
               inp.value = href;
            }

            const syncClear = () => {
               btnClear.style.display = inp.value.length > 0 ? 'flex' : 'none';
            };
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
            renderOmniboxSuggestions(
               '',
               inlineSuggest,
               inp,
               deactivateInlineAddressBar,
            );

            const submitInline = () => {
               const target = resolveUrl(inp.value, config.search_engine);
               deactivateInlineAddressBar();
               if (target) {
                  flushScrollSync();
                  triggerProgress();
                  sendIpc({ type: 'NAVIGATE', url: target });
               }
            };

            inp.addEventListener('input', () => {
               syncClear();
               renderOmniboxSuggestions(
                  inp.value,
                  inlineSuggest,
                  inp,
                  deactivateInlineAddressBar,
               );
            });

            btnClear.onclick = () => {
               inp.value = '';
               syncClear();
               renderOmniboxSuggestions(
                  '',
                  inlineSuggest,
                  inp,
                  deactivateInlineAddressBar,
               );
               inp.focus();
            };

            engBtn.onclick = () => {
               const list = getEngines();
               const curIdx = list.findIndex(
                  (e) => e.id === config.search_engine,
               );
               const next = list[(curIdx + 1) % list.length];
               if (next) {
                  config.search_engine = next.id;
                  engBtn.textContent = next.name;
                  inp.placeholder = `Search ${next.name} or enter !bang / URL...`;
                  updateAddressLabels();
                  sendIpc({
                     type: 'PATCH_HUD_PREFS',
                     search_engine: config.search_engine,
                  });
               }
               inp.focus();
            };

            bindOmniboxKeydown(
               inp,
               inlineSuggest,
               submitInline,
               deactivateInlineAddressBar,
            );

            requestAnimationFrame(() => {
               positionInlineDropdown(dropdown);
               inp.focus();
               if (inp.value) inp.select();
            });
         }

         function openModalHud() {
            if (isInlineActive) deactivateInlineAddressBar();
            const existingInp = shadowRoot.getElementById('__fynexo_input__');
            if (isModalActive && existingInp) {
               existingInp.focus();
               if (existingInp.value) existingInp.select();
               return;
            }

            isModalActive = true;
            modalBar.textContent = '';

            const engineBadge = document.createElement('button');
            engineBadge.type = 'button';
            engineBadge.id = '__fynexo_hud_engine__';
            engineBadge.title = 'Click to switch search engine';
            engineBadge.textContent = currentEngineObj().name || 'Google';
            engineBadge.style.cssText =
               'font-size: 11.5px; font-weight: 500; color: var(--fnx-fg); background: var(--fnx-surface); border: 1px solid var(--fnx-border-hover); padding: 5px 9px; border-radius: 8px; cursor: pointer; white-space: nowrap; flex-shrink: 0;';
            engineBadge.onmousedown = (e) => e.preventDefault();

            const input = document.createElement('input');
            input.id = '__fynexo_input__';
            input.type = 'text';
            input.spellcheck = false;
            input.autocomplete = 'off';
            input.setAttribute('autocapitalize', 'off');
            input.setAttribute('enterkeyhint', 'go');
            input.placeholder = `Search ${currentEngineObj().name} or enter !bang / URL...`;
            input.style.cssText =
               'flex: 1; min-width: 0; background: transparent; border: none; outline: none; color: var(--fnx-fg); letter-spacing: -0.01em; user-select: auto;';

            const btnClear = document.createElement('button');
            btnClear.type = 'button';
            btnClear.title = 'Clear';
            btnClear.style.cssText =
               'width: 26px; height: 26px; border: none; background: transparent; display: none; align-items: center; justify-content: center; cursor: pointer; flex-shrink: 0; opacity: 0.85;';
            btnClear.appendChild(createIcon('clear'));
            btnClear.onmousedown = (e) => e.preventDefault();

            const btnGo = document.createElement('button');
            btnGo.type = 'button';
            btnGo.title = 'Go';
            btnGo.style.cssText =
               'width: 32px; height: 30px; border: none; background: var(--fnx-accent); color: #ffffff; border-radius: 8px; display: flex; align-items: center; justify-content: center; cursor: pointer; flex-shrink: 0;';
            btnGo.appendChild(createIcon('go'));
            btnGo.onmousedown = (e) => e.preventDefault();

            const href = canonicalCurrentUrl();
            if (
               !isInternalAny(href) &&
               !href.startsWith('data:') &&
               href !== 'about:blank'
            ) {
               input.value = href;
            }

            const syncClearVisibility = () => {
               btnClear.style.display =
                  input.value.length > 0 ? 'flex' : 'none';
            };
            syncClearVisibility();

            const submitModal = () => {
               const target = resolveUrl(input.value, config.search_engine);
               deactivateModalHud();
               if (target) {
                  flushScrollSync();
                  triggerProgress();
                  sendIpc({ type: 'NAVIGATE', url: target });
               }
            };

            input.addEventListener('input', () => {
               syncClearVisibility();
               renderOmniboxSuggestions(
                  input.value,
                  hudSuggest,
                  input,
                  deactivateModalHud,
               );
            });

            btnClear.onclick = () => {
               input.value = '';
               syncClearVisibility();
               renderOmniboxSuggestions(
                  '',
                  hudSuggest,
                  input,
                  deactivateModalHud,
               );
               input.focus();
            };

            engineBadge.onclick = () => {
               const list = getEngines();
               const curIdx = list.findIndex(
                  (e) => e.id === config.search_engine,
               );
               const next = list[(curIdx + 1) % list.length];
               if (next) {
                  config.search_engine = next.id;
                  engineBadge.textContent = next.name;
                  input.placeholder = `Search ${next.name} or enter !bang / URL...`;
                  updateAddressLabels();
                  sendIpc({
                     type: 'PATCH_HUD_PREFS',
                     search_engine: config.search_engine,
                  });
               }
               input.focus();
            };

            btnGo.onclick = submitModal;
            bindOmniboxKeydown(
               input,
               hudSuggest,
               submitModal,
               deactivateModalHud,
            );

            modalBar.appendChild(engineBadge);
            modalBar.appendChild(input);
            modalBar.appendChild(btnClear);
            modalBar.appendChild(btnGo);

            renderHudQuickBar(hudQuick, deactivateModalHud);
            renderOmniboxSuggestions('', hudSuggest, input, deactivateModalHud);
            hudRoot.style.display = 'flex';
            updateAddressLabels();

            requestAnimationFrame(() => {
               input.focus();
               if (input.value) input.select();
            });
         }

         function openHud() {
            const useInline =
               window.innerWidth > 768 &&
               (styles.address_bar.mode || 'inline').toLowerCase() !== 'modal';
            if (useInline) {
               activateInlineAddressBar();
            } else {
               openModalHud();
            }
         }

         // Close inline <input> & restore <button> when clicking outside
         // Note: shadowRoot uses mode:'closed', so window.composedPath() cannot see internal nodes;
         // handle internal Shadow DOM hits on shadowRoot itself and outside-HUD clicks on window.
         shadowRoot.addEventListener(
            'pointerdown',
            (e) => {
               if (!isInlineActive) return;
               const path =
                  typeof e.composedPath === 'function' ? e.composedPath() : [];
               const dropEl = shadowRoot.getElementById(
                  '__fynexo_inline_dropdown__',
               );
               const targetEl =
                  e.target && e.target.nodeType === 1
                     ? e.target
                     : e.target && e.target.parentElement;
               const inAddr =
                  path.includes(addrSlot) ||
                  (targetEl && addrSlot.contains(targetEl));
               const inDrop =
                  dropEl &&
                  (path.includes(dropEl) ||
                     (targetEl && dropEl.contains(targetEl)));
               if (!inAddr && !inDrop) {
                  deactivateInlineAddressBar();
               }
            },
            true,
         );

         const isEventInsideHud = (e) => {
            const hostEl = document.getElementById('__fynexo_host__');
            const path =
               typeof e.composedPath === 'function' ? e.composedPath() : [];
            return Boolean(
               hostEl &&
                  (e.target === hostEl ||
                     (hostEl.contains && hostEl.contains(e.target)) ||
                     path.includes(hostEl)),
            );
         };

         window.addEventListener(
            'pointerdown',
            (e) => {
               if (!isInlineActive) return;
               if (isEventInsideHud(e)) return;
               deactivateInlineAddressBar();
            },
            true,
         );

         window.addEventListener(
            'resize',
            () => {
               if (isInlineActive) {
                  if (window.innerWidth <= 768) {
                     deactivateInlineAddressBar();
                  } else {
                     positionInlineDropdown(
                        shadowRoot.getElementById('__fynexo_inline_dropdown__'),
                     );
                  }
               }
            },
            { passive: true },
         );

         syncAllStyles();

         if (document.readyState === 'complete') {
            finishProgress();
         } else {
            window.addEventListener(
               'load',
               () => {
                  finishProgress();
                  if (config.force_dark) applyPageRules();
               },
               { once: true },
            );
         }

         // Top-edge hover peek in Immersive Fullscreen
         window.addEventListener(
            'mousemove',
            (e) => {
               if (!isFullscreen) return;
               if (e.clientY <= 6 && !isTopbarPeek) {
                  isTopbarPeek = true;
                  applyShadowStyles();
               }
            },
            { passive: true },
         );

         // Mobile Webpage Input Auto-Hide
         document.addEventListener(
            'focusin',
            (e) => {
               if (
                  window.innerWidth <= 768 &&
                  hudRoot.style.display !== 'flex'
               ) {
                  const targetEl = isEditableWebElement(e.target)
                     ? getDeepActiveElement() || e.target
                     : null;
                  if (targetEl) {
                     setKeyboardHiddenState(true);
                     try {
                        setTimeout(() => {
                           if (
                              targetEl &&
                              typeof targetEl.scrollIntoView === 'function'
                           ) {
                              targetEl.scrollIntoView({
                                 block: 'nearest',
                                 behavior: 'smooth',
                              });
                           }
                        }, 300);
                     } catch (_) {}
                  }
               }
            },
            true,
         );

         document.addEventListener(
            'focusout',
            () => {
               if (window.innerWidth <= 768) {
                  setTimeout(() => {
                     if (!isEditableWebElement(getDeepActiveElement())) {
                        setKeyboardHiddenState(false);
                     }
                  }, 120);
               }
            },
            true,
         );

         if (window.visualViewport) {
            window.visualViewport.addEventListener(
               'resize',
               () => {
                  if (
                     window.innerWidth <= 768 &&
                     hudRoot.style.display !== 'flex'
                  ) {
                     const ratio =
                        window.visualViewport.height /
                        Math.max(window.screen.height, window.innerHeight, 1);
                     const activeEl = getDeepActiveElement();
                     if (ratio < 0.68 || isEditableWebElement(activeEl)) {
                        setKeyboardHiddenState(true);
                        if (activeEl && isEditableWebElement(activeEl)) {
                           try {
                              if (
                                 typeof activeEl.scrollIntoView === 'function'
                              ) {
                                 activeEl.scrollIntoView({
                                    block: 'nearest',
                                    behavior: 'smooth',
                                 });
                              }
                           } catch (_) {}
                        }
                     } else {
                        setKeyboardHiddenState(false);
                     }
                  }
               },
               { passive: true },
            );
         }

         window.addEventListener(
            'keydown',
            (e) => {
               if (window.__fynexo_recording_keybind__) return;
               if (e.isComposing || e.keyCode === 229) return;

               // Protect plain single-key shortcuts when typing inside webpage inputs
               if (
                  !e.ctrlKey &&
                  !e.metaKey &&
                  !e.altKey &&
                  !/^f\d+$/i.test(e.key || '') &&
                  e.key !== 'Escape'
               ) {
                  if (
                     isEditableWebElement(e.target) ||
                     isEditableWebElement(getDeepActiveElement())
                  )
                     return;
               }

               if (
                  matchesKeybind(e, keybinds.omnibox) ||
                  (e.altKey &&
                     !e.ctrlKey &&
                     !e.shiftKey &&
                     (e.key || '').toLowerCase() === 'd')
               ) {
                  e.preventDefault();
                  e.stopPropagation();
                  openHud();
                  return;
               }
               if (matchesKeybind(e, keybinds.reopen_closed_tab)) {
                  e.preventDefault();
                  e.stopPropagation();
                  triggerProgress();
                  sendIpc({ type: 'REOPEN_CLOSED_TAB' });
                  return;
               }
               if (matchesKeybind(e, keybinds.new_tab)) {
                  e.preventDefault();
                  e.stopPropagation();
                  flushScrollSync();
                  triggerProgress();
                  sendIpc({ type: 'NEW_TAB' });
                  return;
               }
               if (matchesKeybind(e, keybinds.close_tab)) {
                  e.preventDefault();
                  e.stopPropagation();
                  if (tabState.active_id) {
                     sendIpc({ type: 'CLOSE_TAB', id: tabState.active_id });
                  }
                  return;
               }
               if (
                  matchesKeybind(e, keybinds.next_tab) ||
                  (String(keybinds.next_tab || '').toLowerCase() ===
                     'ctrl+tab' &&
                     e.ctrlKey &&
                     e.shiftKey &&
                     e.key === 'Tab')
               ) {
                  e.preventDefault();
                  e.stopPropagation();
                  if (tabState.tabs.length > 1) {
                     const curIdx = tabState.tabs.findIndex(
                        (t) => t.id === tabState.active_id,
                     );
                     const nextIdx =
                        (curIdx +
                           (e.shiftKey ? -1 : 1) +
                           tabState.tabs.length) %
                        tabState.tabs.length;
                     flushScrollSync();
                     triggerProgress();
                     sendIpc({
                        type: 'SWITCH_TAB',
                        id: tabState.tabs[nextIdx].id,
                     });
                  }
                  return;
               }
               if (matchesKeybind(e, keybinds.bookmark_page)) {
                  e.preventDefault();
                  e.stopPropagation();
                  toggleCurrentBookmark();
                  return;
               }
               if (matchesKeybind(e, keybinds.find_in_page)) {
                  if (!isInternalSettings()) {
                     e.preventDefault();
                     e.stopPropagation();
                     openFindBar();
                     return;
                  }
               }
               if (matchesKeybind(e, keybinds.reader_mode)) {
                  if (!isInternalAny()) {
                     e.preventDefault();
                     e.stopPropagation();
                     toggleReaderMode();
                     return;
                  }
               }
               if (matchesKeybind(e, keybinds.force_dark)) {
                  e.preventDefault();
                  e.stopPropagation();
                  config.force_dark = !config.force_dark;
                  sendIpc({
                     type: 'PATCH_HUD_PREFS',
                     force_dark: config.force_dark,
                  });
                  applyPageRules();
                  syncBookmarkButtons();
                  if (config.force_dark && isPageAlreadyDark()) {
                     showHudToast('Smart Dark: Page is already dark (skipped)');
                  } else {
                     showHudToast(
                        config.force_dark
                           ? 'Smart Force Dark enabled'
                           : 'Smart Force Dark off',
                     );
                  }
                  return;
               }
               if (matchesKeybind(e, keybinds.fullscreen)) {
                  e.preventDefault();
                  e.stopPropagation();
                  sendIpc({ type: 'TOGGLE_FULLSCREEN' });
                  return;
               }
               if (matchesKeybind(e, keybinds.preferences)) {
                  e.preventDefault();
                  e.stopPropagation();
                  closeAnyOmnibox();
                  flushScrollSync();
                  togglePreferences();
                  return;
               }
               if (matchesKeybind(e, keybinds.zoom_in)) {
                  e.preventDefault();
                  e.stopPropagation();
                  sendIpc({ type: 'SET_ZOOM', delta: 0.1 });
                  return;
               }
               if (matchesKeybind(e, keybinds.zoom_out)) {
                  e.preventDefault();
                  e.stopPropagation();
                  sendIpc({ type: 'SET_ZOOM', delta: -0.1 });
                  return;
               }
               if (matchesKeybind(e, keybinds.zoom_reset)) {
                  e.preventDefault();
                  e.stopPropagation();
                  sendIpc({ type: 'SET_ZOOM', zoom: 1.0 });
                  return;
               }
               if (
                  (e.ctrlKey || e.metaKey) &&
                  !e.altKey &&
                  !e.shiftKey &&
                  /^[1-9]$/.test(e.key || '')
               ) {
                  const len = tabState.tabs.length;
                  if (len > 0) {
                     const targetIdx =
                        e.key === '9'
                           ? len - 1
                           : Math.min(len - 1, Number(e.key) - 1);
                     const targetTab = tabState.tabs[targetIdx];
                     if (targetTab) {
                        e.preventDefault();
                        e.stopPropagation();
                        if (targetTab.id !== tabState.active_id) {
                           flushScrollSync();
                           triggerProgress();
                           sendIpc({ type: 'SWITCH_TAB', id: targetTab.id });
                        }
                     }
                  }
                  return;
               }
               if (
                  (e.ctrlKey || e.metaKey) &&
                  e.shiftKey &&
                  !e.altKey &&
                  (e.key || '').toLowerCase() === 'c'
               ) {
                  const sel = window.getSelection
                     ? String(window.getSelection()).trim()
                     : '';
                  if (!sel) {
                     e.preventDefault();
                     e.stopPropagation();
                     copyCurrentUrl();
                     return;
                  }
               }
               if (
                  e.altKey &&
                  !e.ctrlKey &&
                  !e.metaKey &&
                  !e.shiftKey &&
                  (e.key === 'ArrowLeft' || e.key === 'ArrowRight')
               ) {
                  e.preventDefault();
                  e.stopPropagation();
                  requestHistoryNav(
                     e.key === 'ArrowRight' ? 'forward' : 'back',
                  );
                  return;
               }
               if (
                  e.altKey &&
                  !e.ctrlKey &&
                  (e.key || '').toLowerCase() === 's'
               ) {
                  e.preventDefault();
                  e.stopPropagation();
                  if (typeof toggleSidePanel === 'function') toggleSidePanel();
                  return;
               }
               if (e.key === 'Escape') {
                  if (isPromptOpenRef && isPromptOpenRef()) {
                     if (closePromptModalRef) closePromptModalRef();
                     return;
                  }
                  if (isHistoryOpenRef && isHistoryOpenRef()) {
                     if (closeHistoryDrawerRef) closeHistoryDrawerRef();
                     return;
                  }
                  if (
                     typeof isSidePanelOpen !== 'undefined' &&
                     isSidePanelOpen
                  ) {
                     toggleSidePanel(false);
                     return;
                  }
                  if (
                     tabContextMenuEl &&
                     tabContextMenuEl.style.display !== 'none'
                  ) {
                     hideTabContextMenu();
                     return;
                  }
                  if (
                     typeof closeAllNavPopovers === 'function' &&
                     activeNavPopoverEl &&
                     activeNavPopoverEl.style.display !== 'none'
                  ) {
                     closeAllNavPopovers();
                     restorePageFocus();
                     return;
                  }
                  if (isInlineActive || isModalActive) {
                     closeAnyOmnibox();
                     return;
                  }
                  if (isReaderOpen) {
                     closeReaderMode();
                     return;
                  }
                  if (isFindOpen) {
                     closeFindBar();
                     return;
                  }
                  if (isDrawerOpen) {
                     closeMobileDrawer();
                     return;
                  }
                  if (isFullscreen) {
                     sendIpc({ type: 'TOGGLE_FULLSCREEN', force: false });
                     return;
                  }
               }
            },
            true,
         );

         let scrollSaveTimer = null;
         window.addEventListener(
            'scroll',
            () => {
               if (isInternalAny() || isReaderOpen) return;
               if (scrollSaveTimer) clearTimeout(scrollSaveTimer);
               scrollSaveTimer = setTimeout(flushScrollSync, 350);
            },
            { passive: true, capture: true },
         );

         window.addEventListener(
            'click',
            (e) => {
               if (isEventInsideHud(e)) return;
               hideTabContextMenu();
               if (typeof closeAllNavPopovers === 'function')
                  closeAllNavPopovers();
            },
            true,
         );
         window.addEventListener(
            'contextmenu',
            (e) => {
               if (isEventInsideHud(e)) return;
               hideTabContextMenu();
            },
            true,
         );
         window.addEventListener(
            'contextmenu',
            (e) => {
               if (e.defaultPrevented || e.shiftKey) return;
               if (isMobilePlatform() || isEventInsideHud(e)) return;
               e.preventDefault();
               showPageContextMenu(e);
            },
            false,
         );
         window.addEventListener('blur', hideTabContextMenu, { passive: true });
         window.addEventListener('resize', hideTabContextMenu, {
            passive: true,
         });
         if (shadowRoot) {
            shadowRoot.addEventListener('click', (e) => {
               if (tabContextMenuEl && !tabContextMenuEl.contains(e.target)) {
                  hideTabContextMenu();
               }
               const targetEl =
                  e.target && e.target.nodeType === 1
                     ? e.target
                     : e.target && e.target.parentElement;
               const inNavPop =
                  targetEl &&
                  targetEl.closest &&
                  targetEl.closest('.fnx-nav-popover');
               const inNavBtn =
                  targetEl &&
                  targetEl.closest &&
                  (targetEl.closest('.fnx-popover-trigger') ||
                     targetEl.closest('#__fynexo_btn_more__') ||
                     targetEl.closest('#__fynexo_mb_more__') ||
                     targetEl.closest('#__fynexo_btn_shield__') ||
                     targetEl.closest('#__fynexo_btn_media__') ||
                     targetEl.closest('#__fynexo_btn_downloads__') ||
                     targetEl.closest('#__fynexo_btn_sidepanel__') ||
                     targetEl.closest('#__fynexo_desk_reader__') ||
                     targetEl.closest('#__fynexo_btn_reader_pill__'));
               if (
                  !inNavPop &&
                  !inNavBtn &&
                  typeof closeAllNavPopovers === 'function'
               ) {
                  closeAllNavPopovers();
               }
            });
            shadowRoot.addEventListener('contextmenu', (e) => {
               if (e.shiftKey) return;
               const targetEl =
                  e.target && e.target.nodeType === 1
                     ? e.target
                     : e.target && e.target.parentElement;
               if (
                  !targetEl ||
                  !targetEl.closest ||
                  (!targetEl.closest('.fnx-tab-item') &&
                     !targetEl.closest('.fnx-drawer-card'))
               ) {
                  hideTabContextMenu();
                  if (!isMobilePlatform()) {
                     e.preventDefault();
                     if (isEditableWebElement(targetEl)) {
                        showPageContextMenu(e);
                     }
                  }
               }
            });
         }

         function isAllowedPopupUrl(rawUrl) {
            if (!rawUrl) return false;
            try {
               const parsed = new URL(String(rawUrl), window.location.href);
               if (isInternalAny()) return true;
               const p = parsed.protocol.toLowerCase();
               return (
                  (p === 'http:' || p === 'https:') &&
                  parsed.hostname.toLowerCase() !== 'fynexo.localhost'
               );
            } catch (_) {
               return false;
            }
         }

         window.addEventListener(
            'mouseup',
            (e) => {
               if (e.button === 1 && pendingMidTab && !pendingMidTab.done) {
                  const snap = pendingMidTab;
                  e.preventDefault();
                  setTimeout(() => {
                     if (pendingMidTab === snap && !snap.done) {
                        finishMiddleCloseTab(snap.id);
                     }
                  }, 0);
               } else if (e.button === 3) {
                  e.preventDefault();
                  requestHistoryNav('back');
               } else if (e.button === 4) {
                  e.preventDefault();
                  requestHistoryNav('forward');
               }
            },
            true,
         );

         function cleanLinkTitleCandidate(raw) {
            const s = String(raw || '')
               .replace(/\s+/g, ' ')
               .trim();
            if (!s) return '';
            if (/^\d{1,2}:\d{2}(?::\d{2})?$/.test(s)) return '';
            if (/^(now playing|watch later|add to queue|live|shorts)$/i.test(s))
               return '';
            return s.slice(0, 120);
         }

         function extractLinkTitle(a) {
            if (!a) return '';
            const direct = [
               a.getAttribute('title'),
               a.textContent,
               a.getAttribute('aria-label'),
            ];
            for (const cand of direct) {
               const cleaned = cleanLinkTitleCandidate(cand);
               if (cleaned) return cleaned;
            }
            const card = a.closest
               ? a.closest(
                    'ytd-rich-item-renderer, ytd-rich-grid-media, ytd-video-renderer, ytd-compact-video-renderer, ytd-grid-video-renderer, yt-lockup-view-model, article, li',
                 )
               : null;
            if (card) {
               const titleNode = card.querySelector(
                  '#video-title-link, #video-title, .yt-lockup-metadata-view-model__title, h1, h2, h3, [role="heading"]',
               );
               if (titleNode) {
                  const cleaned = cleanLinkTitleCandidate(
                     titleNode.getAttribute('title') ||
                        titleNode.textContent ||
                        titleNode.getAttribute('aria-label'),
                  );
                  if (cleaned) return cleaned;
               }
            }
            const img = a.querySelector ? a.querySelector('img[alt]') : null;
            if (img) {
               const cleaned = cleanLinkTitleCandidate(img.getAttribute('alt'));
               if (cleaned) return cleaned;
            }
            return '';
         }

         function handlePageLinkActivation(e, isAuxMiddle) {
            const a = e.target.closest ? e.target.closest('a') : null;
            if (!a || !a.href) return;
            const rawHref = a.getAttribute('href') || '';
            if (rawHref.startsWith('#') || rawHref.startsWith('javascript:'))
               return;
            const isBlank =
               !isAuxMiddle && (a.target === '_blank' || a.target === 'blank');
            const isBgTrigger =
               isAuxMiddle || ((e.ctrlKey || e.metaKey) && e.button === 0);
            if (isBlank || isBgTrigger) {
               e.preventDefault();
               e.stopPropagation();
               if (isAllowedPopupUrl(a.href)) {
                  const openInBg = isBgTrigger && !e.shiftKey;
                  const linkTitle = extractLinkTitle(a);
                  if (!openInBg) triggerProgress();
                  sendIpc({
                     type: 'NEW_TAB',
                     url: a.href,
                     title: linkTitle,
                     background: openInBg,
                  });
               }
            }
         }
         document.addEventListener(
            'mousedown',
            (e) => {
               if (e.button === 1) {
                  const a = e.target.closest
                     ? e.target.closest('a[href]')
                     : null;
                  if (a && !(a.getAttribute('href') || '').startsWith('#')) {
                     e.preventDefault();
                  }
               }
            },
            true,
         );
         document.addEventListener(
            'click',
            (e) => handlePageLinkActivation(e, false),
            true,
         );
         document.addEventListener(
            'auxclick',
            (e) => {
               if (e.button === 1) handlePageLinkActivation(e, true);
            },
            true,
         );

         try {
            window.open = function (url) {
               if (url && isAllowedPopupUrl(url)) {
                  triggerProgress();
                  sendIpc({
                     type: 'NEW_TAB',
                     url: new URL(String(url), window.location.href).href,
                  });
               }
               return null;
            };
         } catch (_) {}

         // Viewport & Dimensions Patch for Web Games, Canvas & IO Games
         try {
            const getActiveTopH = () => {
               if (isFullscreen) return 0;
               if (
                  !document.documentElement ||
                  !document.documentElement.classList.contains('fnx-game-mode')
               )
                  return 0;
               const isDeck =
                  (
                     (styles && styles.topbar && styles.topbar.layout) ||
                     'deck'
                  ).toLowerCase() !== 'compact';
               const bmShown =
                  (styles &&
                     styles.topbar &&
                     (
                        styles.topbar.bookmarks_bar || 'always'
                     ).toLowerCase()) === 'always';
               return isDeck
                  ? 40 + 38 + (bmShown ? 28 : 0)
                  : 38 + (bmShown ? 28 : 0);
            };

            const protoWin = Object.getPrototypeOf(window) || window;
            const origInnerH =
               Object.getOwnPropertyDescriptor(protoWin, 'innerHeight') ||
               Object.getOwnPropertyDescriptor(window, 'innerHeight');
            if (origInnerH && origInnerH.get) {
               Object.defineProperty(window, 'innerHeight', {
                  get: function () {
                     const raw = origInnerH.get.call(this);
                     return Math.max(0, raw - getActiveTopH());
                  },
                  configurable: true,
                  enumerable: true,
               });
            }

            const origClientH = Object.getOwnPropertyDescriptor(
               Element.prototype,
               'clientHeight',
            );
            if (origClientH && origClientH.get) {
               Object.defineProperty(document.documentElement, 'clientHeight', {
                  get: function () {
                     const raw = origClientH.get.call(this);
                     return Math.max(0, raw - getActiveTopH());
                  },
                  configurable: true,
                  enumerable: true,
               });
            }

            if (window.visualViewport) {
               const protoVv = Object.getPrototypeOf(window.visualViewport);
               const origVvH = Object.getOwnPropertyDescriptor(
                  protoVv,
                  'height',
               );
               if (origVvH && origVvH.get) {
                  Object.defineProperty(window.visualViewport, 'height', {
                     get: function () {
                        const raw = origVvH.get.call(this);
                        return Math.max(0, raw - getActiveTopH());
                     },
                     configurable: true,
                     enumerable: true,
                  });
               }
            }
         } catch (_) {}

         // HTML5 Fullscreen API Synchronization for Web Games & Video Players
         document.addEventListener('fullscreenchange', () => {
            const isDomFull = !!document.fullscreenElement;
            if (isDomFull) {
               if (!isFullscreen)
                  sendIpc({ type: 'TOGGLE_FULLSCREEN', force: true });
            } else {
               if (isFullscreen)
                  sendIpc({ type: 'TOGGLE_FULLSCREEN', force: false });
            }
         });

         // Game & Viewport Isolation Engine (Zero Scroll, No Scrollbars for Games & Canvas)
         function checkIsGameMode() {
            try {
               if (isInternalAny()) return false;
               const host = (window.location.hostname || '').toLowerCase();
               // Never flag video/media or major web platforms as games
               if (
                  host.includes('youtube') ||
                  host.includes('bilibili') ||
                  host.includes('netflix') ||
                  host.includes('twitch') ||
                  host.includes('vimeo') ||
                  host.includes('google') ||
                  host.includes('github') ||
                  host.includes('twitter') ||
                  host.includes('x.com') ||
                  host.includes('reddit') ||
                  host.includes('wikipedia')
               ) {
                  return false;
               }

               // 1. Well-known game platforms / IO game domains
               if (
                  host.endsWith('.io') ||
                  host.includes('game') ||
                  host.includes('poki.') ||
                  host.includes('crazygames.') ||
                  host.includes('itch.') ||
                  host.includes('1v1.lol') ||
                  host.includes('y8.') ||
                  host.includes('krunker')
               ) {
                  return true;
               }

               // 2. Pointer lock active (FPS / 3D Canvas Games)
               if (document.pointerLockElement) return true;

               // 3. Fullscreen element is a canvas (e.g. web game fullscreen)
               if (
                  document.fullscreenElement &&
                  document.fullscreenElement.tagName === 'CANVAS'
               ) {
                  return true;
               }
            } catch (_) {}
            return false;
         }

         function syncGameMode() {
            const isGame = checkIsGameMode();
            const docEl = document.documentElement;
            if (!docEl) return;
            if (isGame) {
               if (!docEl.classList.contains('fnx-game-mode')) {
                  docEl.classList.add('fnx-game-mode');
                  if (document.body) {
                     document.body.classList.add('fnx-game-mode');
                     document.body.scrollTop = 0;
                     document.body.scrollLeft = 0;
                  }
                  docEl.scrollTop = 0;
                  docEl.scrollLeft = 0;
               }
            } else {
               if (docEl.classList.contains('fnx-game-mode')) {
                  docEl.classList.remove('fnx-game-mode');
                  if (document.body)
                     document.body.classList.remove('fnx-game-mode');
               }
            }
         }

         syncGameMode();
         document.addEventListener('DOMContentLoaded', syncGameMode, {
            once: true,
         });
         window.addEventListener('load', syncGameMode, { once: true });
         window.addEventListener('resize', syncGameMode, { passive: true });
         document.addEventListener('fullscreenchange', syncGameMode, {
            passive: true,
         });
         document.addEventListener('pointerlockchange', syncGameMode, {
            passive: true,
         });
         document.addEventListener('mozpointerlockchange', syncGameMode, {
            passive: true,
         });

         function patchYoutubeDrawers() {
            try {
               const host = (window.location.hostname || '').toLowerCase();
               if (!host.includes('youtube.com')) return true;
               const drawers = document.querySelectorAll('tp-yt-app-drawer');
               let patchedGuide = false;
               for (let i = 0; i < drawers.length; i++) {
                  const d = drawers[i];
                  const sr = d && d.shadowRoot;
                  if (!sr) continue;
                  if (d.id === 'guide') patchedGuide = true;
                  if (!sr.getElementById('__fnx_yt_drawer_fix__')) {
                     const st = document.createElement('style');
                     st.id = '__fnx_yt_drawer_fix__';
                     st.textContent = `
                                #contentContainer {
                                    padding-top: 0 !important;
                                    padding-bottom: 0 !important;
                                    top: 0 !important;
                                    bottom: 0 !important;
                                    height: 100% !important;
                                    max-height: 100% !important;
                                }
                            `;
                     sr.appendChild(st);
                  }
               }
               return patchedGuide;
            } catch (_) {
               return true;
            }
         }

         // YouTube Auto Ad Skipper & Video Ad Cleaner (Zero-Blackscreen / Fluid Transition)
         function initYouTubeAdSkipper() {
            try {
               const host = (window.location.hostname || '').toLowerCase();
               if (!host.includes('youtube.com')) return;

               if (!patchYoutubeDrawers()) {
                  let dTries = 0;
                  const drawerTimer = setInterval(() => {
                     if (patchYoutubeDrawers() || ++dTries > 30)
                        clearInterval(drawerTimer);
                  }, 250);
               }
               document.addEventListener(
                  'DOMContentLoaded',
                  patchYoutubeDrawers,
                  { once: true },
               );
               window.addEventListener('load', patchYoutubeDrawers, {
                  once: true,
               });
               window.addEventListener(
                  'yt-navigate-finish',
                  () => setTimeout(patchYoutubeDrawers, 120),
                  { passive: true },
               );

               let savedRate = 1.0;
               let playerEl = null;
               let adPollTimer = null;

               const skipAds = () => {
                  if (config.adblock === false) {
                     if (adPollTimer) {
                        clearInterval(adPollTimer);
                        adPollTimer = null;
                     }
                     return;
                  }
                  if (!playerEl || !playerEl.isConnected) {
                     playerEl = document.querySelector('.html5-video-player');
                  }
                  if (!playerEl) return;

                  const isAd =
                     playerEl.classList.contains('ad-showing') ||
                     playerEl.classList.contains('ad-interrupting');
                  const v = playerEl.querySelector('video');

                  if (v) {
                     if (isAd) {
                        // Mute ad and fast-forward at 16x without seeking currentTime to avoid MSE pipeline crash/black screen
                        v.muted = true;
                        v.playbackRate = 16.0;
                     } else {
                        // Restore playback speed and respect the tab's mute state as soon as ad finishes
                        if (v.playbackRate > 2.0) {
                           v.playbackRate = savedRate || 1.0;
                           v.muted = isTabMutedState;
                        } else {
                           savedRate = v.playbackRate;
                        }
                     }
                  }

                  if (isAd) {
                     if (!adPollTimer) {
                        adPollTimer = setInterval(skipAds, 250);
                     }
                     const skipBtn = playerEl.querySelector(
                        'button.ytp-ad-skip-button-modern, button.ytp-ad-skip-button, .ytp-skip-ad-button, .ytp-ad-skip-button-container button',
                     );
                     if (skipBtn && typeof skipBtn.click === 'function') {
                        try {
                           skipBtn.click();
                        } catch (_) {}
                     }
                     const overlayBtn = playerEl.querySelector(
                        '.ytp-ad-overlay-close-button',
                     );
                     if (overlayBtn && typeof overlayBtn.click === 'function') {
                        try {
                           overlayBtn.click();
                        } catch (_) {}
                     }
                  } else if (adPollTimer) {
                     clearInterval(adPollTimer);
                     adPollTimer = null;
                  }
               };

               // Observe player class changes so idle playback incurs zero 250ms DOM polling.
               let observedPlayer = null;
               let playerObserver = null;
               const attachPlayerObserver = () => {
                  const p = document.querySelector('.html5-video-player');
                  if (!p || p === observedPlayer) return false;
                  if (playerObserver) {
                     try {
                        playerObserver.disconnect();
                     } catch (_) {}
                  }
                  observedPlayer = p;
                  playerEl = p;
                  playerObserver = new MutationObserver(skipAds);
                  playerObserver.observe(p, {
                     attributes: true,
                     attributeFilter: ['class'],
                  });
                  skipAds();
                  return true;
               };
               if (!attachPlayerObserver()) {
                  let tries = 0;
                  const bootTimer = setInterval(() => {
                     if (attachPlayerObserver() || ++tries > 40)
                        clearInterval(bootTimer);
                  }, 500);
               }
               window.addEventListener(
                  'yt-navigate-finish',
                  () => setTimeout(attachPlayerObserver, 200),
                  { passive: true },
               );
               window.addEventListener(
                  'popstate',
                  () => setTimeout(attachPlayerObserver, 200),
                  { passive: true },
               );
            } catch (_) {}
         }
         initYouTubeAdSkipper();

         function initGithubDrawerHarmonizer() {
            try {
               const host = (window.location.hostname || '').toLowerCase();
               if (host !== 'github.com' && !host.endsWith('.github.com')) return;

               let ghRafPending = false;
               const resetTopOffsets = () => {
                  const patched = document.querySelectorAll('[data-fnx-gh-offset]');
                  for (let i = 0; i < patched.length; i++) {
                     const n = patched[i];
                     n.removeAttribute('data-fnx-gh-offset');
                     n.removeAttribute('data-fnx-gh-top');
                     n.style.removeProperty('top');
                     n.style.removeProperty('inset-block-start');
                     n.style.removeProperty('bottom');
                     n.style.removeProperty('inset-block-end');
                     n.style.removeProperty('height');
                     n.style.removeProperty('max-height');
                     n.style.removeProperty('margin-top');
                     n.style.removeProperty('margin-bottom');
                     n.style.removeProperty('margin-block-start');
                     n.style.removeProperty('margin-block-end');
                     n.style.removeProperty('transform');
                     n.style.removeProperty('align-self');
                  }
               };

               const applyTopOffset = (node, topPx) => {
                  const topVal = `${topPx}px`;
                  const heightVal = `calc(100vh - ${topPx}px)`;
                  node.setAttribute('data-fnx-gh-offset', '1');
                  node.setAttribute('data-fnx-gh-top', String(topPx));
                  node.style.setProperty('top', topVal, 'important');
                  node.style.setProperty('inset-block-start', topVal, 'important');
                  node.style.setProperty('bottom', '0px', 'important');
                  node.style.setProperty('inset-block-end', '0px', 'important');
                  node.style.setProperty('height', heightVal, 'important');
                  node.style.setProperty('max-height', heightVal, 'important');
                  node.style.setProperty('margin-top', '0px', 'important');
                  node.style.setProperty('margin-bottom', '0px', 'important');
                  node.style.setProperty('margin-block-start', '0px', 'important');
                  node.style.setProperty('margin-block-end', '0px', 'important');
               };

               const patchDrawers = () => {
                  ghRafPending = false;
                  const deskTopVisible =
                     styles.topbar.visible_on_desktop !== false &&
                     !isFullscreen &&
                     !isMobilePlatform();
                  const tb =
                     shadowRoot && shadowRoot.getElementById('__fynexo_topbar__');
                  const tbRect = tb ? tb.getBoundingClientRect() : null;
                  let topPx = 0;
                  if (deskTopVisible && tbRect && tbRect.height > 1 && tbRect.bottom > 1) {
                     topPx = Math.ceil(tbRect.bottom);
                  } else if (deskTopVisible && document.body) {
                     topPx = Math.round(document.body.getBoundingClientRect().top);
                  }
                  if (!Number.isFinite(topPx) || topPx <= 1) {
                     resetTopOffsets();
                     return;
                  }

                  if (
                     document.body &&
                     !document.documentElement.hasAttribute('data-fynexo-internal') &&
                     document.body.getBoundingClientRect().top < topPx - 2
                  ) {
                     document.body.style.setProperty('position', 'fixed', 'important');
                     document.body.style.setProperty('top', `${topPx}px`, 'important');
                     document.body.style.setProperty('left', '0px', 'important');
                     document.body.style.setProperty('right', '0px', 'important');
                     document.body.style.setProperty('bottom', '0px', 'important');
                  }

                  const roots = [document];
                  const customModalHosts = document.querySelectorAll(
                     'modal-dialog, primer-dialog, anchored-position',
                  );
                  for (let m = 0; m < customModalHosts.length; m++) {
                     if (customModalHosts[m] && customModalHosts[m].shadowRoot) {
                        roots.push(customModalHosts[m].shadowRoot);
                     }
                  }

                  const seeds = [];
                  for (let r = 0; r < roots.length; r++) {
                     const list = roots[r].querySelectorAll(
                        'dialog, modal-dialog, [role="dialog"], [aria-modal="true"], [popover], .Overlay, .Overlay-backdrop, .Overlay-backdrop--side, [data-component="SidePanel"], [data-testid="global-navigation-modal"], aside.SidePanel, [class*="prc-Dialog-Backdrop"], [class*="prc-Dialog-Dialog"]',
                     );
                     for (let i = 0; i < list.length; i++) seeds.push(list[i]);
                  }
                  if (typeof document.elementsFromPoint === 'function') {
                     const probeYs = [
                        Math.min(window.innerHeight - 10, topPx + 4),
                        Math.min(window.innerHeight - 10, topPx + 24),
                     ];
                     const probeXs = [28, 120, Math.max(40, window.innerWidth - 28)];
                     for (let py = 0; py < probeYs.length; py++) {
                        for (let px = 0; px < probeXs.length; px++) {
                           const hit = document.elementsFromPoint(
                              probeXs[px],
                              probeYs[py],
                           );
                           for (let h = 0; h < hit.length; h++) {
                              if (hit[h] && hit[h].id !== '__fynexo_host__') {
                                 seeds.push(hit[h]);
                              }
                           }
                        }
                     }
                  }

                  const minH = window.innerHeight * 0.45;
                  const seen = typeof Set === 'function' ? new Set() : null;
                  for (let i = 0; i < seeds.length; i++) {
                     let node = seeds[i];
                     for (
                        let depth = 0;
                        depth < 30 &&
                        node &&
                        node !== document.body &&
                        node !== document.documentElement;
                        depth++
                     ) {
                        if (node.id === '__fynexo_host__') break;
                        if (!seen || !seen.has(node)) {
                           if (seen) seen.add(node);
                           const cs = getComputedStyle(node);
                           if (
                              cs.position === 'fixed' ||
                              cs.position === 'sticky' ||
                              cs.position === 'absolute'
                           ) {
                              const rect = node.getBoundingClientRect();
                              const prevTop = parseInt(
                                 node.getAttribute('data-fnx-gh-top') || '',
                                 10,
                              );
                              if (
                                 rect.height >= minH &&
                                 (rect.top < topPx - 2 ||
                                    (Number.isFinite(prevTop) && prevTop !== topPx))
                              ) {
                                 applyTopOffset(node, topPx);
                                 const after = node.getBoundingClientRect();
                                 if (
                                    after.top < topPx - 2 &&
                                    cs.transform &&
                                    cs.transform !== 'none' &&
                                    typeof DOMMatrix === 'function'
                                 ) {
                                    try {
                                       const mat = new DOMMatrix(cs.transform);
                                       if (mat.f < -1) {
                                          node.style.setProperty(
                                             'transform',
                                             `translateX(${mat.e}px)`,
                                             'important',
                                          );
                                       }
                                    } catch (_) {}
                                 }
                                 const afterTx = node.getBoundingClientRect();
                                 if (afterTx.top < topPx - 2) {
                                    const delta = Math.ceil(topPx - afterTx.top);
                                    if (delta > 0 && delta <= topPx + 24) {
                                       node.style.setProperty(
                                          'margin-top',
                                          `${delta}px`,
                                          'important',
                                       );
                                    }
                                 }
                              }
                           }
                        }
                        node =
                           node.parentElement ||
                           (node.getRootNode && node.getRootNode().host) ||
                           null;
                     }
                  }

                  // Final safety pass if an open dialog/drawer is still partially above topPx (e.g. centered flex child)
                  for (let r = 0; r < roots.length; r++) {
                     const dialogs = roots[r].querySelectorAll(
                        'dialog[open], [role="dialog"], [aria-modal="true"], [data-testid="global-navigation-modal"], .Overlay--drawer',
                     );
                     for (let i = 0; i < dialogs.length; i++) {
                        const d = dialogs[i];
                        const rect = d.getBoundingClientRect();
                        if (rect.height >= minH && rect.top < topPx - 2) {
                           const cs = getComputedStyle(d);
                           if (cs.position === 'static' || cs.position === 'relative') {
                              const parent = d.parentElement;
                              const pcs = parent ? getComputedStyle(parent) : null;
                              if (pcs && (pcs.display === 'flex' || pcs.display === 'inline-flex' || pcs.display === 'grid')) {
                                 d.style.setProperty('align-self', 'stretch', 'important');
                              }
                              const afterSelf = d.getBoundingClientRect();
                              if (afterSelf.top < topPx - 2) {
                                 const delta = Math.ceil(topPx - afterSelf.top);
                                 if (delta > 0 && delta <= topPx + 24 && !d.hasAttribute('data-fnx-gh-offset')) {
                                    d.setAttribute('data-fnx-gh-offset', '1');
                                    d.style.setProperty('margin-top', `${delta}px`, 'important');
                                 }
                              }
                           } else {
                              applyTopOffset(d, topPx);
                           }
                        }
                     }
                  }
               };

               let ghSettleTimer = null;
               const schedulePatch = () => {
                  if (!ghRafPending) {
                     ghRafPending = true;
                     requestAnimationFrame(patchDrawers);
                  }
                  if (ghSettleTimer) clearTimeout(ghSettleTimer);
                  ghSettleTimer = setTimeout(() => {
                     ghSettleTimer = null;
                     patchDrawers();
                  }, 80);
               };

               document.addEventListener('click', schedulePatch, { passive: true, capture: true });
               document.addEventListener('keydown', schedulePatch, { passive: true, capture: true });
               window.addEventListener('turbo:load', schedulePatch, { passive: true });
               window.addEventListener('resize', patchDrawers, { passive: true });

               const attachObserver = () => {
                  const target = document.documentElement || document.body;
                  if (target) {
                     new MutationObserver((mutations) => {
                        for (let i = 0; i < mutations.length; i++) {
                           const t = mutations[i].target;
                           if (
                              t &&
                              (t.id === '__fynexo_host__' ||
                                 (t.closest && t.closest('#__fynexo_host__')) ||
                                 (t.nodeType === 1 && t.hasAttribute('data-fnx-gh-offset')))
                           ) {
                              continue;
                           }
                           schedulePatch();
                           break;
                        }
                     }).observe(target, {
                        childList: true,
                        subtree: true,
                        attributes: true,
                        attributeFilter: ['open', 'hidden', 'aria-hidden', 'style', 'class'],
                     });
                  }
               };
               if (document.readyState === 'loading') {
                  document.addEventListener('DOMContentLoaded', () => {
                     attachObserver();
                     patchDrawers();
                  }, { once: true });
               } else {
                  attachObserver();
                  patchDrawers();
               }
            } catch (_) {}
         }
         initGithubDrawerHarmonizer();

         window.addEventListener(
            'wheel',
            () => {
               if (
                  document.documentElement &&
                  document.documentElement.classList.contains('fnx-game-mode')
               ) {
                  if (
                     document.body &&
                     (document.body.scrollTop !== 0 ||
                        document.body.scrollLeft !== 0)
                  ) {
                     document.body.scrollTop = 0;
                     document.body.scrollLeft = 0;
                  }
                  if (
                     document.documentElement.scrollTop !== 0 ||
                     document.documentElement.scrollLeft !== 0
                  ) {
                     document.documentElement.scrollTop = 0;
                     document.documentElement.scrollLeft = 0;
                  }
               }
            },
            { passive: true },
         );

         let mutLastTitle = '';
         let mutLastUrl = '';
         let titleObserved = false;

         function syncTitle() {
            if (isBackgroundParked) return;
            const isSet = isInternalSettings();
            const isNt = isInternalNewtab();
            const t = isSet
               ? 'Preferences'
               : isNt
                 ? 'Start page'
                 : document.title || window.location.hostname || 'New tab';
            const u = isSet
               ? 'fynexo://settings'
               : isNt
                 ? 'fynexo://newtab'
                 : window.location.href;
            if (!u || u === 'about:blank') return;
            if (t === mutLastTitle && u === mutLastUrl) return;
            mutLastTitle = t;
            mutLastUrl = u;
            sendIpc({
               type: 'UPDATE_TAB_TITLE',
               title: t,
               url: u,
            });
            updateAddressLabels();
         }

         onForegroundResumeCb = function () {
            mutLastTitle = '';
            mutLastUrl = '';
            syncTitle();
         };

         function bindTitleObserver() {
            syncTitle();
            if (titleObserved) return;
            const titleEl = document.querySelector('title');
            if (titleEl) {
               titleObserved = true;
               new MutationObserver(syncTitle).observe(titleEl, {
                  subtree: true,
                  characterData: true,
                  childList: true,
               });
            }
         }

         bindTitleObserver();
         scheduleReaderableCheck();
         document.addEventListener(
            'DOMContentLoaded',
            () => {
               bindTitleObserver();
               scheduleReaderableCheck();
            },
            { once: true },
         );
         window.addEventListener(
            'load',
            () => {
               bindTitleObserver();
               scheduleReaderableCheck();
            },
            { once: true },
         );
         window.addEventListener(
            'popstate',
            () => {
               finishProgress(false);
               requestAnimationFrame(syncTitle);
               sendIpc({ type: 'SYNC_NAV_STATE' });
               scheduleReaderableCheck();
            },
            { passive: true },
         );
         window.addEventListener(
            'hashchange',
            () => {
               finishProgress(false);
               requestAnimationFrame(syncTitle);
               sendIpc({ type: 'SYNC_NAV_STATE' });
               scheduleReaderableCheck();
            },
            { passive: true },
         );
         window.addEventListener(
            'pageshow',
            (e) => {
               if (e && e.persisted) {
                  mutLastTitle = '';
                  mutLastUrl = '';
               }
               finishProgress(false);
               requestAnimationFrame(syncTitle);
               sendIpc({ type: 'SYNC_NAV_STATE' });
               scheduleReaderableCheck();
            },
            { passive: true },
         );
         window.addEventListener('turbo:load', scheduleReaderableCheck, {
            passive: true,
         });
         window.addEventListener(
            'yt-navigate-finish',
            () => {
               finishProgress(false);
               sendIpc({ type: 'SYNC_NAV_STATE' });
               scheduleReaderableCheck();
            },
            { passive: true },
         );

         // Frame-0 Immediate Style & Layout Synchronization (0ms paint, zero overlap)
         syncAllStyles();
         syncNavButtons();

         // Single Combined IPC Sync on Boot (with resilient retry if window.ipc is delayed)
         function sendBootSyncWhenReady(retries) {
            if (window.ipc && typeof window.ipc.postMessage === 'function') {
               sendIpc({ type: 'BOOT_SYNC' });
            } else if (retries < 30) {
               setTimeout(() => sendBootSyncWhenReady(retries + 1), 40);
            }
         }
         sendBootSyncWhenReady(0);

         // O(1) Pointer-Flag Anti-Detachment Observer (microtask coalesced)
         try {
            let antiStripQueued = false;
            const runAntiStrip = () => {
               antiStripQueued = false;
               if (!host.isConnected) {
                  const target = document.documentElement || document.body;
                  if (target) target.appendChild(host);
               }
               if (!pageRulesEl || !pageRulesEl.isConnected) {
                  applyPageRules();
               }
               if (siteRulesEl && !siteRulesEl.isConnected) {
                  applySiteRules();
               }
               syncGameMode();
            };
            const antiStripObserver = new MutationObserver(() => {
               if (!antiStripQueued) {
                  antiStripQueued = true;
                  Promise.resolve().then(runAntiStrip);
               }
            });
            antiStripObserver.observe(document.documentElement, {
               childList: true,
            });
         } catch (_) {}
      } catch (err) {
         console.error('[Fynexo HUD Exception]:', err);
         try {
            const h = document.getElementById('__fynexo_host__');
            if (
               h &&
               (!shadowRoot || !shadowRoot.getElementById('__fynexo_topbar__'))
            ) {
               h.remove();
            }
         } catch (_) {}
      }
   }

   function ensureMounted() {
      const target = document.documentElement || document.body;
      if (!target) {
         if (window.requestAnimationFrame)
            window.requestAnimationFrame(ensureMounted);
         else setTimeout(ensureMounted, 5);
         return;
      }
      const existingHost = document.getElementById('__fynexo_host__');
      if (
         !existingHost ||
         !shadowRoot ||
         !shadowRoot.getElementById('__fynexo_topbar__')
      ) {
         if (
            existingHost &&
            (!shadowRoot || !shadowRoot.getElementById('__fynexo_topbar__'))
         ) {
            try {
               existingHost.remove();
            } catch (_) {}
         }
         initInterface();
      }
   }

   let domReadyForPaint = document.readyState !== 'loading';
   let paintReadyScheduled = false;
   let paintReadySent = false;
   function notifyTabPaintReady() {
      if (paintReadySent) return;
      paintReadySent = true;
      sendIpc({ type: 'TAB_PAINT_READY' });
   }

   function schedulePaintReadyFrames() {
      if (paintReadyScheduled || paintReadySent) return;
      paintReadyScheduled = true;
      const safety = setTimeout(notifyTabPaintReady, 120);
      const fire = () => {
         clearTimeout(safety);
         notifyTabPaintReady();
      };
      if (typeof window.requestAnimationFrame === 'function') {
         window.requestAnimationFrame(() => {
            window.requestAnimationFrame(fire);
         });
      } else {
         fire();
      }
   }

   function maybeSchedulePaintReady(forceWithoutBoot) {
      if (paintReadyScheduled || paintReadySent) return;
      if (!domReadyForPaint) return;
      if (bootSyncDone || forceWithoutBoot) {
         schedulePaintReadyFrames();
      }
   }

   onBootSyncDoneCb = () => maybeSchedulePaintReady(false);

   ensureMounted();
   if (!domReadyForPaint) {
      document.addEventListener(
         'DOMContentLoaded',
         () => {
            domReadyForPaint = true;
            ensureMounted();
            maybeSchedulePaintReady(false);
            setTimeout(() => maybeSchedulePaintReady(true), 250);
         },
         { once: true },
      );
   } else {
      maybeSchedulePaintReady(false);
      setTimeout(() => maybeSchedulePaintReady(true), 250);
   }
})();
