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
      clean_urls: true,
      selection_popup: true,
      active_mode: 'auto',
      mode_profiles: [],
      site_overrides: {},
      homepage: 'fynexo://newtab',
   };
   let rawBaseConfig = Object.assign({}, DEFAULT_CONTENT_CONFIG, bootPayload.config || {});
   let config = Object.assign({}, rawBaseConfig);

   // Early document_start YouTube Ad Shield for Windows Dual-WebView content tabs
   (() => {
      let host = '';
      try {
         host = String((window.location && window.location.hostname) || '').toLowerCase();
      } catch (_) {}
      if (host !== 'youtube.com' && !host.endsWith('.youtube.com')) return;
      const AD_KEYS = new Set(['adPlacements', 'playerAds', 'adSlots', 'adBreakHeartbeatParams']);
      const AD_RENDERERS = new Set([
         'adSlotRenderer',
         'promotedSparklesWebRenderer',
         'promotedSparklesTextSearchRenderer',
         'promotedVideoRenderer',
         'inFeedAdLayoutRenderer',
         'statementBannerRenderer',
         'mealbarPromoRenderer',
         'videoDisplayFullButtonedRenderer',
      ]);
      const shouldStrip = () => config.adblock !== false || !!config.ext_yt_ad_skipper;
      function pruneYtObj(node, depth) {
         if (!node || typeof node !== 'object' || depth > 14) return node;
         if (Array.isArray(node)) {
            let w = 0;
            for (let i = 0; i < node.length; i++) {
               const it = node[i];
               if (it && typeof it === 'object') {
                  let drop = false;
                  for (const rk of AD_RENDERERS) {
                     if (Object.prototype.hasOwnProperty.call(it, rk)) { drop = true; break; }
                  }
                  if (drop) continue;
                  pruneYtObj(it, depth + 1);
               }
               node[w++] = it;
            }
            node.length = w;
            return node;
         }
         for (const ak of AD_KEYS) {
            if (Object.prototype.hasOwnProperty.call(node, ak)) {
               node[ak] = Array.isArray(node[ak]) ? [] : undefined;
            }
         }
         for (const rk of AD_RENDERERS) {
            if (Object.prototype.hasOwnProperty.call(node, rk)) delete node[rk];
         }
         const keys = Object.keys(node);
         for (let i = 0; i < keys.length; i++) {
            const k = keys[i];
            if (k === 'streamingData' || k === 'videoDetails' || k === 'captions' || k === 'storyboards') continue;
            const v = node[k];
            if (v && typeof v === 'object') pruneYtObj(v, depth + 1);
         }
         return node;
      }
      ['ytInitialPlayerResponse', 'ytInitialData'].forEach((prop) => {
         try {
            let backing = window[prop];
            if (backing && typeof backing === 'object' && shouldStrip()) pruneYtObj(backing, 0);
            Object.defineProperty(window, prop, {
               configurable: true,
               enumerable: true,
               get() { return backing; },
               set(val) {
                  if (val && typeof val === 'object' && shouldStrip()) {
                     try { pruneYtObj(val, 0); } catch (_) {}
                  }
                  backing = val;
               },
            });
         } catch (_) {}
      });
      if (typeof window.fetch === 'function') {
         const origFetch = window.fetch.bind(window);
         window.fetch = function (input, init) {
            const urlStr = typeof input === 'string' ? input : (input && input.url) || '';
            const isYtApi = shouldStrip() && /\/youtubei\/v1\/(?:player|next|browse|search)(?:[?#]|$)/i.test(urlStr);
            const p = origFetch(input, init);
            if (!isYtApi) return p;
            return p.then((res) => {
               try {
                  if (!res || !res.ok) return res;
                  const ct = (res.headers && res.headers.get && res.headers.get('content-type')) || '';
                  if (!ct.toLowerCase().includes('json')) return res;
                  return res.clone().text().then((txt) => {
                     try {
                        const data = JSON.parse(txt);
                        pruneYtObj(data, 0);
                        const headers = new Headers(res.headers);
                        headers.delete('content-length');
                        return new Response(JSON.stringify(data), { status: res.status, statusText: res.statusText, headers });
                     } catch (_) { return res; }
                  }).catch(() => res);
               } catch (_) { return res; }
            });
         };
      }
   })();
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
         if (/^https?:\/\/fynexo\.localhost(?::\d+)?\/history/i.test(href)) {
            return 'fynexo://history';
         }
         if (/^https?:\/\/fynexo\.localhost(?::\d+)?\/bookmarks/i.test(href)) {
            return 'fynexo://bookmarks';
         }
         if (/^https?:\/\/fynexo\.localhost(?::\d+)?\/downloads/i.test(href)) {
            return 'fynexo://downloads';
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
            'html:not([data-fynexo-internal]){background-color:#111113!important;}html:not([data-fynexo-internal])>body{filter:invert(0.92) hue-rotate(180deg)!important;background-color:#ffffff;min-height:100dvh!important;}html:not([data-fynexo-internal])>body img,html:not([data-fynexo-internal])>body video,html:not([data-fynexo-internal])>body picture,html:not([data-fynexo-internal])>body canvas,html:not([data-fynexo-internal])>body iframe,html:not([data-fynexo-internal])>body svg:not(:root),html:not([data-fynexo-internal])>body [style*="background-image"]{filter:invert(1) hue-rotate(180deg)!important;}',
         );
      }
      if (config.adblock !== false && !isLocalDev && !isCurrentHostWhitelisted()) {
         chunks.push(
            'ins.adsbygoogle,[id^="div-gpt-ad"],[id^="google_ads_iframe"],iframe[src*="doubleclick.net"],iframe[src*="googlesyndication.com"],.taboola-placeholder,.OUTBRAIN,ytd-ad-slot-renderer,ytd-in-feed-ad-layout-renderer,ytd-banner-promo-renderer,ytd-statement-banner-renderer,#player-ads,#masthead-ad,[data-ad-slot]{display:none!important;}',
         );
      }
      const cleanFont = String(config.page_font_family || '').replace(/[<>{};]/g, '').trim();
      if (cleanFont) {
         chunks.push(
            'body,p,article,main,section,li,td,th,dd,dt,blockquote,h1,h2,h3,h4,h5,h6{font-family:' +
               cleanFont +
               '!important;}',
         );
      }
      const minFs = Number(config.min_font_size) || 0;
      if (minFs >= 10 && minFs <= 24) {
         chunks.push(
            'html{font-size:max(100%,' + minFs + 'px)!important;}body,p,li,td,th,dd,dt,blockquote{font-size:max(1em,' + minFs + 'px);}',
         );
      }
      if (siteOv && siteOv.super_copy) {
         chunks.push(
            '*,*::before,*::after{-webkit-user-select:text!important;user-select:text!important;-webkit-touch-callout:default!important;}',
         );
      }
      const next = chunks.join('\n');
      if (pageRulesEl.textContent !== next) pageRulesEl.textContent = next;
   }

   // SuperCopy capture-phase event unlocker (per-site opt-in via Site Info Popover)
   if (!isInternalAny()) {
      ['copy', 'cut', 'selectstart', 'contextmenu', 'dragstart'].forEach((evName) => {
         window.addEventListener(
            evName,
            (e) => {
               const ov = getCurrentSiteOverride();
               if (!ov || !ov.super_copy) return;
               e.stopImmediatePropagation();
            },
            true,
         );
      });
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

   // Media Detection, Volume Boost & Universal Playback Speed (0.25x - 4.0x)
   let currentMediaEl = null;
   let mediaGainValue = 100;
   let mediaIsMuted = false;
   let mediaPlaybackRate = 1.0;
   let audioCtx = null;

   function applyMediaRate(rate) {
      const r = Math.max(0.25, Math.min(4.0, Math.round((Number(rate) || 1.0) * 100) / 100));
      mediaPlaybackRate = r;
      try {
         document.querySelectorAll('video, audio').forEach((m) => {
            if (Math.abs((m.playbackRate || 1) - r) > 0.005) m.playbackRate = r;
         });
      } catch (_) {}
   }

   function syncSiteMediaRate() {
      const ov = getCurrentSiteOverride();
      if (ov && typeof ov.media_rate === 'number' && ov.media_rate >= 0.25 && ov.media_rate <= 4.0) {
         applyMediaRate(ov.media_rate);
      } else if (mediaPlaybackRate !== 1.0) {
         applyMediaRate(mediaPlaybackRate);
      }
   }

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
         media_rate: Number(mediaPlaybackRate) || 1.0,
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
      if (mediaPlaybackRate !== 1.0) applyMediaRate(mediaPlaybackRate);
      if (playing !== lastMediaPlaying) {
         lastMediaPlaying = playing;
         sendIpc({ type: 'UPDATE_TAB_MEDIA_STATE', playing });
      }
      reportContentPageMetaRef(false);
   }
   document.addEventListener('play', checkMediaPlaybackState, true);
   document.addEventListener('loadedmetadata', () => {
      if (mediaPlaybackRate !== 1.0) applyMediaRate(mediaPlaybackRate);
   }, true);
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
            if (selPopEl && selPopEl.style.display === 'flex') {
               hideSelectionBubble();
               return;
            }
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
         if ((e.ctrlKey || e.metaKey) && e.shiftKey && !e.altKey && (e.key || '').toLowerCase() === 'a') {
            e.preventDefault();
            e.stopPropagation();
            sendIpc({ type: 'HUD_ACTION', action: 'open_tab_search' });
            return;
         }
         if ((e.ctrlKey || e.metaKey) && !e.altKey && !e.shiftKey && (e.key || '').toLowerCase() === 's') {
            if (!isInternalAny()) {
               e.preventDefault();
               e.stopPropagation();
               sendIpc({ type: 'SAVE_PAGE_PDF' });
               return;
            }
         }
         if ((e.ctrlKey || e.metaKey) && !e.altKey && !e.shiftKey && (e.key || '').toLowerCase() === 'u') {
            const cur = canonicalCurrentUrl();
            if (/^https?:\/\//i.test(cur)) {
               e.preventDefault();
               e.stopPropagation();
               sendIpc({ type: 'NEW_TAB', url: 'view-source:' + cur });
               return;
            }
         }
         if ((e.ctrlKey || e.metaKey) && e.shiftKey && !e.altKey && (e.key === 'Delete' || e.key === 'Backspace')) {
            e.preventDefault();
            e.stopPropagation();
            sendIpc({ type: 'HUD_ACTION', action: 'open_clear_data' });
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
      syncSiteMediaRate();
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
   exposeBridge('__fynexo_on_search_suggestions__', function (payload) {
      if (isInternalAny() && typeof window.__fynexo_on_newtab_suggestions__ === 'function') {
         window.__fynexo_on_newtab_suggestions__(payload);
      }
   });
   // In-Place Page Translator (CORS-free via Rust WinInet backend, bilingual or replace)
   const trOriginals = new Map();
   let trActive = false;
   let trMode = 'bilingual';
   let trTargetLang = '';
   let trReqSeq = 1;
   let trActiveReqIds = new Set();
   let trPendingBatches = 0;
   let trBarEl = null;
   let trStatusEl = null;
   let trModeBtn = null;
   let trLangSel = null;

   function ensureTranslateStyle() {
      if (document.getElementById('__fynexo_tr_css__')) return;
      const st = document.createElement('style');
      st.id = '__fynexo_tr_css__';
      st.textContent =
         '.fnx-tr-bi{display:block;margin-top:3px;padding-left:8px;border-left:2px solid #7c86ff;opacity:0.9;font-size:0.94em;line-height:1.5;}';
      (document.head || document.documentElement).appendChild(st);
   }

   function restoreTranslatedPage(silent) {
      trActive = false;
      trActiveReqIds.clear();
      trPendingBatches = 0;
      trOriginals.forEach((rec, el) => {
         try {
            if (!el || !el.isConnected) return;
            if (rec.biEl && rec.biEl.parentNode) rec.biEl.remove();
            if (rec.replaced) el.textContent = rec.origText;
            el.removeAttribute('data-fnx-tr-id');
         } catch (_) {}
      });
      trOriginals.clear();
      if (trBarEl) trBarEl.style.display = 'none';
      if (!silent) showHudToast(isZh() ? '已还原网页原文' : 'Restored original page');
   }

   function ensureTranslateBar() {
      const sr = ensureShadowHost();
      if (!sr || !shellEl) return null;
      if (trBarEl && trBarEl.isConnected) return trBarEl;
      const bar = document.createElement('div');
      bar.id = '__fynexo_tr_bar__';
      bar.style.cssText =
         'position:fixed;top:12px;right:14px;z-index:2147483645;display:none;align-items:center;gap:6px;padding:6px 10px;border-radius:10px;background:var(--fnx-panel);border:1px solid var(--fnx-border-hover);box-shadow:0 10px 28px rgba(0,0,0,0.36);color:var(--fnx-fg);font-size:12px;font-family:system-ui,sans-serif;pointer-events:auto;';
      const status = document.createElement('span');
      status.style.cssText = 'font-weight:600;white-space:nowrap;';
      const sel = document.createElement('select');
      sel.style.cssText =
         'height:24px;padding:0 6px;border-radius:6px;border:1px solid var(--fnx-border);background:var(--fnx-surface);color:var(--fnx-fg);font-size:11.5px;cursor:pointer;';
      [
         ['zh-CN', '简体中文'],
         ['zh-TW', '繁體中文'],
         ['en', 'English'],
         ['ja', '日本語'],
         ['ko', '한국어'],
      ].forEach(([v, lbl]) => {
         const o = document.createElement('option');
         o.value = v;
         o.textContent = lbl;
         sel.appendChild(o);
      });
      sel.onchange = () => {
         trTargetLang = sel.value;
         startInPlaceTranslate(trTargetLang);
      };
      const btnStyle =
         'height:24px;padding:0 8px;border-radius:6px;border:1px solid var(--fnx-border);background:var(--fnx-surface);color:var(--fnx-fg);font-size:11.5px;cursor:pointer;';
      const modeBtn = document.createElement('button');
      modeBtn.type = 'button';
      modeBtn.style.cssText = btnStyle;
      modeBtn.onclick = () => {
         trMode = trMode === 'bilingual' ? 'replace' : 'bilingual';
         applyTranslatedMode();
      };
      const closeBtn = document.createElement('button');
      closeBtn.type = 'button';
      closeBtn.style.cssText = btnStyle;
      closeBtn.textContent = isZh() ? '还原' : 'Restore';
      closeBtn.onclick = () => restoreTranslatedPage(false);
      bar.appendChild(status);
      bar.appendChild(sel);
      bar.appendChild(modeBtn);
      bar.appendChild(closeBtn);
      shellEl.appendChild(bar);
      trBarEl = bar;
      trStatusEl = status;
      trModeBtn = modeBtn;
      trLangSel = sel;
      return bar;
   }

   function applyTranslatedMode() {
      const zh = isZh();
      if (trModeBtn) {
         trModeBtn.textContent = trMode === 'bilingual' ? (zh ? '双语' : 'Bilingual') : (zh ? '仅译文' : 'Replace');
      }
      trOriginals.forEach((rec, el) => {
         if (!el || !el.isConnected || !rec.translated) return;
         if (trMode === 'replace') {
            if (rec.biEl && rec.biEl.parentNode) rec.biEl.remove();
            rec.biEl = null;
            el.textContent = rec.translated;
            rec.replaced = true;
         } else {
            if (rec.replaced) {
               el.textContent = rec.origText;
               rec.replaced = false;
            }
            if (!rec.biEl || !rec.biEl.isConnected) {
               const sp = document.createElement('span');
               sp.className = 'fnx-tr-bi';
               sp.textContent = rec.translated;
               el.appendChild(sp);
               rec.biEl = sp;
            } else {
               rec.biEl.textContent = rec.translated;
            }
         }
      });
   }

   function collectTranslatableBlocks() {
      const skip = /^(SCRIPT|STYLE|NOSCRIPT|CODE|PRE|TEXTAREA|INPUT|SELECT|OPTION|SVG|MATH|KBD|SAMP)$/;
      const nodes = document.querySelectorAll('p,h1,h2,h3,h4,h5,h6,li,td,th,blockquote,figcaption,dt,dd');
      const out = [];
      for (let i = 0; i < nodes.length && out.length < 220; i++) {
         const el = nodes[i];
         if (!el || el.closest('#__fynexo_host__,code,pre,nav,footer,script,style')) continue;
         if (el.querySelector('p,h1,h2,h3,h4,h5,h6,blockquote,ul,ol,table')) continue;
         let bad = false;
         let p = el;
         while (p && p !== document.body) {
            if (skip.test(p.tagName || '')) { bad = true; break; }
            p = p.parentElement;
         }
         if (bad) continue;
         const txt = String(el.textContent || '').replace(/\s+/g, ' ').trim();
         if (txt.length < 3 || txt.length > 1600) continue;
         if (/^[\d\s.,:%$€£¥+\-*/=<>()[\]{}|\\/_#@!?~`^&]+$/.test(txt)) continue;
         out.push([el, txt]);
      }
      return out;
   }

   function startInPlaceTranslate(targetLang) {
      if (isInternalAny()) return;
      restoreTranslatedPage(true);
      ensureTranslateStyle();
      const lang = String(targetLang || (isZh() ? 'zh-CN' : 'en'));
      trTargetLang = lang;
      trActive = true;
      const bar = ensureTranslateBar();
      if (bar) {
         bar.style.display = 'inline-flex';
         if (trLangSel) trLangSel.value = lang;
         if (trStatusEl) trStatusEl.textContent = isZh() ? '正在翻译…' : 'Translating...';
         if (trModeBtn) trModeBtn.textContent = trMode === 'bilingual' ? (isZh() ? '双语' : 'Bilingual') : (isZh() ? '仅译文' : 'Replace');
      }
      const blocks = collectTranslatableBlocks();
      if (blocks.length === 0) {
         if (trStatusEl) trStatusEl.textContent = isZh() ? '无可翻译正文' : 'No text found';
         return;
      }
      const idMap = [];
      for (let i = 0; i < blocks.length; i++) {
         const id = i + 1;
         const [el, txt] = blocks[i];
         el.setAttribute('data-fnx-tr-id', String(id));
         trOriginals.set(el, { id, origText: txt, translated: '', biEl: null, replaced: false });
         idMap.push({ id, text: txt });
      }
      const chunkSize = 24;
      trPendingBatches = Math.ceil(idMap.length / chunkSize);
      for (let i = 0; i < idMap.length; i += chunkSize) {
         const rid = trReqSeq++;
         trActiveReqIds.add(rid);
         sendIpc({
            type: 'TRANSLATE_BATCH',
            req_id: rid,
            target_lang: lang,
            items: idMap.slice(i, i + chunkSize),
         });
      }
   }

   // Selection Floating Bubble (Translate / Search / Copy)
   let selPopEl = null;
   let selResEl = null;
   let selTextCache = '';
   let selTrReqId = 0;

   function hideSelectionBubble() {
      if (selPopEl) selPopEl.style.display = 'none';
      if (selResEl) {
         selResEl.style.display = 'none';
         selResEl.textContent = '';
      }
      selTrReqId = 0;
   }

   function ensureSelectionBubble() {
      const sr = ensureShadowHost();
      if (!sr || !shellEl) return null;
      if (selPopEl && selPopEl.isConnected) return selPopEl;
      const box = document.createElement('div');
      box.id = '__fynexo_sel_pop__';
      box.style.cssText =
         'position:fixed;z-index:2147483645;display:none;flex-direction:column;gap:6px;max-width:min(340px,calc(100vw - 24px));padding:5px;border-radius:10px;background:var(--fnx-panel);border:1px solid var(--fnx-border-hover);box-shadow:0 12px 30px rgba(0,0,0,0.42);color:var(--fnx-fg);font-family:system-ui,-apple-system,sans-serif;pointer-events:auto;';
      const row = document.createElement('div');
      row.style.cssText = 'display:flex;align-items:center;gap:4px;';
      const btnCss =
         'height:26px;padding:0 9px;border-radius:7px;border:1px solid var(--fnx-border);background:var(--fnx-surface);color:var(--fnx-fg);font-size:12px;font-weight:500;cursor:pointer;white-space:nowrap;';
      const bTr = document.createElement('button');
      bTr.type = 'button';
      bTr.style.cssText = btnCss;
      const bSr = document.createElement('button');
      bSr.type = 'button';
      bSr.style.cssText = btnCss;
      const bCp = document.createElement('button');
      bCp.type = 'button';
      bCp.style.cssText = btnCss;
      const res = document.createElement('div');
      res.style.cssText =
         'display:none;padding:7px 9px;border-radius:7px;background:var(--fnx-surface);border:1px solid var(--fnx-border);font-size:12.5px;line-height:1.5;color:var(--fnx-fg);max-height:180px;overflow-y:auto;word-break:break-word;user-select:text;';

      bTr.onmousedown = (e) => { e.preventDefault(); e.stopPropagation(); };
      bSr.onmousedown = (e) => { e.preventDefault(); e.stopPropagation(); };
      bCp.onmousedown = (e) => { e.preventDefault(); e.stopPropagation(); };

      bTr.onclick = (e) => {
         e.stopPropagation();
         if (!selTextCache) return;
         const hasCjk = /[\u4e00-\u9fff]/.test(selTextCache);
         const targetLang = hasCjk ? 'en' : 'zh-CN';
         const rid = trReqSeq++;
         selTrReqId = rid;
         res.textContent = isZh() ? '翻译中…' : 'Translating...';
         res.style.display = 'block';
         sendIpc({
            type: 'TRANSLATE_BATCH',
            req_id: rid,
            target_lang: targetLang,
            items: [{ id: 1, text: selTextCache }],
         });
      };
      bSr.onclick = (e) => {
         e.stopPropagation();
         if (!selTextCache) return;
         const engines = Array.isArray(config.search_engines) ? config.search_engines : [];
         const activeId = String(config.search_engine || 'google');
         const eng = engines.find((x) => x && x.id === activeId) || engines[0];
         const tpl = (eng && eng.url) || 'https://www.google.com/search?q={query}';
         const q = encodeURIComponent(selTextCache);
         const url = tpl.includes('{query}') ? tpl.replace('{query}', q) : tpl.replace('%s', q);
         sendIpc({ type: 'NEW_TAB', url });
         hideSelectionBubble();
      };
      bCp.onclick = (e) => {
         e.stopPropagation();
         if (!selTextCache) return;
         copyTextToClipboard(selTextCache, isZh() ? '已复制选中文本' : 'Selection copied');
         hideSelectionBubble();
      };

      row.append(bTr, bSr, bCp);
      box.append(row, res);
      shellEl.appendChild(box);
      box._bTr = bTr;
      box._bSr = bSr;
      box._bCp = bCp;
      selPopEl = box;
      selResEl = res;
      return box;
   }

   if (!isInternalAny()) {
      document.addEventListener(
         'mouseup',
         (e) => {
            if (config.selection_popup === false) return;
            if (e && e.target && e.target.closest && e.target.closest('#__fynexo_host__')) return;
            if (isEditableTarget(e && e.target)) {
               hideSelectionBubble();
               return;
            }
            setTimeout(() => {
               try {
                  const sel = window.getSelection ? window.getSelection() : null;
                  const txt = sel ? String(sel.toString() || '').trim() : '';
                  if (!sel || sel.isCollapsed || txt.length < 2 || txt.length > 600) {
                     hideSelectionBubble();
                     return;
                  }
                  const rect = sel.rangeCount > 0 ? sel.getRangeAt(0).getBoundingClientRect() : null;
                  if (!rect || (rect.width === 0 && rect.height === 0)) {
                     hideSelectionBubble();
                     return;
                  }
                  const box = ensureSelectionBubble();
                  if (!box) return;
                  selTextCache = txt;
                  const zh = isZh();
                  box._bTr.textContent = zh ? '翻译' : 'Translate';
                  box._bSr.textContent = zh ? '搜索' : 'Search';
                  box._bCp.textContent = zh ? '复制' : 'Copy';
                  if (selResEl) {
                     selResEl.style.display = 'none';
                     selResEl.textContent = '';
                  }
                  const vw = window.innerWidth || 1024;
                  const left = Math.max(10, Math.min(vw - 210, Math.round(rect.left + rect.width / 2 - 86)));
                  const top = rect.top > 56 ? Math.round(rect.top - 44) : Math.round(rect.bottom + 10);
                  box.style.left = left + 'px';
                  box.style.top = Math.max(8, top) + 'px';
                  box.style.display = 'flex';
               } catch (_) {}
            }, 25);
         },
         true,
      );
      document.addEventListener(
         'mousedown',
         (e) => {
            if (!selPopEl || selPopEl.style.display === 'none') return;
            const path = e && typeof e.composedPath === 'function' ? e.composedPath() : [];
            if (path.includes(selPopEl)) return;
            hideSelectionBubble();
         },
         true,
      );
      window.addEventListener('scroll', hideSelectionBubble, { passive: true });
   }

   // Lightweight Article-to-Markdown Exporter (.md)
   function nodeToMarkdown(node) {
      if (!node) return '';
      if (node.nodeType === 3) return String(node.nodeValue || '').replace(/\s+/g, ' ');
      if (node.nodeType !== 1) return '';
      const tag = String(node.tagName || '').toUpperCase();
      if (/^(SCRIPT|STYLE|NOSCRIPT|SVG|IFRAME|NAV|FOOTER|ASIDE|FORM|BUTTON|INPUT|SELECT)$/.test(tag)) return '';
      if (node.id === '__fynexo_host__') return '';
      const inner = () => Array.from(node.childNodes || []).map(nodeToMarkdown).join('');
      if (/^H[1-6]$/.test(tag)) {
         const t = inner().trim();
         return t ? '\n\n' + '#'.repeat(Number(tag[1])) + ' ' + t + '\n\n' : '';
      }
      if (tag === 'P') {
         const t = inner().trim();
         return t ? '\n\n' + t + '\n\n' : '';
      }
      if (tag === 'BR') return '  \n';
      if (tag === 'HR') return '\n\n---\n\n';
      if (tag === 'STRONG' || tag === 'B') {
         const t = inner().trim();
         return t ? '**' + t + '**' : '';
      }
      if (tag === 'EM' || tag === 'I') {
         const t = inner().trim();
         return t ? '*' + t + '*' : '';
      }
      if (tag === 'CODE') {
         if (node.parentElement && node.parentElement.tagName === 'PRE') return node.textContent || '';
         const t = String(node.textContent || '').trim();
         return t ? '`' + t.replace(/`/g, '\\`') + '`' : '';
      }
      if (tag === 'PRE') {
         const codeEl = node.querySelector('code');
         const cls = String((codeEl && codeEl.className) || '');
         const langMatch = cls.match(/language-([a-z0-9_+-]+)/i);
         const lang = langMatch ? langMatch[1] : '';
         const raw = String((codeEl || node).textContent || '').replace(/\n+$/, '');
         return raw ? '\n\n```' + lang + '\n' + raw + '\n```\n\n' : '';
      }
      if (tag === 'BLOCKQUOTE') {
         const t = inner().trim();
         return t ? '\n\n' + t.split('\n').map((l) => '> ' + l).join('\n') + '\n\n' : '';
      }
      if (tag === 'A') {
         const href = String(node.getAttribute('href') || '').trim();
         const t = inner().trim();
         if (!t) return '';
         if (/^https?:\/\//i.test(href)) return '[' + t + '](' + href + ')';
         return t;
      }
      if (tag === 'IMG') {
         const src = String(node.currentSrc || node.getAttribute('src') || '').trim();
         const alt = String(node.getAttribute('alt') || '').trim();
         return /^https?:\/\//i.test(src) ? '![' + alt + '](' + src + ')' : '';
      }
      if (tag === 'UL' || tag === 'OL') {
         const items = Array.from(node.children || []).filter((c) => c.tagName === 'LI');
         if (!items.length) return '';
         return (
            '\n\n' +
            items
               .map((li, idx) => (tag === 'OL' ? idx + 1 + '. ' : '- ') + nodeToMarkdown(li).trim().replace(/\n+/g, ' '))
               .join('\n') +
            '\n\n'
         );
      }
      if (tag === 'LI') return inner();
      if (tag === 'TABLE') {
         const rows = Array.from(node.querySelectorAll('tr')).slice(0, 60);
         if (!rows.length) return '';
         const lines = rows.map((tr) =>
            '| ' +
            Array.from(tr.children || [])
               .map((c) => String(c.textContent || '').replace(/\s+/g, ' ').replace(/\|/g, '\\|').trim())
               .join(' | ') +
            ' |',
         );
         if (lines.length >= 1) {
            const cols = Math.max(1, (rows[0].children || []).length);
            lines.splice(1, 0, '| ' + Array(cols).fill('---').join(' | ') + ' |');
         }
         return '\n\n' + lines.join('\n') + '\n\n';
      }
      return inner();
   }

   function exportCurrentPageAsMarkdown() {
      if (isInternalAny()) return;
      const root = document.querySelector('article, main, [role="main"]') || document.body;
      const title = String((document.querySelector('h1') && document.querySelector('h1').textContent) || document.title || 'Article').trim();
      const url = canonicalCurrentUrl();
      const bodyMd = nodeToMarkdown(root)
         .replace(/\n{3,}/g, '\n\n')
         .trim();
      if (!bodyMd) {
         showHudToast(isZh() ? '当前页面无可导出的文章正文' : 'No article content found');
         return;
      }
      const header = '# ' + title + '\n\n> Source: ' + url + '\n\n';
      sendIpc({
         type: 'SAVE_MARKDOWN_FILE',
         title,
         url,
         markdown: header + bodyMd + '\n',
      });
   }

   function clampSelectionBubbleInViewport() {
      if (!selPopEl || selPopEl.style.display === 'none') return;
      requestAnimationFrame(() => {
         try {
            const r = selPopEl.getBoundingClientRect();
            const vw = window.innerWidth || 1024;
            const vh = window.innerHeight || 720;
            if (r.right > vw - 10) {
               selPopEl.style.left = Math.max(8, vw - r.width - 10) + 'px';
            }
            if (r.bottom > vh - 10) {
               selPopEl.style.top = Math.max(8, vh - r.height - 10) + 'px';
            }
         } catch (_) {}
      });
   }

   function renderSelectionTranslationResult(text, isErr) {
      if (!selResEl) return;
      selResEl.textContent = '';
      const txtEl = document.createElement('div');
      txtEl.textContent = text;
      selResEl.appendChild(txtEl);
      if (!isErr && text) {
         const cpBtn = document.createElement('button');
         cpBtn.type = 'button';
         cpBtn.style.cssText =
            'margin-top:6px;height:22px;padding:0 8px;border-radius:6px;border:1px solid var(--fnx-border);background:var(--fnx-panel);color:var(--fnx-secondary);font-size:11px;cursor:pointer;';
         cpBtn.textContent = isZh() ? '复制译文' : 'Copy translation';
         cpBtn.onmousedown = (e) => { e.preventDefault(); e.stopPropagation(); };
         cpBtn.onclick = (e) => {
            e.stopPropagation();
            copyTextToClipboard(text, isZh() ? '已复制译文' : 'Translation copied');
            hideSelectionBubble();
         };
         selResEl.appendChild(cpBtn);
      }
      clampSelectionBubbleInViewport();
   }

   function handleTranslateResult(payload) {
      if (!payload) return;
      if (selTrReqId && Number(payload.req_id) === selTrReqId) {
         selTrReqId = 0;
         if (!selResEl) return;
         if (payload.error) {
            renderSelectionTranslationResult((isZh() ? '翻译失败：' : 'Error: ') + String(payload.error), true);
            return;
         }
         const it = Array.isArray(payload.items) && payload.items[0];
         renderSelectionTranslationResult(
            (it && it.text) ? String(it.text) : (isZh() ? '无译文' : 'No translation'),
            !(it && it.text),
         );
         return;
      }
      if (!trActive || !trActiveReqIds.has(Number(payload.req_id))) return;
      trActiveReqIds.delete(Number(payload.req_id));
      trPendingBatches = Math.max(0, trPendingBatches - 1);
      if (payload.error) {
         if (trStatusEl) trStatusEl.textContent = isZh() ? '翻译失败' : 'Translate failed';
         showHudToast(String(payload.error));
         return;
      }
      const items = Array.isArray(payload.items) ? payload.items : [];
      for (let i = 0; i < items.length; i++) {
         const it = items[i];
         if (!it || !it.id || !it.text) continue;
         const el = document.querySelector('[data-fnx-tr-id="' + String(it.id) + '"]');
         if (!el) continue;
         const rec = trOriginals.get(el);
         if (!rec) continue;
         if (String(it.text).trim().toLowerCase() === rec.origText.trim().toLowerCase()) continue;
         rec.translated = String(it.text);
      }
      applyTranslatedMode();
      if (trStatusEl) {
         trStatusEl.textContent =
            trPendingBatches > 0
               ? (isZh() ? '翻译中…' : 'Translating...')
               : (isZh() ? '已翻译' : 'Translated');
      }
   }

   // Chrome-style Link Hover Status Bar (bottom-left URL preview, dodges cursor to bottom-right)
   let statusBarEl = null;
   let statusHideTimer = null;
   let activeHoverLink = null;
   function ensureStatusBar() {
      const sr = ensureShadowHost();
      if (!sr || !shellEl) return null;
      if (statusBarEl && statusBarEl.isConnected) return statusBarEl;
      const el = document.createElement('div');
      el.id = '__fynexo_status_bar__';
      el.style.cssText =
         'position:fixed;left:0;bottom:0;z-index:2147483642;display:none;max-width:min(480px,72vw);padding:3px 10px;border-top-right-radius:7px;border-top:1px solid var(--fnx-border);border-right:1px solid var(--fnx-border);background:var(--fnx-panel);color:var(--fnx-secondary);font-size:11.5px;font-family:ui-monospace,SFMono-Regular,Consolas,monospace;white-space:nowrap;overflow:hidden;text-overflow:ellipsis;pointer-events:none;box-shadow:0 -2px 10px rgba(0,0,0,0.22);';
      shellEl.appendChild(el);
      statusBarEl = el;
      return el;
   }
   function showLinkStatus(href, clientX, clientY) {
      if (statusHideTimer) { clearTimeout(statusHideTimer); statusHideTimer = null; }
      const bar = ensureStatusBar();
      if (!bar) return;
      const dodgeRight = clientX < 500 && clientY > (window.innerHeight || 700) - 90;
      bar.style.left = dodgeRight ? 'auto' : '0';
      bar.style.right = dodgeRight ? '0' : 'auto';
      bar.style.borderTopRightRadius = dodgeRight ? '0' : '7px';
      bar.style.borderTopLeftRadius = dodgeRight ? '7px' : '0';
      bar.textContent = href.length > 180 ? href.slice(0, 177) + '...' : href;
      bar.style.display = 'block';
   }
   function hideLinkStatus() {
      activeHoverLink = null;
      if (!statusBarEl) return;
      if (statusHideTimer) clearTimeout(statusHideTimer);
      statusHideTimer = setTimeout(() => {
         statusHideTimer = null;
         if (statusBarEl) statusBarEl.style.display = 'none';
      }, 90);
   }
   if (!isInternalAny()) {
      document.addEventListener('mouseover', (e) => {
         const t = e && e.target && e.target.closest ? e.target.closest('a[href]') : null;
         if (!t || !t.href || t.closest('#__fynexo_host__')) {
            if (activeHoverLink) hideLinkStatus();
            return;
         }
         activeHoverLink = t;
         showLinkStatus(String(t.href), e.clientX || 0, e.clientY || 0);
      }, true);
      document.addEventListener('mouseout', (e) => {
         if (!activeHoverLink) return;
         const rel = e && e.relatedTarget && e.relatedTarget.closest ? e.relatedTarget.closest('a[href]') : null;
         if (!rel) hideLinkStatus();
      }, true);
      window.addEventListener('blur', hideLinkStatus, { passive: true });
   }

   // Video Picture-in-Picture helper
   function toggleLocalPictureInPicture() {
      const zh = isZh();
      try {
         if (document.pictureInPictureElement) {
            document.exitPictureInPicture().catch(() => {});
            showHudToast(zh ? '已退出画中画' : 'Exited Picture-in-Picture');
            return;
         }
         const videos = Array.from(document.querySelectorAll('video')).filter((v) => v && v.readyState > 0);
         const target =
            (currentMediaEl && currentMediaEl.tagName === 'VIDEO' ? currentMediaEl : null) ||
            videos.find((v) => !v.paused && !v.ended) ||
            videos[0];
         if (!target || typeof target.requestPictureInPicture !== 'function') {
            showHudToast(zh ? '当前页面未检测到可画中画的视频' : 'No playable video found for Picture-in-Picture');
            return;
         }
         target.removeAttribute('disablepictureinpicture');
         target.disablePictureInPicture = false;
         target.requestPictureInPicture()
            .then(() => showHudToast(zh ? '已开启视频画中画' : 'Picture-in-Picture active'))
            .catch(() => showHudToast(zh ? '该视频暂不支持画中画' : 'Picture-in-Picture unavailable for this video'));
      } catch (_) {
         showHudToast(zh ? '该视频暂不支持画中画' : 'Picture-in-Picture unavailable');
      }
   }

   exposeBridge('__fynexo_on_translate_result__', handleTranslateResult);

   exposeBridge('__fynexo_content_action__', function (action, payload) {
      const data = payload || {};
      switch (action) {
         case 'toggle_translate':
            if (trActive && !data.force) restoreTranslatedPage(false);
            else startInPlaceTranslate(data.target_lang || (isZh() ? 'zh-CN' : 'en'));
            break;
         case 'toggle_pip':
            toggleLocalPictureInPicture();
            break;
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
         case 'media_rate':
            applyMediaRate(data.rate !== undefined ? data.rate : 1.0);
            reportContentPageMetaRef(true);
            break;
         case 'export_markdown':
            exportCurrentPageAsMarkdown();
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
      syncSiteMediaRate();
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
