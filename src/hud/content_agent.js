(function () {
   // Lightweight Content-Tab Agent (Windows Dual-WebView: zero chrome DOM/CSS bloat)
   if (window.top !== window.self) return;
   if (window.location && window.location.href === 'about:blank') return;

   const __FNX_IPC_TOKEN = '__FNX_IPC_TOKEN__';
   const __FNX_BRIDGE_TOKEN = '__FNX_BRIDGE_TOKEN__';
   const __FNX_ROLE = '__FNX_ROLE__';
   const IS_HUD_SHELL = __FNX_ROLE === 'hud';
   const IS_CONTENT_TAB = __FNX_ROLE === 'content';
   const __FNX_MOBILE_HOST = __FNX_MOBILE_HOST__;
   const __fnxIsInternalOriginInit = (() => {
      try {
         const h = String((window.location && window.location.href) || '').trim().toLowerCase();
         return (
            h.startsWith('fynexo://') ||
            /^https?:\/\/fynexo\.localhost(?::\d+)?(?:[/?#]|$)/.test(h)
         );
      } catch (_) {
         return false;
      }
   })();

   const __fnxJsonStringify = JSON.stringify;
   const __fnxJsonParse = JSON.parse;
   const __fnxDefineProperty = Object.defineProperty;
   const __fnxFreeze = Object.freeze;
   const __fnxReflectApply =
      typeof Reflect === 'object' && Reflect && typeof Reflect.apply === 'function'
         ? Reflect.apply
         : null;
   const __fnxFuncApply = Function.prototype.apply;
   const __fnxArraySlice = Array.prototype.slice;
   let __fnxIpcPost =
      window.ipc && typeof window.ipc.postMessage === 'function'
         ? window.ipc.postMessage.bind(window.ipc)
         : null;

   function shieldExternalWindowIpc() {
      if (__fnxIsInternalOriginInit || !__fnxIpcPost) return;
      try {
         const noopStub = __fnxFreeze({ postMessage: function () {} });
         __fnxDefineProperty(window, 'ipc', {
            value: noopStub,
            writable: false,
            configurable: false,
            enumerable: false,
         });
      } catch (_) {}
   }
   shieldExternalWindowIpc();

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
         const forwarded = __fnxArraySlice.call(arguments, 0, len - 1);
         if (__fnxReflectApply) {
            return __fnxReflectApply(fn, null, forwarded);
         }
         return __fnxFuncApply.call(fn, null, forwarded);
      }
      exposeReadOnly(name, guarded);
   }

   __FNX_I18N__;

   __FNX_DEFAULTS__;

   const bootPayload = __FNX_INITIAL_JSON__ || {};
   const DEFAULT_CONTENT_CONFIG = {
      language: 'zh',
      theme: 'obsidian',
      adblock: true,
      force_dark: false,
      enforce_perf: false,
      reduce_animations: false,
      incognito: false,
      allow_page_eval: false,
      ext_reader_mode: false,
      ext_yt_ad_skipper: false,
      ext_gh_harmonizer: false,
      ext_media_booster: false,
      ext_userscript_engine: true,
      ext_mouse_gestures: true,
      active_mode: 'auto',
      mode_profiles: [],
      site_overrides: {},
      homepage: 'fynexo://newtab',
   };
   let rawBaseConfig = Object.assign({}, DEFAULT_CONTENT_CONFIG, bootPayload.config || {});
   let config = Object.assign({}, rawBaseConfig);
   let styles = bootPayload.styles || {};
   let activeTheme = Object.assign({}, DEFAULT_STYLES.colors, bootPayload.active_theme || {});
   let keybinds = Object.assign({}, DEFAULT_KEYBINDS, bootPayload.keybinds || {});
   let enabledRules = Array.isArray(bootPayload.enabled_rules)
      ? bootPayload.enabled_rules
      : Array.isArray(bootPayload.site_rules)
        ? bootPayload.site_rules.filter((r) => r && r.enabled)
        : [];
   let adblockWhitelist = Array.isArray(bootPayload.adblock_whitelist)
      ? bootPayload.adblock_whitelist
      : [];
   let tabState = bootPayload.tabs || { active_id: 0, tabs: [] };
   let isFullscreen = false;
   let isTabMutedState = false;
   let isBackgroundParked = false;
   let bootSyncDone = false;
   let onBootSyncDoneCb = null;

   const MODE_BOOL_KEYS = [
      'adblock',
      'force_dark',
      'ext_side_panel',
      'ext_reader_mode',
      'ext_yt_ad_skipper',
      'ext_gh_harmonizer',
      'ext_media_booster',
      'ext_userscript_engine',
      'ext_mouse_gestures',
   ];

   function isLocalDevHost(host) {
      const h = String(host || '').toLowerCase();
      if (!h || h === 'fynexo.localhost') return false;
      if (
         h === 'localhost' ||
         h === '0.0.0.0' ||
         h === '::1' ||
         h === '[::1]' ||
         h.endsWith('.localhost') ||
         h.endsWith('.local') ||
         h.endsWith('.test')
      ) {
         return true;
      }
      return h.startsWith('127.') || h.startsWith('10.') || h.startsWith('192.168.');
   }

   function resolveSiteOverrideKey() {
      try {
         if (isInternalAny()) return '';
         const h = String((window.location && window.location.hostname) || '').toLowerCase();
         if (!h) return '';
         const port = String((window.location && window.location.port) || '');
         if (port && (isLocalDevHost(h) || (port !== '80' && port !== '443'))) {
            return `${h}:${port}`;
         }
         return h;
      } catch (_) {
         return '';
      }
   }

   function getCurrentSiteOverride() {
      const key = resolveSiteOverrideKey();
      const map = rawBaseConfig && rawBaseConfig.site_overrides;
      if (!key || !map || typeof map !== 'object') return null;
      const ov = map[key];
      return ov && typeof ov === 'object' ? ov : null;
   }

   function matchModeDomainPattern(hostname, hostPort, pattern) {
      const p = String(pattern || '*').trim().toLowerCase();
      if (!p || p === '*') return true;
      if (p.includes(':')) {
         const cur = String(hostPort || hostname || '').toLowerCase();
         const normP = p.replace(/^www\./, '');
         return cur === normP || cur.replace(/^www\./, '') === normP;
      }
      const clean = p.startsWith('*.') ? p.slice(2) : p.replace(/^www\./, '');
      const h = String(hostname || '').toLowerCase();
      return h === clean || h.endsWith('.' + clean);
   }

   function resolveActiveModeProfile() {
      if (isInternalAny()) return null;
      const modePref = String(rawBaseConfig.active_mode || 'auto').trim().toLowerCase();
      const profiles = Array.isArray(rawBaseConfig.mode_profiles) ? rawBaseConfig.mode_profiles : [];
      if (modePref === 'default' || modePref === 'off') return null;
      if (modePref === 'auto') {
         const curHost = String((window.location && window.location.hostname) || '').toLowerCase();
         const curPort = String((window.location && window.location.port) || '');
         const curHostPort = curPort ? `${curHost}:${curPort}` : curHost;
         if (!curHost) return null;
         for (let i = 0; i < profiles.length; i++) {
            const p = profiles[i];
            if (!p || !Array.isArray(p.auto_domains)) continue;
            for (let j = 0; j < p.auto_domains.length; j++) {
               const pat = String(p.auto_domains[j] || '').trim().toLowerCase();
               if (!pat) continue;
               if (
                  matchModeDomainPattern(curHost, curHostPort, pat) ||
                  (pat === 'localhost' && isLocalDevHost(curHost))
               ) {
                  return p;
               }
            }
         }
         return null;
      }
      for (let i = 0; i < profiles.length; i++) {
         const p = profiles[i];
         if (p && String(p.id || '').toLowerCase() === modePref) return p;
      }
      return null;
   }

   function recomputeEffectiveConfig() {
      const next = Object.assign({}, rawBaseConfig);
      const prof = resolveActiveModeProfile();
      if (prof) {
         for (let i = 0; i < MODE_BOOL_KEYS.length; i++) {
            const k = MODE_BOOL_KEYS[i];
            if (typeof prof[k] === 'boolean') next[k] = prof[k];
         }
      }
      config = next;
   }

   recomputeEffectiveConfig();

   function isZh() {
      return typeof FynexoI18n !== 'undefined'
         ? FynexoI18n.isZhLocale(config.language)
         : String(config.language || 'zh').toLowerCase().startsWith('zh');
   }

   function isMobilePlatform() {
      return !!__FNX_MOBILE_HOST;
   }

   function isInternalAny(u) {
      const s = String(u || (window.location && window.location.href) || '').trim().toLowerCase();
      return (
         s.startsWith('fynexo://') ||
         /^https?:\/\/fynexo\.localhost(?::\d+)?(?:[/?#]|$)/.test(s)
      );
   }

   function isInternalSettings(u) {
      const s = String(u || (window.location && window.location.href) || '').trim().toLowerCase();
      return s.startsWith('fynexo://settings') || s.includes('fynexo.localhost/settings');
   }

   function canonicalCurrentUrl() {
      try {
         const href = String((window.location && window.location.href) || '');
         if (/^https?:\/\/fynexo\.localhost(?::\d+)?\/settings/i.test(href)) {
            return 'fynexo://settings';
         }
         if (/^https?:\/\/fynexo\.localhost(?::\d+)?(?:\/newtab|\/?$)/i.test(href)) {
            return 'fynexo://newtab';
         }
         return href;
      } catch (_) {
         return '';
      }
   }

   function sendIpc(payload) {
      try {
         if (!__fnxIpcPost && window.ipc && typeof window.ipc.postMessage === 'function') {
            __fnxIpcPost = window.ipc.postMessage.bind(window.ipc);
         }
         if (!__fnxIpcPost) return;
         const outgoing = Object.assign({}, payload, { __t: __FNX_IPC_TOKEN });
         __fnxIpcPost(__fnxJsonStringify(outgoing));
      } catch (_) {}
   }

   function requestHistoryNav(dir) {
      sendIpc({ type: dir === 'forward' ? 'GO_FORWARD' : 'GO_BACK' });
   }

   function copyTextToClipboard(text, msg) {
      try {
         if (navigator.clipboard && typeof navigator.clipboard.writeText === 'function') {
            navigator.clipboard.writeText(String(text || '')).catch(() => {});
         }
         if (msg) showHudToast(msg);
      } catch (_) {}
   }

   // Minimal Shadow DOM host mounted ONLY when an in-page tool or toast is needed
   let hostEl = null;
   let shadowRoot = null;
   let shellEl = null;
   let toastEl = null;
   let toastTimer = null;

   function ensureShadowHost() {
      if (shadowRoot && hostEl && hostEl.isConnected) return shadowRoot;
      const root = document.documentElement || document.body;
      if (!root) return null;
      hostEl = document.getElementById('__fynexo_host__');
      if (!hostEl) {
         hostEl = document.createElement('div');
         hostEl.id = '__fynexo_host__';
         hostEl.setAttribute('data-fynexo-role', 'content');
         root.appendChild(hostEl);
      }
      shadowRoot = hostEl.shadowRoot || hostEl.attachShadow({ mode: 'open' });
      if (!shadowRoot.getElementById('__fynexo_content_style__')) {
         const st = document.createElement('style');
         st.id = '__fynexo_content_style__';
         st.textContent = __FNX_CONTENT_CSS__;
         shadowRoot.appendChild(st);
      }
      shellEl = shadowRoot.getElementById('__fynexo_shell__');
      if (!shellEl) {
         shellEl = document.createElement('div');
         shellEl.id = '__fynexo_shell__';
         shadowRoot.appendChild(shellEl);
      }
      syncShadowThemeVars();
      return shadowRoot;
   }

   function syncShadowThemeVars() {
      if (!hostEl) return;
      const src =
         styles && styles.colors && styles.colors.use_custom_colors
            ? styles.colors
            : activeTheme || {};
      const vars = {
         '--fnx-bg': src.bg || '#0b0c10',
         '--fnx-panel': src.panel || '#12141c',
         '--fnx-surface': src.surface || src.panel || '#171a24',
         '--fnx-elevated': src.elevated || '#1f2330',
         '--fnx-border': src.border || '#262b3a',
         '--fnx-border-hover': src.border_hover || src.borderHover || '#384056',
         '--fnx-fg': src.fg || '#f3f4f8',
         '--fnx-secondary': src.secondary || '#a6adbe',
         '--fnx-accent': src.accent || '#7c86ff',
      };
      const keys = Object.keys(vars);
      for (let i = 0; i < keys.length; i++) {
         hostEl.style.setProperty(keys[i], vars[keys[i]]);
      }
   }

   function showHudToast(msg) {
      if (!msg) return;
      const sr = ensureShadowHost();
      if (!sr || !shellEl) return;
      if (!toastEl || !toastEl.isConnected) {
         toastEl = document.createElement('div');
         toastEl.id = '__fynexo_hud_toast__';
         shellEl.appendChild(toastEl);
      }
      toastEl.textContent = String(msg);
      toastEl.style.display = 'inline-flex';
      if (toastTimer) clearTimeout(toastTimer);
      toastTimer = setTimeout(() => {
         toastTimer = null;
         if (toastEl) toastEl.style.display = 'none';
      }, 2200);
   }

   // Lightweight Console & Network Capture Ring Buffers (SSE-safe, lazy body capture)
   const MAX_LOGS = 180;
   const MAX_NET = 160;
   const devConsoleLogs = [];
   const devNetLogs = [];
   let devDrawerOpen = false;
   let devDrawerEl = null;
   let devActiveTab = 'console';
   let devBodyEl = null;
   let devReplBarEl = null;

   function pushConsoleLog(level, args) {
      try {
         const parts = [];
         for (let i = 0; i < args.length; i++) {
            const a = args[i];
            if (typeof a === 'string') parts.push(a);
            else if (a instanceof Error) parts.push(a.stack || a.message || String(a));
            else {
               try {
                  parts.push(__fnxJsonStringify(a));
               } catch (_) {
                  parts.push(String(a));
               }
            }
         }
         devConsoleLogs.push({ level, time: new Date().toLocaleTimeString(), text: parts.join(' ') });
         if (devConsoleLogs.length > MAX_LOGS) devConsoleLogs.shift();
         if (devDrawerOpen && devActiveTab === 'console') renderDevDrawerBody();
      } catch (_) {}
   }

   const __fnxIsSensitiveAuthOrigin = (() => {
      try {
         const host = String((window.location && window.location.hostname) || '').toLowerCase();
         const path = String((window.location && window.location.pathname) || '').toLowerCase();
         if (!host) return false;
         if (
            host === 'accounts.google.com' ||
            host === 'login.microsoftonline.com' ||
            host === 'login.live.com' ||
            host === 'appleid.apple.com' ||
            host === 'open.weixin.qq.com' ||
            host === 'auth.alipay.com' ||
            host === 'x.com' ||
            host === 'twitter.com' ||
            host.endsWith('.x.com') ||
            host.endsWith('.twitter.com') ||
            host.startsWith('auth.') ||
            host.startsWith('login.') ||
            host.startsWith('sso.') ||
            host.startsWith('oauth.')
         ) {
            return true;
         }
         if (host === 'github.com' && (path.startsWith('/login') || path.startsWith('/session'))) {
            return true;
         }
         return false;
      } catch (_) {
         return false;
      }
   })();

   function ensureDevCaptureHooks() {
      if (__fnxIsInternalOriginInit || __fnxIsSensitiveAuthOrigin) return;
      if (!window.__fnx_console_hooked__) {
         window.__fnx_console_hooked__ = true;
         ['log', 'info', 'warn', 'error'].forEach((lvl) => {
            try {
               const orig = console[lvl];
               if (typeof orig === 'function') {
                  console[lvl] = function () {
                     pushConsoleLog(lvl, arguments);
                     return orig.apply(this, arguments);
                  };
               }
            } catch (_) {}
         });
      }
      if (!window.__fnx_net_hooked__) {
         window.__fnx_net_hooked__ = true;
         try {
            const origFetch = window.fetch;
            if (typeof origFetch === 'function') {
               window.fetch = function (input, init) {
                  const started = Date.now();
                  const method = String((init && init.method) || (input && input.method) || 'GET').toUpperCase();
                  const url = typeof input === 'string' ? input : (input && input.url) || '';
                  return origFetch.apply(this, arguments).then(
                     (res) => {
                        devNetLogs.push({
                           method,
                           url,
                           status: res ? res.status : 0,
                           ms: Date.now() - started,
                        });
                        if (devNetLogs.length > MAX_NET) devNetLogs.shift();
                        if (devDrawerOpen && devActiveTab === 'network') renderDevDrawerBody();
                        return res;
                     },
                     (err) => {
                        devNetLogs.push({ method, url, status: 'ERR', ms: Date.now() - started });
                        if (devNetLogs.length > MAX_NET) devNetLogs.shift();
                        if (devDrawerOpen && devActiveTab === 'network') renderDevDrawerBody();
                        throw err;
                     },
                  );
               };
            }
         } catch (_) {}
      }
   }

   // ScriptForge & Site Rules Engine
   const scriptForgeExecutedIds = Object.create(null);
   const scriptForgeListeners = Object.create(null);
   let scriptForgeListenerSeq = 1;
   let scriptForgeCmdSeq = 1;
   let scriptForgeMenuCommands = [];
   let activePageScriptCount = 0;
   let siteRulesEl = null;
   let pageRulesEl = null;
   let reportContentPageMetaRef = null;
   let syncShieldPopoverUIRef = null;

   __FNX_SCRIPTFORGE__;

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
            const m = bg.match(/rgba?\(\s*(\d+)\s*,\s*(\d+)\s*,\s*(\d+)(?:\s*,\s*([\d.]+))?\s*\)/i);
            if (!m) return null;
            const a = m[4] !== undefined ? parseFloat(m[4]) : 1;
            if (a < 0.12) return null;
            return 0.299 * Number(m[1]) + 0.587 * Number(m[2]) + 0.114 * Number(m[3]);
         };

         let dark = false;
         const lumBody = checkEl(document.body);
         if (lumBody !== null) {
            dark = lumBody < 112;
         } else {
            const lumHtml = checkEl(document.documentElement);
            if (lumHtml !== null) dark = lumHtml < 112;
         }

         if (pr) pr.disabled = wasDisabled;
         return dark;
      } catch (_) {
         return false;
      }
   }

   function isCurrentHostWhitelisted() {
      const host = String((window.location && window.location.hostname) || '').toLowerCase();
      if (!host || !Array.isArray(adblockWhitelist) || !adblockWhitelist.length) return false;
      for (let i = 0; i < adblockWhitelist.length; i++) {
         const w = String(adblockWhitelist[i] || '').toLowerCase();
         if (w && (host === w || host.endsWith('.' + w))) return true;
      }
      return false;
   }

   function getDetectedAdCount() {
      if (isInternalAny() || !config.adblock || isCurrentHostWhitelisted()) return 0;
      try {
         return document.querySelectorAll(
            'ins.adsbygoogle, [id^="google_ads_iframe"], .ad-container, .ad-banner, [data-ad-slot]',
         ).length;
      } catch (_) {
         return 0;
      }
   }

   function applyPageRules() {
      const parent = document.head || document.documentElement || document.body;
      if (!parent) return;
      if (!pageRulesEl || !pageRulesEl.isConnected) {
         pageRulesEl = document.getElementById('__fynexo_page_rules__');
         if (!pageRulesEl) {
            pageRulesEl = document.createElement('style');
            pageRulesEl.id = '__fynexo_page_rules__';
            parent.appendChild(pageRulesEl);
         }
      }
      if (isInternalAny()) {
         pageRulesEl.textContent = '';
         return;
      }
      const host = String((window.location && window.location.hostname) || '').toLowerCase();
      const isLocalDev = isLocalDevHost(host);
      const siteOv = getCurrentSiteOverride();
      const siteDm = siteOv && siteOv.dark_mode ? String(siteOv.dark_mode).toLowerCase() : 'auto';
      const wantForceDark =
         siteDm === 'on'
            ? true
            : siteDm === 'off' || (isLocalDev && siteDm === 'auto')
              ? false
              : !!config.force_dark;

      const chunks = [];
      chunks.push('::highlight(fynexo-find){background:rgba(124,134,255,0.36);color:inherit;}');
      chunks.push('::highlight(fynexo-find-active){background:#7c86ff;color:#ffffff;}');
      if (config.reduce_animations) {
         chunks.push(
            '*,*::before,*::after{animation-duration:0.01ms!important;animation-iteration-count:1!important;transition-duration:0.01ms!important;scroll-behavior:auto!important;}',
         );
      }
      if (wantForceDark && (siteDm === 'on' || !isPageAlreadyDark())) {
         chunks.push(
            'html:not([data-fynexo-internal]){background-color:#111113!important;}html:not([data-fynexo-internal])>body{filter:invert(0.92) hue-rotate(180deg)!important;background-color:#ffffff;}html:not([data-fynexo-internal])>body img,html:not([data-fynexo-internal])>body video,html:not([data-fynexo-internal])>body picture,html:not([data-fynexo-internal])>body canvas,html:not([data-fynexo-internal])>body iframe,html:not([data-fynexo-internal])>body svg:not(:root),html:not([data-fynexo-internal])>body [style*="background-image"]{filter:invert(1) hue-rotate(180deg)!important;}',
         );
      }
      if (config.adblock !== false && !isLocalDev && !isCurrentHostWhitelisted()) {
         chunks.push(
            'ins.adsbygoogle,[id^="div-gpt-ad"],[id^="google_ads_iframe"],iframe[src*="doubleclick.net"],iframe[src*="googlesyndication.com"],.taboola-placeholder,.OUTBRAIN,ytd-ad-slot-renderer,ytd-in-feed-ad-layout-renderer,ytd-banner-promo-renderer,ytd-statement-banner-renderer,#player-ads,#masthead-ad,[data-ad-slot]{display:none!important;}',
         );
      }
      const next = chunks.join('\n');
      if (pageRulesEl.textContent !== next) pageRulesEl.textContent = next;
   }

   function applySiteRules() {
      if (isInternalAny()) return;
      const engineOn = config.ext_userscript_engine !== false;
      if (!engineOn) {
         try {
            const gmStyles = document.querySelectorAll('style[data-fnx-gm-style]');
            for (let i = 0; i < gmStyles.length; i++) gmStyles[i].remove();
         } catch (_) {}
         scriptForgeMenuCommands = [];
         for (const k in scriptForgeExecutedIds) delete scriptForgeExecutedIds[k];
      }
      const host = String((window.location && window.location.hostname) || '').toLowerCase();
      const href = String((window.location && window.location.href) || '');
      let combinedCss = '';
      let matchedCount = 0;

      for (let i = 0; i < enabledRules.length; i++) {
         const r = enabledRules[i];
         if (!r || !r.enabled) continue;
         const jsText = String(r.js || '');
         const isPureBoostOrPicked =
            typeof r.id === 'string' && (r.id.startsWith('boost-') || r.id.startsWith('picked-'));
         if (!engineOn && !isPureBoostOrPicked) continue;
         const meta = jsText ? parseUserscriptMeta(jsText) : null;
         const matched = meta
            ? urlMatchesUserscriptMeta(href, meta)
            : domainMatchesRule(host, r.domain);
         if (!matched) continue;
         matchedCount++;
         if (r.css) combinedCss += '\n' + String(r.css);

         const execKey = String(r.id || r.name || i);
         if (engineOn && jsText.trim() && !scriptForgeExecutedIds[execKey]) {
            scriptForgeExecutedIds[execKey] = true;
            const runAt = (meta && meta.runAt) || 'document-end';
            const runRuleJs = () => {
               try {
                  const sb = createScriptForgeSandbox(r, meta);
                  const fn = new Function(
                     'unsafeWindow',
                     'GM',
                     'GM_info',
                     'GM_addStyle',
                     'GM_getValue',
                     'GM_setValue',
                     'GM_deleteValue',
                     'GM_listValues',
                     'GM_addValueChangeListener',
                     'GM_removeValueChangeListener',
                     'GM_setClipboard',
                     'GM_notification',
                     'GM_openInTab',
                     'GM_registerMenuCommand',
                     'GM_unregisterMenuCommand',
                     'GM_xmlhttpRequest',
                     'GM_log',
                     jsText,
                  );
                  fn.call(
                     sb.unsafeWindow,
                     sb.unsafeWindow,
                     sb.GM,
                     sb.GM_info,
                     sb.GM_addStyle,
                     sb.GM_getValue,
                     sb.GM_setValue,
                     sb.GM_deleteValue,
                     sb.GM_listValues,
                     sb.GM_addValueChangeListener,
                     sb.GM_removeValueChangeListener,
                     sb.GM_setClipboard,
                     sb.GM_notification,
                     sb.GM_openInTab,
                     sb.GM_registerMenuCommand,
                     sb.GM_unregisterMenuCommand,
                     sb.GM_xmlhttpRequest,
                     sb.GM_log,
                  );
               } catch (err) {
                  console.warn('[ScriptForge Userscript Error]:', (meta && meta.name) || r.name || r.id, err);
               }
            };
            if (runAt === 'document-start') runRuleJs();
            else if (runAt === 'document-body') {
               if (document.body) runRuleJs();
               else document.addEventListener('DOMContentLoaded', runRuleJs, { once: true });
            } else if (runAt === 'document-idle') {
               if (document.readyState === 'complete') setTimeout(runRuleJs, 0);
               else window.addEventListener('load', () => setTimeout(runRuleJs, 0), { once: true });
            } else if (document.readyState === 'loading') {
               document.addEventListener('DOMContentLoaded', runRuleJs, { once: true });
            } else {
               runRuleJs();
            }
         }
      }

      activePageScriptCount = matchedCount;
      if (typeof reportContentPageMetaRef === 'function') reportContentPageMetaRef(false);

      if (!combinedCss) {
         if (siteRulesEl && siteRulesEl.isConnected) siteRulesEl.remove();
         siteRulesEl = null;
         return;
      }
      const parent = document.head || document.documentElement || document.body;
      if (!parent) return;
      if (!siteRulesEl || !siteRulesEl.isConnected) {
         siteRulesEl = document.getElementById('__fynexo_site_rules__') || document.createElement('style');
         siteRulesEl.id = '__fynexo_site_rules__';
         if (!siteRulesEl.isConnected) parent.appendChild(siteRulesEl);
      }
      if (siteRulesEl.textContent !== combinedCss) siteRulesEl.textContent = combinedCss;
   }

   // Media Detection & Volume Boost
   let currentMediaEl = null;
   let mediaGainValue = 100;
   let mediaIsMuted = false;
   let audioCtx = null;

   function isControllableMedia(el) {
      return !!(el && el.isConnected && (el.tagName === 'VIDEO' || el.tagName === 'AUDIO'));
   }

   function findActiveMedia() {
      try {
         const list = document.querySelectorAll('video, audio');
         let best = null;
         for (let i = 0; i < list.length; i++) {
            const m = list[i];
            if (!m.paused && !m.ended) return m;
            if (!best && (m.readyState >= 1 || m.currentSrc || m.src)) best = m;
         }
         return best;
      } catch (_) {
         return null;
      }
   }

   function ensureMediaGainNode(mediaEl) {
      if (!mediaEl || mediaEl.__fnx_gain_node) return mediaEl ? mediaEl.__fnx_gain_node : null;
      const AC = window.AudioContext || window.webkitAudioContext;
      if (!AC) return null;
      try {
         if (!audioCtx) audioCtx = new AC();
         if (audioCtx.state === 'suspended') audioCtx.resume().catch(() => {});
         const src = audioCtx.createMediaElementSource(mediaEl);
         const gain = audioCtx.createGain();
         src.connect(gain);
         gain.connect(audioCtx.destination);
         mediaEl.__fnx_gain_node = gain;
         return gain;
      } catch (_) {
         return null;
      }
   }

   function applyMediaVolume(val) {
      const pct = Math.max(0, Math.min(300, Math.round(Number(val) || 0)));
      mediaGainValue = pct;
      const m = isControllableMedia(currentMediaEl) ? currentMediaEl : findActiveMedia();
      if (!m) return;
      currentMediaEl = m;
      m.muted = !!mediaIsMuted || !!isTabMutedState || pct === 0;
      if (pct <= 100) {
         m.volume = pct / 100;
         if (m.__fnx_gain_node) m.__fnx_gain_node.gain.value = 1;
      } else {
         m.volume = 1;
         const gain = ensureMediaGainNode(m);
         if (gain) gain.gain.value = pct / 100;
      }
   }

   function toggleLocalMediaPlayPause() {
      const m = isControllableMedia(currentMediaEl) ? currentMediaEl : findActiveMedia();
      if (!m) return;
      currentMediaEl = m;
      if (m.paused || m.ended) m.play().catch(() => {});
      else m.pause();
   }

   function applyTabMuted(muted) {
      isTabMutedState = !!muted;
      try {
         document.querySelectorAll('video, audio').forEach((m) => {
            m.muted = isTabMutedState || mediaIsMuted;
         });
      } catch (_) {}
   }

   // Pure-DOM Find Bar (Lazy-Mounted)
   let findBarEl = null;
   let findInputEl = null;
   let findCountEl = null;
   let findRanges = [];
   let findIdx = -1;

   function clearFindHighlights() {
      findRanges = [];
      findIdx = -1;
      try {
         if (typeof CSS !== 'undefined' && CSS.highlights) {
            CSS.highlights.delete('fynexo-find');
            CSS.highlights.delete('fynexo-find-active');
         }
      } catch (_) {}
   }

   function updateFindHighlight(step) {
      const q = findInputEl ? findInputEl.value : '';
      clearFindHighlights();
      if (!q || !document.body) {
         if (findCountEl) findCountEl.textContent = '0/0';
         return;
      }
      const lowerQ = q.toLowerCase();
      try {
         const walker = document.createTreeWalker(document.body, NodeFilter.SHOW_TEXT);
         let node;
         while ((node = walker.nextNode())) {
            if (!node.nodeValue) continue;
            const parent = node.parentElement;
            if (!parent || /^(SCRIPT|STYLE|NOSCRIPT|TEXTAREA)$/i.test(parent.tagName)) continue;
            if (parent.closest && parent.closest('#__fynexo_host__')) continue;
            const hay = node.nodeValue.toLowerCase();
            let pos = 0;
            while ((pos = hay.indexOf(lowerQ, pos)) !== -1) {
               const r = document.createRange();
               r.setStart(node, pos);
               r.setEnd(node, pos + q.length);
               findRanges.push(r);
               pos += q.length;
               if (findRanges.length >= 600) break;
            }
            if (findRanges.length >= 600) break;
         }
      } catch (_) {}

      if (!findRanges.length) {
         if (findCountEl) findCountEl.textContent = '0/0';
         return;
      }
      findIdx = step ? (findIdx + step + findRanges.length) % findRanges.length : 0;
      const cur = findRanges[findIdx];
      try {
         if (typeof Highlight === 'function' && typeof CSS !== 'undefined' && CSS.highlights) {
            CSS.highlights.set('fynexo-find', new Highlight(...findRanges));
            CSS.highlights.set('fynexo-find-active', new Highlight(cur));
         }
         const rect = cur.getBoundingClientRect();
         if (rect.top < 40 || rect.bottom > window.innerHeight - 40) {
            window.scrollBy({ top: rect.top - window.innerHeight / 3, behavior: 'smooth' });
         }
      } catch (_) {}
      if (findCountEl) findCountEl.textContent = `${findIdx + 1}/${findRanges.length}`;
   }

   function closeFindBar() {
      clearFindHighlights();
      if (findBarEl) findBarEl.style.display = 'none';
   }

   function openFindBar() {
      if (isInternalSettings()) return;
      ensureShadowHost();
      if (!shellEl) return;
      if (!findBarEl) {
         findBarEl = document.createElement('div');
         findBarEl.id = '__fynexo_find_bar__';
         findInputEl = document.createElement('input');
         findInputEl.id = '__fynexo_find_input__';
         findInputEl.type = 'text';
         findInputEl.placeholder = isZh() ? '页面内查找...' : 'Find in page...';
         findCountEl = document.createElement('span');
         findCountEl.id = '__fynexo_find_count__';
         findCountEl.textContent = '0/0';

         const btnPrev = document.createElement('button');
         btnPrev.type = 'button';
         btnPrev.className = 'fnx-ca-btn';
         btnPrev.textContent = '↑';
         btnPrev.onclick = () => updateFindHighlight(-1);

         const btnNext = document.createElement('button');
         btnNext.type = 'button';
         btnNext.className = 'fnx-ca-btn';
         btnNext.textContent = '↓';
         btnNext.onclick = () => updateFindHighlight(1);

         const btnClose = document.createElement('button');
         btnClose.type = 'button';
         btnClose.className = 'fnx-ca-btn';
         btnClose.textContent = '×';
         btnClose.onclick = closeFindBar;

         findInputEl.addEventListener('input', () => updateFindHighlight(0));
         findInputEl.addEventListener('keydown', (e) => {
            if (e.key === 'Enter') {
               e.preventDefault();
               updateFindHighlight(e.shiftKey ? -1 : 1);
            } else if (e.key === 'Escape') {
               e.preventDefault();
               closeFindBar();
            }
         });

         findBarEl.append(findInputEl, findCountEl, btnPrev, btnNext, btnClose);
         shellEl.appendChild(findBarEl);
      }
      findBarEl.style.display = 'flex';
      findInputEl.focus();
      findInputEl.select();
   }

   // Pure-DOM Reader Mode (Lazy-Mounted)
   let isReaderOpen = false;
   let readerEl = null;
   let readerFontPx = 17;
   let readerTheme = 'dark';
   let cachedReaderable = false;

   function detectReaderable() {
      if (isInternalAny() || !document.body) return false;
      try {
         const art = document.querySelector('article, main, [role="main"]');
         const target = art || document.body;
         const paras = target.querySelectorAll('p');
         let len = 0;
         for (let i = 0; i < paras.length && i < 60; i++) {
            len += (paras[i].textContent || '').trim().length;
            if (len >= 520) return true;
         }
         return false;
      } catch (_) {
         return false;
      }
   }

   function closeReaderMode() {
      isReaderOpen = false;
      if (readerEl) readerEl.style.display = 'none';
      if (typeof reportContentPageMetaRef === 'function') reportContentPageMetaRef(true);
   }

   function toggleReaderMode() {
      if (isInternalAny()) return;
      if (isReaderOpen) {
         closeReaderMode();
         return;
      }
      ensureShadowHost();
      if (!shellEl) return;
      if (!readerEl) {
         readerEl = document.createElement('div');
         readerEl.id = '__fynexo_reader__';
         shellEl.appendChild(readerEl);
      }
      readerEl.textContent = '';
      readerEl.className = readerTheme === 'paper' ? 'theme-paper' : readerTheme === 'sepia' ? 'theme-sepia' : '';

      const bar = document.createElement('div');
      bar.className = 'fnx-reader-bar';
      const titleSpan = document.createElement('span');
      titleSpan.textContent = isZh() ? '阅读模式' : 'Reader Mode';
      const ctrls = document.createElement('div');
      ctrls.className = 'fnx-reader-ctrl';

      const makeBtn = (label, fn) => {
         const b = document.createElement('button');
         b.type = 'button';
         b.className = 'fnx-ca-btn';
         b.textContent = label;
         b.onclick = fn;
         return b;
      };
      const artBox = document.createElement('article');
      artBox.id = '__fynexo_reader_article__';
      artBox.style.fontSize = readerFontPx + 'px';

      ctrls.append(
         makeBtn('A-', () => {
            readerFontPx = Math.max(13, readerFontPx - 1);
            artBox.style.fontSize = readerFontPx + 'px';
         }),
         makeBtn('A+', () => {
            readerFontPx = Math.min(26, readerFontPx + 1);
            artBox.style.fontSize = readerFontPx + 'px';
         }),
         makeBtn(isZh() ? '深色' : 'Dark', () => {
            readerTheme = 'dark';
            readerEl.className = '';
         }),
         makeBtn(isZh() ? '纸白' : 'Paper', () => {
            readerTheme = 'paper';
            readerEl.className = 'theme-paper';
         }),
         makeBtn(isZh() ? '护眼' : 'Sepia', () => {
            readerTheme = 'sepia';
            readerEl.className = 'theme-sepia';
         }),
         makeBtn(isZh() ? '退出' : 'Close', closeReaderMode),
      );
      bar.append(titleSpan, ctrls);

      const h1 = document.createElement('h1');
      const pageH1 = document.querySelector('h1');
      h1.textContent = (pageH1 && pageH1.textContent.trim()) || document.title || '';
      const byline = document.createElement('div');
      byline.className = 'fnx-reader-byline';
      byline.textContent = window.location.hostname || '';
      artBox.append(h1, byline);

      const srcRoot = document.querySelector('article, main, [role="main"]') || document.body;
      if (srcRoot) {
         const nodes = srcRoot.querySelectorAll('h2, h3, p, pre, blockquote');
         for (let i = 0; i < nodes.length && i < 400; i++) {
            const txt = (nodes[i].textContent || '').trim();
            if (txt.length < 2) continue;
            const clone = document.createElement(nodes[i].tagName.toLowerCase());
            clone.textContent = txt;
            artBox.appendChild(clone);
         }
      }
      readerEl.append(bar, artBox);
      readerEl.style.display = 'flex';
      isReaderOpen = true;
      if (typeof reportContentPageMetaRef === 'function') reportContentPageMetaRef(true);
   }

   // Visual Element Picker (Lazy-Mounted)
   let pickerActive = false;
   let pickerBoxEl = null;
   let pickerBarEl = null;
   let pickerSelEl = null;
   let pickerHoverTarget = null;
   let pickerSessionCss = '';

   function buildUniqueCssSelector(el) {
      if (!el || el.nodeType !== 1) return '';
      if (el.id && /^[A-Za-z_][A-Za-z0-9_-]*$/.test(el.id)) return '#' + el.id;
      const parts = [];
      let cur = el;
      for (let d = 0; cur && cur.nodeType === 1 && cur !== document.body && d < 4; d++) {
         let seg = cur.tagName.toLowerCase();
         if (cur.id && /^[A-Za-z_][A-Za-z0-9_-]*$/.test(cur.id)) {
            parts.unshift('#' + cur.id);
            break;
         }
         const cls = Array.from(cur.classList || [])
            .filter((c) => c && /^[A-Za-z_][A-Za-z0-9_-]*$/.test(c))
            .slice(0, 2);
         if (cls.length) seg += '.' + cls.join('.');
         parts.unshift(seg);
         cur = cur.parentElement;
      }
      return parts.join(' > ');
   }

   function stopVisualElementPicker() {
      if (!pickerActive) return;
      pickerActive = false;
      pickerHoverTarget = null;
      if (pickerBoxEl) pickerBoxEl.style.display = 'none';
      if (pickerBarEl) pickerBarEl.style.display = 'none';
      window.removeEventListener('mousemove', onPickerMove, true);
      window.removeEventListener('click', onPickerClick, true);
      if (typeof reportContentPageMetaRef === 'function') reportContentPageMetaRef(true);
   }

   function onPickerMove(e) {
      if (!pickerActive) return;
      const t = e.target;
      if (!t || t === hostEl || (t.closest && t.closest('#__fynexo_host__'))) return;
      pickerHoverTarget = t;
      const r = t.getBoundingClientRect();
      if (pickerBoxEl) {
         pickerBoxEl.style.display = 'block';
         pickerBoxEl.style.left = r.left + 'px';
         pickerBoxEl.style.top = r.top + 'px';
         pickerBoxEl.style.width = r.width + 'px';
         pickerBoxEl.style.height = r.height + 'px';
      }
      if (pickerSelEl) pickerSelEl.textContent = buildUniqueCssSelector(t);
   }

   function onPickerClick(e) {
      if (!pickerActive) return;
      const t = e.target;
      if (!t || t === hostEl || (t.closest && t.closest('#__fynexo_host__'))) return;
      e.preventDefault();
      e.stopPropagation();
      const sel = buildUniqueCssSelector(t);
      if (sel) {
         pickerSessionCss += '\n' + sel + ' { display: none !important; }';
         const st = document.getElementById('__fynexo_picker_rules__') || document.createElement('style');
         st.id = '__fynexo_picker_rules__';
         st.textContent = pickerSessionCss;
         (document.head || document.documentElement).appendChild(st);
         showHudToast((isZh() ? '已隐藏元素：' : 'Hidden: ') + sel);
      }
      stopVisualElementPicker();
   }

   function startVisualElementPicker() {
      if (isInternalAny()) return;
      if (pickerActive) {
         stopVisualElementPicker();
         return;
      }
      ensureShadowHost();
      if (!shellEl) return;
      if (!pickerBoxEl) {
         pickerBoxEl = document.createElement('div');
         pickerBoxEl.id = '__fynexo_picker_box__';
         shellEl.appendChild(pickerBoxEl);
      }
      if (!pickerBarEl) {
         pickerBarEl = document.createElement('div');
         pickerBarEl.id = '__fynexo_picker_bar__';
         const lbl = document.createElement('span');
         lbl.textContent = isZh() ? '点击元素以隐藏' : 'Click element to hide';
         pickerSelEl = document.createElement('span');
         pickerSelEl.id = '__fynexo_picker_sel__';
         const cancelBtn = document.createElement('button');
         cancelBtn.type = 'button';
         cancelBtn.className = 'fnx-ca-btn';
         cancelBtn.textContent = isZh() ? '取消 (Esc)' : 'Cancel (Esc)';
         cancelBtn.onclick = stopVisualElementPicker;
         pickerBarEl.append(lbl, pickerSelEl, cancelBtn);
         shellEl.appendChild(pickerBarEl);
      }
      pickerBarEl.style.display = 'flex';
      pickerActive = true;
      window.addEventListener('mousemove', onPickerMove, true);
      window.addEventListener('click', onPickerClick, true);
      if (typeof reportContentPageMetaRef === 'function') reportContentPageMetaRef(true);
   }

   // Lazy DevDrawer (Mounted ONLY when opened)
   function renderDevDrawerBody() {
      if (!devBodyEl) return;
      devBodyEl.textContent = '';
      if (devReplBarEl) devReplBarEl.style.display = devActiveTab === 'console' ? 'flex' : 'none';
      if (devActiveTab === 'console') {
         for (let i = 0; i < devConsoleLogs.length; i++) {
            const item = devConsoleLogs[i];
            const row = document.createElement('div');
            row.className = 'fnx-dev-row ' + item.level;
            row.textContent = `[${item.time}] [${item.level.toUpperCase()}] ${item.text}`;
            devBodyEl.appendChild(row);
         }
      } else if (devActiveTab === 'network') {
         for (let i = 0; i < devNetLogs.length; i++) {
            const n = devNetLogs[i];
            const row = document.createElement('div');
            row.className = 'fnx-dev-row' + (n.status === 'ERR' || n.status >= 400 ? ' error' : '');
            row.textContent = `${n.method} [${n.status}] ${n.ms}ms  ${n.url}`;
            devBodyEl.appendChild(row);
         }
      } else {
         const row = document.createElement('div');
         row.className = 'fnx-dev-row';
         row.textContent = `URL: ${window.location.href}\nTitle: ${document.title}\nNodes: ${document.getElementsByTagName('*').length}\nScripts: ${activePageScriptCount}`;
         devBodyEl.appendChild(row);
      }
   }

   function toggleDevDrawer() {
      ensureDevCaptureHooks();
      ensureShadowHost();
      if (!shellEl) return;
      if (!devDrawerEl) {
         devDrawerEl = document.createElement('div');
         devDrawerEl.id = '__fynexo_dev_drawer__';
         const head = document.createElement('div');
         head.className = 'fnx-dev-head';
         const tabs = document.createElement('div');
         tabs.className = 'fnx-dev-tabs';
         const tabBtns = {};
         ['console', 'network', 'info'].forEach((tKey) => {
            const b = document.createElement('button');
            b.type = 'button';
            b.className = 'fnx-dev-tab' + (tKey === devActiveTab ? ' active' : '');
            b.textContent = tKey.toUpperCase();
            b.onclick = () => {
               devActiveTab = tKey;
               Object.keys(tabBtns).forEach((k) => tabBtns[k].classList.toggle('active', k === tKey));
               renderDevDrawerBody();
            };
            tabBtns[tKey] = b;
            tabs.appendChild(b);
         });
         const right = document.createElement('div');
         right.className = 'fnx-dev-tabs';
         const clrBtn = document.createElement('button');
         clrBtn.type = 'button';
         clrBtn.className = 'fnx-ca-btn';
         clrBtn.textContent = isZh() ? '清空' : 'Clear';
         clrBtn.onclick = () => {
            if (devActiveTab === 'console') devConsoleLogs.length = 0;
            else if (devActiveTab === 'network') devNetLogs.length = 0;
            renderDevDrawerBody();
         };
         const clsBtn = document.createElement('button');
         clsBtn.type = 'button';
         clsBtn.className = 'fnx-ca-btn';
         clsBtn.textContent = '×';
         clsBtn.onclick = () => {
            devDrawerOpen = false;
            devDrawerEl.style.display = 'none';
            if (typeof reportContentPageMetaRef === 'function') reportContentPageMetaRef(true);
         };
         right.append(clrBtn, clsBtn);
         head.append(tabs, right);

         devBodyEl = document.createElement('div');
         devBodyEl.className = 'fnx-dev-body';

         devReplBarEl = document.createElement('div');
         devReplBarEl.className = 'fnx-dev-repl';
         const replInput = document.createElement('input');
         replInput.type = 'text';
         replInput.placeholder = isZh() ? '执行 JS 表达式（需开启设置中 Allow Page Eval）' : 'Run JS expression (requires Allow Page Eval)';
         replInput.addEventListener('keydown', (e) => {
            if (e.key !== 'Enter') return;
            const code = replInput.value.trim();
            if (!code) return;
            replInput.value = '';
            if (!config.allow_page_eval) {
               pushConsoleLog('warn', ['Blocked: enable Allow Console JS Eval in Settings -> Browsing']);
               return;
            }
            try {
               const res = window.eval(code);
               pushConsoleLog('log', ['=>', res]);
            } catch (err) {
               pushConsoleLog('error', [err]);
            }
         });
         devReplBarEl.appendChild(replInput);

         devDrawerEl.append(head, devBodyEl, devReplBarEl);
         shellEl.appendChild(devDrawerEl);
      }
      devDrawerOpen = !devDrawerOpen;
      devDrawerEl.style.display = devDrawerOpen ? 'flex' : 'none';
      if (devDrawerOpen) renderDevDrawerBody();
      if (typeof reportContentPageMetaRef === 'function') reportContentPageMetaRef(true);
   }

   function clearCurrentSiteStorageLocal() {
      try { window.localStorage && window.localStorage.clear(); } catch (_) {}
      try { window.sessionStorage && window.sessionStorage.clear(); } catch (_) {}
      try {
         const cookies = String(document.cookie || '').split(';');
         for (let i = 0; i < cookies.length; i++) {
            const name = cookies[i].split('=')[0].trim();
            if (name) {
               document.cookie = name + '=;expires=Thu, 01 Jan 1970 00:00:00 GMT;path=/';
            }
         }
      } catch (_) {}
      showHudToast(isZh() ? '已清除当前站点本地存储' : 'Cleared site storage');
   }

   function performHardCacheBustReload() {
      try {
         window.location.reload();
      } catch (_) {
         sendIpc({ type: 'RELOAD' });
      }
   }

   // Built-in Opt-In Extensions (YouTube Ad Skipper, GitHub Harmonizer, Mouse Gestures)
   __FNX_EXTENSIONS__;

   function syncExtensions() {
      if (!isInternalAny() && config.ext_mouse_gestures !== false) {
         ensureShadowHost();
      }
      try { if (typeof syncYouTubeAdSkipper === 'function') syncYouTubeAdSkipper(); } catch (_) {}
      try { if (typeof syncGitHubHarmonizer === 'function') syncGitHubHarmonizer(); } catch (_) {}
      try { if (typeof syncMouseGestures === 'function') syncMouseGestures(); } catch (_) {}
   }

   // Keybinding Matcher
   function matchesKeybind(e, combo) {
      if (!combo || typeof combo !== 'string') return false;
      const parts = combo.toLowerCase().split('+').map((s) => s.trim()).filter(Boolean);
      if (!parts.length) return false;
      const wantCtrl = parts.includes('ctrl') || parts.includes('cmd') || parts.includes('meta');
      const wantShift = parts.includes('shift');
      const wantAlt = parts.includes('alt');
      const wantKey = parts[parts.length - 1];
      if (!!(e.ctrlKey || e.metaKey) !== wantCtrl) return false;
      if (!!e.shiftKey !== wantShift) return false;
      if (!!e.altKey !== wantAlt) return false;
      const evKey = String(e.key || '').toLowerCase();
      if (wantKey === 'plus' || wantKey === '=') return evKey === '=' || evKey === '+';
      if (wantKey === 'minus' || wantKey === '-') return evKey === '-' || evKey === '_';
      return evKey === wantKey;
   }

   function isEditableTarget(t) {
      if (!t || t.nodeType !== 1) return false;
      const tag = t.tagName;
      return tag === 'INPUT' || tag === 'TEXTAREA' || tag === 'SELECT' || !!t.isContentEditable;
   }

   // Scroll Sync & Restore
   let lastSentScrollY = -1;
   let scrollSyncTimer = null;
   function flushScrollSync() {
      if (isInternalAny()) return;
      const y = Math.max(0, Math.round(window.scrollY || 0));
      if (Math.abs(y - lastSentScrollY) < 12) return;
      lastSentScrollY = y;
      sendIpc({ type: 'SAVE_SCROLL', scroll_y: y });
   }

   window.addEventListener(
      'scroll',
      () => {
         if (scrollSyncTimer) return;
         scrollSyncTimer = setTimeout(() => {
            scrollSyncTimer = null;
            flushScrollSync();
         }, 450);
      },
      { passive: true },
   );
   window.addEventListener('pagehide', flushScrollSync, { passive: true });

   // Content Page Meta Reporter -> HUD WebView
   let pageMetaTimer = null;
   let lastPageMetaJson = '';

   function buildContentPageMeta() {
      const curUrl = canonicalCurrentUrl();
      const internal = isInternalAny(curUrl);
      const activeMedia = !internal
         ? isControllableMedia(currentMediaEl)
            ? currentMediaEl
            : findActiveMedia()
         : null;
      const hasMedia = !!activeMedia;
      let mediaTitle = '';
      if (hasMedia) {
         try {
            const pageH1 = document.querySelector('h1.ytd-watch-metadata, h1.title, h1');
            mediaTitle = (pageH1 && pageH1.textContent.trim()) || document.title || '';
         } catch (_) {}
      }
      return {
         url: curUrl || '',
         readerable: !!config.ext_reader_mode && !internal && !!cachedReaderable,
         reader_open: !!isReaderOpen,
         dev_drawer_open: !!devDrawerOpen,
         picker_active: !!pickerActive,
         has_media: hasMedia,
         media_paused: hasMedia ? !!(activeMedia.paused || activeMedia.ended) : true,
         media_title: mediaTitle,
         media_muted: !!mediaIsMuted,
         media_gain: Number(mediaGainValue) || 100,
         blocked_ads: !internal ? getDetectedAdCount() : 0,
         script_count: Number(activePageScriptCount) || 0,
         sf_commands:
            !internal && Array.isArray(scriptForgeMenuCommands)
               ? scriptForgeMenuCommands.map((c) => ({
                    id: c.id,
                    caption: c.caption,
                    scriptName: c.scriptName || '',
                 }))
               : [],
         page_is_dark: !internal && isPageAlreadyDark(),
      };
   }

   function sendContentPageMetaNow(force) {
      pageMetaTimer = null;
      if (isBackgroundParked) return;
      const meta = buildContentPageMeta();
      const payload = { type: 'CONTENT_PAGE_META', meta };
      const json = __fnxJsonStringify(payload);
      if (!force && json === lastPageMetaJson) return;
      lastPageMetaJson = json;
      sendIpc(payload);
   }

   reportContentPageMetaRef = function (force) {
      if (force) {
         if (pageMetaTimer) {
            clearTimeout(pageMetaTimer);
            pageMetaTimer = null;
         }
         sendContentPageMetaNow(true);
         return;
      }
      if (pageMetaTimer) return;
      pageMetaTimer = setTimeout(() => sendContentPageMetaNow(false), 80);
   };

   // Title & SPA URL Sync
   let lastSentTitle = '';
   let lastSentUrl = '';
   function reportTitleAndUrl(force) {
      const curUrl = canonicalCurrentUrl();
      const curTitle = String(document.title || '').trim();
      if (!force && curTitle === lastSentTitle && curUrl === lastSentUrl) return;
      lastSentTitle = curTitle;
      lastSentUrl = curUrl;
      sendIpc({ type: 'UPDATE_TAB_TITLE', title: curTitle, url: curUrl });
      reportContentPageMetaRef(false);
   }

   if (!isInternalAny() && window.history) {
      ['pushState', 'replaceState'].forEach((m) => {
         try {
            const orig = window.history[m];
            if (typeof orig === 'function') {
               window.history[m] = function () {
                  const r = orig.apply(this, arguments);
                  setTimeout(() => {
                     applySiteRules();
                     reportTitleAndUrl(false);
                  }, 40);
                  return r;
               };
            }
         } catch (_) {}
      });
      window.addEventListener('popstate', () => {
         setTimeout(() => {
            applySiteRules();
            reportTitleAndUrl(false);
         }, 40);
      });
   }

   // Media Play/Pause State Sync
   let lastMediaPlaying = false;
   function checkMediaPlaybackState() {
      const m = findActiveMedia();
      const playing = !!(m && !m.paused && !m.ended);
      if (playing) currentMediaEl = m;
      if (playing !== lastMediaPlaying) {
         lastMediaPlaying = playing;
         sendIpc({ type: 'UPDATE_TAB_MEDIA_STATE', playing });
      }
      reportContentPageMetaRef(false);
   }
   document.addEventListener('play', checkMediaPlaybackState, true);
   document.addEventListener('pause', checkMediaPlaybackState, true);
   document.addEventListener('ended', checkMediaPlaybackState, true);

   // Dismiss HUD Popovers on Content Click & Fullscreen Top-Edge Peek
   let lastPointerSentAt = 0;
   window.addEventListener(
      'pointerdown',
      () => {
         const now = Date.now();
         if (now - lastPointerSentAt > 60) {
            lastPointerSentAt = now;
            sendIpc({ type: 'CONTENT_POINTER_DOWN' });
         }
      },
      { capture: true, passive: true },
   );

   let lastRemotePeekAt = 0;
   window.addEventListener(
      'mousemove',
      (e) => {
         if (!isFullscreen) return;
         if (e.clientY <= 4) {
            const now = Date.now();
            if (now - lastRemotePeekAt > 250) {
               lastRemotePeekAt = now;
               sendIpc({ type: 'HUD_ACTION', action: 'fullscreen_peek' });
            }
         }
      },
      { passive: true },
   );

   // Keyboard Shortcuts
   window.addEventListener(
      'keydown',
      (e) => {
         if (e.key === 'Escape') {
            if (pickerActive) {
               stopVisualElementPicker();
               return;
            }
            if (findBarEl && findBarEl.style.display === 'flex') {
               closeFindBar();
               return;
            }
            if (isReaderOpen) {
               closeReaderMode();
               return;
            }
            if (devDrawerOpen) {
               toggleDevDrawer();
               return;
            }
            sendIpc({ type: 'HUD_ACTION', action: 'dismiss_overlays' });
            return;
         }

         const editable = isEditableTarget(e.target);
         const hasMod = e.ctrlKey || e.metaKey || e.altKey;
         if (editable && !hasMod && e.key !== 'F5' && e.key !== 'F11' && e.key !== 'F12') {
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
            sendIpc({ type: 'HUD_ACTION', action: 'open_omnibox' });
            return;
         }
         if (
            (e.ctrlKey || e.metaKey) &&
            !e.altKey &&
            ((!e.shiftKey && (e.key || '').toLowerCase() === 'k') ||
               (e.shiftKey && (e.key || '').toLowerCase() === 'p'))
         ) {
            e.preventDefault();
            e.stopPropagation();
            sendIpc({ type: 'HUD_ACTION', action: 'open_command_palette' });
            return;
         }
         if (!e.ctrlKey && !e.metaKey && !e.altKey && !e.shiftKey && e.key === 'F5') {
            e.preventDefault();
            e.stopPropagation();
            sendIpc({ type: 'RELOAD' });
            return;
         }
         if (matchesKeybind(e, keybinds.reopen_closed_tab)) {
            e.preventDefault();
            e.stopPropagation();
            sendIpc({ type: 'REOPEN_CLOSED_TAB' });
            return;
         }
         if (matchesKeybind(e, keybinds.new_incognito_window || 'Ctrl+Shift+N')) {
            e.preventDefault();
            e.stopPropagation();
            sendIpc({ type: 'NEW_WINDOW', incognito: true });
            return;
         }
         if (matchesKeybind(e, keybinds.new_window || 'Ctrl+N')) {
            e.preventDefault();
            e.stopPropagation();
            sendIpc({ type: 'NEW_WINDOW', incognito: false });
            return;
         }
         if (matchesKeybind(e, keybinds.new_tab)) {
            e.preventDefault();
            e.stopPropagation();
            flushScrollSync();
            sendIpc({ type: 'NEW_TAB' });
            return;
         }
         if (matchesKeybind(e, keybinds.close_tab)) {
            e.preventDefault();
            e.stopPropagation();
            if (tabState.active_id) sendIpc({ type: 'CLOSE_TAB', id: tabState.active_id });
            return;
         }
         const cycleRelativeTab = (step) => {
            if (Array.isArray(tabState.tabs) && tabState.tabs.length > 1) {
               const curIdx = tabState.tabs.findIndex((t) => t.id === tabState.active_id);
               const nextIdx = (curIdx + step + tabState.tabs.length) % tabState.tabs.length;
               flushScrollSync();
               sendIpc({ type: 'SWITCH_TAB', id: tabState.tabs[nextIdx].id });
            }
         };
         if (matchesKeybind(e, keybinds.prev_tab)) {
            e.preventDefault();
            e.stopPropagation();
            cycleRelativeTab(-1);
            return;
         }
         if (
            matchesKeybind(e, keybinds.next_tab) ||
            (String(keybinds.next_tab || '').toLowerCase() === 'ctrl+tab' && e.ctrlKey && e.shiftKey && e.key === 'Tab')
         ) {
            e.preventDefault();
            e.stopPropagation();
            cycleRelativeTab(e.shiftKey ? -1 : 1);
            return;
         }
         if (
            ((e.ctrlKey || e.metaKey) && e.shiftKey && !e.altKey && (e.key || '').toLowerCase() === 'r') ||
            ((e.ctrlKey || e.metaKey) && !e.shiftKey && !e.altKey && e.key === 'F5')
         ) {
            e.preventDefault();
            e.stopPropagation();
            performHardCacheBustReload();
            return;
         }
         if (matchesKeybind(e, keybinds.reload)) {
            e.preventDefault();
            e.stopPropagation();
            sendIpc({ type: 'RELOAD' });
            return;
         }
         if (matchesKeybind(e, keybinds.go_back)) {
            e.preventDefault();
            e.stopPropagation();
            requestHistoryNav('back');
            return;
         }
         if (matchesKeybind(e, keybinds.go_forward)) {
            e.preventDefault();
            e.stopPropagation();
            requestHistoryNav('forward');
            return;
         }
         if (matchesKeybind(e, keybinds.toggle_side_panel)) {
            e.preventDefault();
            e.stopPropagation();
            sendIpc({ type: 'HUD_ACTION', action: 'toggle_side_panel' });
            return;
         }
         if (matchesKeybind(e, keybinds.pin_tab)) {
            e.preventDefault();
            e.stopPropagation();
            if (tabState.active_id) sendIpc({ type: 'TOGGLE_PIN_TAB', id: tabState.active_id });
            return;
         }
         if (matchesKeybind(e, keybinds.mute_tab)) {
            e.preventDefault();
            e.stopPropagation();
            if (tabState.active_id) sendIpc({ type: 'TOGGLE_MUTE_TAB', id: tabState.active_id });
            return;
         }
         if (matchesKeybind(e, keybinds.bookmark_page)) {
            e.preventDefault();
            e.stopPropagation();
            sendIpc({ type: 'HUD_ACTION', action: 'toggle_bookmark' });
            return;
         }
         if (matchesKeybind(e, keybinds.find_in_page) && !isInternalSettings()) {
            e.preventDefault();
            e.stopPropagation();
            openFindBar();
            return;
         }
         if (matchesKeybind(e, keybinds.reader_mode) && !isInternalAny()) {
            e.preventDefault();
            e.stopPropagation();
            toggleReaderMode();
            return;
         }
         if (matchesKeybind(e, keybinds.force_dark)) {
            e.preventDefault();
            e.stopPropagation();
            sendIpc({ type: 'HUD_ACTION', action: 'toggle_force_dark' });
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
            flushScrollSync();
            sendIpc({ type: 'HUD_ACTION', action: 'toggle_preferences' });
            return;
         }
         if (matchesKeybind(e, keybinds.devtools)) {
            e.preventDefault();
            e.stopPropagation();
            sendIpc({ type: 'OPEN_DEVTOOLS' });
            return;
         }
         if (matchesKeybind(e, keybinds.dev_drawer)) {
            e.preventDefault();
            e.stopPropagation();
            toggleDevDrawer();
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
         if ((e.ctrlKey || e.metaKey) && !e.altKey && !e.shiftKey && /^[1-9]$/.test(e.key || '')) {
            const len = Array.isArray(tabState.tabs) ? tabState.tabs.length : 0;
            if (len > 0) {
               const idx = e.key === '9' ? len - 1 : Math.min(len - 1, Number(e.key) - 1);
               const targetTab = tabState.tabs[idx];
               if (targetTab && targetTab.id !== tabState.active_id) {
                  e.preventDefault();
                  e.stopPropagation();
                  flushScrollSync();
                  sendIpc({ type: 'SWITCH_TAB', id: targetTab.id });
               }
            }
            return;
         }
         if ((e.ctrlKey || e.metaKey) && !e.altKey && !e.shiftKey && (e.key || '').toLowerCase() === 'p') {
            e.preventDefault();
            e.stopPropagation();
            try { window.print(); } catch (_) {}
            return;
         }
         if ((e.ctrlKey || e.metaKey) && !e.altKey && !e.shiftKey && (e.key || '').toLowerCase() === 'h') {
            e.preventDefault();
            e.stopPropagation();
            sendIpc({ type: 'HUD_ACTION', action: 'open_history' });
            return;
         }
         if ((e.ctrlKey || e.metaKey) && !e.altKey && !e.shiftKey && (e.key || '').toLowerCase() === 'j') {
            e.preventDefault();
            e.stopPropagation();
            sendIpc({ type: 'HUD_ACTION', action: 'toggle_downloads' });
            return;
         }
         if ((e.ctrlKey || e.metaKey) && e.shiftKey && !e.altKey && (e.key || '').toLowerCase() === 'c') {
            const sel = window.getSelection ? String(window.getSelection()) : '';
            if (!sel) {
               e.preventDefault();
               e.stopPropagation();
               sendIpc({ type: 'HUD_ACTION', action: 'copy_url' });
            }
         }
      },
      true,
   );

   // Apply Runtime Payload Helper
   function applyRuntimePayload(rtPayload) {
      if (!rtPayload) return;
      if (rtPayload.config) {
         rawBaseConfig = Object.assign({}, DEFAULT_CONTENT_CONFIG, rawBaseConfig, rtPayload.config);
         recomputeEffectiveConfig();
      }
      if (rtPayload.styles) styles = rtPayload.styles;
      if (rtPayload.active_theme) activeTheme = Object.assign(activeTheme, rtPayload.active_theme);
      if (rtPayload.keybinds) keybinds = Object.assign({}, DEFAULT_KEYBINDS, rtPayload.keybinds);
      if (Array.isArray(rtPayload.enabled_rules)) enabledRules = rtPayload.enabled_rules;
      else if (Array.isArray(rtPayload.site_rules)) {
         enabledRules = rtPayload.site_rules.filter((r) => r && r.enabled);
      }
      if (Array.isArray(rtPayload.adblock_whitelist)) adblockWhitelist = rtPayload.adblock_whitelist;
      syncShadowThemeVars();
      applyPageRules();
      applySiteRules();
      syncExtensions();
      if (isInternalAny() && typeof window.__fynexo_on_settings_runtime__ === 'function') {
         window.__fynexo_on_settings_runtime__(rtPayload);
      }
      reportContentPageMetaRef(false);
   }

   // Rust -> JS Authenticated Bridges
   exposeBridge('__fynexo_boot_sync__', function (rtPayload, tabsPayload) {
      if (tabsPayload) tabState = tabsPayload;
      applyRuntimePayload(rtPayload);
      bootSyncDone = true;
      if (typeof onBootSyncDoneCb === 'function') onBootSyncDoneCb();
   });
   exposeBridge('__fynexo_set_runtime__', function (payload) {
      applyRuntimePayload(payload);
   });
   exposeBridge('__fynexo_set_config__', function (newCfg) {
      if (newCfg) {
         rawBaseConfig = Object.assign({}, DEFAULT_CONTENT_CONFIG, rawBaseConfig, newCfg);
         recomputeEffectiveConfig();
         applyPageRules();
         applySiteRules();
         syncExtensions();
         reportContentPageMetaRef(false);
      }
   });
   exposeBridge('__fynexo_set_tabs__', function (state) {
      if (state) tabState = state;
      reportContentPageMetaRef(false);
   });
   exposeBridge('__fynexo_set_fullscreen__', function (full) {
      isFullscreen = !!full;
   });
   exposeBridge('__fynexo_restore_scroll__', function (targetY) {
      const y = Number(targetY) || 0;
      if (y <= 24 || isInternalAny()) return;
      let tries = 0;
      const attempt = () => {
         window.scrollTo(0, y);
         if (Math.abs((window.scrollY || 0) - y) > 40 && ++tries < 6) {
            setTimeout(attempt, 180);
         }
      };
      attempt();
   });
   exposeBridge('__fnx_set_tab_muted__', applyTabMuted);
   exposeBridge('__fnx_set_background__', function (isBg) {
      isBackgroundParked = !!isBg;
      if (!isBackgroundParked) reportContentPageMetaRef(true);
   });
   exposeBridge('__fynexo_request_reveal_frame__', function () {
      const sendReady = () => sendIpc({ type: 'TAB_PAINT_READY' });
      if (typeof window.requestAnimationFrame === 'function') {
         window.requestAnimationFrame(() => window.requestAnimationFrame(sendReady));
      } else {
         sendReady();
      }
   });
   exposeBridge('__fynexo_on_resource_stats__', function (stats) {
      if (isInternalSettings() && typeof window.__fynexo_on_settings_perf__ === 'function') {
         window.__fynexo_on_settings_perf__(stats);
      }
   });
   exposeBridge('__fynexo_content_action__', function (action, payload) {
      const data = payload || {};
      switch (action) {
         case 'open_find':
            openFindBar();
            break;
         case 'print_page':
            try { window.print(); } catch (_) {}
            break;
         case 'toggle_reader':
            toggleReaderMode();
            break;
         case 'toggle_dev_drawer':
            toggleDevDrawer();
            break;
         case 'start_element_picker':
            startVisualElementPicker();
            break;
         case 'clear_site_storage':
            clearCurrentSiteStorageLocal();
            break;
         case 'hard_reload':
            performHardCacheBustReload();
            break;
         case 'media_play_pause':
            toggleLocalMediaPlayPause();
            reportContentPageMetaRef(true);
            break;
         case 'media_mute':
            mediaIsMuted = typeof data.muted === 'boolean' ? data.muted : !mediaIsMuted;
            applyMediaVolume(mediaGainValue);
            reportContentPageMetaRef(true);
            break;
         case 'media_volume':
            if (typeof data.muted === 'boolean') mediaIsMuted = data.muted;
            applyMediaVolume(data.gain !== undefined ? data.gain : mediaGainValue);
            reportContentPageMetaRef(true);
            break;
         case 'run_sf_command':
            if (Array.isArray(scriptForgeMenuCommands)) {
               const cmd = scriptForgeMenuCommands.find((c) => c && c.id === data.id);
               if (cmd && typeof cmd.onClick === 'function') {
                  try { cmd.onClick(); } catch (err) { console.warn('[ScriptForge Command Error]:', err); }
               }
            }
            break;
         case 'preview_site_boost': {
            const ruleId = String(data.ruleId || '');
            const cleanCss = String(data.css || '').replace(/[<>]/g, '').trim();
            const cleanDom = String(data.domain || '');
            if (!ruleId || !cleanDom) break;
            if (!cleanCss) {
               enabledRules = enabledRules.filter((r) => !r || r.id !== ruleId);
            } else {
               const existing = enabledRules.find((r) => r && r.id === ruleId);
               if (!existing) {
                  enabledRules.push({ id: ruleId, name: 'Site Boost (' + cleanDom + ')', domain: cleanDom, enabled: true, css: cleanCss, js: '' });
               } else {
                  existing.enabled = true;
                  existing.css = cleanCss;
               }
            }
            applySiteRules();
            break;
         }
      }
   });

   // Safe No-Op Chrome Bridges (owned by hud_webview in Dual-WebView mode)
   exposeBridge('__fynexo_toast__', function () {});
   exposeBridge('__fynexo_set_downloads__', function () {});
   exposeBridge('__fynexo_trigger_progress__', function () {});
   exposeBridge('__fynexo_finish_progress__', function () {});
   exposeBridge('__fynexo_set_nav_state__', function () {});
   exposeBridge('__fynexo_hud_action__', function () {});
   exposeBridge('__fynexo_set_page_meta__', function () {});

   // Boot Sync & Paint-Ready Handshake
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
         window.requestAnimationFrame(() => window.requestAnimationFrame(fire));
      } else {
         fire();
      }
   }

   function maybeSchedulePaintReady(forceWithoutBoot) {
      if (paintReadyScheduled || paintReadySent || !domReadyForPaint) return;
      if (bootSyncDone || forceWithoutBoot) schedulePaintReadyFrames();
   }

   onBootSyncDoneCb = () => maybeSchedulePaintReady(false);

   function sendBootSyncWhenReady(retries) {
      if (__fnxIpcPost || (window.ipc && typeof window.ipc.postMessage === 'function')) {
         sendIpc({ type: 'BOOT_SYNC' });
      } else if (retries < 30) {
         setTimeout(() => sendBootSyncWhenReady(retries + 1), 40);
      }
   }

   function onDomReady() {
      domReadyForPaint = true;
      applyPageRules();
      applySiteRules();
      syncExtensions();
      cachedReaderable = detectReaderable();
      reportTitleAndUrl(true);
      maybeSchedulePaintReady(false);
      setTimeout(() => maybeSchedulePaintReady(true), 250);

      try {
         const titleEl = document.querySelector('title');
         if (titleEl) {
            new MutationObserver(() => reportTitleAndUrl(false)).observe(titleEl, { childList: true });
         }
      } catch (_) {}
   }

   applyPageRules();
   applySiteRules();
   sendBootSyncWhenReady(0);

   if (!domReadyForPaint) {
      document.addEventListener('DOMContentLoaded', onDomReady, { once: true });
   } else {
      onDomReady();
   }
   window.addEventListener(
      'load',
      () => {
         cachedReaderable = detectReaderable();
         applyPageRules();
         reportTitleAndUrl(false);
         reportContentPageMetaRef(true);
      },
      { once: true },
   );
})();
