(function () {
   // 1. Iframe & Blank-Park Isolation: Only top-level real pages host Fynexo Chrome
   if (window.top !== window.self) return;
   if (window.location && window.location.href === 'about:blank') return;

   // Capture pristine built-ins & IPC reference at document_start before any page script runs.
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
   const __fnxStringSlice = String.prototype.slice;
   const __fnxArraySlice = Array.prototype.slice;
   let __fnxIpcPost =
      window.ipc && typeof window.ipc.postMessage === 'function'
         ? window.ipc.postMessage.bind(window.ipc)
         : null;
   const __fnxAndroidGetPageBlocked =
      window.__fynexo_android__ &&
      typeof window.__fynexo_android__.getPageBlockedAds === 'function'
         ? window.__fynexo_android__.getPageBlockedAds.bind(window.__fynexo_android__)
         : null;
   const __fnxAndroidPrintPage =
      window.__fynexo_android__ &&
      typeof window.__fynexo_android__.printPage === 'function'
         ? window.__fynexo_android__.printPage.bind(window.__fynexo_android__)
         : null;
   const __fnxAndroidSetZoom =
      window.__fynexo_android__ &&
      typeof window.__fynexo_android__.setZoomPercent === 'function'
         ? window.__fynexo_android__.setZoomPercent.bind(window.__fynexo_android__)
         : null;
   const __fnxAndroidOpenDownloads =
      window.__fynexo_android__ &&
      typeof window.__fynexo_android__.openDownloads === 'function'
         ? window.__fynexo_android__.openDownloads.bind(window.__fynexo_android__)
         : null;
   const __fnxAndroidSetUaMode =
      window.__fynexo_android__ &&
      typeof window.__fynexo_android__.setUserAgentMode === 'function'
         ? window.__fynexo_android__.setUserAgentMode.bind(window.__fynexo_android__)
         : null;

   function applyAndroidZoomNow(zoomFactor) {
      if (!__fnxAndroidSetZoom) return;
      const pct = Math.max(50, Math.min(250, Math.round((Number(zoomFactor) || 1.0) * 100)));
      try {
         __fnxAndroidSetZoom(pct);
      } catch (_) {}
   }

   function triggerNativeOrWindowPrint() {
      if (__fnxAndroidPrintPage) {
         try {
            __fnxAndroidPrintPage(String(document.title || 'Fynexo Page'));
            return;
         } catch (_) {}
      }
      try {
         window.print();
      } catch (_) {}
   }
   let __fnxIpcShielded = false;
   let __fnxAndroidShielded = false;

   function shieldExternalWindowIpc() {
      if (__fnxIsInternalOriginInit) return;
      if (!__fnxAndroidShielded && window.__fynexo_android__) {
         __fnxAndroidShielded = true;
         try {
            const safeAndroidStub = __fnxFreeze({
               getPageBlockedAds:
                  __fnxAndroidGetPageBlocked ||
                  function () {
                     return 0;
                  },
            });
            __fnxDefineProperty(window, '__fynexo_android__', {
               value: safeAndroidStub,
               writable: false,
               configurable: false,
               enumerable: false,
            });
         } catch (_) {}
      }
      if (!__fnxIpcPost || __fnxIpcShielded) return;
      __fnxIpcShielded = true;
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

   __FNX_ICONS__;

   __FNX_I18N__;

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
      if (!__fnxIsInternalOriginInit) return null;
      try {
         const v = window.sessionStorage && window.sessionStorage.getItem(key);
         return v ? __fnxJsonParse(v) : null;
      } catch (_) {
         return null;
      }
   }

   function writeSessionCache(key, obj) {
      if (!__fnxIsInternalOriginInit) return;
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
         enforce_perf: false,
         adblock: true,
         restore_session: true,
         force_dark: false,
         incognito: false,
         https_first: true,
         allow_page_eval: false,
         window_mode: 'multi',
         language: 'zh',
         default_zoom: 1.0,
         user_agent_mode: 'default',
         custom_user_agent: '',
         tab_cache_mode: 'balanced',
         perf_preset: 'low',
         efficiency_mode: true,
         auto_trim_ram: true,
         tab_sleep_minutes: 15,
         hw_accel: true,
         reduce_animations: false,
         ext_side_panel: false,
         ext_reader_mode: false,
         ext_yt_ad_skipper: false,
         ext_gh_harmonizer: false,
         ext_media_booster: false,
         ext_userscript_engine: true,
         ext_mouse_gestures: true,
         chrome_extensions_enabled: true,
         active_mode: 'auto',
         mode_profiles:
            typeof DEFAULT_MODE_PROFILES !== 'undefined'
               ? DEFAULT_MODE_PROFILES
               : [],
         site_overrides: {},
         search_engine: 'google',
         homepage: 'fynexo://newtab',
         search_engines: DEFAULT_ENGINES,
         bangs: DEFAULT_BANGS,
      },
      bootPayload.config || {},
   );
   if (!Array.isArray(config.mode_profiles) || config.mode_profiles.length === 0) {
      config.mode_profiles =
         typeof DEFAULT_MODE_PROFILES !== 'undefined'
            ? DEFAULT_MODE_PROFILES
            : [];
   }
   let rawBaseConfig = Object.assign({}, config);
   let activeModeMatch = null;
   let modeLayoutBackup = null;
   let modeThemeBackup = null;
   let pageManualFeatureOverrides = Object.create(null);
   let userFpsHudOverride = null;
   function isZh() {
      return typeof FynexoI18n !== 'undefined'
         ? FynexoI18n.isZhLocale(config.language)
         : String(config.language || 'zh').toLowerCase().startsWith('zh');
   }
   let styles = mergeStyles(bootPayload.styles);
   let activeTheme = Object.assign(
      {},
      DEFAULT_STYLES.colors,
      bootPayload.active_theme || {},
   );
   let modeThemes = Object.assign({}, bootPayload.mode_themes || {});
   if (Array.isArray(bootPayload.themes)) {
      for (let i = 0; i < bootPayload.themes.length; i++) {
         const t = bootPayload.themes[i];
         if (t && t.id && t.colors) modeThemes[t.id] = t.colors;
      }
   }
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
   let recentDownloads = Array.isArray(bootPayload.downloads)
      ? bootPayload.downloads
      : [];
   let chromeExtensions = Array.isArray(bootPayload.chrome_extensions)
      ? bootPayload.chrome_extensions
      : [];
   let chromeExtEngineActive = !!bootPayload.chrome_ext_engine_active;
   let sitePermissions =
      bootPayload.site_permissions && typeof bootPayload.site_permissions === 'object'
         ? bootPayload.site_permissions
         : {};
   let browserProfiles = Array.isArray(bootPayload.profiles)
      ? bootPayload.profiles
      : [];
   let activeProfileId = String(bootPayload.active_profile_id || 'default');
   let activeHudBmFolder = '';
   let syncDownloadsUIRef = null;
   let syncChromeExtsUIRef = null;
   let syncChromeStoreBannerRef = null;

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
         mode_themes: modeThemes,
         keybinds: keybinds,
         enabled_rules: enabledRules,
         bookmarks: bookmarks,
         history: historyItems.slice(0, 25),
         downloads: recentDownloads,
         chrome_ext_engine_active: chromeExtEngineActive,
         chrome_extensions: chromeExtensions.map((e) => ({
            id: e.id,
            runtime_id: e.runtime_id,
            name: e.name,
            version: e.version,
            popup_path: e.popup_path,
            options_path: e.options_path,
            enabled: e.enabled,
            pinned: e.pinned,
         })),
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
   let syncExtensionsRef = null;
   let syncFixedHeadersRef = null;
   let ensureReaderMountedRef = null;
   let openHistoryDrawerRef = null;
   let closeHistoryDrawerRef = null;
   let isHistoryOpenRef = null;
   let openPromptModalRef = null;
   let closePromptModalRef = null;
   let isPromptOpenRef = null;
   let toggleDevDrawerRef = null;
   let startVisualElementPickerRef = null;
   let isDevDrawerOpenRef = null;
   let renderDevConsoleRef = null;
   let renderDevNetworkRef = null;
   let toggleDevFpsHudRef = null;
   let syncMainMenuUIRef = null;
   let syncHudLocalizationRef = null;
   let syncSidePanelLocalizationRef = null;
   let syncHudLayoutRef = null;
   let reportContentPageMetaRef = null;
   let onContentActionCb = null;
   let onHudActionCb = null;
   let onPageMetaCb = null;
   let remotePageMeta = {
      url: '',
      readerable: false,
      reader_open: false,
      dev_drawer_open: false,
      picker_active: false,
      has_media: false,
      media_paused: true,
      media_title: '',
      media_muted: false,
      media_gain: 100,
      blocked_ads: 0,
      script_count: 0,
      sf_commands: [],
      page_is_dark: false,
   };
   let syncDevDrawerLocalizationRef = null;
   const DEV_NET_MAX = 60;
   let devNetSeq = 1;
   const devNetworkLogs = [];
   const DEV_CONSOLE_MAX = 200;
   const devConsoleLogs = [];
   function formatDevConsoleArg(a) {
      if (a === undefined) return 'undefined';
      if (a === null) return 'null';
      if (typeof a === 'string') return a;
      if (a instanceof Error) {
         return (a.name || 'Error') + ': ' + (a.message || '');
      }
      try {
         const seen = new WeakSet();
         return JSON.stringify(a, (_k, v) => {
            if (typeof v === 'object' && v !== null) {
               if (seen.has(v)) return '[Circular]';
               seen.add(v);
            }
            return v;
         });
      } catch (_) {
         return String(a);
      }
   }
   function pushDevConsoleEntry(level, argsLike) {
      try {
         const d = new Date();
         const pad2 = (n) => (n < 10 ? '0' + n : String(n));
         const time =
            pad2(d.getHours()) +
            ':' +
            pad2(d.getMinutes()) +
            ':' +
            pad2(d.getSeconds());
         const parts = [];
         for (let i = 0; i < argsLike.length; i++) {
            parts.push(formatDevConsoleArg(argsLike[i]));
         }
         devConsoleLogs.push({
            level: level || 'log',
            time: time,
            text: parts.join(' '),
         });
         if (devConsoleLogs.length > DEV_CONSOLE_MAX) {
            devConsoleLogs.splice(0, devConsoleLogs.length - DEV_CONSOLE_MAX);
         }
         if (typeof renderDevConsoleRef === 'function') {
            renderDevConsoleRef();
         }
      } catch (_) {}
   }
   if (!IS_HUD_SHELL && !window.__fnx_console_hooked__) {
      window.__fnx_console_hooked__ = true;
      try {
         ['log', 'info', 'warn', 'error'].forEach((lvl) => {
            const orig = console[lvl];
            if (typeof orig !== 'function') return;
            console[lvl] = function () {
               pushDevConsoleEntry(lvl, arguments);
               return orig.apply(console, arguments);
            };
         });
         window.addEventListener('error', (ev) => {
            const loc = ev.filename
               ? ' (' + ev.filename.split('/').pop() + ':' + (ev.lineno || 0) + ')'
               : '';
            pushDevConsoleEntry('error', [
               (ev.message || 'Uncaught Error') + loc,
            ]);
         });
         window.addEventListener('unhandledrejection', (ev) => {
            pushDevConsoleEntry('error', [
               'Unhandled Promise Rejection:',
               ev.reason,
            ]);
         });
      } catch (_) {}
   }

   function pushDevNetworkEntry(entry) {
      try {
         const d = new Date();
         const pad2 = (n) => (n < 10 ? '0' + n : String(n));
         entry.id = devNetSeq++;
         entry.time =
            pad2(d.getHours()) +
            ':' +
            pad2(d.getMinutes()) +
            ':' +
            pad2(d.getSeconds());
         devNetworkLogs.push(entry);
         if (devNetworkLogs.length > DEV_NET_MAX) {
            devNetworkLogs.splice(0, devNetworkLogs.length - DEV_NET_MAX);
         }
         if (typeof renderDevNetworkRef === 'function') {
            renderDevNetworkRef();
         }
      } catch (_) {}
   }
   function truncateDevPayload(val) {
      if (val === null || val === undefined) return '';
      if (typeof val === 'string') {
         return val.length > 12000 ? val.slice(0, 12000) + '\n... (truncated)' : val;
      }
      if (typeof FormData !== 'undefined' && val instanceof FormData) {
         return '[FormData]';
      }
      if (typeof Blob !== 'undefined' && val instanceof Blob) {
         return `[Blob ${val.size} bytes]`;
      }
      if (val instanceof ArrayBuffer || ArrayBuffer.isView(val)) {
         return `[Binary ${val.byteLength || 0} bytes]`;
      }
      try {
         const s = String(val);
         return s.length > 12000 ? s.slice(0, 12000) + '\n... (truncated)' : s;
      } catch (_) {
         return '[Payload]';
      }
   }
   if (!IS_HUD_SHELL && !window.__fnx_net_hooked__) {
      window.__fnx_net_hooked__ = true;
      try {
         if (typeof window.fetch === 'function') {
            const origFetch = window.fetch.bind(window);
            window.fetch = function (input, init) {
               const t0 = performance.now ? performance.now() : Date.now();
               let reqUrl = '';
               let method = 'GET';
               const reqHeaders = Object.create(null);
               let reqBody = '';
               try {
                  if (typeof input === 'string') {
                     reqUrl = input;
                  } else if (input && typeof input.url === 'string') {
                     reqUrl = input.url;
                     method = input.method || method;
                  }
                  if (init && init.method) method = String(init.method);
                  method = method.toUpperCase();
                  const hSrc = (init && init.headers) || (input && input.headers);
                  if (hSrc) {
                     if (typeof hSrc.forEach === 'function') {
                        hSrc.forEach((v, k) => {
                           reqHeaders[String(k)] = String(v);
                        });
                     } else if (Array.isArray(hSrc)) {
                        hSrc.forEach((pair) => {
                           if (pair && pair.length >= 2) {
                              reqHeaders[String(pair[0])] = String(pair[1]);
                           }
                        });
                     } else if (typeof hSrc === 'object') {
                        Object.keys(hSrc).forEach((k) => {
                           reqHeaders[k] = String(hSrc[k]);
                        });
                     }
                  }
                  if (init && init.body !== undefined) {
                     reqBody = truncateDevPayload(init.body);
                  }
               } catch (_) {}

               const absUrl = (() => {
                  try {
                     return new URL(reqUrl, window.location.href).href;
                  } catch (_) {
                     return reqUrl;
                  }
               })();
               const shouldTrack =
                  /^https?:/i.test(absUrl) && !absUrl.includes('fynexo.localhost');

               return origFetch(input, init).then(
                  (res) => {
                     if (shouldTrack) {
                        const dur = Math.max(
                           0,
                           Math.round((performance.now ? performance.now() : Date.now()) - t0),
                        );
                        const ct = String(
                           (res.headers && res.headers.get && res.headers.get('content-type')) || '',
                        ).toLowerCase();
                        const isEventStream = ct.includes('event-stream');
                        const isDevOpen =
                           typeof isDevDrawerOpenRef === 'function' &&
                           isDevDrawerOpenRef();
                        const canReadText =
                           !isEventStream &&
                           (!ct ||
                              ct.includes('json') ||
                              ct.includes('text') ||
                              ct.includes('xml') ||
                              ct.includes('javascript') ||
                              ct.includes('form'));
                        if (isDevOpen && canReadText && typeof res.clone === 'function') {
                           res.clone()
                              .text()
                              .then((txt) => {
                                 pushDevNetworkEntry({
                                    kind: 'fetch',
                                    method: method,
                                    url: absUrl,
                                    status: res.status || 0,
                                    duration: dur,
                                    reqHeaders: reqHeaders,
                                    reqBody: reqBody,
                                    resBody: truncateDevPayload(txt),
                                 });
                              })
                              .catch(() => {
                                 pushDevNetworkEntry({
                                    kind: 'fetch',
                                    method: method,
                                    url: absUrl,
                                    status: res.status || 0,
                                    duration: dur,
                                    reqHeaders: reqHeaders,
                                    reqBody: reqBody,
                                    resBody: '',
                                 });
                              });
                        } else {
                           pushDevNetworkEntry({
                              kind: 'fetch',
                              method: method,
                              url: absUrl,
                              status: res.status || 0,
                              duration: dur,
                              reqHeaders: reqHeaders,
                              reqBody: reqBody,
                              resBody: isEventStream
                                 ? '[EventStream]'
                                 : canReadText
                                   ? ''
                                   : ct
                                     ? `[${ct}]`
                                     : '[Binary Response]',
                           });
                        }
                     }
                     return res;
                  },
                  (err) => {
                     if (shouldTrack) {
                        pushDevNetworkEntry({
                           kind: 'fetch',
                           method: method,
                           url: absUrl,
                           status: 0,
                           duration: Math.max(
                              0,
                              Math.round((performance.now ? performance.now() : Date.now()) - t0),
                           ),
                           reqHeaders: reqHeaders,
                           reqBody: reqBody,
                           resBody: 'Network Error: ' + (err && err.message ? err.message : String(err)),
                        });
                     }
                     throw err;
                  },
               );
            };
         }
         if (typeof window.XMLHttpRequest === 'function') {
            const xp = window.XMLHttpRequest.prototype;
            const origOpen = xp.open;
            const origSetHdr = xp.setRequestHeader;
            const origSend = xp.send;
            xp.open = function (method, url) {
               this.__fnx_xhr = {
                  method: String(method || 'GET').toUpperCase(),
                  url: String(url || ''),
                  reqHeaders: Object.create(null),
                  reqBody: '',
                  t0: 0,
               };
               return origOpen.apply(this, arguments);
            };
            xp.setRequestHeader = function (k, v) {
               if (this.__fnx_xhr) {
                  this.__fnx_xhr.reqHeaders[String(k)] = String(v);
               }
               return origSetHdr.apply(this, arguments);
            };
            xp.send = function (body) {
               const meta = this.__fnx_xhr;
               if (meta) {
                  meta.t0 = performance.now ? performance.now() : Date.now();
                  meta.reqBody = truncateDevPayload(body);
                  let absUrl = meta.url;
                  try {
                     absUrl = new URL(meta.url, window.location.href).href;
                  } catch (_) {}
                  if (/^https?:/i.test(absUrl) && !absUrl.includes('fynexo.localhost')) {
                     this.addEventListener(
                        'loadend',
                        () => {
                           const dur = Math.max(
                              0,
                              Math.round((performance.now ? performance.now() : Date.now()) - meta.t0),
                           );
                           let resText = '';
                           try {
                              if (!this.responseType || this.responseType === 'text') {
                                 resText = truncateDevPayload(this.responseText || '');
                              } else if (this.responseType === 'json') {
                                 resText = truncateDevPayload(JSON.stringify(this.response, null, 2));
                              } else {
                                 resText = `[responseType: ${this.responseType}]`;
                              }
                           } catch (_) {}
                           pushDevNetworkEntry({
                              kind: 'xhr',
                              method: meta.method,
                              url: absUrl,
                              status: this.status || 0,
                              duration: dur,
                              reqHeaders: meta.reqHeaders,
                              reqBody: meta.reqBody,
                              resBody: resText,
                           });
                        },
                        { once: true },
                     );
                  }
               }
               return origSend.apply(this, arguments);
            };
         }
      } catch (_) {}
   }
   let readerFontPx = 18;
   let readerWide = false;
   let readerSavedScroll = 0;
   let findRanges = [];
   let findIdx = -1;
   let appliedRuleJsIds = Object.create(null);
   let scriptForgeMenuCommands = [];
   let scriptForgeCmdSeq = 1;
   let activePageScriptCount = 0;
   let syncShieldPopoverUIRef = null;
   const scriptForgeMemStore = Object.create(null);
   const scriptForgeListeners = Object.create(null);
   let scriptForgeListenerSeq = 1;
   let lastDeskTabsSig = '';
   let lastDrawerTabsSig = '';
   let suggestItems = [];
   let suggestIndex = -1;
   let hudToastTimer = null;
   let progressTimer = null;
   let progressCrawlTimer = null;
   let progressFadeTimer = null;
   let progressResetTimer = null;
   let progressSafetyTimer = null;
   let progressBars = [];
   let isPageLoading = false;
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
         if (IS_HUD_SHELL) {
            sendIpc({ type: 'CONTENT_ACTION', action: 'focus_content', payload: {} });
            return;
         }
         if (IS_CONTENT_TAB) return;
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

   function performHardCacheBustReload() {
      if (IS_HUD_SHELL) {
         try {
            showHudToast(
               isZh()
                  ? '已清理 CacheStorage 与 ServiceWorker 并硬刷新'
                  : 'Cleared CacheStorage & ServiceWorkers, hard reloading...',
            );
         } catch (_) {}
         triggerProgress();
         sendIpc({ type: 'CONTENT_ACTION', action: 'hard_reload', payload: {} });
         return;
      }
      try {
         showHudToast(
            isZh()
               ? '已清理 CacheStorage 与 ServiceWorker 并硬刷新'
               : 'Cleared CacheStorage & ServiceWorkers, hard reloading...',
         );
      } catch (_) {}
      const ops = [];
      try {
         if (
            navigator.serviceWorker &&
            typeof navigator.serviceWorker.getRegistrations === 'function'
         ) {
            ops.push(
               navigator.serviceWorker
                  .getRegistrations()
                  .then((regs) =>
                     Promise.all(
                        regs.map((r) => r.unregister().catch(() => false)),
                     ),
                  )
                  .catch(() => {}),
            );
         }
      } catch (_) {}
      try {
         if (window.caches && typeof window.caches.keys === 'function') {
            ops.push(
               window.caches
                  .keys()
                  .then((keys) =>
                     Promise.all(
                        keys.map((k) => window.caches.delete(k).catch(() => false)),
                     ),
                  )
                  .catch(() => {}),
            );
         }
      } catch (_) {}
      triggerProgress();
      Promise.all(ops).finally(() => {
         try {
            window.location.reload();
         } catch (_) {
            sendIpc({ type: 'RELOAD' });
         }
      });
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
         shieldExternalWindowIpc();
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

   function getActiveTabObj() {
      if (!tabState || !Array.isArray(tabState.tabs)) return null;
      return (
         tabState.tabs.find((t) => t && t.id === tabState.active_id) ||
         tabState.tabs[0] ||
         null
      );
   }

   function getCurrentRawUrl() {
      if (IS_HUD_SHELL) {
         const t = getActiveTabObj();
         return (t && t.url) || 'fynexo://newtab';
      }
      return (window.location && window.location.href) || '';
   }

   function getCurrentUrlParts() {
      const raw = canonicalCurrentUrl();
      const lower = String(raw || '').toLowerCase();
      if (lower.startsWith('fynexo://') || !raw || raw === 'about:blank') {
         return { hostname: '', host: '', port: '', href: raw };
      }
      try {
         const parsed = new URL(raw);
         return {
            hostname: String(parsed.hostname || '').toLowerCase(),
            host: String(parsed.host || '').toLowerCase(),
            port: String(parsed.port || ''),
            href: raw,
         };
      } catch (_) {
         return { hostname: '', host: '', port: '', href: raw };
      }
   }

   function parseInternalRoute(rawUrl) {
      const u = rawUrl || getCurrentRawUrl() || '';
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
      if (IS_HUD_SHELL) {
         const raw = getCurrentRawUrl();
         if (isInternalSettings(raw)) return 'fynexo://settings';
         if (isInternalNewtab(raw)) return 'fynexo://newtab';
         return raw || 'fynexo://newtab';
      }
      if (isInternalSettings()) return 'fynexo://settings';
      if (isInternalNewtab()) return 'fynexo://newtab';
      return window.location.href || '';
   }

   function isLocalDevHost(rawHost) {
      const h = String(rawHost || '')
         .trim()
         .toLowerCase();
      if (!h || h === 'fynexo.localhost') return false;
      if (
         h === 'localhost' ||
         h === '0.0.0.0' ||
         h === '[::1]' ||
         h === '::1' ||
         /^127\.\d+\.\d+\.\d+$/.test(h) ||
         /\.(?:localhost|local|test)$/.test(h)
      ) {
         return true;
      }
      if (
         /^10\.\d+\.\d+\.\d+$/.test(h) ||
         /^192\.168\.\d+\.\d+$/.test(h) ||
         /^172\.(?:1[6-9]|2\d|3[01])\.\d+\.\d+$/.test(h)
      ) {
         return true;
      }
      return false;
   }

   function formatHostLabel(rawUrl) {
      const zh = isZh();
      const ntTxt = zh ? '搜索或输入网址' : 'Search or enter URL';
      const prefTxt = zh ? '偏好设置' : 'Preferences';
      if (!rawUrl && isInternalNewtab()) return ntTxt;
      if (!rawUrl && isInternalSettings()) return prefTxt;
      if (isInternalNewtab(rawUrl)) return ntTxt;
      if (isInternalSettings(rawUrl)) return prefTxt;
      if (!rawUrl || rawUrl === 'about:blank') return ntTxt;
      try {
         const parsed = new URL(rawUrl);
         const cleanHost = parsed.hostname.replace(/^www\./, '');
         const hostWithPort = parsed.port ? `${cleanHost}:${parsed.port}` : cleanHost;
         if (!hostWithPort) return rawUrl;
         return hostWithPort;
      } catch (_) {
         return rawUrl;
      }
   }

   function extractMonogram(rawUrl, title) {
      if (isInternalNewtab(rawUrl)) return 'F';
      if (!rawUrl || isInternalSettings(rawUrl)) return 'S';
      try {
         const parsed = new URL(rawUrl);
         const host = parsed.hostname.replace(/^(www|m)\./, '');
         if (parsed.port && isLocalDevHost(host)) {
            return ':' + parsed.port.slice(-2);
         }
         if (host) return host.charAt(0).toUpperCase();
      } catch (_) {}
      const clean = (title || '').trim();
      return clean ? clean.charAt(0).toUpperCase() : '•';
   }

   function monogramAccentColor(rawUrl) {
      if (isInternalNewtab(rawUrl)) return 'var(--fnx-accent)';
      if (!rawUrl || isInternalSettings(rawUrl)) return 'var(--fnx-secondary)';
      let host = '';
      let hostKey = '';
      try {
         const parsed = new URL(rawUrl);
         host = parsed.hostname
            .replace(/^(www|m)\./, '')
            .toLowerCase();
         hostKey = parsed.port ? `${host}:${parsed.port}` : host;
      } catch (_) {
         host = String(rawUrl).toLowerCase();
         hostKey = host;
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
      for (let i = 0; i < hostKey.length; i++) {
         h = (h << 5) - h + hostKey.charCodeAt(i);
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
      targetEl.classList.remove('has-fav');
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
      targetEl.textContent = '';
      const txtSpan = document.createElement('span');
      txtSpan.className = 'fnx-tab-fav-txt';
      txtSpan.textContent = extractMonogram(rawUrl, title);
      targetEl.style.color = monogramAccentColor(rawUrl);
      targetEl.appendChild(txtSpan);

      try {
         const u = new URL(String(rawUrl || ''));
         if (
            (u.protocol === 'https:' || u.protocol === 'http:') &&
            u.hostname &&
            !isLocalDevHost(u.hostname)
         ) {
            const img = document.createElement('img');
            img.className = 'fnx-tab-fav-img';
            img.alt = '';
            img.loading = 'lazy';
            img.decoding = 'async';
            img.referrerPolicy = 'no-referrer';
            img.onload = () => {
               if (img.naturalWidth > 0) {
                  targetEl.classList.add('has-fav');
               }
            };
            img.onerror = () => {
               if (img.parentNode) img.parentNode.removeChild(img);
            };
            img.src = u.origin + '/favicon.ico';
            targetEl.appendChild(img);
         }
      } catch (_) {}
   }

   function contrastOnAccent(hex) {
      const m = String(hex || '').trim().match(/^#([0-9a-f]{6})$/i);
      if (!m) return '#ffffff';
      const n = parseInt(m[1], 16);
      const lin = (c) => {
         const v = c / 255;
         return v <= 0.04045 ? v / 12.92 : Math.pow((v + 0.055) / 1.055, 2.4);
      };
      const lum = 0.2126 * lin((n >> 16) & 255) + 0.7152 * lin((n >> 8) & 255) + 0.0722 * lin(n & 255);
      return lum > 0.185 ? '#0b0e17' : '#ffffff';
   }

   function resolvePalette() {
      const src =
         styles.colors && styles.colors.use_custom_colors
            ? styles.colors
            : (activeTheme || {});
      const d = (typeof DEFAULT_STYLES !== 'undefined' && DEFAULT_STYLES.colors) || {};
      const accent = src.accent || d.accent;
      return {
         bg: src.bg || d.bg,
         panel: src.panel || d.panel,
         surface: src.surface || d.surface || src.panel || d.panel,
         elevated: src.elevated || d.elevated,
         border: src.border || d.border,
         borderHover: src.border_hover || src.borderHover || d.border_hover,
         fg: src.fg || d.fg,
         secondary: src.secondary || d.secondary,
         muted: src.muted || d.muted,
         accent: accent,
         onAccent: contrastOnAccent(accent),
         accentBg: src.accent_bg || src.accentBg || d.accent_bg,
         hover: src.hover || d.hover,
         tabActive: src.tab_active || src.tabActive || d.tab_active,
         tabInactive: src.tab_inactive || src.tabInactive || d.tab_inactive || 'transparent',
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
      if (__FNX_MOBILE_HOST) return true;
      if (IS_HUD_SHELL) return false;
      if (/Android|iPhone|iPad|iPod|Mobile/i.test(navigator.userAgent)) {
         return true;
      }
      try {
         if (
            window.matchMedia &&
            window.matchMedia('(hover: none) and (pointer: coarse)').matches &&
            Math.min(window.innerWidth || 9999, window.innerHeight || 9999) <= 900
         ) {
            return true;
         }
      } catch (_) {}
      return false;
   }

   function computeDesktopTopHeight() {
      const tb = styles.topbar || {};
      const isDeck = (tb.layout || 'deck').toLowerCase() === 'deck';
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

   function resolveSiteOverrideKey() {
      if (isInternalAny()) return '';
      const parts = getCurrentUrlParts();
      const hostname = parts.hostname;
      if (!hostname) return '';
      const port = parts.port;
      if (port && (isLocalDevHost(hostname) || (port !== '80' && port !== '443'))) {
         return `${hostname}:${port}`;
      }
      return hostname;
   }

   function getCurrentSiteOverride() {
      const host = resolveSiteOverrideKey();
      if (!host || !config.site_overrides || typeof config.site_overrides !== 'object') {
         return null;
      }
      const ov = config.site_overrides[host];
      return ov && typeof ov === 'object' ? ov : null;
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

   __FNX_SCRIPTFORGE__;

   function applySiteRules() {
      if (IS_HUD_SHELL || isInternalAny()) {
         if (siteRulesEl && siteRulesEl.isConnected) siteRulesEl.remove();
         scriptForgeMenuCommands = [];
         activePageScriptCount = 0;
         if (typeof syncShieldPopoverUIRef === 'function') {
            syncShieldPopoverUIRef();
         }
         return;
      }

      const engineOn = config.ext_userscript_engine !== false;
      if (!engineOn) {
         try {
            const gmStyles = document.querySelectorAll('style[data-fnx-gm-style]');
            for (let i = 0; i < gmStyles.length; i++) gmStyles[i].remove();
         } catch (_) {}
         scriptForgeMenuCommands = [];
         appliedRuleJsIds = Object.create(null);
      }

      const host = window.location.hostname || '';
      const href = window.location.href || '';
      let combinedCss = '';
      let matchedCount = 0;

      for (let i = 0; i < enabledRules.length; i++) {
         const r = enabledRules[i];
         if (!r || !r.enabled) continue;
         if (!domainMatchesRule(host, r.domain)) continue;
         const meta = r.js ? parseUserscriptMeta(r.js) : null;
         if (meta && !urlMatchesUserscriptMeta(href, meta)) continue;
         matchedCount += 1;

         if (r.css && r.css.trim()) {
            combinedCss += '\n/* Rule: ' + (r.name || r.id) + ' */\n' + r.css;
         }
         if (engineOn && r.js && r.js.trim() && !appliedRuleJsIds[r.id]) {
            appliedRuleJsIds[r.id] = true;
            const code = r.js;
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
                     code,
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
                  console.warn(
                     '[ScriptForge Userscript Error]:',
                     (meta && meta.name) || r.name || r.id,
                     err,
                  );
               }
            };

            if (runAt === 'document-start') {
               runRuleJs();
            } else if (runAt === 'document-body') {
               if (document.body) {
                  runRuleJs();
               } else {
                  document.addEventListener('DOMContentLoaded', runRuleJs, {
                     once: true,
                  });
               }
            } else if (runAt === 'document-idle') {
               if (document.readyState === 'complete') {
                  setTimeout(runRuleJs, 0);
               } else {
                  window.addEventListener(
                     'load',
                     () => setTimeout(runRuleJs, 0),
                     { once: true },
                  );
               }
            } else if (document.readyState === 'loading') {
               document.addEventListener('DOMContentLoaded', runRuleJs, {
                  once: true,
               });
            } else {
               runRuleJs();
            }
         }
      }

      activePageScriptCount = matchedCount;
      if (typeof syncShieldPopoverUIRef === 'function') {
         syncShieldPopoverUIRef();
      }
      if (IS_CONTENT_TAB && typeof reportContentPageMetaRef === 'function') {
         reportContentPageMetaRef(false);
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

      if (IS_HUD_SHELL) {
         const hudBaseCss =
            'html, body { background: transparent !important; overflow: hidden !important; margin: 0 !important; padding: 0 !important; }';
         if (pageRulesEl.textContent !== hudBaseCss) {
            pageRulesEl.textContent = hudBaseCss;
         }
         return;
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

      const isInt = isInternalAny();
      const animCss = config.reduce_animations
         ? `
            *, *::before, *::after {
                animation-duration: 0.001ms !important;
                animation-iteration-count: 1 !important;
                transition-duration: 0.001ms !important;
                scroll-behavior: auto !important;
            }
        `
         : '';
      const perfCss = animCss;
      const siteOv = isInt ? null : getCurrentSiteOverride();
      if (!IS_CONTENT_TAB) {
         try {
            if (document.documentElement) {
               const zVal =
                  siteOv &&
                  typeof siteOv.zoom === 'number' &&
                  siteOv.zoom >= 0.5 &&
                  siteOv.zoom <= 2.5
                     ? String(siteOv.zoom)
                     : '';
               if (document.documentElement.style.zoom !== zVal) {
                  document.documentElement.style.zoom = zVal;
               }
            }
         } catch (_) {}
      }

      const isLocalDev = !isInt && isLocalDevHost(window.location.hostname || '');
      const siteDm =
         siteOv && siteOv.dark_mode
            ? String(siteOv.dark_mode).toLowerCase()
            : 'auto';
      const isDarkAlready = isPageAlreadyDark();
      const wantForceDark =
         siteDm === 'on'
            ? true
            : siteDm === 'off' || (isLocalDev && siteDm === 'auto')
              ? false
              : !!config.force_dark;
      const forceDarkCss =
         wantForceDark && !isInt && (siteDm === 'on' || !isDarkAlready)
            ? `
            html:not([data-fynexo-internal]) {
                background-color: #111113 !important;
            }
            html:not([data-fynexo-internal]) > body {
                filter: invert(0.92) hue-rotate(180deg) !important;
                background-color: #ffffff;
                min-height: 100dvh !important;
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
         config.adblock !== false && !isInternalAny() && !isLocalDev
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
      const layoutMode = (styles.topbar.layout || 'deck').toLowerCase();
      const isVert = layoutMode === 'vertical';
      const vertW = isVert ? styles.topbar.vertical_width || '216px' : '0px';
      const pageHost = String(window.location.hostname || '').toLowerCase();
      const pagePath = String(window.location.pathname || '').toLowerCase();
      const isCwsHost =
         !isInt &&
         (pageHost === 'chromewebstore.google.com' ||
            (pageHost === 'chrome.google.com' &&
               pagePath.startsWith('/webstore')));
      const deskOffsetCss =
         !IS_CONTENT_TAB && deskTopVisible && !isMob
            ? `
            :root {
                --fnx-host-top-offset: ${topH};
                --fnx-host-left-offset: ${vertW};
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
                left: ${isVert ? vertW : '0'} !important;
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
                --fnx-host-left-offset: 0px !important;
            }
            html[data-fynexo-internal] > body {
                position: fixed !important;
                top: ${topH} !important;
                left: ${isVert ? vertW : '0'} !important;
                right: 0 !important;
                bottom: 0 !important;
                width: ${isVert ? `calc(100% - ${vertW})` : '100%'} !important;
                max-width: ${isVert ? `calc(100% - ${vertW})` : '100%'} !important;
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
            ${
               isCwsHost
                  ? `
            /* Chrome Web Store Top Header Harmonization */
            header[role="banner"],
            #gb {
                top: var(--fnx-host-top-offset, ${topH}) !important;
                inset-block-start: var(--fnx-host-top-offset, ${topH}) !important;
                ${isVert ? `left: var(--fnx-host-left-offset, ${vertW}) !important; max-width: calc(100vw - var(--fnx-host-left-offset, ${vertW})) !important;` : ''}
            }
            `
                  : ''
            }
        `
            : `
            :root { --fnx-host-top-offset: 0px; --fnx-host-left-offset: 0px; }
        `;

      let mobPaddingRules = '';
      if (mobTopVisible) {
         mobPaddingRules += `padding-top: ${topH} !important;`;
      }
      if (mobBarVisible) {
         if (mobPos === 'top') {
            mobPaddingRules += `padding-top: calc(${mobH} + env(safe-area-inset-top, 0px)) !important;`;
         } else {
            const mobSafeB = __FNX_MOBILE_HOST
               ? 'env(safe-area-inset-bottom, 0px)'
               : 'max(env(safe-area-inset-bottom, 0px), 12px)';
            mobPaddingRules += `padding-bottom: calc(${mobH} + ${mobSafeB}) !important;`;
         }
      }

      const mobOffsetCss = !IS_CONTENT_TAB && isMob
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
      const shellRoot = shadowRoot.getElementById('__fynexo_shell__');
      if (shellRoot) {
         shellRoot.classList.toggle('zero-fx', !!config.enforce_perf);
         shellRoot.classList.toggle('reduce-motion', !!config.reduce_animations);
         shellRoot.classList.toggle('fnx-role-hud', IS_HUD_SHELL);
         shellRoot.classList.toggle('fnx-role-content', IS_CONTENT_TAB);
      }
      const topbarEl = shadowRoot.getElementById('__fynexo_topbar__');
      if (topbarEl) {
         topbarEl.classList.toggle(
            'fnx-mode-active',
            Boolean(!isInternalAny() && activeModeMatch && activeModeMatch.profile),
         );
      }
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

      const layoutMode = (tb.layout || 'deck').toLowerCase();
      const isDeck = layoutMode === 'deck';
      const isVert = layoutMode === 'vertical' && !isMobilePlatform();
      const vertW = tb.vertical_width || '216px';
      const bmShown = isBookmarksBarShown();
      const mobIsTop = (mb.position || 'bottom').toLowerCase() === 'top';
      const showDeskTop =
         !IS_CONTENT_TAB &&
         tb.visible_on_desktop !== false &&
         (!isFullscreen || isTopbarPeek);
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
                --fnx-on-accent: ${p.onAccent};
                --fnx-accent-bg: ${p.accentBg};
                --fnx-hover: ${p.hover};
                --fnx-tab-active: ${p.tabActive};
                --fnx-tab-inactive: ${p.tabInactive};
                --fnx-topbar-live: ${liveTopOffset};
                ${IS_HUD_SHELL ? `--fnx-host-top-offset: ${liveTopOffset};` : ''}
                ${IS_CONTENT_TAB ? `--fnx-host-top-offset: 0px;` : ''}
            }

            #__fynexo_topbar__ {
                background: ${tb.background || 'var(--fnx-panel)'};
                border-bottom: ${tb.border_bottom && tb.border_bottom !== '1px solid var(--fnx-border)' ? tb.border_bottom : '1px solid color-mix(in srgb, var(--fnx-fg) 11%, var(--fnx-panel))'};
                transition: border-color 0.18s ease, box-shadow 0.18s ease;
                ${tb.custom_css || ''}
            }
            #__fynexo_topbar__.fnx-mode-active {
                border-bottom-color: color-mix(in srgb, var(--fnx-accent) 48%, var(--fnx-border));
                box-shadow: inset 0 -1px 0 color-mix(in srgb, var(--fnx-accent) 24%, transparent);
            }

            #__fynexo_tab_row__ {
                ${
                   isVert
                      ? `
                position: fixed;
                top: ${liveTopOffset};
                left: 0;
                bottom: 0;
                width: ${vertW};
                height: calc(100vh - ${liveTopOffset});
                display: flex;
                flex-direction: column;
                align-items: stretch;
                justify-content: flex-start;
                padding: 8px 6px;
                background: var(--fnx-bg);
                border-right: 1px solid var(--fnx-border);
                border-bottom: none;
                z-index: 2147483645;
                box-sizing: border-box;
                `
                      : `
                display: ${isDeck ? 'flex' : 'none'};
                height: ${tb.titlebar_height || '42px'};
                padding: 0 0 0 10px;
                background: var(--fnx-bg);
                border-bottom: 1px solid color-mix(in srgb, var(--fnx-fg) 4%, transparent);
                `
                }
            }

            ${
               isVert
                  ? `
            #__fynexo_tabs_deck_cluster__ {
                flex-direction: column !important;
                align-items: stretch !important;
                width: 100% !important;
                height: 100% !important;
                gap: 4px !important;
            }
            #__fynexo_tabs_container__ {
                flex-direction: column !important;
                width: 100% !important;
                max-width: 100% !important;
                height: auto !important;
                max-height: calc(100% - 38px) !important;
                flex: 0 1 auto !important;
                overflow-y: auto !important;
                overflow-x: hidden !important;
                align-items: stretch !important;
            }
            #__fynexo_btn_newtab__ {
                width: 100% !important;
                height: 30px !important;
                margin: 2px 0 0 !important;
                border-radius: 8px !important;
                border: 1px dashed var(--fnx-border) !important;
            }
            .fnx-tab-item:not(.pinned) {
                width: 100% !important;
                max-width: 100% !important;
                min-width: 0 !important;
                flex: 0 0 ${ts.height || '32px'} !important;
            }
            `
                  : ''
            }

            #__fynexo_win_controls__ {
                height: ${isDeck ? tb.titlebar_height || '42px' : '100%'};
            }

            #__fynexo_nav_row__ {
                height: ${tb.height || '40px'};
                padding: 0 ${isDeck || isVert ? tb.padding_x || '10px' : '0'};
                background: ${tb.background || 'var(--fnx-panel)'};
            }

            ${
               isDeck || isVert
                  ? `
            #__fynexo_left_zone__ {
                flex: 1 1 0% !important;
                min-width: max-content !important;
                justify-content: flex-start !important;
            }
            #__fynexo_right_zone__ {
                flex: 1 1 0% !important;
                min-width: max-content !important;
                justify-content: flex-end !important;
            }
            #__fynexo_center_zone__ {
                flex: 0 1 clamp(320px, 46vw, 560px) !important;
                width: clamp(320px, 46vw, 560px) !important;
            }
            `
                  : ''
            }

            ${tb.show_copy_btn === false ? '#__fynexo_desk_copy__ { display: none !important; }' : ''}
            ${tb.show_star_btn === false ? '#__fynexo_desk_star__ { display: none !important; }' : ''}
            ${tb.show_shield_btn === false ? '#__fynexo_btn_shield__ { display: none !important; }' : ''}
            ${tb.show_downloads_btn === false ? '#__fynexo_btn_downloads__ { display: none !important; }' : ''}

            #__fynexo_bm_row__ {
                display: ${bmShown ? 'flex' : 'none'};
                height: 28px;
                padding: 0 ${tb.padding_x || '8px'} 0 12px;
                background: ${tb.background || 'var(--fnx-panel)'};
                border-top: none;
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
                border: 1px solid color-mix(in srgb, var(--fnx-border-hover) 68%, transparent);
                border-top-color: color-mix(in srgb, var(--fnx-fg) 22%, var(--fnx-border-hover));
                background: var(--fnx-tab-active, var(--fnx-elevated));
                box-shadow: 0 2px 8px rgba(0, 0, 0, 0.24);
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
                background: ${ab.background || 'color-mix(in srgb, var(--fnx-surface) 92%, var(--fnx-bg))'};
                border: ${ab.border || '1px solid color-mix(in srgb, var(--fnx-border-hover) 68%, var(--fnx-border))'};
                box-shadow: inset 0 1px 0 color-mix(in srgb, var(--fnx-fg) 5%, transparent);
                font-size: ${ab.font_size || '12.5px'};
                box-sizing: border-box;
                transition: background 0.14s ease, border-color 0.14s ease, box-shadow 0.14s ease, opacity 0.14s ease;
                ${ab.custom_css || ''}
            }
            #__fynexo_addr_slot__:hover:not(.editing) {
                background: color-mix(in srgb, var(--fnx-surface) 75%, var(--fnx-elevated));
                border-color: var(--fnx-border-hover);
                opacity: 1;
            }
            #__fynexo_addr_slot__.is-newtab-route:not(.editing) {
                background: color-mix(in srgb, var(--fnx-surface) 58%, var(--fnx-panel));
                border-color: color-mix(in srgb, var(--fnx-border) 78%, transparent);
                opacity: 0.78;
            }
            #__fynexo_addr_slot__.is-newtab-route:not(.editing):hover {
                background: var(--fnx-surface);
                border-color: var(--fnx-border-hover);
                opacity: 1;
            }
            #__fynexo_addr_slot__.editing {
                width: 100%;
                max-width: ${isDeck ? 'clamp(360px, 54vw, 640px)' : '46vw'};
                border-color: var(--fnx-accent);
                background: var(--fnx-bg);
                box-shadow: 0 0 0 2px color-mix(in srgb, var(--fnx-accent) 24%, transparent);
                opacity: 1;
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
                height: calc(${mb.height || '54px'} + ${mobIsTop ? 'env(safe-area-inset-top, 0px)' : (__FNX_MOBILE_HOST ? 'env(safe-area-inset-bottom, 0px)' : 'max(env(safe-area-inset-bottom, 0px), 12px)')});
                padding-left: max(env(safe-area-inset-left, 0px), ${mb.padding_x || '8px'});
                padding-right: max(env(safe-area-inset-right, 0px), ${mb.padding_x || '8px'});
                padding-bottom: ${mobIsTop ? '0px' : (__FNX_MOBILE_HOST ? 'env(safe-area-inset-bottom, 0px)' : 'max(env(safe-area-inset-bottom, 0px), 12px)')};
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
                #__fynexo_progress__ {
                    display: ${showDeskTop ? 'none' : 'block'} !important;
                }
            `
                  : `
                #__fynexo_topbar__ {
                    display: ${tb.visible_on_mobile === true && !isFullscreen && !isReaderOpen ? 'flex' : 'none'} !important;
                }
                #__fynexo_bottom_bar__ {
                    display: ${mb.visible !== false && !isPageKeyboardOpen && !isFullscreen && !isReaderOpen ? 'flex' : 'none'} !important;
                }
                #__fynexo_progress__ {
                    display: ${mb.visible !== false && !isPageKeyboardOpen && !isFullscreen && !isReaderOpen ? 'none' : (isReaderOpen ? 'none' : 'block')} !important;
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
         if (isDeck || isVert) {
            if (tabsDeckCluster.parentNode !== tabRow) {
               if (winControls.parentNode === tabRow) {
                  tabRow.insertBefore(tabsDeckCluster, winControls);
               } else {
                  tabRow.appendChild(tabsDeckCluster);
               }
            }
         } else {
            if (tabsDeckCluster.parentNode !== centerZone) {
               centerZone.insertBefore(tabsDeckCluster, centerZone.firstChild);
            }
         }
         if (isDeck) {
            if (winControls.parentNode !== tabRow) {
               tabRow.appendChild(winControls);
            }
            if (addrSlot.parentNode !== centerZone) {
               centerZone.appendChild(addrSlot);
            }
         } else {
            if (winControls.parentNode !== rightZone) {
               rightZone.appendChild(winControls);
            }
            const pos = (ab.position || (isVert ? 'center' : 'right')).toLowerCase();
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
      if (IS_HUD_SHELL && typeof syncHudLayoutRef === 'function') {
         syncHudLayoutRef();
      }
   }

   function renderBookmarksBar() {
      if (!shadowRoot || IS_CONTENT_TAB) return;
      const bmList = shadowRoot.getElementById('__fynexo_bm_list__');
      if (!bmList) return;
      bmList.textContent = '';

      const zh = isZh();
      const frag = document.createDocumentFragment();
      const items = Array.isArray(bookmarks) ? bookmarks : [];

      if (items.length === 0) {
         const emptyHint = document.createElement('span');
         emptyHint.style.cssText =
            'font-size: 11.5px; color: var(--fnx-muted); padding: 0 6px;';
         emptyHint.textContent = zh
            ? '暂无快捷方式 — 点击地址栏星标即可固定到这里'
            : 'No shortcuts saved — click the star icon in the address bar to pin sites here';
         frag.appendChild(emptyHint);
         bmList.appendChild(frag);
         return;
      }

      const folderMap = Object.create(null);
      const folderOrder = [];
      const rootItems = [];
      for (let i = 0; i < items.length; i++) {
         const b = items[i];
         if (!b || !b.url) continue;
         const f = String(b.folder || '').trim();
         if (!f) {
            rootItems.push(b);
         } else {
            if (!folderMap[f]) {
               folderMap[f] = [];
               folderOrder.push(f);
            }
            folderMap[f].push(b);
         }
      }
      if (activeHudBmFolder && !folderMap[activeHudBmFolder]) {
         activeHudBmFolder = '';
      }

      function createBookmarkPill(b) {
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
            showHudToast((zh ? '已移除快捷方式：' : 'Removed shortcut: ') + label);
         };
         return pill;
      }

      if (activeHudBmFolder && folderMap[activeHudBmFolder]) {
         const backPill = document.createElement('button');
         backPill.type = 'button';
         backPill.className = 'fnx-bm-pill';
         backPill.style.cssText =
            'border-color: var(--fnx-accent); color: var(--fnx-accent); font-weight: 600;';
         backPill.textContent = (zh ? '← 全部 · ' : '← All · ') + activeHudBmFolder;
         backPill.onclick = () => {
            activeHudBmFolder = '';
            renderBookmarksBar();
         };
         frag.appendChild(backPill);
         folderMap[activeHudBmFolder].forEach((b) => frag.appendChild(createBookmarkPill(b)));
      } else {
         folderOrder.forEach((fname) => {
            const fp = document.createElement('button');
            fp.type = 'button';
            fp.className = 'fnx-bm-pill';
            fp.title = fname + ' (' + folderMap[fname].length + ')';
            const fIcon = createIcon('folder');
            fIcon.style.cssText = 'width: 12px; height: 12px; color: var(--fnx-accent); flex-shrink: 0;';
            const fLbl = document.createElement('span');
            fLbl.className = 'fnx-bm-label';
            fLbl.textContent = fname;
            fp.appendChild(fIcon);
            fp.appendChild(fLbl);
            fp.onclick = () => {
               activeHudBmFolder = fname;
               renderBookmarksBar();
            };
            frag.appendChild(fp);
         });
         rootItems.forEach((b) => frag.appendChild(createBookmarkPill(b)));
      }

      bmList.appendChild(frag);
   }

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

   function matchModeDomainPattern(hostname, hostPort, pattern) {
      const p = String(pattern || '*')
         .trim()
         .toLowerCase();
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
      const modePref = String(rawBaseConfig.active_mode || 'auto')
         .trim()
         .toLowerCase();
      const profiles = Array.isArray(rawBaseConfig.mode_profiles)
         ? rawBaseConfig.mode_profiles
         : [];
      if (modePref === 'default' || modePref === 'off') return null;
      if (modePref === 'auto') {
         const urlParts = getCurrentUrlParts();
         const curHost = urlParts.hostname;
         const curHostPort = urlParts.port ? `${curHost}:${urlParts.port}` : curHost;
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
                  return { profile: p, auto: true };
               }
            }
         }
         return null;
      }
      for (let i = 0; i < profiles.length; i++) {
         const p = profiles[i];
         if (p && String(p.id || '').toLowerCase() === modePref) {
            return { profile: p, auto: false };
         }
      }
      return null;
   }

   function applyModeOverlayToConfig() {
      for (let i = 0; i < MODE_BOOL_KEYS.length; i++) {
         const k = MODE_BOOL_KEYS[i];
         if (Object.prototype.hasOwnProperty.call(rawBaseConfig, k)) {
            config[k] = rawBaseConfig[k];
         }
      }
      activeModeMatch = resolveActiveModeProfile();
      if (activeModeMatch && activeModeMatch.profile) {
         const p = activeModeMatch.profile;
         for (let i = 0; i < MODE_BOOL_KEYS.length; i++) {
            const k = MODE_BOOL_KEYS[i];
            if (typeof p[k] === 'boolean') {
               config[k] = p[k];
            }
         }
      }
      const mp = activeModeMatch && activeModeMatch.profile;
      const wantLayout = mp && ['deck', 'compact', 'vertical'].indexOf(String(mp.layout || '')) >= 0 ? mp.layout : '';
      if (wantLayout) {
         if (modeLayoutBackup === null) modeLayoutBackup = styles.topbar.layout || 'deck';
         styles.topbar.layout = wantLayout;
      } else if (modeLayoutBackup !== null) {
         styles.topbar.layout = modeLayoutBackup;
         modeLayoutBackup = null;
      }
      const wantThemeId = mp && mp.theme ? String(mp.theme) : '';
      const wantTheme = wantThemeId
         ? (modeThemes && modeThemes[wantThemeId]) ||
           (typeof BUILTIN_THEME_COLORS !== 'undefined'
              ? BUILTIN_THEME_COLORS[wantThemeId]
              : null)
         : null;
      if (wantTheme) {
         if (modeThemeBackup === null) modeThemeBackup = Object.assign({}, activeTheme);
         activeTheme = Object.assign({}, modeThemeBackup, wantTheme);
      } else if (modeThemeBackup !== null) {
         activeTheme = modeThemeBackup;
         modeThemeBackup = null;
      }

      const manualKeys = Object.keys(pageManualFeatureOverrides);
      for (let i = 0; i < manualKeys.length; i++) {
         const mk = manualKeys[i];
         config[mk] = pageManualFeatureOverrides[mk];
      }
      if (
         userFpsHudOverride === null &&
         typeof toggleDevFpsHudRef === 'function'
      ) {
         toggleDevFpsHudRef(
            !!(activeModeMatch && activeModeMatch.profile && activeModeMatch.profile.fps_hud),
            true,
         );
      }
   }

   function syncAllStyles() {
      applyModeOverlayToConfig();
      applyPageRules();
      applySiteRules();
      applyShadowStyles();
      if (!IS_CONTENT_TAB) {
         renderTabs(false);
         if (isDrawerOpen) renderDrawerCards();
         renderBookmarksBar();
         updateAddressLabels();
      }
      if (typeof syncExtensionsRef === 'function') syncExtensionsRef();
      if (!IS_HUD_SHELL && !IS_CONTENT_TAB && typeof syncFixedHeadersRef === 'function') {
         syncFixedHeadersRef(true);
      }
      if (!IS_CONTENT_TAB) {
         if (typeof syncChromeExtsUIRef === 'function') syncChromeExtsUIRef();
         if (typeof syncMainMenuUIRef === 'function') syncMainMenuUIRef();
      }
      if (IS_HUD_SHELL && typeof syncHudLayoutRef === 'function') {
         syncHudLayoutRef();
      }
      if (IS_CONTENT_TAB && typeof reportContentPageMetaRef === 'function') {
         reportContentPageMetaRef(false);
      }
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

   function isInsecureExternalHttp(rawUrl) {
      if (!rawUrl || !/^http:\/\//i.test(rawUrl)) return false;
      try {
         const u = new URL(rawUrl);
         return !isLocalDevHost(u.hostname || '');
      } catch (_) {
         return false;
      }
   }

   function updateProtocolIcon(containerEl, rawUrl) {
      if (!containerEl) return;
      const isSet = isInternalSettings(rawUrl);
      const isNt = isInternalNewtab(rawUrl);
      const isHttps = rawUrl && rawUrl.startsWith('https://');
      const isUnsafeHttp = isInsecureExternalHttp(rawUrl);
      const kind = isSet
         ? 'settings'
         : isNt
           ? 'search'
           : isUnsafeHttp
             ? 'alert'
             : isHttps
               ? 'lock'
               : 'search';
      if (containerEl.dataset.iconKind !== kind) {
         containerEl.dataset.iconKind = kind;
         containerEl.textContent = '';
         containerEl.appendChild(createIcon(kind));
      }
      containerEl.style.color = isSet
         ? 'var(--fnx-accent)'
         : isUnsafeHttp
           ? '#F59E0B'
           : isHttps
             ? '#10B981'
             : 'var(--fnx-muted)';
      containerEl.title = isUnsafeHttp
         ? (isZh() ? 'HTTP 明文连接（不安全）' : 'Not secure (HTTP connection)')
         : isHttps
           ? (isZh() ? 'HTTPS 安全加密连接' : 'Connection is secure (HTTPS)')
           : '';
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
            ? (isZh() ? `取消收藏 (${keybinds.bookmark_page || 'Ctrl+D'})` : `Remove shortcut (${keybinds.bookmark_page || 'Ctrl+D'})`)
            : (isZh() ? `收藏此页 (${keybinds.bookmark_page || 'Ctrl+D'})` : `Save shortcut (${keybinds.bookmark_page || 'Ctrl+D'})`);
      }

      cachedReaderable = !internal && (
         IS_HUD_SHELL ? !!remotePageMeta.readerable : isPageReaderable()
      );
      const rdActive = IS_HUD_SHELL ? !!remotePageMeta.reader_open : isReaderOpen;
      const deskReader = shadowRoot.getElementById('__fynexo_desk_reader__');
      if (deskReader) {
         deskReader.style.display =
            !internal && (cachedReaderable || rdActive) ? 'flex' : 'none';
         deskReader.classList.toggle('active-route', rdActive);
         deskReader.title = (isZh() ? '阅读模式' : 'Reader mode') + ` (${keybinds.reader_mode || 'Alt+R'})`;
      }

      const deskDark = shadowRoot.getElementById('__fynexo_desk_dark__');
      if (deskDark) {
         deskDark.classList.toggle('active-route', !!config.force_dark);
         deskDark.title = isZh()
            ? `智能夜间：${config.force_dark ? '已开启' : '已关闭'} (${keybinds.force_dark || 'Alt+Shift+D'})`
            : `Smart Force Dark: ${config.force_dark ? 'On' : 'Off'} (${keybinds.force_dark || 'Alt+Shift+D'})`;
      }

      const bmAddBtn = shadowRoot.getElementById('__fynexo_bm_add__');
      if (bmAddBtn) {
         bmAddBtn.textContent = '';
         const zh = isZh();
         if (internal) {
            bmAddBtn.appendChild(createIcon('settings'));
            bmAddBtn.appendChild(document.createTextNode(zh ? '管理' : 'Manage'));
            bmAddBtn.title = zh ? '在偏好设置中管理常用网站' : 'Manage shortcuts in Studio Preferences';
         } else {
            bmAddBtn.appendChild(createIcon(starred ? 'starFilled' : 'plus'));
            bmAddBtn.appendChild(
               document.createTextNode(starred ? (zh ? '已收藏' : 'Saved') : (zh ? '收藏本页' : 'Save page')),
            );
            bmAddBtn.title = starred
               ? (zh ? '从书签栏移除当前页' : 'Remove current page from shortcuts')
               : (zh ? '把当前页固定到书签栏' : 'Pin current page to shortcuts bar');
         }
      }

      const incogBadge = shadowRoot.getElementById('__fynexo_incog_badge__');
      if (incogBadge) {
         incogBadge.textContent = isZh() ? '无痕' : 'Private';
         incogBadge.title = isZh()
            ? '无痕模式开启中 — 点击退出'
            : 'Private mode active — click to exit Private mode';
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
      const tObj = IS_HUD_SHELL ? getActiveTabObj() : null;
      const curTitle =
         (tObj && tObj.title) || document.title || formatHostLabel(curUrl);
      sendIpc({ type: 'TOGGLE_BOOKMARK', title: curTitle, url: curUrl });
   }

   function updateAddressLabels() {
      if (!shadowRoot || IS_CONTENT_TAB) return;
      const href = canonicalCurrentUrl();
      if (typeof syncChromeStoreBannerRef === 'function') syncChromeStoreBannerRef(href);
      const label = formatHostLabel(href);
      const isSet = isInternalSettings(href);
      const isNt = isInternalNewtab(href);
      const engName = currentEngineObj().name || 'Google';

      const schemeEl = shadowRoot.getElementById('__fynexo_addr_scheme__');
      const domEl = shadowRoot.getElementById('__fynexo_addr_domain__');
      const pathEl = shadowRoot.getElementById('__fynexo_addr_path__');
      const addrSlotEl = shadowRoot.getElementById('__fynexo_addr_slot__');
      if (addrSlotEl) {
         addrSlotEl.classList.toggle('is-newtab-route', Boolean(isNt || !href || href === 'about:blank'));
      }
      const zh = isZh();
      if (domEl && pathEl) {
         if (isNt || !href || href === 'about:blank') {
            if (schemeEl) schemeEl.textContent = '';
            domEl.textContent = '';
            pathEl.textContent = zh
               ? `输入网址或按 Ctrl+L 搜索...`
               : `Enter URL or press Ctrl+L...`;
            pathEl.style.color = 'var(--fnx-muted)';
         } else if (isSet) {
            if (schemeEl) schemeEl.textContent = 'fynexo://';
            domEl.textContent = 'settings';
            domEl.style.color = 'var(--fnx-fg)';
            pathEl.textContent = zh ? ' · 偏好设置' : ' · Studio Preferences';
            pathEl.style.color = 'var(--fnx-muted)';
         } else {
            try {
               const u = new URL(href);
               const isUnsafeHttp = isInsecureExternalHttp(href);
               if (schemeEl) {
                  schemeEl.textContent = isUnsafeHttp
                     ? (zh ? '不安全 · http://' : 'Not secure · http://')
                     : u.protocol === 'https:'
                       ? ''
                       : u.protocol + '//';
                  schemeEl.style.color = isUnsafeHttp ? '#F59E0B' : '';
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
               ? (zh ? `在 ${engName} 搜索或输入网址` : `Search ${engName} or type URL`)
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
      if (IS_HUD_SHELL) {
         if (typeof closeAllNavPopoversRef === 'function') closeAllNavPopoversRef();
         sendIpc({ type: 'CONTENT_ACTION', action: 'open_find', payload: {} });
         return;
      }
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
      if (IS_HUD_SHELL) {
         cachedReaderable = !internal && !!remotePageMeta.readerable;
         const rdOpen = !internal && !!remotePageMeta.reader_open;
         const deskReader =
            shadowRoot && shadowRoot.getElementById('__fynexo_desk_reader__');
         if (deskReader) {
            deskReader.style.display =
               !internal && (cachedReaderable || rdOpen) ? 'flex' : 'none';
            deskReader.classList.toggle('active-route', rdOpen);
         }
         return;
      }
      const next = !!config.ext_reader_mode && !internal && isPageReaderable();
      cachedReaderable = next;
      const deskReader =
         shadowRoot && shadowRoot.getElementById('__fynexo_desk_reader__');
      if (deskReader) {
         deskReader.style.display =
            !internal && (cachedReaderable || isReaderOpen) ? 'flex' : 'none';
      }
      if (IS_CONTENT_TAB && typeof reportContentPageMetaRef === 'function') {
         reportContentPageMetaRef(false);
      }
   }

   function scheduleReaderableCheck() {
      if (!config.ext_reader_mode) {
         if (readerCheckTimer) {
            clearTimeout(readerCheckTimer);
            readerCheckTimer = null;
         }
         refreshReaderableState();
         return;
      }
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
      applyShadowStyles();
      try {
         window.scrollTo(0, readerSavedScroll);
      } catch (_) {}
      syncBookmarkButtons();
      if (IS_CONTENT_TAB && typeof reportContentPageMetaRef === 'function') {
         reportContentPageMetaRef(true);
      }
   }

   function toggleReaderMode() {
      if (IS_HUD_SHELL) {
         if (isInternalAny()) return;
         if (typeof closeAllNavPopoversRef === 'function') closeAllNavPopoversRef();
         sendIpc({ type: 'CONTENT_ACTION', action: 'toggle_reader', payload: {} });
         return;
      }
      if (isInternalAny()) return;
      if (isReaderOpen) {
         closeReaderMode();
         return;
      }
      if (!config.ext_reader_mode) {
         showHudToast('Reader Mode extension is off (enable in 内建 Extension)');
         return;
      }
      if (!shadowRoot) return;
      if (typeof ensureReaderMountedRef === 'function') {
         ensureReaderMountedRef();
      }
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
      applyShadowStyles();
      syncReaderTypography();
      overlay.style.display = 'flex';
      overlay.scrollTop = 0;
      if (document.documentElement)
         document.documentElement.style.overflow = 'hidden';
      syncBookmarkButtons();
      if (IS_CONTENT_TAB && typeof reportContentPageMetaRef === 'function') {
         reportContentPageMetaRef(true);
      }
   }

   function computeTabsSig() {
      let s =
         String(tabState.active_id) +
         ':' +
         String(tabState.closed_count || 0) +
         ':' +
         (isZh() ? 'zh' : 'en') +
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
      const zh = isZh();
      const drawerTitleEl = shadowRoot.getElementById('__fynexo_drawer_title__');
      if (drawerTitleEl) drawerTitleEl.textContent = zh ? '打开的标签页' : 'Open tabs';
      const drawerPrefsTxtEl = shadowRoot.getElementById('__fynexo_drawer_prefs_txt__');
      if (drawerPrefsTxtEl) drawerPrefsTxtEl.textContent = zh ? '设置' : 'Settings';
      const drawerNewTxtEl = shadowRoot.getElementById('__fynexo_drawer_new_txt__');
      if (drawerNewTxtEl) drawerNewTxtEl.textContent = zh ? '新建标签' : 'New tab';
      const drawerDoneEl = shadowRoot.getElementById('__fynexo_drawer_done__');
      if (drawerDoneEl) drawerDoneEl.textContent = zh ? '完成' : 'Done';
      const drawerCountEl = shadowRoot.getElementById('__fynexo_drawer_count__');
      if (drawerCountEl) {
         drawerCountEl.textContent = zh
            ? `${tabState.tabs.length} 个标签`
            : `${tabState.tabs.length} ${tabState.tabs.length === 1 ? 'tab' : 'tabs'}`;
      }
      const reopenBtn = shadowRoot.getElementById('__fynexo_drawer_reopen__');
      if (reopenBtn) {
         const cc = Number(tabState.closed_count) || 0;
         reopenBtn.style.display = cc > 0 ? 'inline-flex' : 'none';
         const reopenTxt = reopenBtn.querySelector('.fnx-drawer-reopen-txt');
         if (reopenTxt) {
            reopenTxt.textContent = zh
               ? (cc > 1 ? `恢复 (${cc})` : '恢复关闭')
               : (cc > 1 ? `Reopen (${cc})` : 'Reopen');
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
            ? (zh ? '偏好设置' : 'Preferences')
            : isNt
              ? (zh ? '起始页' : 'Start page')
              : tab.title || (zh ? '新标签页' : 'New tab');
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
               if (!IS_HUD_SHELL) triggerProgress();
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

      tabContextMenuEl.setAttribute('role', 'menu');
      for (let i = 0; i < cleaned.length; i++) {
         const entry = cleaned[i];
         if (entry.sep) {
            const sep = document.createElement('div');
            sep.className = 'fnx-menu-sep';
            sep.setAttribute('role', 'separator');
            tabContextMenuEl.appendChild(sep);
            continue;
         }

         const item = document.createElement('div');
         let cls = 'fnx-menu-item';
         if (entry.danger) cls += ' danger';
         if (entry.disabled) cls += ' disabled';
         item.className = cls;
         item.setAttribute('role', 'menuitem');
         item.tabIndex = entry.disabled ? -1 : 0;
         if (entry.disabled) item.setAttribute('aria-disabled', 'true');

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
         item.onkeydown = (ev) => {
            if (ev.key === 'Enter' || ev.key === ' ') {
               ev.preventDefault();
               item.click();
            } else if (ev.key === 'Escape') {
               ev.preventDefault();
               hideTabContextMenu();
            } else if (ev.key === 'ArrowDown' || ev.key === 'ArrowUp') {
               ev.preventDefault();
               const list = Array.from(tabContextMenuEl.querySelectorAll('[role="menuitem"]:not([aria-disabled="true"])'));
               const idx = list.indexOf(item);
               if (idx >= 0 && list.length > 0) {
                  const next = list[(idx + (ev.key === 'ArrowDown' ? 1 : -1) + list.length) % list.length];
                  if (next) next.focus();
               }
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

   function showBackForwardHistoryMenu(e, anchorEl) {
      if (e && e.preventDefault) {
         e.preventDefault();
         e.stopPropagation();
      }
      const zh = isZh();
      const entries = [];
      const curHref = canonicalCurrentUrl();
      const seen = Object.create(null);
      if (curHref) seen[normalizeUrlKey(curHref)] = true;

      for (let i = 0; i < historyItems.length && entries.length < 8; i++) {
         const h = historyItems[i];
         if (!h || !h.url) continue;
         const k = normalizeUrlKey(h.url);
         if (!k || seen[k]) continue;
         seen[k] = true;
         const label = (h.title || formatHostLabel(h.url) || h.url).slice(0, 42);
         const targetUrl = h.url;
         entries.push({
            label: label,
            icon: 'history',
            onClick: () => {
               flushScrollSync();
               triggerProgress();
               sendIpc({ type: 'NAVIGATE', url: targetUrl });
            },
         });
      }

      if (!isInternalNewtab(curHref)) {
         if (entries.length > 0) entries.push({ sep: true });
         entries.push({
            label: zh ? '返回起始页 (Start Page)' : 'Go to Start Page',
            icon: 'home',
            onClick: () => {
               flushScrollSync();
               triggerProgress();
               sendIpc({ type: 'NAVIGATE', url: 'fynexo://newtab' });
            },
         });
      }

      if (entries.length === 0) return;
      const rect = anchorEl && anchorEl.getBoundingClientRect
         ? anchorEl.getBoundingClientRect()
         : null;
      const cx = rect ? rect.left : (e && e.clientX) || 16;
      const cy = rect ? rect.bottom + 2 : (e && e.clientY) || 44;
      renderContextMenu(cx, cy, entries);
   }

   function showTabContextMenu(e, tab, idx) {
      const zh = isZh();
      const entries = [
         {
            label: zh ? '重新加载标签页' : 'Reload tab',
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
            label: zh ? '复制标签页' : 'Duplicate tab',
            icon: 'copy',
            onClick: () => sendIpc({ type: 'DUPLICATE_TAB', id: tab.id }),
         },
         !isMobilePlatform() && config.window_mode !== 'single'
            ? {
                 label: zh ? '移至新窗口' : 'Move tab to new window',
                 icon: 'external',
                 onClick: () => sendIpc({ type: 'DETACH_TAB', id: tab.id }),
              }
            : null,
         {
            label: tab.pinned ? (zh ? '取消固定标签页' : 'Unpin tab') : (zh ? '固定标签页' : 'Pin tab'),
            icon: 'pin',
            onClick: () => sendIpc({ type: 'TOGGLE_PIN_TAB', id: tab.id }),
         },
         {
            label: tab.background_play
               ? (zh ? '禁止后台保活运行' : 'Disable background running')
               : (zh ? '允许后台持续运行' : 'Allow running in background'),
            icon: 'media',
            onClick: () =>
               sendIpc({ type: 'TOGGLE_BACKGROUND_PLAY', id: tab.id }),
         },
         {
            label: tab.muted ? (zh ? '取消标签页静音' : 'Unmute tab') : (zh ? '静音此标签页' : 'Mute tab'),
            icon: tab.muted ? 'volumeUp' : 'volumeMute',
            onClick: () => sendIpc({ type: 'TOGGLE_MUTE_TAB', id: tab.id }),
         },
         { sep: true },
         {
            label: zh ? '关闭标签页' : 'Close tab',
            shortcut: 'Ctrl+W',
            icon: 'close',
            onClick: () =>
               sendIpc({ type: 'CLOSE_TAB', id: tab.id, force: true }),
         },
         tabState.tabs.length > 1
            ? {
                 label: zh ? '关闭其他标签页' : 'Close other tabs',
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
                          title: zh ? '关闭其他标签页' : 'Close other tabs',
                          badge: zh ? `${otherCount} 个标签` : `${otherCount} tabs`,
                          description: zh
                             ? '确认关闭当前工作区内其余未固定的标签页？'
                             : 'Close all other unpinned tabs in the current workspace?',
                          confirmText: zh ? '确认关闭' : 'Close tabs',
                          hideInput: true,
                       },
                       () => sendIpc({ type: 'CLOSE_OTHER_TABS', id: tab.id }),
                    );
                 },
              }
            : null,
         idx < tabState.tabs.length - 1
            ? {
                 label: zh ? '关闭右侧标签页' : 'Close tabs to the right',
                 onClick: () =>
                    sendIpc({ type: 'CLOSE_TABS_TO_RIGHT', id: tab.id }),
              }
            : null,
         {
            label: zh ? '关闭全部标签页' : 'Close all tabs',
            danger: true,
            onClick: () => {
               const unpinned = tabState.tabs.filter((t) => !t.pinned).length;
               if (unpinned <= 1 || !openPromptModalRef) {
                  sendIpc({ type: 'CLOSE_ALL_TABS' });
                  return;
               }
               openPromptModalRef(
                  {
                     title: zh ? '关闭全部标签页' : 'Close all tabs',
                     badge: zh ? `${unpinned} 个标签` : `${unpinned} tabs`,
                     description: zh
                        ? '确认关闭所有未固定标签页并回到起始页？'
                        : 'Close all unpinned tabs and reset to Start Page?',
                     confirmText: zh ? '全部关闭' : 'Close all tabs',
                     hideInput: true,
                  },
                  () => sendIpc({ type: 'CLOSE_ALL_TABS' }),
               );
            },
         },
         { sep: true },
         {
            label: zh ? '重新打开关闭的标签页' : 'Reopen closed tab',
            shortcut: 'Ctrl+Shift+T',
            icon: 'undo',
            onClick: () => sendIpc({ type: 'REOPEN_CLOSED_TAB' }),
         },
         {
            label: zh ? '新建标签页' : 'New tab',
            shortcut: 'Ctrl+T',
            icon: 'plus',
            onClick: () => {
               if (!IS_HUD_SHELL) triggerProgress();
               sendIpc({ type: 'NEW_TAB' });
            },
         },
         { sep: true },
         {
            label: zh ? '保存当前工作区快照...' : 'Save current workspace...',
            icon: 'workspace',
            onClick: () => {
               if (!openPromptModalRef) return;
               openPromptModalRef(
                  {
                     title: zh ? '保存工作区快照' : 'Save workspace',
                     badge: zh ? '会话' : 'Session',
                     description: zh
                        ? '把当前所有标签页保存为命名工作区（例如：工作、学习、开发）。'
                        : 'Save all open tabs into a named workspace (e.g. Work, Music, Dev).',
                     placeholder: zh ? '输入工作区名称…' : 'Workspace name...',
                     confirmText: zh ? '保存工作区' : 'Save workspace',
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

      const zh = isZh();
      const entries = [];

      // 1. Link actions
      if (linkEl && linkEl.href) {
         const linkHref = linkEl.href;
         const canOpenTab = /^(https?:|fynexo:)/i.test(linkHref);
         if (canOpenTab) {
            entries.push({
               label: zh ? '在新标签页打开链接' : 'Open link in new tab',
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
            if (!isMobilePlatform() && String(config.window_mode || 'multi').toLowerCase() !== 'single') {
               entries.push({
                  label: zh ? '在新窗口打开链接' : 'Open link in new window',
                  icon: 'workspace',
                  onClick: () => {
                     sendIpc({
                        type: 'NEW_WINDOW',
                        url: linkHref,
                        incognito: false,
                     });
                  },
               });
            }
         }
         entries.push({
            label: zh ? '复制链接地址' : 'Copy link address',
            icon: 'link',
            onClick: () => copyTextToClipboard(linkHref, zh ? '链接已复制' : 'Link copied'),
         });
         entries.push({ sep: true });
      }

      // 2. Image actions
      if (imgEl) {
         const imgSrc = imgEl.currentSrc || imgEl.src || '';
         if (imgSrc) {
            if (/^https?:/i.test(imgSrc)) {
               entries.push({
                  label: zh ? '在新标签页打开图片' : 'Open image in new tab',
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
               label: zh ? '复制图片地址' : 'Copy image URL',
               icon: 'copy',
               onClick: () =>
                  copyTextToClipboard(imgSrc, zh ? '图片地址已复制' : 'Image URL copied'),
            });
            entries.push({ sep: true });
         }
      }

      // 3. Editable input or text selection actions
      if (isEditable) {
         const hasEditableSel = !isPassword && selText.length > 0;
         entries.push({
            label: zh ? '剪切' : 'Cut',
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
            label: zh ? '复制' : 'Copy',
            shortcut: 'Ctrl+C',
            icon: 'copy',
            disabled: !hasEditableSel,
            onClick: () => copyTextToClipboard(selText, zh ? '已复制' : 'Copied'),
         });
         entries.push({
            label: zh ? '粘贴' : 'Paste',
            shortcut: 'Ctrl+V',
            icon: 'clipboard',
            onClick: () => {
               const pasteTip = zh ? '请按 Ctrl+V 粘贴' : 'Press Ctrl+V to paste';
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
                        .catch(() => showHudToast(pasteTip));
                  } else {
                     showHudToast(pasteTip);
                  }
               } catch (_) {
                  showHudToast(pasteTip);
               }
            },
         });
         entries.push({
            label: zh ? '全选' : 'Select all',
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
            label: zh ? '复制' : 'Copy',
            shortcut: 'Ctrl+C',
            icon: 'copy',
            onClick: () => copyTextToClipboard(selText, zh ? '已复制' : 'Copied'),
         });
         entries.push({
            label: zh
               ? `使用 ${eng.name || '搜索'} 搜索“${shortSel}”`
               : `Search ${eng.name || 'web'} for "${shortSel}"`,
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
         label: zh ? '后退' : 'Back',
         shortcut: 'Alt+Left',
         icon: 'back',
         disabled: !navState.canBack,
         onClick: () => requestHistoryNav('back'),
      });
      entries.push({
         label: zh ? '前进' : 'Forward',
         shortcut: 'Alt+Right',
         icon: 'forward',
         disabled: !navState.canForward,
         onClick: () => requestHistoryNav('forward'),
      });
      entries.push({
         label: zh ? '重新加载' : 'Reload',
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
            label: zh ? '复制本页网址' : 'Copy page URL',
            icon: 'link',
            onClick: () => copyCurrentUrl(),
         });
         entries.push({
            label: starred ? (zh ? '取消收藏' : 'Remove bookmark') : (zh ? '收藏本页' : 'Bookmark page'),
            icon: starred ? 'starFilled' : 'star',
            onClick: () => toggleCurrentBookmark(),
         });
         entries.push({
            label: isReaderOpen ? (zh ? '退出阅读模式' : 'Exit reader mode') : (zh ? '纯净阅读模式' : 'Reader mode'),
            shortcut: 'Alt+R',
            icon: 'reader',
            onClick: () => toggleReaderMode(),
         });
      }
      if (!isSet) {
         entries.push({
            label: zh ? '页内查找' : 'Find in page',
            shortcut: keybinds.find_in_page || 'Ctrl+F',
            icon: 'search',
            onClick: () => openFindBar(),
         });
      }
      entries.push({ sep: true });

      entries.push({
         label: zh ? '打印...' : 'Print...',
         shortcut: 'Ctrl+P',
         icon: 'printer',
         onClick: () => {
            setTimeout(() => {
               triggerNativeOrWindowPrint();
            }, 30);
         },
      });
      entries.push({
         label: isZh() ? '开发者调试抽屉' : 'Developer Drawer',
         shortcut: keybinds.dev_drawer || 'Ctrl+Shift+J',
         icon: 'code',
         onClick: () => {
            if (typeof toggleDevDrawerRef === 'function') toggleDevDrawerRef(true);
         },
      });
      if (!isMobilePlatform()) {
         entries.push({
            label: isZh() ? '检查元素 (DevTools)' : 'Inspect',
            shortcut: keybinds.devtools || 'F12',
            icon: 'code',
            onClick: () => sendIpc({ type: 'OPEN_DEVTOOLS' }),
         });
      }

      renderContextMenu(e && e.clientX, e && e.clientY, entries);
   }

   let pendingMidTab = null;
   let tabHoverTimer = null;

   function hideTabHoverCard() {
      if (tabHoverTimer) {
         clearTimeout(tabHoverTimer);
         tabHoverTimer = null;
      }
      if (!shadowRoot) return;
      const card = shadowRoot.getElementById('__fynexo_tab_hover_card__');
      if (card) card.style.display = 'none';
   }

   function getTabHoverDomain(rawUrl) {
      if (isInternalNewtab(rawUrl) || !rawUrl || rawUrl === 'about:blank') {
         return 'fynexo://newtab';
      }
      if (isInternalSettings(rawUrl)) {
         return 'fynexo://settings';
      }
      return formatHostLabel(rawUrl) || rawUrl;
   }

   function scheduleTabHoverCard(tabEl, tab, cleanTitle, isActive) {
      hideTabHoverCard();
      if (!tabEl || !shadowRoot || draggingTabId) return;
      tabHoverTimer = setTimeout(() => {
         tabHoverTimer = null;
         if (!shadowRoot || !tabEl.isConnected || draggingTabId) return;
         const shell =
            shadowRoot.getElementById('__fynexo_shell__') || shadowRoot;
         let card = shadowRoot.getElementById('__fynexo_tab_hover_card__');
         let titleEl, domainEl, badgeEl;
         if (!card) {
            card = document.createElement('div');
            card.id = '__fynexo_tab_hover_card__';
            titleEl = document.createElement('div');
            titleEl.className = 'fnx-thc-title';
            const metaEl = document.createElement('div');
            metaEl.className = 'fnx-thc-meta';
            domainEl = document.createElement('span');
            domainEl.className = 'fnx-thc-domain';
            badgeEl = document.createElement('span');
            badgeEl.className = 'fnx-thc-badge';
            metaEl.appendChild(domainEl);
            metaEl.appendChild(badgeEl);
            card.appendChild(titleEl);
            card.appendChild(metaEl);
            shell.appendChild(card);
         } else {
            titleEl = card.querySelector('.fnx-thc-title');
            domainEl = card.querySelector('.fnx-thc-domain');
            badgeEl = card.querySelector('.fnx-thc-badge');
         }

         if (titleEl) titleEl.textContent = cleanTitle;
         if (domainEl) domainEl.textContent = getTabHoverDomain(tab.url || '');

         const zh = isZh();
         let badgeTxt = '';
         if (tab.suspended && !isActive) {
            badgeTxt = zh ? '已休眠 · 省内存' : 'Suspended';
         } else if (tab.muted) {
            badgeTxt = zh ? '已静音' : 'Muted';
         } else if (tab.playing_audio) {
            badgeTxt = zh ? '正在播放音频' : 'Playing Audio';
         } else if (tab.pinned) {
            badgeTxt = zh ? '已固定' : 'Pinned';
         }
         if (badgeEl) {
            if (badgeTxt) {
               badgeEl.textContent = badgeTxt;
               badgeEl.style.display = '';
            } else {
               badgeEl.textContent = '';
               badgeEl.style.display = 'none';
            }
         }

         card.style.visibility = 'hidden';
         card.style.display = 'flex';
         const rect = tabEl.getBoundingClientRect();
         const cw = card.offsetWidth || 240;
         const ch = card.offsetHeight || 54;
         const isVert =
            (styles.topbar.layout || 'deck').toLowerCase() === 'vertical' &&
            !isMobilePlatform();
         const maxLeft = Math.max(8, window.innerWidth - cw - 8);
         const maxTop = Math.max(8, window.innerHeight - ch - 8);
         const left = isVert
            ? Math.min(maxLeft, Math.round(rect.right + 8))
            : Math.max(8, Math.min(maxLeft, Math.round(rect.left)));
         const top = isVert
            ? Math.max(8, Math.min(maxTop, Math.round(rect.top)))
            : Math.max(8, Math.min(maxTop, Math.round(rect.bottom + 6)));
         card.style.left = left + 'px';
         card.style.top = top + 'px';
         card.style.visibility = '';
      }, 240);
   }

   function finishMiddleCloseTab(tabId) {
      if (pendingMidTab && pendingMidTab.id === tabId && pendingMidTab.done) {
         return;
      }
      pendingMidTab = { id: tabId, done: true };
      sendIpc({ type: 'CLOSE_TAB', id: tabId, force: true });
   }

   function renderTabs(force) {
      if (!shadowRoot || IS_CONTENT_TAB) return;
      hideTabHoverCard();

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
         drawerCount.textContent = isZh()
            ? `${tabState.tabs.length} 个标签`
            : `${tabState.tabs.length} ${tabState.tabs.length === 1 ? 'tab' : 'tabs'}`;
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
                  const zh = isZh();
                  const cleanTitle = isSet
                     ? (zh ? '偏好设置' : 'Preferences')
                     : isNt
                       ? (zh ? '起始页' : 'Start page')
                       : tab.title || (zh ? '新标签页' : 'New tab');

                  const el = document.createElement('div');
                  let cls = isActive ? 'fnx-tab-item active' : 'fnx-tab-item';
                  if (isActive && isPageLoading) cls += ' is-loading';
                  if (tab.pinned) cls += ' pinned';
                  if (tab.suspended && !isActive) cls += ' suspended';
                  el.className = cls;
                  el.setAttribute('role', 'tab');
                  el.setAttribute('aria-selected', isActive ? 'true' : 'false');
                  el.tabIndex = 0;
                  if (isActive) activeTabEl = el;

                  el.draggable = true;
                  el.onmouseenter = () => {
                     scheduleTabHoverCard(el, tab, cleanTitle, isActive);
                  };
                  el.onmouseleave = () => {
                     hideTabHoverCard();
                  };

                  el.ondragstart = (e) => {
                     hideTabHoverCard();
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
                  el.ondragend = (e) => {
                     const wasDraggingId = draggingTabId;
                     draggingTabId = null;
                     el.classList.remove('dragging');
                     container
                        .querySelectorAll('.drag-over')
                        .forEach((n) => n.classList.remove('drag-over'));
                     if (
                        wasDraggingId === tab.id &&
                        e &&
                        (e.clientY > 118 || e.clientY < -16) &&
                        tabState.tabs.length > 1 &&
                        !isMobilePlatform() &&
                        config.window_mode !== 'single'
                     ) {
                        sendIpc({ type: 'DETACH_TAB', id: tab.id });
                     }
                  };

                  el.onclick = (e) => {
                     hideTabHoverCard();
                     if (
                        e.target.closest &&
                        (e.target.closest('.fnx-tab-close') ||
                           e.target.closest('.fnx-tab-audio-btn'))
                     )
                        return;
                     if (!isActive) {
                        flushScrollSync();
                        if (!IS_HUD_SHELL) triggerProgress();
                        sendIpc({ type: 'SWITCH_TAB', id: tab.id });
                     }
                  };
                  el.oncontextmenu = (e) => {
                     hideTabHoverCard();
                     e.preventDefault();
                     e.stopPropagation();
                     showTabContextMenu(e, tab, idx);
                  };
                  el.onmousedown = (e) => {
                     hideTabHoverCard();
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

   function omniboxPlaceholder(engName) {
      const name = engName || 'Google';
      return isZh()
         ? `在 ${name} 搜索，或输入 !bang、>命令、算式、网址...`
         : `Search ${name}, or enter !bang, >cmd, math, URL...`;
   }

   function getPaletteCommands() {
      const zh = isZh();
      const setMode = (id, label) => () => {
         rawBaseConfig.active_mode = id;
         config.active_mode = id;
         pageManualFeatureOverrides = Object.create(null);
         userFpsHudOverride = null;
         sendIpc({ type: 'PATCH_HUD_PREFS', active_mode: id });
         syncAllStyles();
         showHudToast((zh ? '模式已切换：' : 'Mode: ') + label);
      };
      const cmds = [
         { zh: '切换到自动场景模式', en: 'Mode: Auto by site', kw: 'auto mode 自动 模式', run: setMode('auto', zh ? '自动' : 'Auto') },
      ];
      const profiles = Array.isArray(rawBaseConfig.mode_profiles) && rawBaseConfig.mode_profiles.length
         ? rawBaseConfig.mode_profiles
         : (Array.isArray(config.mode_profiles) ? config.mode_profiles : []);
      for (let i = 0; i < profiles.length; i++) {
         const p = profiles[i];
         if (!p || !p.id) continue;
         const nmZh = p.name_zh || p.name_en || p.id;
         const nmEn = p.name_en || p.name_zh || p.id;
         const icon = p.icon ? p.icon + ' ' : '';
         cmds.push({
            zh: '切换' + icon + nmZh,
            en: 'Mode: ' + icon + nmEn,
            kw: (p.id + ' ' + nmZh + ' ' + nmEn + ' mode 模式').toLowerCase(),
            run: setMode(p.id, zh ? nmZh : nmEn),
         });
      }
      cmds.push(
         { zh: '新建浏览窗口', en: 'New browser window', kw: 'new window win 新建 窗口', run: () => sendIpc({ type: 'NEW_WINDOW', incognito: false }) },
         { zh: '新建独立无痕窗口', en: 'New InPrivate window', kw: 'new incognito private window 无痕 新建 窗口', run: () => sendIpc({ type: 'NEW_WINDOW', incognito: true }) },
         { zh: '切换标准模式（无特殊模式）', en: 'Mode: Standard', kw: 'default standard 标准 模式', run: setMode('default', zh ? '标准' : 'Standard') },
         {
            zh: '开关智能夜间深色', en: 'Toggle Smart Force Dark', kw: 'dark night 夜间 深色',
            run: () => {
               config.force_dark = !config.force_dark;
               sendIpc({ type: 'PATCH_HUD_PREFS', force_dark: config.force_dark });
               applyPageRules();
               showHudToast(config.force_dark ? (zh ? '智能夜间已开启' : 'Smart Force Dark on') : (zh ? '智能夜间已关闭' : 'Smart Force Dark off'));
            },
         },
         {
            zh: '开关无痕模式', en: 'Toggle Private mode', kw: 'private incognito 无痕 隐私',
            run: () => {
               config.incognito = !config.incognito;
               sendIpc({ type: 'PATCH_HUD_PREFS', incognito: config.incognito });
               syncBookmarkButtons();
               showHudToast(config.incognito ? (zh ? '无痕模式已开启' : 'Private mode on') : (zh ? '无痕模式已关闭' : 'Private mode off'));
            },
         },
         { zh: '打开开发者调试抽屉', en: 'Open Developer Drawer', kw: 'dev drawer console network 调试 开发者 抽屉', run: () => { if (typeof toggleDevDrawerRef === 'function') toggleDevDrawerRef(true); } },
         { zh: '点选隐藏页面元素', en: 'Pick element to hide', kw: 'pick hide element 点选 隐藏 元素', run: () => { if (typeof startVisualElementPickerRef === 'function') startVisualElementPickerRef(); } },
         {
            zh: '保存当前标签组为工作区', en: 'Save tabs as workspace', kw: 'workspace snapshot save 工作区 快照 保存',
            run: () => {
               if (!openPromptModalRef) return;
               openPromptModalRef(
                  {
                     title: zh ? '保存工作区' : 'Save workspace',
                     badge: 'Session',
                     description: zh ? '把当前所有标签页保存为命名工作区（例如 工作、学习）。' : 'Save all open tabs into a named workspace.',
                     placeholder: zh ? '工作区名称…' : 'Workspace name...',
                     confirmText: zh ? '保存' : 'Save workspace',
                  },
                  (name) => { if (name && name.trim()) sendIpc({ type: 'SAVE_WORKSPACE', name: name.trim() }); },
               );
            },
         },
         { zh: '将当前标签移至新窗口', en: 'Move active tab to new window', kw: 'detach move window tab 标签 新窗口 拆出', run: () => sendIpc({ type: 'DETACH_TAB' }) },
         { zh: '释放浏览器闲置内存', en: 'Flush idle browser RAM', kw: 'ram memory trim flush 内存 释放 清理', run: () => sendIpc({ type: 'FORCE_TRIM_MEMORY' }) },
         { zh: '打开性能与内存设置', en: 'Open Performance & Memory Settings', kw: 'performance ram memory eco preset 性能 内存 设置', run: () => sendIpc({ type: 'NAVIGATE', url: 'fynexo://settings#sec-performance' }) },
         { zh: '硬刷新并清理缓存', en: 'Hard reload & clear cache', kw: 'reload refresh cache 刷新 缓存', run: () => performHardCacheBustReload() },
         { zh: '打开偏好设置', en: 'Open Preferences', kw: 'settings preferences 设置 偏好', run: () => togglePreferences() },
      );
      return cmds.map((c) => ({ kind: 'cmd', title: '> ' + (zh ? c.zh : c.en), kw: (c.zh + ' ' + c.en + ' ' + c.kw).toLowerCase(), hint: zh ? '回车执行' : 'Enter to run', badge: zh ? '命令' : 'Command', run: c.run }));
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
               : item.kind === 'calc' || item.kind === 'cmd'
                 ? item.kind + ':' + item.title
                 : item.kind === 'bang'
                   ? 'bang:' + item.trigger
                   : normalizeUrlKey(item.url);
         if (!key || seen[key]) return;
         seen[key] = true;
         results.push(item);
      }

      const calc = evalSafeMathExpr(trimmed);
      if (calc) {
         pushCandidate({
            kind: 'calc',
            title: '= ' + calc.text,
            hint: isZh() ? '点击或回车复制结果' : 'Click or Enter to copy',
            badge: isZh() ? '计算' : 'Calc',
            run: () => copyTextToClipboard(String(calc.value), (isZh() ? '已复制：' : 'Copied: ') + calc.text),
         });
      }
      const isPalette = q.startsWith('>');
      if (isPalette) {
         const needle = q.slice(1).trim();
         getPaletteCommands().forEach((c) => {
            if (!needle || c.kw.includes(needle)) pushCandidate(c);
         });
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
               badge: isZh() ? '标签' : 'Tab',
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
               badge: isZh() ? '收藏' : 'Saved',
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
               badge: isZh() ? '历史' : 'History',
            });
         } else if (!q && results.length < 5) {
            pushCandidate({
               kind: 'history',
               title: title,
               url: url,
               badge: isZh() ? '最近' : 'Recent',
            });
         }
      }

      // 4. Smart Popular Domain Completions & Direct Search Engine Suggestion Rows
      if (q && !isPalette && !q.startsWith('!')) {
         const popularDomains = [
            ['GitHub', 'https://github.com', 'github git gh code repo'],
            ['YouTube', 'https://www.youtube.com', 'youtube yt video'],
            ['Bilibili', 'https://www.bilibili.com', 'bilibili bili b站 哔哩哔哩'],
            ['Google', 'https://www.google.com', 'google search'],
            ['Bing', 'https://www.bing.com', 'bing search'],
            ['Wikipedia', 'https://www.wikipedia.org', 'wikipedia wiki 维基百科'],
            ['Reddit', 'https://www.reddit.com', 'reddit forum'],
            ['X / Twitter', 'https://x.com', 'twitter x.com'],
            ['Stack Overflow', 'https://stackoverflow.com', 'stackoverflow stack qa'],
            ['MDN Web Docs', 'https://developer.mozilla.org', 'mdn mozilla web docs css js'],
            ['Crates.io (Rust)', 'https://crates.io', 'crates rust cargo'],
            ['Docs.rs (Rust)', 'https://docs.rs', 'docs.rs rust documentation'],
            ['npm Registry', 'https://www.npmjs.com', 'npm node package js'],
            ['Zhihu 知乎', 'https://www.zhihu.com', 'zhihu 知乎'],
            ['V2EX', 'https://www.v2ex.com', 'v2ex developer'],
            ['ChatGPT', 'https://chatgpt.com', 'chatgpt openai ai'],
            ['Claude', 'https://claude.ai', 'claude anthropic ai'],
            ['Gemini', 'https://gemini.google.com', 'gemini google ai']
         ];
         for (let i = 0; i < popularDomains.length; i++) {
            const d = popularDomains[i];
            if (
               d[0].toLowerCase().includes(q) ||
               d[1].toLowerCase().includes(q) ||
               d[2].includes(q)
            ) {
               pushCandidate({
                  kind: 'url',
                  title: d[0],
                  url: d[1],
                  badge: isZh() ? '推荐站点' : 'Site',
               });
            }
         }
         const engObj = currentEngineObj(config.search_engine);
         const engName = (engObj && engObj.name) || 'Google';
         const searchUrl = resolveUrl(trimmed, config.search_engine);
         pushCandidate({
            kind: 'search',
            title: isZh()
               ? `使用 ${engName} 搜索 “${trimmed}”`
               : `Search ${engName} for "${trimmed}"`,
            url: searchUrl,
            badge: engName,
         });
      }

      suggestItems = results.slice(0, isPalette ? 12 : 7);
      suggestIndex = calc || (isPalette && suggestItems.length) ? 0 : -1;

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
                    : item.kind === 'cmd' || item.kind === 'calc'
                      ? 'settings'
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
             item.kind === 'calc' || item.kind === 'cmd'
                ? item.hint
                : item.kind === 'bang'
                  ? item.url
                  : formatHostLabel(item.url);
         textWrap.appendChild(tSpan);
         textWrap.appendChild(uSpan);

         const tag = document.createElement('span');
         tag.className = 'fnx-suggest-badge';
         tag.textContent = item.badge;

         row.appendChild(iconWrap);
         row.appendChild(textWrap);
         row.appendChild(tag);

         row.onclick = () => {
             if (item.kind === 'calc' || item.kind === 'cmd') {
                if (item.kind === 'cmd' && closeFn) closeFn();
                item.run();
                return;
             }
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
            if (item.kind === 'tab' && item.tabId) {
               if (!IS_HUD_SHELL) triggerProgress();
               flushScrollSync();
               sendIpc({ type: 'SWITCH_TAB', id: item.tabId });
            } else {
               triggerProgress();
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

      const zh = isZh();
      leftGroup.appendChild(
         makeChip(zh ? '起始页' : 'Start', false, () => {
            if (closeFn) closeFn();
            triggerProgress();
            sendIpc({ type: 'NAVIGATE', url: 'fynexo://newtab' });
         }),
      );

      if ((Number(tabState.closed_count) || 0) > 0) {
         leftGroup.appendChild(
            makeChip(
               zh ? '恢复标签' : 'Reopen',
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
            makeChip(zh ? '查找' : 'Find', isFindOpen, () => {
               if (closeFn) closeFn();
               openFindBar();
            }),
         );
         leftGroup.appendChild(
            makeChip(zh ? '阅读' : 'Reader', isReaderOpen, () => {
               if (closeFn) closeFn();
               toggleReaderMode();
            }),
         );
      }

      leftGroup.appendChild(
         makeChip(
            config.force_dark ? (zh ? '夜间: 开' : 'Dark: On') : (zh ? '夜间: 关' : 'Dark: Off'),
            !!config.force_dark,
            () => {
               config.force_dark = !config.force_dark;
               sendIpc({ type: 'PATCH_HUD_PREFS', force_dark: config.force_dark });
               applyPageRules();
               renderHudQuickBar(quickRow, closeFn);
               if ((IS_HUD_SHELL ? !!remotePageMeta.page_is_dark : isPageAlreadyDark()) && config.force_dark) {
                  showHudToast(zh ? '智能深色：当前网页已是深色背景（已自动跳过）' : 'Smart Dark: Page is already dark (skipped)');
               } else {
                  showHudToast(
                     config.force_dark
                        ? (zh ? '智能夜间深色已开启' : 'Smart Force Dark enabled')
                        : (zh ? '智能夜间深色已关闭' : 'Smart Force Dark off'),
                  );
               }
            },
         ),
      );

      leftGroup.appendChild(
         makeChip(
            config.incognito ? (zh ? '无痕: 开' : 'Private: On') : (zh ? '无痕' : 'Private'),
            !!config.incognito,
            () => {
               config.incognito = !config.incognito;
               sendIpc({ type: 'PATCH_HUD_PREFS', incognito: config.incognito });
               syncBookmarkButtons();
               renderHudQuickBar(quickRow, closeFn);
               showHudToast(
                  config.incognito
                     ? (zh ? '无痕模式已开启（暂停历史记录）' : 'Private mode on (history paused)')
                     : (zh ? '无痕模式已关闭' : 'Private mode off'),
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
         makeChip(zh ? '设置' : 'Prefs', false, () => {
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
                if (item.kind === 'calc' || item.kind === 'cmd') {
                   if (item.kind === 'cmd' && closeFn) closeFn();
                   item.run();
                   return;
                }
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
               if (item.kind === 'tab' && item.tabId) {
                  if (!IS_HUD_SHELL) triggerProgress();
                  sendIpc({ type: 'SWITCH_TAB', id: item.tabId });
               } else {
                  triggerProgress();
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

      if (/^https?:\/\//i.test(raw)) {
         if (/^http:\/\//i.test(raw) && config.https_first !== false) {
            try {
               const u = new URL(raw);
               if (!isLocalDevHost(u.hostname || '')) {
                  u.protocol = 'https:';
                  return u.toString();
               }
            } catch (_) {}
         }
         return raw;
      }
      if (/^:(\d{2,5})(?:[/?#].*)?$/.test(raw)) return 'http://localhost' + raw;
      if (
         /^(?:localhost|127\.\d+\.\d+\.\d+|0\.0\.0\.0|\[::1\]|[a-z0-9-]+(?:\.[a-z0-9-]+)*\.(?:localhost|local|test)|10\.\d+\.\d+\.\d+|192\.168\.\d+\.\d+|172\.(?:1[6-9]|2\d|3[01])\.\d+\.\d+)(?::\d+)?(?:[/?#].*)?$/i.test(
            raw,
         )
      ) {
         return 'http://' + raw;
      }
      if (/^[a-z0-9.-]+:\d{2,5}(?:[/?#].*)?$/i.test(raw)) return 'http://' + raw;
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
      if (progressCrawlTimer) {
         clearTimeout(progressCrawlTimer);
         progressCrawlTimer = null;
      }
      if (progressFadeTimer) {
         clearTimeout(progressFadeTimer);
         progressFadeTimer = null;
      }
      if (progressResetTimer) {
         clearTimeout(progressResetTimer);
         progressResetTimer = null;
      }
      if (progressSafetyTimer) {
         clearTimeout(progressSafetyTimer);
         progressSafetyTimer = null;
      }
   }

   function forEachProgressBar(fn) {
      for (let i = 0; i < progressBars.length; i++) {
         if (progressBars[i]) fn(progressBars[i]);
      }
   }

   function syncActiveTabLoadingClass(loading) {
      isPageLoading = !!loading;
      if (!shadowRoot) return;
      const act = shadowRoot.querySelector('.fnx-tab-item.active');
      if (act) {
         act.classList.toggle('is-loading', isPageLoading);
      }
   }

   function triggerProgress() {
      syncActiveTabLoadingClass(true);
      clearProgressTimers();
      progressSafetyTimer = setTimeout(() => {
         progressSafetyTimer = null;
         finishProgress(true);
      }, 15000);
      if (!shadowRoot || progressBars.length === 0) return;
      forEachProgressBar((pb) => {
         pb.style.transition = 'none';
         pb.style.width = '0%';
         pb.style.opacity = '1';
      });
      progressTimer = setTimeout(() => {
         progressTimer = null;
         forEachProgressBar((pb) => {
            pb.style.transition =
               'width 0.26s cubic-bezier(0.22, 1, 0.36, 1), opacity 0.16s ease';
            pb.style.width = '58%';
         });
         progressCrawlTimer = setTimeout(() => {
            progressCrawlTimer = null;
            forEachProgressBar((pb) => {
               pb.style.transition =
                  'width 2.4s cubic-bezier(0.08, 0.72, 0.15, 1), opacity 0.16s ease';
               pb.style.width = '88%';
            });
         }, 270);
      }, 16);
   }

   function finishProgress(immediate) {
      syncActiveTabLoadingClass(false);
      if (!shadowRoot || progressBars.length === 0) return;
      clearProgressTimers();
      if (immediate === true) {
         forEachProgressBar((pb) => {
            pb.style.transition = 'none';
            pb.style.opacity = '0';
            pb.style.width = '0%';
         });
         return;
      }
      forEachProgressBar((pb) => {
         pb.style.transition =
            'width 0.16s cubic-bezier(0.22, 1, 0.36, 1), opacity 0.2s ease';
         pb.style.width = '100%';
      });
      progressFadeTimer = setTimeout(() => {
         progressFadeTimer = null;
         forEachProgressBar((pb) => {
            pb.style.opacity = '0';
         });
         progressResetTimer = setTimeout(() => {
            progressResetTimer = null;
            forEachProgressBar((pb) => {
               pb.style.transition = 'none';
               pb.style.width = '0%';
            });
         }, 220);
      }, 120);
   }

   function openMobileDrawer() {
      if (!shadowRoot) return;
      isDrawerOpen = true;
      renderDrawerCards();
      const sheet = shadowRoot.getElementById('__fynexo_drawer_sheet__');
      if (sheet) {
         sheet.style.transition = '';
         sheet.style.transform = '';
      }
      const drawer = shadowRoot.getElementById('__fynexo_mobile_drawer__');
      if (drawer) drawer.style.display = 'flex';
   }

   function closeMobileDrawer() {
      if (!shadowRoot) return;
      isDrawerOpen = false;
      const sheet = shadowRoot.getElementById('__fynexo_drawer_sheet__');
      if (sheet) {
         sheet.style.transition = '';
         sheet.style.transform = '';
      }
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
            ? (isZh() ? '后退 (Alt+Left)' : 'Back (Alt+Left)')
            : (isZh() ? '后退 (Alt+Left) — 暂无上一页' : 'Back (Alt+Left) — No back history');
      }
      const fwdIds = ['__fynexo_btn_forward__', '__fynexo_sheet_fwd__'];
      for (let i = 0; i < fwdIds.length; i++) {
         const el = shadowRoot.getElementById(fwdIds[i]);
         if (!el) continue;
         el.disabled = !canFwd;
         el.setAttribute('aria-disabled', canFwd ? 'false' : 'true');
         el.title = canFwd
            ? (isZh() ? '前进 (Alt+Right)' : 'Forward (Alt+Right)')
            : (isZh() ? '前进 (Alt+Right) — 暂无下一页' : 'Forward (Alt+Right) — No forward history');
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
         if (rtPayload.config) {
            rawBaseConfig = Object.assign({}, rawBaseConfig, rtPayload.config);
            config = Object.assign(config, rtPayload.config);
         }
         if (rtPayload.styles) {
            styles = mergeStyles(rtPayload.styles);
            modeLayoutBackup = null;
         }
         if (rtPayload.active_theme) modeThemeBackup = null;
         if (rtPayload.active_theme)
            activeTheme = Object.assign(activeTheme, rtPayload.active_theme);
         if (rtPayload.mode_themes || Array.isArray(rtPayload.themes)) {
            modeThemes = Object.assign({}, rtPayload.mode_themes || {});
            if (Array.isArray(rtPayload.themes)) {
               for (let i = 0; i < rtPayload.themes.length; i++) {
                  const t = rtPayload.themes[i];
                  if (t && t.id && t.colors) modeThemes[t.id] = t.colors;
               }
            }
            modeThemeBackup = null;
         }
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
         if (Array.isArray(rtPayload.downloads)) recentDownloads = rtPayload.downloads;
         if (Array.isArray(rtPayload.chrome_extensions))
            chromeExtensions = rtPayload.chrome_extensions;
         if (typeof rtPayload.chrome_ext_engine_active === 'boolean')
            chromeExtEngineActive = rtPayload.chrome_ext_engine_active;
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
      if (payload.config) {
         rawBaseConfig = Object.assign({}, rawBaseConfig, payload.config);
         config = Object.assign(config, payload.config);
      }
      if (payload.styles) {
         styles = mergeStyles(payload.styles);
         modeLayoutBackup = null;
      }
      if (payload.active_theme) modeThemeBackup = null;
      if (payload.active_theme)
         activeTheme = Object.assign(activeTheme, payload.active_theme);
      if (payload.mode_themes || Array.isArray(payload.themes)) {
         modeThemes = Object.assign({}, payload.mode_themes || {});
         if (Array.isArray(payload.themes)) {
            for (let i = 0; i < payload.themes.length; i++) {
               const t = payload.themes[i];
               if (t && t.id && t.colors) modeThemes[t.id] = t.colors;
            }
         }
         modeThemeBackup = null;
      }
      if (payload.keybinds)
         keybinds = Object.assign({}, DEFAULT_KEYBINDS, payload.keybinds);
      if (Array.isArray(payload.enabled_rules)) {
         enabledRules = payload.enabled_rules;
      } else if (Array.isArray(payload.site_rules)) {
         enabledRules = payload.site_rules.filter((r) => r && r.enabled);
      }
      if (Array.isArray(payload.bookmarks)) bookmarks = payload.bookmarks;
      if (Array.isArray(payload.history)) historyItems = payload.history;
      if (Array.isArray(payload.chrome_extensions))
         chromeExtensions = payload.chrome_extensions;
      if (typeof payload.chrome_ext_engine_active === 'boolean')
         chromeExtEngineActive = payload.chrome_ext_engine_active;
      if (payload.site_permissions && typeof payload.site_permissions === 'object')
         sitePermissions = payload.site_permissions;
      if (Array.isArray(payload.profiles))
         browserProfiles = payload.profiles;
      if (typeof payload.active_profile_id === 'string' && payload.active_profile_id)
         activeProfileId = payload.active_profile_id;
      cacheSlimRuntime();
      syncAllStyles();
      if (typeof syncHudLocalizationRef === 'function') {
         syncHudLocalizationRef();
      }
      const input = shadowRoot
         ? shadowRoot.getElementById('__fynexo_inline_input__') ||
           shadowRoot.getElementById('__fynexo_input__')
         : null;
      if (input) {
         input.placeholder = omniboxPlaceholder(currentEngineObj().name);
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
      if (IS_CONTENT_TAB) return;
      showHudToast(msg);
   });

   exposeBridge('__fynexo_set_downloads__', function (list) {
      recentDownloads = Array.isArray(list) ? list : [];
      cacheSlimRuntime();
      if (typeof syncDownloadsUIRef === 'function') {
         syncDownloadsUIRef();
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
      if (isDevDrawerOpenRef && isDevDrawerOpenRef()) {
         if (toggleDevDrawerRef) toggleDevDrawerRef(false);
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

   exposeReadOnly('__fynexo_on_ime_inset__', function (imeOpen) {
      if (!imeOpen) {
         setKeyboardHiddenState(false);
         return;
      }
      const hudRootEl = shadowRoot
         ? shadowRoot.getElementById('__fynexo_hud_root__')
         : null;
      if (hudRootEl && hudRootEl.style.display !== 'flex' && !isFindOpen) {
         setKeyboardHiddenState(true);
      }
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

   if (!IS_HUD_SHELL) {
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
   }

   exposeBridge('__fynexo_set_tabs__', function (state) {
      if (!state) return;
      const prevActiveId = tabState ? tabState.active_id : null;
      const prevActiveTab = getActiveTabObj();
      const prevUrl = prevActiveTab ? prevActiveTab.url : '';
      tabState = state;
      if (IS_CONTENT_TAB) {
         applyModeOverlayToConfig();
         if (typeof syncExtensionsRef === 'function') syncExtensionsRef();
         return;
      }
      writeSessionCache('__fnx_tabs__', state);
      applyModeOverlayToConfig();
      renderTabs(false);
      updateAddressLabels();
      const currentTab =
         state.tabs && state.tabs.find((t) => t.id === state.active_id);
      if (IS_HUD_SHELL) {
         const nextUrl = currentTab ? currentTab.url : '';
         if (state.active_id !== prevActiveId || nextUrl !== prevUrl) {
            const emptyMeta = {
               url: nextUrl || '',
               readerable: false,
               reader_open: false,
               dev_drawer_open: false,
               picker_active: false,
               has_media: false,
               media_paused: true,
               media_title: '',
               media_muted: false,
               media_gain: 100,
               blocked_ads: 0,
               script_count: 0,
               sf_commands: [],
               page_is_dark: false,
            };
            if (typeof onPageMetaCb === 'function') {
               onPageMetaCb(emptyMeta);
            } else {
               remotePageMeta = emptyMeta;
               refreshReaderableState();
            }
            syncBookmarkButtons();
         }
      }
      try {
         if (currentTab && typeof currentTab.muted === 'boolean') {
            applyTabMuted(currentTab.muted);
         }
      } catch (_) {}
      if (IS_HUD_SHELL && typeof syncHudLayoutRef === 'function') {
         syncHudLayoutRef();
      }
   });

   exposeBridge('__fynexo_on_resource_stats__', function (stats) {
      if (typeof onResourceStatsCb === 'function') {
         onResourceStatsCb(stats);
      }
      if (
         typeof window.__fynexo_on_settings_perf__ === 'function' &&
         isInternalSettings()
      ) {
         window.__fynexo_on_settings_perf__(stats);
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

   exposeBridge('__fynexo_content_action__', function (action, payload) {
      if (typeof onContentActionCb === 'function') {
         onContentActionCb(action, payload || {});
      }
   });

   exposeBridge('__fynexo_hud_action__', function (action) {
      if (typeof onHudActionCb === 'function') {
         onHudActionCb(action);
      }
   });

   exposeBridge('__fynexo_set_page_meta__', function (meta) {
      if (typeof onPageMetaCb === 'function') {
         onPageMetaCb(meta || {});
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

         if (!IS_HUD_SHELL && !IS_CONTENT_TAB) {
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
         }

         // Conditional Background Media Keep-Alive (only activates for background_play tabs or playing media,
         // keeping normal browsing tabs 100% compatible with visibility/anti-bot checks)
         try {
            if (!window.__fynexo_bg_audio_hooked__) {
               window.__fynexo_bg_audio_hooked__ = true;
               const origHiddenDesc = Object.getOwnPropertyDescriptor(
                  Document.prototype,
                  'hidden',
               );
               const origVisDesc = Object.getOwnPropertyDescriptor(
                  Document.prototype,
                  'visibilityState',
               );
               const shouldKeepTabAwake = () => {
                  try {
                     const curTab =
                        tabState &&
                        Array.isArray(tabState.tabs) &&
                        tabState.tabs.find((t) => t.id === tabState.active_id);
                     if (curTab && curTab.background_play) return true;
                     if (
                        activeModeMatch &&
                        activeModeMatch.profile &&
                        activeModeMatch.profile.id === 'media'
                     ) {
                        return true;
                     }
                     const mediaList = document.querySelectorAll('video, audio');
                     for (let i = 0; i < mediaList.length; i++) {
                        const m = mediaList[i];
                        if (m && !m.paused && !m.ended) return true;
                     }
                  } catch (_) {}
                  return false;
               };
               if (origHiddenDesc && origHiddenDesc.get) {
                  try {
                     Object.defineProperty(Document.prototype, 'hidden', {
                        get: function () {
                           if (shouldKeepTabAwake()) return false;
                           return origHiddenDesc.get.call(this);
                        },
                        configurable: true,
                     });
                  } catch (_) {}
               }
               if (origVisDesc && origVisDesc.get) {
                  try {
                     Object.defineProperty(Document.prototype, 'visibilityState', {
                        get: function () {
                           if (shouldKeepTabAwake()) return 'visible';
                           return origVisDesc.get.call(this);
                        },
                        configurable: true,
                     });
                  } catch (_) {}
               }

               window.addEventListener(
                  'visibilitychange',
                  (e) => {
                     if (shouldKeepTabAwake()) {
                        e.stopImmediatePropagation();
                     }
                  },
                  true,
               );

               window.addEventListener(
                  'blur',
                  () => {
                     if (!shouldKeepTabAwake()) return;
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
         progressBars = [progressBar];

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
            if (tooltip) b.setAttribute('aria-label', tooltip);
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
            if ((styles.topbar.layout || 'deck').toLowerCase() === 'vertical') {
               return;
            }
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
               if (!IS_HUD_SHELL) triggerProgress();
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
            if ((styles.topbar.layout || 'deck').toLowerCase() === 'vertical') {
               return;
            }
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

         const handleNavBarBlankDrag = (e, zoneEl) => {
            if ((styles.topbar.layout || 'deck').toLowerCase() === 'deck') {
               return;
            }
            if (
               e.button === 0 &&
               (e.target === zoneEl || isDragTarget(e.target))
            ) {
               if (e.detail === 2) {
                  triggerMaximize();
                  return;
               }
               sendIpc({ type: 'WINDOW_DRAG' });
            }
         };

         const leftZone = document.createElement('div');
         leftZone.id = '__fynexo_left_zone__';
         leftZone.style.cssText =
            'display: flex; align-items: center; gap: 2px; height: 100%; flex-shrink: 0;';
         leftZone.onmousedown = (e) => handleNavBarBlankDrag(e, leftZone);

         const btnBack = makeDesktopIconBtn('back', 'Back (Alt+Left · Right-click for history)', () => {
            requestHistoryNav('back');
         });
         btnBack.id = '__fynexo_btn_back__';
         btnBack.oncontextmenu = (e) => showBackForwardHistoryMenu(e, btnBack);
         leftZone.appendChild(btnBack);

         const btnForward = makeDesktopIconBtn(
            'forward',
            'Forward (Alt+Right · Right-click for history)',
            () => {
               requestHistoryNav('forward');
            },
         );
         btnForward.id = '__fynexo_btn_forward__';
         btnForward.oncontextmenu = (e) => showBackForwardHistoryMenu(e, btnForward);
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
         centerZone.onmousedown = (e) => handleNavBarBlankDrag(e, centerZone);

         const addrSlot = document.createElement('div');
         addrSlot.id = '__fynexo_addr_slot__';

         const addrBtn = document.createElement('button');
         addrBtn.type = 'button';
         addrBtn.id = '__fynexo_address_btn__';
         addrBtn.title = 'Click to edit URL or search (!bangs supported)';
         const addrIcon = document.createElement('span');
         addrIcon.id = '__fynexo_desk_proto__';
         addrIcon.className = 'fnx-popover-trigger';
         addrIcon.title = isZh()
            ? '查看站点连接安全、权限与清理本站缓存'
            : 'View site security, permissions & clear site storage';
         addrIcon.style.cssText =
            'display: flex; align-items: center; justify-content: center; width: 20px; height: 20px; margin-left: -3px; border-radius: 5px; cursor: pointer; flex-shrink: 0;';
         addrIcon.onclick = (e) => {
            if (!isInternalAny()) {
               e.preventDefault();
               e.stopPropagation();
               toggleSiteInfoPopover(addrIcon);
            }
         };

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
         const deskPillProgress = document.createElement('div');
         deskPillProgress.className = 'fnx-pill-progress';
         addrSlot.appendChild(deskPillProgress);
         progressBars.push(deskPillProgress);
         centerZone.appendChild(addrSlot);
         navRow.appendChild(centerZone);

         // Right Studio Utility Zone
         const rightZone = document.createElement('div');
         rightZone.id = '__fynexo_right_zone__';
         rightZone.style.cssText =
            'display: flex; align-items: center; height: 100%; gap: 6px; flex-shrink: 0;';
         rightZone.onmousedown = (e) => handleNavBarBlankDrag(e, rightZone);

         // 1. Incognito / Private Badge (Conditional: only shown in private mode)
         const incogBadge = document.createElement('button');
         incogBadge.type = 'button';
         incogBadge.id = '__fynexo_incog_badge__';
         incogBadge.className = 'fnx-incog-pill';
         incogBadge.title = isZh()
            ? '无痕模式开启中 — 点击退出'
            : 'Private mode active — click to exit Private mode';
         incogBadge.textContent = isZh() ? '无痕' : 'Private';
         incogBadge.style.display = config.incognito ? 'inline-flex' : 'none';
         incogBadge.onclick = () => {
            config.incognito = false;
            sendIpc({ type: 'PATCH_HUD_PREFS', incognito: false });
            incogBadge.style.display = 'none';
            syncBookmarkButtons();
            showHudToast(isZh() ? '已退出无痕模式' : 'Private mode turned off');
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
               bgTab.title || formatHostLabel(bgTab.url) || (isZh() ? '后台媒体' : 'Background Media');
            bgMediaPill.title = isZh()
               ? `${bgMediaLabel.textContent} — 点击跳转至标签页`
               : `${bgMediaLabel.textContent} — click to jump to tab`;
            bgMediaMute.textContent = '';
            const mIcon = createIcon(bgTab.muted ? 'volumeMute' : 'volumeUp');
            mIcon.style.cssText = 'width: 11px; height: 11px; color: var(--fnx-secondary);';
            bgMediaMute.title = bgTab.muted
               ? (isZh() ? '取消后台标签静音' : 'Unmute background tab')
               : (isZh() ? '静音后台标签' : 'Mute background tab');
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
               if (!IS_HUD_SHELL) triggerProgress();
               sendIpc({ type: 'SWITCH_TAB', id: bgDockTargetTabId });
            }
         };
         onTabsRenderedCb = syncBackgroundMediaDock;
         syncBackgroundMediaDock();
         rightZone.appendChild(bgMediaPill);

         // 2. Media Control Button (Conditional: only shown when audio/video is playing)
         const btnMedia = makeDesktopIconBtn(
            'media',
            isZh() ? '媒体控制与音量增强' : 'Media Control & Volume Booster',
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
            isZh() ? '下载管理' : 'Downloads',
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

         // 3.8 Active Mode Indicator Pill (shows only when a mode profile is active)
         const modeTopPill = document.createElement('button');
         modeTopPill.id = '__fynexo_mode_pill__';
         modeTopPill.type = 'button';
         modeTopPill.className = 'fnx-studio-pill fnx-popover-trigger';
         modeTopPill.style.cssText =
            'display: none; height: 24px; padding: 0 8px; border-radius: 6px; font-size: 11px; font-weight: 600; gap: 4px; border-color: var(--fnx-accent); background: var(--fnx-accent-bg); color: var(--fnx-fg);';
         modeTopPill.onclick = (e) => {
            if (e && e.stopPropagation) e.stopPropagation();
            toggleModePopover(modeTopPill);
         };
         rightZone.appendChild(modeTopPill);

         // 3.85 Non-Default User-Agent Indicator Pill (1-click reset to System Default)
         const uaTopPill = document.createElement('button');
         uaTopPill.id = '__fynexo_ua_pill__';
         uaTopPill.type = 'button';
         uaTopPill.className = 'fnx-studio-pill';
         uaTopPill.style.cssText =
            'display: none; height: 24px; padding: 0 8px; border-radius: 6px; font-size: 11px; font-weight: 600; gap: 4px; border-color: var(--fnx-border-hover); background: var(--fnx-elevated); color: var(--fnx-fg);';
         uaTopPill.onclick = (e) => {
            if (e && e.stopPropagation) e.stopPropagation();
            config.user_agent_mode = 'default';
            rawBaseConfig.user_agent_mode = 'default';
            uaTopPill.style.display = 'none';
            sendIpc({ type: 'PATCH_HUD_PREFS', user_agent_mode: 'default' });
            showHudToast(
               isZh()
                  ? '已恢复系统默认 User-Agent'
                  : 'Restored System default User-Agent',
            );
            if (!isInternalAny()) {
               triggerProgress();
               sendIpc({ type: 'RELOAD' });
            }
         };
         rightZone.appendChild(uaTopPill);

         // 3.9 Pinned Chrome Extensions & Extensions Hub Button (Desktop)
         const pinnedExtsBar = document.createElement('div');
         pinnedExtsBar.id = '__fynexo_pinned_exts__';
         pinnedExtsBar.style.cssText =
            'display: none; align-items: center; gap: 2px;';
         rightZone.appendChild(pinnedExtsBar);

         const btnChromeExts = makeDesktopIconBtn(
            'puzzle',
            isZh() ? 'Chrome 扩展程序' : 'Chrome Extensions',
            (e) => {
               if (e && e.stopPropagation) e.stopPropagation();
               toggleChromeExtsPopover();
            },
         );
         btnChromeExts.id = '__fynexo_btn_exts__';
         btnChromeExts.classList.add('fnx-popover-trigger');
         btnChromeExts.style.display =
            !isMobilePlatform() && (chromeExtEngineActive || chromeExtensions.length > 0)
               ? 'flex'
               : 'none';
         btnChromeExts.style.position = 'relative';
         rightZone.appendChild(btnChromeExts);

         // 4. Adblock & Privacy Shield (Always visible)
         const btnShield = makeDesktopIconBtn(
            'shield',
            isZh() ? '隐私拦截与站点开关' : 'Adblock & Privacy Shield',
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

         // 4.5 Side Panel Toggle (Alt+S, Opt-In Extension)
         const btnSidePanel = makeDesktopIconBtn(
            'sidepanel',
            isZh() ? '分屏侧边栏 (Alt+S)' : 'Side Panel (Alt+S)',
            (e) => {
               if (e && e.stopPropagation) e.stopPropagation();
               toggleSidePanel();
            },
         );
         btnSidePanel.id = '__fynexo_btn_sidepanel__';
         btnSidePanel.style.display = config.ext_side_panel ? '' : 'none';
         rightZone.appendChild(btnSidePanel);

         // 5. Main Menu 3-Dots Button (Always visible)
         const btnMore = makeDesktopIconBtn('more', isZh() ? '主菜单' : 'Menu', (e) => {
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
         let siteInfoPopoverEl = null;
         let modePopoverEl = null;
         let chromeExtsPopoverEl = null;
         let taskMgrPopoverEl = null;
         let studioPopoverEl = null;
         let sidePanelEl = null;
         let isSidePanelOpen = false;

         function closeAllNavPopovers() {
            const pops = [
               mainMenuEl,
               mediaPopoverEl,
               shieldPopoverEl,
               siteInfoPopoverEl,
               modePopoverEl,
               downloadsPopoverEl,
               chromeExtsPopoverEl,
               taskMgrPopoverEl,
               studioPopoverEl,
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

         function positionNavPopover(btn, popoverEl, align) {
            if (!popoverEl) return;
            const anchor = resolvePopoverAnchor(btn);
            if (!anchor) return;
            const rect = anchor.getBoundingClientRect();
            const vw = window.innerWidth || 1024;
            const vh = window.innerHeight || 768;

            popoverEl.style.visibility = 'hidden';
            popoverEl.style.display = 'flex';
            if (isMobilePlatform()) {
               popoverEl.style.maxWidth = Math.max(220, Math.min(320, vw - 16)) + 'px';
            } else {
               popoverEl.style.maxWidth = '';
            }
            const popW = popoverEl.getBoundingClientRect().width || 240;

            if (align === 'left') {
               const leftOffset = Math.max(8, Math.min(rect.left, vw - popW - 8));
               popoverEl.style.left = Math.round(leftOffset) + 'px';
               popoverEl.style.right = 'auto';
            } else {
               const rightOffset = Math.max(
                  8,
                  Math.min(vw - rect.right, vw - popW - 8),
               );
               popoverEl.style.right = Math.round(rightOffset) + 'px';
               popoverEl.style.left = 'auto';
            }

            const openUp = rect.top > vh * 0.55;
            if (openUp) {
               let anchorTop = rect.top;
               const mobBarEl = shadowRoot
                  ? shadowRoot.getElementById('__fynexo_bottom_bar__')
                  : null;
               if (mobBarEl) {
                  const br = mobBarEl.getBoundingClientRect();
                  if (br.height > 0 && br.top > vh * 0.5) {
                     anchorTop = Math.min(anchorTop, br.top);
                  }
               }
               const bottomOffset = Math.max(8, vh - anchorTop + 10);
               popoverEl.style.top = 'auto';
               popoverEl.style.bottom = Math.round(bottomOffset) + 'px';
               popoverEl.style.maxHeight =
                  Math.max(180, Math.round(anchorTop - 20)) + 'px';
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
         mainMenuEl.setAttribute('role', 'menu');
         mainMenuEl.style.width = '260px';

         const menuI18nEntries = [];
         function addMenuItem(iconName, label, shortcut, onClick, zhLabel) {
            const item = document.createElement('div');
            item.className = 'fnx-menu-item';
            item.setAttribute('role', 'menuitem');
            item.tabIndex = 0;
            item.onkeydown = (ev) => {
               if (ev.key === 'Enter' || ev.key === ' ') {
                  ev.preventDefault();
                  if (ev.isTrusted === false && !isInternalAny()) return;
                  closeAllNavPopovers();
                  onClick(ev);
               } else if (ev.key === 'Escape') {
                  ev.preventDefault();
                  closeAllNavPopovers();
               } else if (ev.key === 'ArrowDown' || ev.key === 'ArrowUp') {
                  ev.preventDefault();
                  const list = Array.from(mainMenuEl.querySelectorAll('[role="menuitem"]')).filter(
                     (el) => el.offsetParent !== null,
                  );
                  const idx = list.indexOf(item);
                  if (idx >= 0 && list.length > 0) {
                     const next = list[(idx + (ev.key === 'ArrowDown' ? 1 : -1) + list.length) % list.length];
                     if (next) next.focus();
                  }
               }
            };
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
            item.__fnxLabelSpan = labelSpan;
            menuI18nEntries.push([labelSpan, label, zhLabel || label]);
            if (shortcut) {
               const scSpan = document.createElement('span');
               scSpan.className = 'fnx-menu-kbd';
               scSpan.textContent = shortcut;
               item.appendChild(scSpan);
            }
            item.onclick = (e) => {
               e.stopPropagation();
               closeAllNavPopovers();
               onClick(e);
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
            'palette',
            'UI & Site Studio',
            '',
            () => {
               toggleStudioPopover(btnMore, isInternalAny() ? 'ui' : 'ui');
            },
            '外观与站点定制',
         );
         addMenuItem(
            'settings',
            'Preferences',
            keybinds.preferences || 'Ctrl+,',
            () => {
               togglePreferences();
            },
            '偏好设置',
         );

         // Task Manager / Resource Monitor
         addMenuItem(
            'cpu',
            'Resource Monitor',
            '',
            () => {
               openTaskMgrPopover();
            },
            '内存与资源监视',
         );

         // Mobile-only quick popover launchers (desktop exposes these directly on topbar)
         const mobSiteInfoItem = addMenuItem(
            'lock',
            'Site info & permissions',
            '',
            () => {
               toggleSiteInfoPopover();
            },
            '站点信息与权限',
         );
         mobSiteInfoItem.classList.add('fnx-mobile-only-row');

         const mobShieldItem = addMenuItem(
            'shield',
            'Adblock Shield',
            '',
            () => {
               toggleShieldPopover();
            },
            '广告拦截与脚本',
         );
         mobShieldItem.classList.add('fnx-mobile-only-row');
         const mobShieldStatus = document.createElement('span');
         mobShieldStatus.className = 'fnx-pop-sub';
         mobShieldStatus.style.fontSize = '11px';
         mobShieldItem.appendChild(mobShieldStatus);

         const mobMediaItem = addMenuItem(
            'media',
            'Media Control',
            '',
            () => {
               toggleMediaPopover();
            },
            '音量与媒体控制',
         );
         mobMediaItem.classList.add('fnx-mobile-only-row');
         const mobMediaStatus = document.createElement('span');
         mobMediaStatus.className = 'fnx-pop-sub';
         mobMediaStatus.style.fontSize = '11px';
         mobMediaItem.appendChild(mobMediaStatus);

         const mobDownloadsItem = addMenuItem(
            'download',
            'Downloads',
            '',
            () => {
               toggleDownloadsPopover();
            },
            '下载列表',
         );
         mobDownloadsItem.classList.add('fnx-mobile-only-row');

         const mobSidePanelItem = addMenuItem(
            'sidepanel',
            'Side Panel',
            '',
            () => {
               toggleSidePanel();
            },
            '分屏侧边栏',
         );
         mobSidePanelItem.classList.add('fnx-mobile-only-row');
         mobSidePanelItem.style.display = config.ext_side_panel ? '' : 'none';

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
            const zh = isZh();
            for (let i = 0; i < menuI18nEntries.length; i++) {
               const [span, enTxt, zhTxt] = menuI18nEntries[i];
               span.textContent = zh ? zhTxt : enTxt;
            }
            darkLabel.textContent = zh ? '智能夜间深色' : 'Smart Force Dark';
            if (typeof desktopSiteLabel !== 'undefined' && desktopSiteLabel) {
               desktopSiteLabel.textContent = zh ? '桌面版网站' : 'Desktop site';
            }
            if (typeof desktopSiteSwitch !== 'undefined' && desktopSiteSwitch) {
               const isDeskUa =
                  String(config.user_agent_mode || 'default').toLowerCase() === 'desktop';
               desktopSiteSwitch.classList.toggle('on', isDeskUa);
            }
            zoomLabel.textContent = zh ? '网页缩放' : 'Zoom';
            darkSwitch.classList.toggle('on', !!config.force_dark);
            zoomValSpan.textContent =
               Math.round((Number(config.default_zoom) || 1.0) * 100) + '%';
            const internal = isInternalAny();
            const showMultiWin =
               !isMobilePlatform() &&
               String(config.window_mode || 'multi').toLowerCase() !== 'single';
            if (typeof newWindowMenuItem !== 'undefined' && newWindowMenuItem) {
               newWindowMenuItem.style.display = showMultiWin ? '' : 'none';
            }
            if (typeof newIncogWindowMenuItem !== 'undefined' && newIncogWindowMenuItem) {
               newIncogWindowMenuItem.style.display = showMultiWin ? '' : 'none';
            }
            if (typeof devDrawerMenuItem !== 'undefined' && devDrawerMenuItem) {
               const showDevItem =
                  !activeModeMatch ||
                  !activeModeMatch.profile ||
                  activeModeMatch.profile.dev_drawer_access !== false;
               devDrawerMenuItem.style.display = showDevItem ? '' : 'none';
            }
            if (typeof renderMenuModesRow === 'function') {
               renderMenuModesRow();
            }
            if (typeof renderMenuProfilesRow === 'function') {
               renderMenuProfilesRow();
            }
            if (typeof modeTopPill !== 'undefined' && modeTopPill) {
               if (!internal && activeModeMatch && activeModeMatch.profile) {
                  const pr = activeModeMatch.profile;
                  const nm = zh ? pr.name_zh || pr.name_en : pr.name_en || pr.name_zh;
                  const iconKey = resolveModeIconName(pr.icon, pr.id);
                  const shortNm =
                     String(nm || '')
                        .replace(/\s*(?:&[^Mode]*\s*)?Mode$/i, '')
                        .replace(/模式$/, '')
                        .trim() || String(nm || '').slice(0, 8);
                  modeTopPill.textContent = '';
                  const mSvg = createIcon(iconKey);
                  mSvg.style.cssText = 'width: 12.5px; height: 12.5px; flex-shrink: 0;';
                  const mLbl = document.createElement('span');
                  mLbl.textContent = shortNm;
                  modeTopPill.appendChild(mSvg);
                  modeTopPill.appendChild(mLbl);
                  modeTopPill.title =
                     (zh ? '当前模式：' : 'Active Mode: ') +
                     nm +
                     (activeModeMatch.auto
                        ? (zh ? '（自动匹配，点击切换）' : ' (Auto, click to switch)')
                        : (zh ? '（手动锁定，点击切换）' : ' (Locked, click to switch)'));
                  modeTopPill.style.display = 'inline-flex';
               } else {
                  modeTopPill.style.display = 'none';
               }
            }
            if (typeof uaTopPill !== 'undefined' && uaTopPill) {
               const uaMode = String(config.user_agent_mode || 'default').toLowerCase();
               if (uaMode !== 'default' && uaMode !== '') {
                  const uaLabel =
                     uaMode === 'mobile'
                        ? (zh ? '手机 UA' : 'Mobile UA')
                        : uaMode === 'desktop'
                          ? (zh ? '桌面 UA' : 'Desktop UA')
                          : (zh ? '自定义 UA' : 'Custom UA');
                  uaTopPill.textContent = '';
                  const uSvg = createIcon(uaMode === 'mobile' ? 'mobile' : 'globe');
                  uSvg.style.cssText = 'width: 12px; height: 12px; flex-shrink: 0; color: var(--fnx-accent);';
                  const uLbl = document.createElement('span');
                  uLbl.textContent = uaLabel;
                  uaTopPill.appendChild(uSvg);
                  uaTopPill.appendChild(uLbl);
                  uaTopPill.title = zh
                     ? `当前使用 ${uaLabel}（已同步 Client Hints）— 点击一键恢复系统默认`
                     : `Using ${uaLabel} (Client Hints synced) — click to reset to System default`;
                  uaTopPill.style.display = 'inline-flex';
               } else {
                  uaTopPill.style.display = 'none';
               }
            }
            mobShieldStatus.textContent = internal
               ? (zh ? '无需拦截' : 'Exempt')
               : config.adblock !== false
                 ? (zh ? '已开启' : 'Active')
                 : (zh ? '已暂停' : 'Paused');
            const hasMenuMedia = IS_HUD_SHELL
               ? !internal && !!remotePageMeta.has_media
               : !!currentMediaEl;
            if (hasMenuMedia) {
               mobMediaItem.style.removeProperty('display');
               const playing = IS_HUD_SHELL
                  ? !remotePageMeta.media_paused
                  : !currentMediaEl.paused && !currentMediaEl.ended;
               mobMediaStatus.textContent = playing
                  ? (zh ? '播放中' : 'Playing')
                  : (zh ? '已暂停' : 'Paused');
            } else {
               mobMediaItem.style.display = 'none';
            }
         }
         syncMainMenuUIRef = syncMainMenuUI;
         darkItem.onclick = (e) => {
            e.stopPropagation();
            config.force_dark = !config.force_dark;
            rawBaseConfig.force_dark = config.force_dark;
            pageManualFeatureOverrides.force_dark = config.force_dark;
            sendIpc({ type: 'PATCH_HUD_PREFS', force_dark: config.force_dark });
            applyPageRules();
            syncBookmarkButtons();
            syncMainMenuUI();
            if ((IS_HUD_SHELL ? !!remotePageMeta.page_is_dark : isPageAlreadyDark()) && config.force_dark) {
               showHudToast(
                  isZh()
                     ? '智能深色：当前网页已是深色背景（已自动跳过）'
                     : 'Smart Dark: Page is already dark (skipped)',
               );
            } else {
               showHudToast(
                  isZh()
                     ? (config.force_dark ? '已开启智能夜间深色' : '已关闭智能夜间深色')
                     : (config.force_dark ? 'Smart Force Dark enabled' : 'Smart Force Dark off'),
               );
            }
         };
         mainMenuEl.appendChild(darkItem);

         // Mobile 1-Tap Desktop Site Toggle (switches UA to Desktop Chrome & reloads current page)
         const desktopSiteItem = document.createElement('div');
         desktopSiteItem.className = 'fnx-menu-item fnx-mobile-only-row';
         desktopSiteItem.setAttribute('role', 'menuitem');
         desktopSiteItem.tabIndex = 0;
         const desktopSiteLeft = document.createElement('div');
         desktopSiteLeft.style.cssText =
            'display: flex; align-items: center; gap: 9px; min-width: 0;';
         const desktopSiteIconSlot = document.createElement('span');
         desktopSiteIconSlot.className = 'fnx-menu-icon';
         desktopSiteIconSlot.appendChild(createIcon('globe'));
         desktopSiteLeft.appendChild(desktopSiteIconSlot);
         const desktopSiteLabel = document.createElement('span');
         desktopSiteLabel.textContent = isZh() ? '桌面版网站' : 'Desktop site';
         desktopSiteLeft.appendChild(desktopSiteLabel);
         desktopSiteItem.appendChild(desktopSiteLeft);
         const desktopSiteSwitch = document.createElement('span');
         desktopSiteSwitch.className =
            String(config.user_agent_mode || 'default').toLowerCase() === 'desktop'
               ? 'fnx-pop-switch on'
               : 'fnx-pop-switch';
         desktopSiteItem.appendChild(desktopSiteSwitch);
         desktopSiteItem.onclick = (e) => {
            e.stopPropagation();
            const isDeskNow =
               String(config.user_agent_mode || 'default').toLowerCase() === 'desktop';
            const nextUa = isDeskNow ? 'default' : 'desktop';
            config.user_agent_mode = nextUa;
            rawBaseConfig.user_agent_mode = nextUa;
            if (__fnxAndroidSetUaMode) {
               try {
                  __fnxAndroidSetUaMode(nextUa);
               } catch (_) {}
            }
            sendIpc({ type: 'PATCH_HUD_PREFS', user_agent_mode: nextUa });
            syncMainMenuUI();
            closeAllNavPopovers();
            const zh = isZh();
            const internal = isInternalAny();
            showHudToast(
               nextUa === 'desktop'
                  ? (zh
                       ? (internal ? '已开启桌面版网站（访问外网时生效）' : '已切换桌面版网站，正在刷新...')
                       : (internal ? 'Desktop site ON (applies to websites)' : 'Desktop site ON, reloading...'))
                  : (zh
                       ? (internal ? '已恢复移动版网站' : '已恢复移动版网站，正在刷新...')
                       : (internal ? 'Mobile site restored' : 'Mobile site restored, reloading...')),
            );
            if (!internal) {
               triggerProgress();
               setTimeout(() => {
                  try {
                     window.location.reload();
                  } catch (_) {}
               }, 80);
            }
         };
         mainMenuEl.appendChild(desktopSiteItem);

         // Compact Contextual Mode Entry (opens dedicated Mode Switcher popover; no duplicated chip grid)
         const modeMenuRow = document.createElement('div');
         modeMenuRow.className = 'fnx-menu-item';
         modeMenuRow.setAttribute('role', 'menuitem');
         modeMenuRow.tabIndex = 0;
         const modeLeft = document.createElement('div');
         modeLeft.style.cssText =
            'display: flex; align-items: center; gap: 9px; min-width: 0;';
         const modeMenuIconSlot = document.createElement('span');
         modeMenuIconSlot.className = 'fnx-menu-icon';
         modeMenuIconSlot.appendChild(createIcon('spark'));
         const modeTitleLbl = document.createElement('span');
         modeLeft.appendChild(modeMenuIconSlot);
         modeLeft.appendChild(modeTitleLbl);
         const modeActiveSub = document.createElement('span');
         modeActiveSub.style.cssText = 'font-size: 11px; color: var(--fnx-accent); font-weight: 600; flex-shrink: 0; max-width: 115px; overflow: hidden; text-overflow: ellipsis; white-space: nowrap;';
         modeMenuRow.appendChild(modeLeft);
         modeMenuRow.appendChild(modeActiveSub);
         const openModeFromMenu = (ev) => {
            if (ev && ev.stopPropagation) ev.stopPropagation();
            const anchor =
               modeTopPill && modeTopPill.style.display !== 'none'
                  ? modeTopPill
                  : btnMore;
            if (typeof toggleModePopover === 'function') {
               toggleModePopover(anchor);
            }
         };
         modeMenuRow.onclick = openModeFromMenu;
         modeMenuRow.onkeydown = (ev) => {
            if (ev.key === 'Enter' || ev.key === ' ') {
               ev.preventDefault();
               openModeFromMenu(ev);
            }
         };
         mainMenuEl.appendChild(modeMenuRow);

         function renderMenuModesRow() {
            const zh = isZh();
            const pref = String(rawBaseConfig.active_mode || 'auto').toLowerCase();
            modeTitleLbl.textContent = zh ? '场景模式' : 'Browser mode';
            if (activeModeMatch && activeModeMatch.profile) {
               const pr = activeModeMatch.profile;
               const nm = String(
                  (zh ? pr.name_zh || pr.name_en : pr.name_en || pr.name_zh) ||
                     pr.name ||
                     pr.id ||
                     '',
               );
               modeActiveSub.textContent =
                  nm + (activeModeMatch.auto ? (zh ? ' (自动)' : ' (Auto)') : '');
            } else {
               modeActiveSub.textContent =
                  pref === 'default'
                     ? (zh ? '标准' : 'Standard')
                     : (zh ? '自动' : 'Auto');
            }
            if (modePopoverEl && typeof renderModePopoverUI === 'function') {
               renderModePopoverUI();
            }
         }
         renderMenuModesRow();

         addMenuSep();

         // New Tab
         addMenuItem(
            'plus',
            'New tab',
            keybinds.new_tab || 'Ctrl+T',
            () => {
               if (!IS_HUD_SHELL) triggerProgress();
               sendIpc({ type: 'NEW_TAB' });
            },
            '新建标签页',
         );

         // New Window & New Incognito Window (shown in Multi-Window desktop mode)
         const newWindowMenuItem = addMenuItem(
            'workspace',
            'New window',
            keybinds.new_window || 'Ctrl+N',
            () => {
               sendIpc({ type: 'NEW_WINDOW', incognito: false });
            },
            '新建窗口',
         );
         const newIncogWindowMenuItem = addMenuItem(
            'shield',
            'New InPrivate window',
            keybinds.new_incognito_window || 'Ctrl+Shift+N',
            () => {
               sendIpc({ type: 'NEW_WINDOW', incognito: true });
            },
            '新建无痕窗口',
         );

         // History Drawer
         addMenuItem(
            'history',
            'History drawer',
            '',
            () => {
               if (openHistoryDrawerRef) openHistoryDrawerRef();
            },
            '浏览历史记录',
         );

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
                  isZh()
                     ? (styles.topbar.bookmarks_bar === 'hidden' ? '已隐藏常用网站栏' : '已显示常用网站栏')
                     : (styles.topbar.bookmarks_bar === 'hidden' ? 'Shortcuts bar hidden' : 'Shortcuts bar visible'),
               );
            },
            '显示 / 隐藏书签栏',
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
               applyAndroidZoomNow(next);
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
            applyAndroidZoomNow(1.0);
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
               applyAndroidZoomNow(next);
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
            '页内查找文字',
         );

         // Print Page (Ctrl+P)
         addMenuItem(
            'printer',
            'Print page...',
            'Ctrl+P',
            () => {
               if (IS_HUD_SHELL) {
                  sendIpc({
                     type: 'CONTENT_ACTION',
                     action: 'print_page',
                     payload: {},
                  });
               } else {
                  triggerNativeOrWindowPrint();
               }
            },
            '打印当前页面...',
         );

         // One-Click Translate Page
         addMenuItem(
            'translate',
            'Translate this page',
            '',
            () => {
               const cur = canonicalCurrentUrl();
               if (!cur || isInternalAny(cur) || !/^https?:\/\//i.test(cur)) {
                  showHudToast(
                     isZh()
                        ? '当前页面无需翻译（仅支持外网网页）'
                        : 'Translate is available for web pages only',
                  );
                  return;
               }
               const provider = String(config.translate_provider || 'google').toLowerCase();
               const targetLang = isZh() ? 'zh-CN' : 'en';
               const enc = encodeURIComponent(cur);
               let transUrl =
                  'https://translate.google.com/translate?sl=auto&tl=' +
                  targetLang +
                  '&u=' +
                  enc;
               if (provider === 'bing') {
                  const bingLang = isZh() ? 'zh-Hans' : 'en';
                  transUrl =
                     'https://www.bing.com/translator?from=auto&to=' +
                     bingLang +
                     '&url=' +
                     enc;
               } else if (provider === 'youdao') {
                  transUrl =
                     'https://fanyi.youdao.com/web2/#/index?url=' + enc;
               }
               sendIpc({
                  type: 'NEW_TAB',
                  url: transUrl,
                  title: (isZh() ? '翻译: ' : 'Translate: ') + formatHostLabel(cur),
               });
            },
            '一键翻译此网页',
         );

         // Reader Mode
         const readerMenuItem = addMenuItem(
            'reader',
            'Reader mode',
            keybinds.reader_mode || 'Alt+R',
            () => {
               toggleReaderMode();
            },
            '纯净阅读模式',
         );
         readerMenuItem.style.display = config.ext_reader_mode ? '' : 'none';

         // Downloads Folder (Desktop OS)
         const downloadsDirMenuItem = addMenuItem(
            'download',
            'Open downloads folder',
            '',
            (ev) => {
               if (ev && ev.isTrusted === false && !isInternalAny()) return;
               sendIpc({ type: 'OPEN_DOWNLOADS_DIR' });
            },
            '打开下载文件夹',
         );
         downloadsDirMenuItem.classList.add('fnx-desktop-only-row');

         // Built-in Developer Drawer
         const devDrawerMenuItem = addMenuItem(
            'code',
            'Developer Drawer',
            keybinds.dev_drawer || 'Ctrl+Shift+J',
            () => {
               if (typeof toggleDevDrawerRef === 'function') {
                  toggleDevDrawerRef();
               }
            },
            '开发者调试抽屉',
         );

         // Multi-Profile Switcher Strip (Desktop)
         const profileBox = document.createElement('div');
         profileBox.className = 'fnx-desktop-only-row';
         profileBox.style.cssText =
            'display: flex; flex-direction: column; gap: 5px; padding: 6px 9px; margin-top: 2px; border-top: 1px solid var(--fnx-border);';
         const profileHeadRow = document.createElement('div');
         profileHeadRow.style.cssText =
            'display: flex; align-items: center; justify-content: space-between; font-size: 11px; color: var(--fnx-secondary);';
         const profileTitleLbl = document.createElement('span');
         const profileManageBtn = document.createElement('button');
         profileManageBtn.type = 'button';
         profileManageBtn.className = 'fnx-pop-link';
         profileManageBtn.style.fontSize = '10.5px';
         profileManageBtn.onclick = (ev) => {
            ev.stopPropagation();
            closeAllNavPopovers();
            sendIpc({ type: 'NAVIGATE', url: 'fynexo://settings#sec-general' });
         };
         profileHeadRow.appendChild(profileTitleLbl);
         profileHeadRow.appendChild(profileManageBtn);
         const profilePillsRow = document.createElement('div');
         profilePillsRow.style.cssText = 'display: flex; flex-wrap: wrap; gap: 4px;';
         profileBox.appendChild(profileHeadRow);
         profileBox.appendChild(profilePillsRow);
         mainMenuEl.appendChild(profileBox);

         function renderMenuProfilesRow() {
            const zh = isZh();
            profileTitleLbl.textContent = zh ? '浏览器配置文件' : 'Profile';
            profileManageBtn.textContent = zh ? '管理 / 新建' : 'Manage';
            profilePillsRow.textContent = '';
            const list =
               Array.isArray(browserProfiles) && browserProfiles.length > 0
                  ? browserProfiles
                  : [{ id: 'default', name: zh ? '默认' : 'Default', color: '#7C86FF' }];
            for (let i = 0; i < list.length; i++) {
               const p = list[i];
               if (!p || !p.id) continue;
               const isCur = p.id === activeProfileId;
               const chip = document.createElement('button');
               chip.type = 'button';
               chip.style.cssText =
                  'display: inline-flex; align-items: center; gap: 5px; height: 22px; padding: 0 8px; border-radius: 999px; font-size: 11px; cursor: pointer; border: 1px solid ' +
                  (isCur ? 'var(--fnx-accent)' : 'var(--fnx-border)') +
                  '; background: ' +
                  (isCur ? 'var(--fnx-accent-bg)' : 'var(--fnx-surface)') +
                  '; color: ' +
                  (isCur ? 'var(--fnx-fg)' : 'var(--fnx-secondary)') +
                  ';';
               const dot = document.createElement('span');
               dot.style.cssText =
                  'width: 6px; height: 6px; border-radius: 50%; background: ' +
                  (p.color || '#7C86FF') +
                  ';';
               const txt = document.createElement('span');
               txt.textContent = p.name || p.id;
               chip.appendChild(dot);
               chip.appendChild(txt);
               chip.title = isCur
                  ? (zh ? '当前窗口使用的配置' : 'Active profile in this window')
                  : (zh ? '在新独立窗口中打开此配置' : 'Open this profile in a new window');
               chip.onclick = (ev) => {
                  ev.stopPropagation();
                  closeAllNavPopovers();
                  if (!isCur) {
                     sendIpc({ type: 'LAUNCH_PROFILE', id: p.id });
                  }
               };
               profilePillsRow.appendChild(chip);
            }
         }
         renderMenuProfilesRow();

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
            const maxCap = config.ext_media_booster ? 300 : 100;
            mediaGainValue = Math.max(0, Math.min(maxCap, Number(val) || 100));
            if (IS_HUD_SHELL) {
               remotePageMeta.media_gain = mediaGainValue;
               remotePageMeta.media_muted = mediaIsMuted;
               sendIpc({
                  type: 'CONTENT_ACTION',
                  action: 'media_volume',
                  payload: { gain: mediaGainValue, muted: mediaIsMuted },
               });
               return;
            }
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
               if (IS_CONTENT_TAB && typeof reportContentPageMetaRef === 'function') {
                  reportContentPageMetaRef(true);
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
            if (IS_CONTENT_TAB && typeof reportContentPageMetaRef === 'function') {
               reportContentPageMetaRef(true);
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
         function toggleLocalMediaPlayPause() {
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
         }
         btnPlayPause.onclick = (e) => {
            e.stopPropagation();
            if (IS_HUD_SHELL) {
               const wantPlay = !!remotePageMeta.media_paused;
               remotePageMeta.media_paused = !wantPlay;
               syncMediaPopoverUI();
               sendIpc({ type: 'CONTENT_ACTION', action: 'media_play_pause', payload: {} });
               return;
            }
            toggleLocalMediaPlayPause();
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
            const zh = isZh();
            mediaHeaderTxt.textContent = zh ? '媒体控制与音量' : 'Media Control';
            btnMute.title = zh ? '静音 / 取消静音' : 'Toggle Mute';
            btnPlayPause.title = zh ? '播放 / 暂停' : 'Play / Pause';
            const isPlaying = IS_HUD_SHELL
               ? !!remotePageMeta.has_media && !remotePageMeta.media_paused
               : currentMediaEl &&
                 !currentMediaEl.paused &&
                 !currentMediaEl.ended;
            mediaStatusPill.textContent = isPlaying
               ? (zh ? '播放中' : 'Playing')
               : (zh ? '已暂停' : 'Paused');
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
               volPercentLabel.textContent = zh ? '已静音' : 'Muted';
               volPercentLabel.style.color = 'var(--fnx-muted)';
            } else if (mediaGainValue > 100) {
               volPercentLabel.textContent = mediaGainValue + '%';
               volPercentLabel.style.color = 'var(--fnx-accent)';
            } else {
               volPercentLabel.textContent = mediaGainValue + '%';
               volPercentLabel.style.color = 'var(--fnx-fg)';
            }

            const boosterOn = !!config.ext_media_booster;
            volTextSpan.textContent = boosterOn
               ? (zh ? '音量与增益' : 'Volume & Boost')
               : (zh ? '音量' : 'Volume');
            volSlider.max = boosterOn ? '300' : '100';
            if (!boosterOn && mediaGainValue > 100) {
               mediaGainValue = 100;
               volSlider.value = '100';
               applyMediaVolume(100);
            }
            presetsRow.style.gridTemplateColumns = boosterOn
               ? 'repeat(5, 1fr)'
               : 'repeat(2, 1fr)';

            for (let i = 0; i < volPresetBtns.length; i++) {
               const b = volPresetBtns[i];
               const v = Number(b.dataset.vol);
               b.style.display = !boosterOn && v > 100 ? 'none' : '';
               b.classList.toggle(
                  'active',
                  !mediaIsMuted && v === Number(mediaGainValue),
               );
            }

            if (IS_HUD_SHELL) {
               const actTab = getActiveTabObj();
               mediaTitleEl.textContent =
                  remotePageMeta.media_title ||
                  (actTab && actTab.title) ||
                  (zh ? '媒体播放器' : 'Media Player');
               mediaDomainEl.textContent = isInternalAny()
                  ? (zh ? '内置系统页面' : 'Internal Studio Page')
                  : getCurrentUrlParts().hostname.replace(/^www\./, '') || 'fynexo';
            } else {
               const pageH1 = document.querySelector(
                  'h1.ytd-watch-metadata, h1.title, h1',
               );
               const t =
                  (pageH1 && pageH1.textContent.trim()) ||
                  document.title ||
                  (zh ? '媒体播放器' : 'Media Player');
               mediaTitleEl.textContent = t;
               mediaDomainEl.textContent = isInternalAny()
                  ? (zh ? '内置系统页面' : 'Internal Studio Page')
                  : (window.location.hostname || '').replace(/^www\./, '') ||
                    'fynexo';
            }
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
            if (IS_HUD_SHELL) {
               const mbMoreDot = shadowRoot
                  ? shadowRoot.getElementById('__fynexo_mb_more_dot__')
                  : null;
               const hasM = !isInternalAny() && !!remotePageMeta.has_media;
               const isPlay = hasM && !remotePageMeta.media_paused;
               if (hasM) {
                  btnMedia.style.display = 'flex';
                  mediaPulse.style.display = isPlay ? 'inline-block' : 'none';
                  if (mbMoreDot) mbMoreDot.classList.toggle('active', isPlay);
               } else {
                  btnMedia.style.display = 'none';
                  if (mbMoreDot) mbMoreDot.classList.remove('active');
                  if (mediaPopoverEl && mediaPopoverEl.style.display === 'flex') {
                     mediaPopoverEl.style.display = 'none';
                  }
               }
               return;
            }
            reportMediaPlaybackState(false);
            const mbMoreDot = shadowRoot
               ? shadowRoot.getElementById('__fynexo_mb_more_dot__')
               : null;
            if (isInternalAny()) {
               currentMediaEl = null;
               btnMedia.style.display = 'none';
               if (mbMoreDot) mbMoreDot.classList.remove('active');
               if (IS_CONTENT_TAB && typeof reportContentPageMetaRef === 'function') {
                  reportContentPageMetaRef(false);
               }
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
            if (IS_CONTENT_TAB && typeof reportContentPageMetaRef === 'function') {
               reportContentPageMetaRef(false);
            }
         }

         if (!IS_HUD_SHELL) {
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
                     (!IS_CONTENT_TAB && btnMedia.style.display === 'none') ||
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
         }
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

         shieldToggleRow.appendChild(shieldToggleLabel);
         shieldToggleRow.appendChild(shieldSwitch);
         shieldBody.appendChild(shieldToggleRow);

         // ScriptForge Userscript Engine row + registered GM_registerMenuCommand buttons
         const sfToggleRow = document.createElement('div');
         sfToggleRow.className = 'fnx-pop-row';
         sfToggleRow.style.cssText =
            'padding-top: 8px; border-top: 1px solid var(--fnx-border);';
         const sfToggleMeta = document.createElement('div');
         sfToggleMeta.style.cssText =
            'min-width: 0; display: flex; flex-direction: column; gap: 1px;';
         const sfToggleLabel = document.createElement('span');
         sfToggleLabel.className = 'fnx-pop-label';
         sfToggleLabel.textContent = 'ScriptForge userscripts';
         const sfScriptCountSub = document.createElement('span');
         sfScriptCountSub.className = 'fnx-pop-sub';
         sfToggleMeta.appendChild(sfToggleLabel);
         sfToggleMeta.appendChild(sfScriptCountSub);
         const sfSwitch = document.createElement('button');
         sfSwitch.type = 'button';
         sfSwitch.role = 'switch';
         sfSwitch.className = 'fnx-pop-switch';
         sfToggleRow.appendChild(sfToggleMeta);
         sfToggleRow.appendChild(sfSwitch);
         shieldBody.appendChild(sfToggleRow);

         const sfCommandsWrap = document.createElement('div');
         sfCommandsWrap.style.cssText =
            'display: none; flex-direction: column; gap: 5px; padding-top: 8px; border-top: 1px solid var(--fnx-border);';
         shieldBody.appendChild(sfCommandsWrap);

         // Page Customization Tools (Element Picker + Site Boost CSS) inside Shield Popover
         const shieldPageToolsSection = document.createElement('div');
         shieldPageToolsSection.style.cssText =
            'display: none; flex-direction: column; gap: 6px; padding-top: 8px; border-top: 1px solid var(--fnx-border);';

         // Visual Element Picker ("点哪藏哪") controls
         const pickerRow = document.createElement('div');
         pickerRow.style.cssText =
            'display: flex; align-items: center; gap: 6px; width: 100%;';
         const btnPickElement = document.createElement('button');
         btnPickElement.type = 'button';
         btnPickElement.className = 'fnx-pop-btn';
         setBtnIconAndText(btnPickElement, 'target', isZh() ? '点选隐藏页面元素' : 'Pick element to hide');
         btnPickElement.style.cssText = 'flex: 1; width: auto; min-width: 0; font-weight: 600; display: inline-flex; align-items: center; justify-content: center; gap: 5px;';

         const btnResetPickedElements = document.createElement('button');
         btnResetPickedElements.type = 'button';
         btnResetPickedElements.className = 'fnx-pop-btn';
         btnResetPickedElements.style.cssText =
            'width: auto; flex-shrink: 0; padding: 0 10px; display: none;';
         btnResetPickedElements.textContent = 'Restore';

         pickerRow.appendChild(btnPickElement);
         pickerRow.appendChild(btnResetPickedElements);
         shieldPageToolsSection.appendChild(pickerRow);

         const btnOpenSiteBoost = document.createElement('button');
         btnOpenSiteBoost.type = 'button';
         btnOpenSiteBoost.className = 'fnx-pop-btn';
         setBtnIconAndText(
            btnOpenSiteBoost,
            'palette',
            isZh() ? '魔改此网站 (Site Boost)' : 'Site Boost CSS',
         );
         btnOpenSiteBoost.style.cssText =
            'width: 100%; font-weight: 600; display: inline-flex; align-items: center; justify-content: center; gap: 5px; border-color: var(--fnx-accent); color: var(--fnx-fg);';
         btnOpenSiteBoost.onclick = (e) => {
            e.stopPropagation();
            toggleStudioPopover(btnShield, 'boost');
         };
         shieldPageToolsSection.appendChild(btnOpenSiteBoost);
         shieldBody.appendChild(shieldPageToolsSection);

         // Popover 3.2: Dedicated Site Info, Security & Permissions (Left Lock Icon)
         siteInfoPopoverEl = document.createElement('div');
         siteInfoPopoverEl.id = '__fynexo_site_popover__';
         siteInfoPopoverEl.className = 'fnx-nav-popover fnx-nav-popover--panel';
         siteInfoPopoverEl.style.width = '272px';

         const siteInfoHeader = document.createElement('div');
         siteInfoHeader.className = 'fnx-pop-head';
         const siteInfoHeadLeft = document.createElement('div');
         siteInfoHeadLeft.className = 'fnx-pop-title';
         const siteInfoIconSlot = document.createElement('span');
         siteInfoIconSlot.style.cssText = 'display: inline-flex; align-items: center; justify-content: center; flex-shrink: 0;';
         siteInfoIconSlot.appendChild(createIcon('lock'));
         const siteInfoTitleTxt = document.createElement('span');
         siteInfoTitleTxt.textContent = 'Connection is secure';
         siteInfoHeadLeft.appendChild(siteInfoIconSlot);
         siteInfoHeadLeft.appendChild(siteInfoTitleTxt);
         const siteInfoProtoBadge = document.createElement('span');
         siteInfoProtoBadge.className = 'fnx-pop-sub';
         siteInfoProtoBadge.style.cssText =
            'font-size: 10.5px; font-weight: 600; padding: 1px 6px; border-radius: 5px; background: var(--fnx-surface); border: 1px solid var(--fnx-border); flex-shrink: 0;';
         siteInfoHeader.appendChild(siteInfoHeadLeft);
         siteInfoHeader.appendChild(siteInfoProtoBadge);
         siteInfoPopoverEl.appendChild(siteInfoHeader);

         const siteInfoBody = document.createElement('div');
         siteInfoBody.className = 'fnx-pop-body';

         const siteInfoHostBlock = document.createElement('div');
         siteInfoHostBlock.style.cssText = 'display: flex; flex-direction: column; gap: 2px;';
         const siteInfoHostEl = document.createElement('div');
         siteInfoHostEl.className = 'fnx-pop-label';
         siteInfoHostEl.style.cssText = 'overflow: hidden; text-overflow: ellipsis; white-space: nowrap;';
         const siteInfoDescEl = document.createElement('div');
         siteInfoDescEl.className = 'fnx-pop-sub';
         siteInfoHostBlock.appendChild(siteInfoHostEl);
         siteInfoHostBlock.appendChild(siteInfoDescEl);
         siteInfoBody.appendChild(siteInfoHostBlock);

         // Per-Site Overrides (Dark Mode + Zoom)
         const siteOvSection = document.createElement('div');
         siteOvSection.style.cssText =
            'display: none; flex-direction: column; gap: 7px; padding-top: 8px; border-top: 1px solid var(--fnx-border);';

         function applySiteOverridePatch(patch) {
            const host = resolveSiteOverrideKey();
            if (!host) return;
            config.site_overrides = config.site_overrides || {};
            const cur = Object.assign(
               { dark_mode: '' },
               config.site_overrides[host] || {},
            );
            if (Object.prototype.hasOwnProperty.call(patch, 'dark_mode')) {
               cur.dark_mode =
                  patch.dark_mode === 'auto'
                     ? ''
                     : String(patch.dark_mode || '');
            }
            if (Object.prototype.hasOwnProperty.call(patch, 'zoom')) {
               if (patch.zoom === null || patch.zoom === undefined) {
                  delete cur.zoom;
               } else {
                  cur.zoom = Math.max(
                     0.5,
                     Math.min(2.5, Math.round(Number(patch.zoom) * 100) / 100),
                  );
               }
            }
            if (
               (cur.zoom === undefined || cur.zoom === null) &&
               (!cur.dark_mode || cur.dark_mode === 'auto')
            ) {
               delete config.site_overrides[host];
            } else {
               config.site_overrides[host] = cur;
            }
            applyPageRules();
            syncShieldPopoverUI();
            sendIpc({
               type: 'PATCH_HUD_PREFS',
               site_override: Object.assign({ domain: host }, patch),
            });
         }

         const dmRow = document.createElement('div');
         dmRow.className = 'fnx-pop-row';
         const dmLabel = document.createElement('span');
         dmLabel.className = 'fnx-pop-label';
         dmLabel.textContent = 'Site dark';
         const dmSeg = document.createElement('div');
         dmSeg.className = 'fnx-pop-seg';
         dmSeg.style.gridTemplateColumns = 'repeat(3, 1fr)';
         const dmBtns = ['auto', 'on', 'off'].map((mode) => {
            const b = document.createElement('button');
            b.type = 'button';
            b.className = 'fnx-pop-seg-btn';
            b.dataset.dm = mode;
            b.textContent =
               mode === 'on' ? 'On' : mode === 'off' ? 'Off' : 'Auto';
            b.onclick = (e) => {
               e.stopPropagation();
               applySiteOverridePatch({ dark_mode: mode });
            };
            dmSeg.appendChild(b);
            return b;
         });
         dmRow.appendChild(dmLabel);
         dmRow.appendChild(dmSeg);
         siteOvSection.appendChild(dmRow);

         const zmRow = document.createElement('div');
         zmRow.className = 'fnx-pop-row';
         const zmLabel = document.createElement('span');
         zmLabel.className = 'fnx-pop-label';
         zmLabel.textContent = 'Site zoom';
         const zmSeg = document.createElement('div');
         zmSeg.className = 'fnx-pop-seg';
         zmSeg.style.gridTemplateColumns = '28px 46px 28px';
         const btnSiteZoomOut = document.createElement('button');
         btnSiteZoomOut.type = 'button';
         btnSiteZoomOut.className = 'fnx-pop-seg-btn';
         btnSiteZoomOut.textContent = '-';
         btnSiteZoomOut.title = 'Decrease zoom for this site';
         btnSiteZoomOut.onclick = (e) => {
            e.stopPropagation();
            const ov = getCurrentSiteOverride();
            const base =
               ov && typeof ov.zoom === 'number'
                  ? ov.zoom
                  : Number(config.default_zoom) || 1.0;
            const next = Math.max(0.5, Number((base - 0.1).toFixed(1)));
            applySiteOverridePatch({ zoom: next });
         };
         const btnSiteZoomReset = document.createElement('button');
         btnSiteZoomReset.type = 'button';
         btnSiteZoomReset.className = 'fnx-pop-seg-btn';
         btnSiteZoomReset.textContent = 'Auto';
         btnSiteZoomReset.title = 'Reset per-site zoom to global default';
         btnSiteZoomReset.onclick = (e) => {
            e.stopPropagation();
            applySiteOverridePatch({ zoom: null });
         };
         const btnSiteZoomIn = document.createElement('button');
         btnSiteZoomIn.type = 'button';
         btnSiteZoomIn.className = 'fnx-pop-seg-btn';
         btnSiteZoomIn.textContent = '+';
         btnSiteZoomIn.title = 'Increase zoom for this site';
         btnSiteZoomIn.onclick = (e) => {
            e.stopPropagation();
            const ov = getCurrentSiteOverride();
            const base =
               ov && typeof ov.zoom === 'number'
                  ? ov.zoom
                  : Number(config.default_zoom) || 1.0;
            const next = Math.min(2.5, Number((base + 0.1).toFixed(1)));
            applySiteOverridePatch({ zoom: next });
         };
         zmSeg.appendChild(btnSiteZoomOut);
         zmSeg.appendChild(btnSiteZoomReset);
         zmSeg.appendChild(btnSiteZoomIn);
         zmRow.appendChild(zmLabel);
         zmRow.appendChild(zmSeg);
         siteOvSection.appendChild(zmRow);

         // Per-Site Permissions Inspector (Camera, Microphone, Location, Notifications, Clipboard)
         const sitePermsWrap = document.createElement('div');
         sitePermsWrap.style.cssText =
            'display: none; flex-direction: column; gap: 5px; padding-top: 8px; border-top: 1px solid var(--fnx-border);';
         const sitePermsHead = document.createElement('div');
         sitePermsHead.className = 'fnx-pop-sub';
         sitePermsHead.style.cssText = 'font-size: 11px; font-weight: 600; color: var(--fnx-secondary);';
         sitePermsWrap.appendChild(sitePermsHead);
         const sitePermsList = document.createElement('div');
         sitePermsList.style.cssText = 'display: flex; flex-direction: column; gap: 4px;';
         sitePermsWrap.appendChild(sitePermsList);
         siteOvSection.appendChild(sitePermsWrap);

         const btnClearSiteData = document.createElement('button');
         btnClearSiteData.type = 'button';
         btnClearSiteData.className = 'fnx-pop-btn';
         setBtnIconAndText(
            btnClearSiteData,
            'broom',
            isZh() ? '清理本站 Cookie 与缓存' : 'Clear site cookies & storage',
         );
         btnClearSiteData.style.cssText =
            'width: 100%; margin-top: 2px; font-weight: 500; display: inline-flex; align-items: center; justify-content: center; gap: 5px;';
         function clearCurrentSiteStorageLocal() {
            try {
               const cookies = String(document.cookie || '').split(';');
               const hostParts = String(window.location.hostname || '').split('.');
               for (let i = 0; i < cookies.length; i++) {
                  const eq = cookies[i].indexOf('=');
                  const name = (eq > -1 ? cookies[i].slice(0, eq) : cookies[i]).trim();
                  if (!name) continue;
                  document.cookie = name + '=;expires=Thu, 01 Jan 1970 00:00:00 GMT;path=/';
                  for (let d = 0; d < hostParts.length; d++) {
                     const dom = hostParts.slice(d).join('.');
                     if (dom) {
                        document.cookie =
                           name +
                           '=;expires=Thu, 01 Jan 1970 00:00:00 GMT;path=/;domain=.' +
                           dom;
                     }
                  }
               }
            } catch (_) {}
            try {
               window.localStorage.clear();
            } catch (_) {}
            try {
               window.sessionStorage.clear();
            } catch (_) {}
            try {
               if (window.caches && typeof window.caches.keys === 'function') {
                  window.caches.keys().then((keys) => {
                     keys.forEach((k) => window.caches.delete(k));
                  });
               }
            } catch (_) {}
            try {
               if (
                  window.indexedDB &&
                  typeof window.indexedDB.databases === 'function'
               ) {
                  window.indexedDB.databases().then((dbs) => {
                     (dbs || []).forEach((db) => {
                        if (db && db.name) window.indexedDB.deleteDatabase(db.name);
                     });
                  });
               }
            } catch (_) {}
         }

         btnClearSiteData.onclick = (e) => {
            e.stopPropagation();
            if (isInternalAny()) return;
            if (IS_HUD_SHELL) {
               closeAllNavPopovers();
               sendIpc({
                  type: 'CONTENT_ACTION',
                  action: 'clear_site_storage',
                  payload: {},
               });
               showHudToast(
                  isZh()
                     ? '已清理本站 Cookie 与本地存储数据'
                     : 'Cleared cookies & local storage for this site',
               );
               return;
            }
            clearCurrentSiteStorageLocal();
            closeAllNavPopovers();
            showHudToast(
               isZh()
                  ? '已清理本站 Cookie 与本地存储数据'
                  : 'Cleared cookies & local storage for this site',
            );
         };
         siteOvSection.appendChild(btnClearSiteData);
         siteInfoBody.appendChild(siteOvSection);
         siteInfoPopoverEl.appendChild(siteInfoBody);

         const btnOpenSitePermsSettings = document.createElement('button');
         btnOpenSitePermsSettings.type = 'button';
         btnOpenSitePermsSettings.className = 'fnx-pop-footer-row';
         const sitePermsLinkLabel = document.createElement('span');
         sitePermsLinkLabel.textContent = 'All site permissions';
         const sitePermsLinkArrow = document.createElement('span');
         sitePermsLinkArrow.style.cssText = 'opacity: 0.65; font-size: 12px;';
         sitePermsLinkArrow.textContent = '→';
         btnOpenSitePermsSettings.appendChild(sitePermsLinkLabel);
         btnOpenSitePermsSettings.appendChild(sitePermsLinkArrow);
         btnOpenSitePermsSettings.onclick = () => {
            closeAllNavPopovers();
            sendIpc({ type: 'NAVIGATE', url: 'fynexo://settings#sec-privacy' });
         };
         siteInfoPopoverEl.appendChild(btnOpenSitePermsSettings);

         function resolveCurrentPermissionOrigin() {
            const cur = canonicalCurrentUrl();
            if (!cur || !/^https?:\/\//i.test(cur)) return '';
            try {
               const u = new URL(cur);
               if (!u.hostname || u.hostname.toLowerCase() === 'fynexo.localhost') return '';
               return (u.protocol + '//' + u.host).toLowerCase();
            } catch (_) {
               return '';
            }
         }

         function renderSitePermsControls(zh) {
            const origin = resolveCurrentPermissionOrigin();
            if (!origin) {
               sitePermsWrap.style.display = 'none';
               return;
            }
            sitePermsWrap.style.display = 'flex';
            sitePermsHead.textContent = zh ? '本站权限控制' : 'Site permissions';
            sitePermsList.textContent = '';
            const curPerms = (sitePermissions && sitePermissions[origin]) || {};
            const defs = [
               ['camera', zh ? '摄像头' : 'Camera'],
               ['microphone', zh ? '麦克风' : 'Microphone'],
               ['geolocation', zh ? '位置信息' : 'Location'],
               ['notifications', zh ? '通知提醒' : 'Notifications'],
               ['clipboard', zh ? '剪贴板读取' : 'Clipboard'],
            ];
            for (let i = 0; i < defs.length; i++) {
               const [kind, label] = defs[i];
               const row = document.createElement('div');
               row.className = 'fnx-pop-row';
               row.style.padding = '1px 0';
               const lbl = document.createElement('span');
               lbl.className = 'fnx-pop-sub';
               lbl.style.color = 'var(--fnx-fg)';
               lbl.textContent = label;
               const sel = document.createElement('select');
               sel.style.cssText =
                  'height: 22px; font-size: 11px; padding: 0 6px; border-radius: 6px; border: 1px solid var(--fnx-border); background: var(--fnx-surface); color: var(--fnx-fg); outline: none; cursor: pointer;';
               const opts = [
                  ['ask', zh ? '询问' : 'Ask'],
                  ['allow', zh ? '允许' : 'Allow'],
                  ['deny', zh ? '拒绝' : 'Block'],
               ];
               const curVal = curPerms[kind] || 'ask';
               for (let j = 0; j < opts.length; j++) {
                  const o = document.createElement('option');
                  o.value = opts[j][0];
                  o.textContent = opts[j][1];
                  if (opts[j][0] === curVal) o.selected = true;
                  sel.appendChild(o);
               }
               sel.onclick = (ev) => ev.stopPropagation();
               sel.onchange = (ev) => {
                  ev.stopPropagation();
                  const nextState = sel.value || 'ask';
                  if (!sitePermissions || typeof sitePermissions !== 'object') sitePermissions = {};
                  if (!sitePermissions[origin]) sitePermissions[origin] = {};
                  if (nextState === 'ask') {
                     delete sitePermissions[origin][kind];
                  } else {
                     sitePermissions[origin][kind] = nextState;
                  }
                  sendIpc({
                     type: 'SET_SITE_PERMISSION',
                     origin: origin,
                     kind: kind,
                     state: nextState,
                  });
               };
               row.appendChild(lbl);
               row.appendChild(sel);
               sitePermsList.appendChild(row);
            }
         }

         function syncSiteInfoPopoverUI() {
            const zh = isZh();
            const internal = isInternalAny();
            const uParts = getCurrentUrlParts();
            const localDev = !internal && isLocalDevHost(uParts.hostname || '');
            const unsafeHttp = !internal && isInsecureExternalHttp(canonicalCurrentUrl());
            const displayHost =
               (uParts.host || uParts.hostname || '').replace(/^www\./, '');

            siteInfoIconSlot.textContent = '';
            if (internal) {
               siteInfoIconSlot.appendChild(createIcon('spark'));
               siteInfoTitleTxt.textContent = zh ? 'Fynexo 内置页面' : 'Fynexo Studio Page';
               siteInfoProtoBadge.textContent = 'Internal';
               siteInfoProtoBadge.style.color = 'var(--fnx-accent)';
               siteInfoHostEl.textContent = canonicalCurrentUrl() || 'fynexo://';
               siteInfoDescEl.textContent = zh ? '系统内置页面，运行于本地安全沙箱。' : 'Built-in browser page.';
            } else if (localDev) {
               siteInfoIconSlot.appendChild(createIcon('code'));
               siteInfoTitleTxt.textContent = zh ? '本地开发服务' : 'Local Development';
               siteInfoProtoBadge.textContent = 'Local';
               siteInfoProtoBadge.style.color = 'var(--fnx-accent)';
               siteInfoHostEl.textContent = displayHost || 'localhost';
               siteInfoDescEl.textContent = zh ? '本地端口自动豁免强制深色与广告拦截。' : 'Auto-exempt from adblock & auto-dark.';
            } else if (unsafeHttp) {
               siteInfoIconSlot.appendChild(createIcon('globe'));
               siteInfoIconSlot.style.color = '#F59E0B';
               siteInfoTitleTxt.textContent = zh ? '连接未加密' : 'Not secure';
               siteInfoProtoBadge.textContent = 'HTTP';
               siteInfoProtoBadge.style.color = '#F59E0B';
               siteInfoHostEl.textContent = displayHost || (zh ? '当前网站' : 'Current site');
               siteInfoDescEl.textContent = zh ? '此站点使用明文 HTTP，请勿输入敏感密码。' : 'Unencrypted HTTP connection.';
            } else {
               siteInfoIconSlot.appendChild(createIcon('lock'));
               siteInfoIconSlot.style.color = '#34D399';
               siteInfoTitleTxt.textContent = zh ? '连接安全' : 'Connection is secure';
               siteInfoProtoBadge.textContent = 'HTTPS';
               siteInfoProtoBadge.style.color = '#34D399';
               siteInfoHostEl.textContent = displayHost || (zh ? '当前网站' : 'Current site');
               siteInfoDescEl.textContent = zh ? '与该站点之间的数据传输已加密。' : 'Encrypted HTTPS connection.';
            }

            dmLabel.textContent = zh ? '本页夜间' : 'Site dark';
            zmLabel.textContent = zh ? '本页缩放' : 'Site zoom';
            sitePermsLinkLabel.textContent = zh ? '管理所有网站权限' : 'All site permissions';

            const siteHost = resolveSiteOverrideKey();
            if (internal || !siteHost) {
               siteOvSection.style.display = 'none';
            } else {
               siteOvSection.style.display = 'flex';
               const ov = getCurrentSiteOverride();
               const curDm =
                  ov && ov.dark_mode
                     ? String(ov.dark_mode).toLowerCase()
                     : 'auto';
               for (let j = 0; j < dmBtns.length; j++) {
                  const m = dmBtns[j].dataset.dm;
                  dmBtns[j].textContent = zh
                     ? (m === 'on' ? '开启' : m === 'off' ? '关闭' : '自动')
                     : (m === 'on' ? 'On' : m === 'off' ? 'Off' : 'Auto');
                  dmBtns[j].classList.toggle('active', m === curDm);
               }
               const hasZoom =
                  ov &&
                  typeof ov.zoom === 'number' &&
                  ov.zoom >= 0.5 &&
                  ov.zoom <= 2.5;
               btnSiteZoomReset.textContent = hasZoom
                  ? Math.round(ov.zoom * 100) + '%'
                  : (zh ? '自动' : 'Auto');
               btnSiteZoomReset.classList.toggle('active', !hasZoom);
               setBtnIconAndText(
                  btnClearSiteData,
                  'broom',
                  zh ? '清理本站 Cookie 与缓存' : 'Clear site cookies & storage',
               );
               btnClearSiteData.style.cssText =
                  'width: 100%; margin-top: 2px; font-weight: 500; display: inline-flex; align-items: center; justify-content: center; gap: 5px;';
               renderSitePermsControls(zh);
            }
         }

         function pickedRuleIdForHost(rawHost) {
            const dom = String(rawHost || '')
               .trim()
               .toLowerCase()
               .replace(/^www\./, '');
            if (!dom) return '';
            return 'picked-' + dom.replace(/[^a-z0-9-]/g, '-');
         }

         function escCssIdent(raw) {
            if (window.CSS && typeof window.CSS.escape === 'function') {
               return window.CSS.escape(String(raw || ''));
            }
            return String(raw || '').replace(/[^a-zA-Z0-9_-]/g, '\\$&');
         }

         function buildSingleElemPart(el) {
            if (!el || el.nodeType !== 1) return '';
            const tag = String(el.tagName || 'div').toLowerCase();
            if (tag === 'html' || tag === 'body') return tag;
            if (
               el.id &&
               /^[a-zA-Z][a-zA-Z0-9_-]{1,56}$/.test(el.id) &&
               !/\d{5,}/.test(el.id)
            ) {
               const idSel = '#' + escCssIdent(el.id);
               try {
                  if (document.querySelectorAll(idSel).length === 1) {
                     return idSel;
                  }
               } catch (_) {}
            }
            const cleanClasses = [];
            if (el.classList) {
               for (let i = 0; i < el.classList.length && cleanClasses.length < 2; i++) {
                  const cls = String(el.classList[i] || '').trim();
                  if (/^[a-zA-Z_-][a-zA-Z0-9_-]{1,36}$/.test(cls)) {
                     cleanClasses.push('.' + escCssIdent(cls));
                  }
               }
            }
            let part = tag + cleanClasses.join('');
            const parent = el.parentElement;
            if (parent && parent.children) {
               let sameTagCount = 0;
               let myTagIndex = 0;
               for (let i = 0; i < parent.children.length; i++) {
                  const sib = parent.children[i];
                  if (sib && sib.tagName === el.tagName) {
                     sameTagCount += 1;
                     if (sib === el) myTagIndex = sameTagCount;
                  }
               }
               if (sameTagCount > 1 && myTagIndex > 0) {
                  let matchesMore = true;
                  try {
                     matchesMore =
                        cleanClasses.length === 0 ||
                        parent.querySelectorAll(':scope > ' + part).length > 1;
                  } catch (_) {}
                  if (matchesMore) {
                     part += ':nth-of-type(' + myTagIndex + ')';
                  }
               }
            }
            return part;
         }

         function buildUniqueCssSelector(el) {
            if (!el || el.nodeType !== 1) return '';
            const parts = [];
            let cur = el;
            for (let depth = 0; depth < 3 && cur && cur.nodeType === 1; depth++) {
               const t = String(cur.tagName || '').toLowerCase();
               if (t === 'html' || t === 'body') break;
               const p = buildSingleElemPart(cur);
               if (!p) break;
               parts.unshift(p);
               if (p.charAt(0) === '#') break;
               const candidate = parts.join(' > ');
               try {
                  if (document.querySelectorAll(candidate).length <= 2) {
                     break;
                  }
               } catch (_) {
                  break;
               }
               cur = cur.parentElement;
            }
            return parts.join(' > ').slice(0, 480);
         }

         let pickerActive = false;
         let pickerHighlightBox = null;
         let pickerBanner = null;
         let pickerBannerText = null;
         let pickerBannerTag = null;
         let pickerCancelBtn = null;
         let lastPickerTarget = null;

         function ensurePickerUi() {
            if (!pickerHighlightBox) {
               pickerHighlightBox = document.createElement('div');
               pickerHighlightBox.id = '__fynexo_picker_box__';
               pickerHighlightBox.style.cssText =
                  'position: fixed; pointer-events: none; z-index: 2147483645; border: 2px dashed var(--fnx-accent, #38bdf8); background: rgba(56, 189, 248, 0.16); border-radius: 5px; box-sizing: border-box; transition: left 0.04s ease-out, top 0.04s ease-out, width 0.04s ease-out, height 0.04s ease-out; display: none;';
               shell.appendChild(pickerHighlightBox);
            }
            if (!pickerBanner) {
               pickerBanner = document.createElement('div');
               pickerBanner.id = '__fynexo_picker_banner__';
               pickerBanner.style.cssText =
                  'position: fixed; left: 50%; transform: translateX(-50%); z-index: 2147483646; display: none; align-items: center; gap: 10px; max-width: calc(100vw - 24px); padding: 8px 14px; border-radius: 999px; background: var(--fnx-bg, #141417); color: var(--fnx-fg, #f4f4f6); border: 1px solid var(--fnx-accent, #38bdf8); box-shadow: 0 12px 32px rgba(0,0,0,0.48); font: 600 12px/1.3 system-ui, sans-serif; pointer-events: auto;';
               pickerBannerText = document.createElement('span');
               pickerBannerText.style.cssText =
                  'white-space: nowrap; overflow: hidden; text-overflow: ellipsis;';
               pickerBannerTag = document.createElement('span');
               pickerBannerTag.style.cssText =
                  'font-family: ui-monospace, Consolas, monospace; font-size: 11px; color: var(--fnx-accent, #38bdf8); background: rgba(56, 189, 248, 0.12); padding: 2px 7px; border-radius: 999px; max-width: 170px; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; display: none;';
               pickerCancelBtn = document.createElement('button');
               pickerCancelBtn.type = 'button';
               pickerCancelBtn.style.cssText =
                  'border: 1px solid var(--fnx-border, rgba(255,255,255,0.18)); background: var(--fnx-surface, rgba(255,255,255,0.08)); color: var(--fnx-fg, #f4f4f6); border-radius: 999px; padding: 3px 10px; font-size: 11px; font-weight: 600; cursor: pointer; flex-shrink: 0;';
               pickerCancelBtn.onclick = (ev) => {
                  ev.preventDefault();
                  ev.stopPropagation();
                  stopVisualElementPicker();
               };
               pickerBanner.appendChild(pickerBannerText);
               pickerBanner.appendChild(pickerBannerTag);
               pickerBanner.appendChild(pickerCancelBtn);
               shell.appendChild(pickerBanner);
            }
         }

         function isHudOwnTarget(t) {
            if (!t) return false;
            return t === host || (typeof host.contains === 'function' && host.contains(t));
         }

         function onPickerHover(e) {
            if (!pickerActive || !pickerHighlightBox) return;
            const rawTarget = e.target;
            if (isHudOwnTarget(rawTarget)) {
               pickerHighlightBox.style.display = 'none';
               return;
            }
            const el =
               rawTarget && rawTarget.nodeType === 1
                  ? rawTarget
                  : document.elementFromPoint(e.clientX || 0, e.clientY || 0);
            if (
               !el ||
               el.nodeType !== 1 ||
               el === document.documentElement ||
               el === document.body ||
               isHudOwnTarget(el)
            ) {
               pickerHighlightBox.style.display = 'none';
               return;
            }
            const rect = el.getBoundingClientRect();
            if (!rect || rect.width < 4 || rect.height < 4) {
               pickerHighlightBox.style.display = 'none';
               return;
            }
            lastPickerTarget = el;
            pickerHighlightBox.style.left = Math.max(0, rect.left) + 'px';
            pickerHighlightBox.style.top = Math.max(0, rect.top) + 'px';
            pickerHighlightBox.style.width = rect.width + 'px';
            pickerHighlightBox.style.height = rect.height + 'px';
            pickerHighlightBox.style.display = 'block';
            if (pickerBannerTag) {
               const previewTag =
                  el.tagName.toLowerCase() +
                  (el.id
                     ? '#' + String(el.id).slice(0, 18)
                     : el.classList && el.classList[0]
                       ? '.' + String(el.classList[0]).slice(0, 18)
                       : '');
               pickerBannerTag.textContent = previewTag;
               pickerBannerTag.style.display = 'inline-block';
            }
         }

         function onPickerIntercept(e) {
            if (!pickerActive) return;
            if (isHudOwnTarget(e.target)) return;
            e.preventDefault();
            e.stopPropagation();
            if (typeof e.stopImmediatePropagation === 'function') {
               e.stopImmediatePropagation();
            }
         }

         function onPickerClick(e) {
            if (!pickerActive) return;
            if (isHudOwnTarget(e.target)) return;
            e.preventDefault();
            e.stopPropagation();
            if (typeof e.stopImmediatePropagation === 'function') {
               e.stopImmediatePropagation();
            }
            let el =
               e.target && e.target.nodeType === 1
                  ? e.target
                  : document.elementFromPoint(e.clientX || 0, e.clientY || 0);
            if (
               !el ||
               el.nodeType !== 1 ||
               el === document.documentElement ||
               el === document.body ||
               isHudOwnTarget(el)
            ) {
               el = lastPickerTarget;
            }
            if (
               !el ||
               el.nodeType !== 1 ||
               el === document.documentElement ||
               el === document.body
            ) {
               stopVisualElementPicker();
               return;
            }
            const selector = buildUniqueCssSelector(el);
            const domain = resolveSiteOverrideKey();
            if (
               !selector ||
               !domain ||
               selector.length > 512 ||
               /[{};\r\n]/.test(selector)
            ) {
               showHudToast(
                  isZh() ? '无法生成该元素的稳定规则' : 'Could not build selector for this element',
               );
               stopVisualElementPicker();
               return;
            }
            const cssLine = selector + ' { display: none !important; }';
            const ruleId = pickedRuleIdForHost(domain);
            const cleanDom = String(domain).replace(/^www\./, '');
            let existing = enabledRules.find((r) => r && r.id === ruleId);
            if (!existing) {
               existing = {
                  id: ruleId,
                  name: 'Picked hidden elements (' + cleanDom + ')',
                  domain: cleanDom,
                  enabled: true,
                  css: cssLine,
                  js: '',
               };
               enabledRules.push(existing);
            } else {
               existing.enabled = true;
               if (!String(existing.css || '').includes(cssLine)) {
                  existing.css =
                     existing.css && existing.css.trim()
                        ? existing.css.trimEnd() + '\n' + cssLine
                        : cssLine;
               }
            }
            applySiteRules();
            syncShieldPopoverUI();
            sendIpc({
               type: 'PATCH_HUD_PREFS',
               append_site_css_rule: { domain: domain, selector: selector },
            });
            showHudToast(
               (isZh() ? '已永久隐藏区块：' : 'Hidden & saved rule: ') + selector,
            );
            stopVisualElementPicker();
         }

         function onPickerKeydown(e) {
            if (!pickerActive) return;
            if (e.key === 'Escape') {
               e.preventDefault();
               e.stopPropagation();
               stopVisualElementPicker();
            }
         }

         function stopVisualElementPicker() {
            if (!pickerActive) return;
            pickerActive = false;
            lastPickerTarget = null;
            window.removeEventListener('mousemove', onPickerHover, true);
            window.removeEventListener('mouseover', onPickerHover, true);
            window.removeEventListener('mousedown', onPickerIntercept, true);
            window.removeEventListener('click', onPickerClick, true);
            window.removeEventListener('keydown', onPickerKeydown, true);
            if (pickerHighlightBox) pickerHighlightBox.style.display = 'none';
            if (pickerBanner) pickerBanner.style.display = 'none';
            if (pickerBannerTag) pickerBannerTag.style.display = 'none';
            if (IS_CONTENT_TAB && typeof reportContentPageMetaRef === 'function') {
               reportContentPageMetaRef(true);
            }
         }

         function startVisualElementPicker() {
            if (isInternalAny()) return;
            closeAllNavPopovers();
            if (IS_HUD_SHELL) {
               sendIpc({
                  type: 'CONTENT_ACTION',
                  action: 'start_element_picker',
                  payload: {},
               });
               return;
            }
            ensurePickerUi();
            const zh = isZh();
            pickerBanner.style.top = isMobilePlatform()
               ? '16px'
               : IS_CONTENT_TAB
                 ? '14px'
                 : '56px';
            pickerBannerText.textContent = zh
               ? '移动或点击要隐藏的页面区块 · 按 Esc 退出'
               : 'Click any page element to hide it · Press Esc to exit';
            pickerCancelBtn.textContent = zh ? '退出' : 'Cancel';
            pickerBannerTag.style.display = 'none';
            pickerBanner.style.display = 'flex';
            pickerHighlightBox.style.display = 'none';
            if (!pickerActive) {
               pickerActive = true;
               window.addEventListener('mousemove', onPickerHover, true);
               window.addEventListener('mouseover', onPickerHover, true);
               window.addEventListener('mousedown', onPickerIntercept, true);
               window.addEventListener('click', onPickerClick, true);
               window.addEventListener('keydown', onPickerKeydown, true);
               if (IS_CONTENT_TAB && typeof reportContentPageMetaRef === 'function') {
                  reportContentPageMetaRef(true);
               }
            }
         }

         startVisualElementPickerRef = startVisualElementPicker;
         btnPickElement.onclick = (e) => {
            e.stopPropagation();
            startVisualElementPicker();
         };
         btnResetPickedElements.onclick = (e) => {
            e.stopPropagation();
            const hostDom = resolveSiteOverrideKey();
            if (!hostDom) return;
            const ruleId = pickedRuleIdForHost(hostDom);
            enabledRules = enabledRules.filter((r) => !r || r.id !== ruleId);
            applySiteRules();
            sendIpc({
               type: 'PATCH_HUD_PREFS',
               clear_picked_site_rules: hostDom,
            });
            syncShieldPopoverUI();
            showHudToast(
               isZh() ? '已恢复本页点选隐藏的元素' : 'Restored picked elements on this site',
            );
         };

         function syncShieldPopoverUI() {
            const zh = isZh();
            const internal = isInternalAny();
            const adOn = config.adblock !== false;
            const sfOn = config.ext_userscript_engine !== false;
            shieldHTxt.textContent = zh ? '隐私拦截与脚本' : 'Adblock Shield';
            shieldCountDesc.textContent = zh ? '当前页面已拦截广告' : 'Blocked elements';
            shieldToggleLabel.textContent = zh ? '启用广告拦截' : 'Protection enabled';
            sfToggleLabel.textContent = zh ? 'ScriptForge 用户脚本' : 'ScriptForge userscripts';
            dmLabel.textContent = zh ? '本页夜间' : 'Site dark';
            zmLabel.textContent = zh ? '本页缩放' : 'Site zoom';
            const manageSpan = shieldPopoverEl.querySelector('.fnx-pop-footer-row > span');
            if (manageSpan) {
               manageSpan.textContent = zh ? '管理规则与白名单' : 'Manage rules & whitelist';
            }

            const uParts = getCurrentUrlParts();
            const localDev = !internal && isLocalDevHost(uParts.hostname || '');
            const unsafeHttp = !internal && isInsecureExternalHttp(canonicalCurrentUrl());
            const displayHost =
               (uParts.host || uParts.hostname || '').replace(
                  /^www\./,
                  '',
               );
            shieldHostSub.textContent = internal
               ? (zh ? '系统页面 · 无需拦截' : 'System page · Exempt')
               : displayHost || (zh ? '当前页面' : 'this page');
            shieldStatusMeta.textContent = internal
               ? (zh ? '已豁免' : 'Exempt')
               : localDev
                 ? (zh ? '本地开发 · 免误拦/免反色' : 'Local Dev · Auto-exempt')
                 : unsafeHttp
                   ? (zh ? 'HTTP 明文连接（不安全）' : 'HTTP Not Secure')
                   : adOn
                     ? (zh ? 'HTTPS · 65+ 拦截规则' : 'HTTPS · 65+ rules')
                     : (zh ? '已暂停' : 'Paused');
            shieldStatusMeta.style.color = unsafeHttp ? '#F59E0B' : 'var(--fnx-muted)';
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

            sfSwitch.disabled = internal;
            sfSwitch.classList.toggle('on', !internal && sfOn);
            sfSwitch.setAttribute(
               'aria-checked',
               !internal && sfOn ? 'true' : 'false',
            );
            sfSwitch.setAttribute(
               'aria-disabled',
               internal ? 'true' : 'false',
            );
            sfToggleLabel.style.color = internal
               ? 'var(--fnx-muted)'
               : 'var(--fnx-fg)';
            const scriptCount = IS_HUD_SHELL
               ? Number(remotePageMeta.script_count) || 0
               : activePageScriptCount;
            sfScriptCountSub.textContent = internal
               ? (zh ? '系统页面 · 无需运行' : 'System page · Exempt')
               : sfOn
                 ? (zh
                    ? scriptCount + ' 个脚本运行中'
                    : scriptCount +
                      (scriptCount === 1
                         ? ' script active'
                         : ' scripts active'))
                 : (zh
                    ? '已暂停 · 匹配 ' + scriptCount + ' 个'
                    : 'Paused · ' + scriptCount + ' matched');

            const siteHost = resolveSiteOverrideKey();
            if (internal || !siteHost) {
               shieldPageToolsSection.style.display = 'none';
            } else {
               shieldPageToolsSection.style.display = 'flex';
               const pickedId = pickedRuleIdForHost(siteHost);
               const hasPickedRule = Boolean(
                  pickedId &&
                     enabledRules.some(
                        (r) =>
                           r &&
                           r.enabled &&
                           r.id === pickedId &&
                           String(r.css || '').trim(),
                     ),
               );
               setBtnIconAndText(
                  btnPickElement,
                  'target',
                  zh ? '点选隐藏页面元素' : 'Pick element to hide',
               );
               btnPickElement.style.cssText = 'flex: 1; width: auto; min-width: 0; font-weight: 600; display: inline-flex; align-items: center; justify-content: center; gap: 5px;';
               btnResetPickedElements.textContent = zh ? '恢复已藏' : 'Restore';
               btnResetPickedElements.title = zh
                  ? '恢复本页所有点选隐藏的元素'
                  : 'Restore picked hidden elements on this site';
               btnResetPickedElements.style.display = hasPickedRule
                  ? 'inline-flex'
                  : 'none';
               setBtnIconAndText(
                  btnOpenSiteBoost,
                  'palette',
                  zh ? '魔改此网站 (Site Boost)' : 'Site Boost CSS',
               );
               btnOpenSiteBoost.style.cssText =
                  'width: 100%; font-weight: 600; display: inline-flex; align-items: center; justify-content: center; gap: 5px; border-color: var(--fnx-accent); color: var(--fnx-fg);';
            }
            syncSiteInfoPopoverUI();

            sfCommandsWrap.textContent = '';
            const sfCmds = IS_HUD_SHELL
               ? (Array.isArray(remotePageMeta.sf_commands)
                    ? remotePageMeta.sf_commands
                    : [])
               : scriptForgeMenuCommands;
            if (!internal && sfOn && sfCmds.length > 0) {
               sfCommandsWrap.style.display = 'flex';
               const cmdHead = document.createElement('div');
               cmdHead.className = 'fnx-pop-sub';
               cmdHead.textContent = zh ? '脚本菜单命令' : 'Script commands';
               sfCommandsWrap.appendChild(cmdHead);
               for (let i = 0; i < sfCmds.length; i++) {
                  const cmd = sfCmds[i];
                  const btn = document.createElement('button');
                  btn.type = 'button';
                  btn.className = 'fnx-pop-btn';
                  btn.style.justifyContent = 'flex-start';
                  btn.textContent = cmd.caption;
                  btn.title = cmd.scriptName || '';
                  btn.onclick = (e) => {
                     e.stopPropagation();
                     closeAllNavPopovers();
                     if (IS_HUD_SHELL) {
                        sendIpc({
                           type: 'CONTENT_ACTION',
                           action: 'run_sf_command',
                           payload: { id: cmd.id },
                        });
                     } else {
                        try {
                           cmd.onClick();
                        } catch (err) {
                           console.warn('[ScriptForge Menu Command Error]:', err);
                        }
                     }
                  };
                  sfCommandsWrap.appendChild(btn);
               }
            } else {
               sfCommandsWrap.style.display = 'none';
            }
         }
         syncShieldPopoverUIRef = syncShieldPopoverUI;
         syncShieldPopoverUI();

         shieldSwitch.onclick = (e) => {
            e.stopPropagation();
            if (isInternalAny()) return;
            config.adblock = config.adblock === false;
            rawBaseConfig.adblock = config.adblock;
            pageManualFeatureOverrides.adblock = config.adblock;
            sendIpc({ type: 'PATCH_HUD_PREFS', adblock: config.adblock });
            applyPageRules();
            syncShieldPopoverUI();
            showHudToast(
               config.adblock
                  ? 'Adblock shield enabled'
                  : 'Adblock shield paused',
            );
         };
         sfSwitch.onclick = (e) => {
            e.stopPropagation();
            if (isInternalAny()) return;
            config.ext_userscript_engine = config.ext_userscript_engine === false;
            rawBaseConfig.ext_userscript_engine = config.ext_userscript_engine;
            pageManualFeatureOverrides.ext_userscript_engine =
               config.ext_userscript_engine;
            sendIpc({
               type: 'PATCH_HUD_PREFS',
               ext_userscript_engine: config.ext_userscript_engine,
            });
            applySiteRules();
            syncShieldPopoverUI();
            showHudToast(
               config.ext_userscript_engine
                  ? 'ScriptForge userscripts enabled'
                  : 'ScriptForge userscripts paused',
            );
         };
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
            sendIpc({ type: 'NAVIGATE', url: 'fynexo://settings#sec-rules' });
         };
         shieldPopoverEl.appendChild(btnOpenAdbStudio);

         function getDetectedAdCount() {
            if (IS_HUD_SHELL) {
               if (isLocalDevHost(getCurrentUrlParts().hostname || '')) return 0;
               return Number(remotePageMeta.blocked_ads) || 0;
            }
            if (isLocalDevHost(window.location.hostname || '')) return 0;
            const ads = document.querySelectorAll(
               'ins.adsbygoogle, [id^="div-gpt-ad"], [id^="google_ads_iframe"], iframe[src*="doubleclick.net"], iframe[src*="googlesyndication.com"]',
            );
            const nativeBlocked =
               window.__fynexo_android__ &&
               typeof window.__fynexo_android__.getPageBlockedAds === 'function'
                  ? Number(window.__fynexo_android__.getPageBlockedAds()) || 0
                  : 0;
            return ads.length + nativeBlocked;
         }

         function updateShieldState() {
            if (isInternalAny()) {
               shieldBadge.style.display = 'none';
               return;
            }
            const count = getDetectedAdCount();
            if (count > 0 && config.adblock !== false) {
               shieldBadge.textContent = count > 99 ? '99+' : String(count);
               shieldBadge.style.display = 'flex';
            } else {
               shieldBadge.style.display = 'none';
            }
            if (IS_CONTENT_TAB && typeof reportContentPageMetaRef === 'function') {
               reportContentPageMetaRef(false);
            }
         }

         // Popover 4: Downloads Tray
         downloadsPopoverEl = document.createElement('div');
         downloadsPopoverEl.id = '__fynexo_downloads_popover__';
         downloadsPopoverEl.className = 'fnx-nav-popover fnx-nav-popover--panel';
         downloadsPopoverEl.style.width = '296px';

         const dlHeader = document.createElement('div');
         dlHeader.className = 'fnx-pop-head';
         const dlHeaderLeft = document.createElement('div');
         dlHeaderLeft.className = 'fnx-pop-title';
         dlHeaderLeft.appendChild(createIcon('download'));
         const dlHTxt = document.createElement('span');
         dlHTxt.textContent = isZh() ? '下载管理' : 'Downloads';
         dlHeaderLeft.appendChild(dlHTxt);
         dlHeader.appendChild(dlHeaderLeft);
         const dlHeaderActions = document.createElement('div');
         dlHeaderActions.style.cssText =
            'display: inline-flex; align-items: center; gap: 8px;';
         const btnClearDlList = document.createElement('button');
         btnClearDlList.type = 'button';
         btnClearDlList.className = 'fnx-pop-link';
         btnClearDlList.textContent = isZh() ? '清空' : 'Clear';
         btnClearDlList.onclick = (ev) => {
            if (ev && ev.stopPropagation) ev.stopPropagation();
            recentDownloads.length = 0;
            syncDownloadsUI();
            closeAllNavPopovers();
         };
         const btnOpenDlFolder = document.createElement('button');
         btnOpenDlFolder.type = 'button';
         btnOpenDlFolder.className = 'fnx-pop-link';
         btnOpenDlFolder.textContent = isZh() ? '打开文件夹' : 'Open folder';
         btnOpenDlFolder.onclick = (ev) => {
            if (ev && ev.isTrusted === false && !isInternalAny()) return;
            if (__fnxAndroidOpenDownloads) {
               closeAllNavPopovers();
               try { __fnxAndroidOpenDownloads(); } catch (_) {}
               return;
            }
            sendIpc({ type: 'OPEN_DOWNLOADS_DIR' });
         };
         dlHeaderActions.appendChild(btnClearDlList);
         dlHeaderActions.appendChild(btnOpenDlFolder);
         dlHeader.appendChild(dlHeaderActions);
         downloadsPopoverEl.appendChild(dlHeader);

         const dlListContainer = document.createElement('div');
         dlListContainer.id = '__fynexo_dl_list__';
         dlListContainer.style.cssText =
            'display: flex; flex-direction: column; gap: 2px; padding: 6px; max-height: 224px; overflow-y: auto;';
         downloadsPopoverEl.appendChild(dlListContainer);

         function formatDlTime(dl) {
            if (dl.updated_at) {
               try {
                  return new Date(Number(dl.updated_at) * 1000).toLocaleTimeString([], {
                     hour: '2-digit',
                     minute: '2-digit',
                  });
               } catch (_) {}
            }
            return dl.time || '';
         }

         function syncDownloadsUI() {
            const zh = isZh();
            dlHTxt.textContent = zh ? '下载管理' : 'Downloads';
            btnClearDlList.textContent = zh ? '清空' : 'Clear';
            btnOpenDlFolder.textContent = zh ? '打开文件夹' : 'Open folder';
            btnDownloads.title = zh ? '下载管理' : 'Downloads';
            if (recentDownloads.length > 0) {
               btnDownloads.style.display = 'flex';
               const activeCount = recentDownloads.filter(
                  (d) => d && d.status === 'downloading',
               ).length;
               dlBadge.textContent = String(
                  activeCount > 0 ? activeCount : recentDownloads.length,
               );
               dlBadge.style.background =
                  activeCount > 0 ? '#10B981' : '';
               dlBadge.style.display = 'flex';
            } else {
               btnDownloads.style.display = 'none';
               dlBadge.style.display = 'none';
            }
            renderDownloadsList();
         }
         syncDownloadsUIRef = syncDownloadsUI;

         function formatBytesShort(n) {
            const v = Number(n) || 0;
            if (v < 1024) return v + ' B';
            if (v < 1048576) return (v / 1024).toFixed(1) + ' KB';
            if (v < 1073741824) return (v / 1048576).toFixed(1) + ' MB';
            return (v / 1073741824).toFixed(2) + ' GB';
         }

         function renderDownloadsList() {
            const zh = isZh();
            dlListContainer.textContent = '';
            if (recentDownloads.length === 0) {
               const emptyTip = document.createElement('div');
               emptyTip.style.cssText =
                  'padding: 18px 8px; text-align: center; color: var(--fnx-muted); font-size: 12px;';
               emptyTip.textContent = zh
                  ? '本次浏览暂无下载记录'
                  : 'No downloads in this session';
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
                  'min-width: 0; flex: 1; display: flex; flex-direction: column; gap: 2px;';
               const nameSpan = document.createElement('span');
               nameSpan.style.cssText =
                  'font-size: 12px; font-weight: 500; white-space: nowrap; overflow: hidden; text-overflow: ellipsis; color: var(--fnx-fg);';
               nameSpan.textContent = dl.name;
               const st = String(dl.status || '').toLowerCase();
               const recv = Number(dl.bytes_received) || 0;
               const total = Number(dl.total_bytes) || 0;
               const spd = Number(dl.speed_bps) || 0;
               const pct = total > 0 ? Math.min(100, Math.round((recv * 100) / total)) : 0;

               let metaLine = '';
               if (st === 'downloading') {
                  const sizePart =
                     total > 0
                        ? formatBytesShort(recv) + ' / ' + formatBytesShort(total) + ' (' + pct + '%)'
                        : recv > 0
                          ? formatBytesShort(recv)
                          : (zh ? '准备中...' : 'Starting...');
                  const spdPart = spd > 0 ? ' · ' + formatBytesShort(spd) + '/s' : '';
                  metaLine = sizePart + spdPart;
               } else if (st === 'cancelled') {
                  const ts = formatDlTime(dl);
                  metaLine = (ts ? ts + ' · ' : '') + (zh ? '已取消' : 'Cancelled');
               } else if (st === 'failed') {
                  const ts = formatDlTime(dl);
                  metaLine = (ts ? ts + ' · ' : '') + (zh ? '下载失败' : 'Failed');
               } else {
                  const ts = formatDlTime(dl);
                  const sz = total > 0 ? formatBytesShort(total) : (recv > 0 ? formatBytesShort(recv) : '');
                  metaLine = [sz, ts, zh ? '已完成' : 'Completed'].filter(Boolean).join(' · ');
               }

               const timeSpan = document.createElement('span');
               timeSpan.className = 'fnx-pop-sub';
               timeSpan.style.fontSize = '10.5px';
               timeSpan.textContent = metaLine;
               nameWrap.appendChild(nameSpan);
               nameWrap.appendChild(timeSpan);

               if (st === 'downloading') {
                  const track = document.createElement('div');
                  track.style.cssText =
                     'width: 100%; height: 3px; border-radius: 99px; background: var(--fnx-border); overflow: hidden; margin-top: 2px;';
                  const fill = document.createElement('div');
                  fill.style.cssText =
                     'height: 100%; border-radius: 99px; background: var(--fnx-accent); transition: width 0.18s ease; width: ' +
                     (total > 0 ? pct : 32) +
                     '%;';
                  track.appendChild(fill);
                  nameWrap.appendChild(track);
               }

               rowLeft.appendChild(nameWrap);
               row.appendChild(rowLeft);

               if (st === 'downloading' && Number(dl.id) > 0) {
                  const btnCancel = document.createElement('button');
                  btnCancel.type = 'button';
                  btnCancel.className = 'fnx-icon-btn';
                  btnCancel.style.cssText =
                     'width: 22px; height: 22px; border-radius: 5px; color: var(--fnx-secondary); flex-shrink: 0;';
                  btnCancel.title = zh ? '取消下载' : 'Cancel download';
                  btnCancel.appendChild(createIcon('closeSmall'));
                  btnCancel.onclick = (ev) => {
                     ev.stopPropagation();
                     if (ev.isTrusted === false && !isInternalAny()) return;
                     sendIpc({ type: 'CANCEL_DOWNLOAD', id: Number(dl.id) });
                  };
                  row.appendChild(btnCancel);
               } else if (st === 'completed') {
                  const actWrap = document.createElement('div');
                  actWrap.style.cssText =
                     'display: inline-flex; align-items: center; gap: 4px; flex-shrink: 0;';
                  const openSpan = document.createElement('span');
                  openSpan.className = 'fnx-pop-dl-open';
                  openSpan.textContent = zh ? '打开' : 'Open';
                  actWrap.appendChild(openSpan);

                  if (!isMobilePlatform()) {
                     const btnReveal = document.createElement('button');
                     btnReveal.type = 'button';
                     btnReveal.className = 'fnx-icon-btn';
                     btnReveal.style.cssText =
                        'width: 22px; height: 22px; border-radius: 5px; color: var(--fnx-secondary);';
                     btnReveal.title = zh
                        ? '在文件夹中显示'
                        : 'Show in folder';
                     btnReveal.appendChild(createIcon('folder'));
                     btnReveal.onclick = (ev) => {
                        ev.stopPropagation();
                        if (ev.isTrusted === false && !isInternalAny()) return;
                        if (__fnxAndroidOpenDownloads) {
                           closeAllNavPopovers();
                           try { __fnxAndroidOpenDownloads(); } catch (_) {}
                           return;
                        }
                        sendIpc({
                           type: 'REVEAL_DOWNLOAD_FILE',
                           id: Number(dl.id) || 0,
                           filename: dl.name,
                        });
                     };
                     actWrap.appendChild(btnReveal);
                  }
                  row.appendChild(actWrap);
               }
               row.onclick = (ev) => {
                  if (ev && ev.isTrusted === false && !isInternalAny()) return;
                  if (__fnxAndroidOpenDownloads) {
                     closeAllNavPopovers();
                     try { __fnxAndroidOpenDownloads(); } catch (_) {}
                     return;
                  }
                  if (st === 'completed') {
                     sendIpc({
                        type: 'OPEN_DOWNLOAD_FILE',
                        id: Number(dl.id) || 0,
                        filename: dl.name,
                     });
                  } else {
                     sendIpc({ type: 'OPEN_DOWNLOADS_DIR' });
                  }
               };
               dlListContainer.appendChild(row);
            });
         }
         syncDownloadsUI();

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

         // Popover 3.3: Dedicated Contextual Mode Switcher (Mode Pill)
         modePopoverEl = document.createElement('div');
         modePopoverEl.id = '__fynexo_mode_popover__';
         modePopoverEl.className = 'fnx-nav-popover fnx-nav-popover--panel';
         modePopoverEl.style.width = '248px';

         const modePopHead = document.createElement('div');
         modePopHead.className = 'fnx-pop-head';
         const modePopTitleWrap = document.createElement('div');
         modePopTitleWrap.className = 'fnx-pop-title';
         const modePopIconSlot = document.createElement('span');
         modePopIconSlot.style.cssText = 'display: inline-flex; align-items: center; justify-content: center; flex-shrink: 0;';
         modePopIconSlot.appendChild(createIcon('spark'));
         const modePopTitleTxt = document.createElement('span');
         modePopTitleTxt.textContent = 'Browser Modes';
         modePopTitleWrap.appendChild(modePopIconSlot);
         modePopTitleWrap.appendChild(modePopTitleTxt);
         const modePopStatusPill = document.createElement('span');
         modePopStatusPill.className = 'fnx-pop-sub';
         modePopStatusPill.style.cssText =
            'font-size: 10.5px; font-weight: 600; color: var(--fnx-accent); flex-shrink: 0;';
         modePopHead.appendChild(modePopTitleWrap);
         modePopHead.appendChild(modePopStatusPill);
         modePopoverEl.appendChild(modePopHead);

         const modePopBody = document.createElement('div');
         modePopBody.className = 'fnx-pop-body';
         modePopBody.style.gap = '6px';
         const modePopList = document.createElement('div');
         modePopList.style.cssText = 'display: flex; flex-direction: column; gap: 3px;';
         const modePopFeatsWrap = document.createElement('div');
         modePopFeatsWrap.style.cssText =
            'display: none; flex-wrap: wrap; gap: 4px; padding-top: 7px; border-top: 1px solid var(--fnx-border);';
         modePopBody.appendChild(modePopList);
         modePopBody.appendChild(modePopFeatsWrap);
         modePopoverEl.appendChild(modePopBody);

         const btnOpenModesCfg = document.createElement('button');
         btnOpenModesCfg.type = 'button';
         btnOpenModesCfg.className = 'fnx-pop-footer-row';
         const modesCfgLabel = document.createElement('span');
         modesCfgLabel.textContent = 'Configure modes';
         const modesCfgArrow = document.createElement('span');
         modesCfgArrow.style.cssText = 'opacity: 0.65; font-size: 12px;';
         modesCfgArrow.textContent = '→';
         btnOpenModesCfg.appendChild(modesCfgLabel);
         btnOpenModesCfg.appendChild(modesCfgArrow);
         btnOpenModesCfg.onclick = () => {
            closeAllNavPopovers();
            sendIpc({ type: 'NAVIGATE', url: 'fynexo://settings#sec-modes' });
         };
         modePopoverEl.appendChild(btnOpenModesCfg);

         function renderModePopoverUI() {
            const zh = isZh();
            const pref = String(rawBaseConfig.active_mode || 'auto').toLowerCase();
            modePopTitleTxt.textContent = zh ? '场景模式切换' : 'Browser Modes';
            modesCfgLabel.textContent = zh ? '配置模式与自动规则' : 'Configure modes';
            modePopIconSlot.textContent = '';
            if (activeModeMatch && activeModeMatch.profile) {
               modePopIconSlot.appendChild(
                  createIcon(
                     resolveModeIconName(
                        activeModeMatch.profile.icon,
                        activeModeMatch.profile.id,
                     ),
                  ),
               );
               modePopStatusPill.textContent = activeModeMatch.auto
                  ? (zh ? '域名自动' : 'Auto')
                  : (zh ? '已锁定' : 'Locked');
            } else {
               modePopIconSlot.appendChild(createIcon('spark'));
               modePopStatusPill.textContent =
                  pref === 'default'
                     ? (zh ? '标准' : 'Standard')
                     : (zh ? '自动监听' : 'Auto');
            }

            modePopList.textContent = '';
            const entries = [
               {
                  id: 'auto',
                  name: zh ? '按网站自动切换' : 'Auto by Site',
                  icon: 'spark',
                  sub:
                     activeModeMatch && activeModeMatch.auto && activeModeMatch.profile
                        ? String(
                             (zh
                                ? activeModeMatch.profile.name_zh || activeModeMatch.profile.name_en
                                : activeModeMatch.profile.name_en || activeModeMatch.profile.name_zh) ||
                                activeModeMatch.profile.id,
                          )
                        : '',
               },
               {
                  id: 'default',
                  name: zh ? '标准模式（关闭场景）' : 'Standard (No Mode)',
                  icon: 'globe',
                  sub: '',
               },
            ];
            const profiles = Array.isArray(rawBaseConfig.mode_profiles)
               ? rawBaseConfig.mode_profiles
               : [];
            for (let i = 0; i < profiles.length; i++) {
               const pr = profiles[i];
               if (!pr || !pr.id) continue;
               const nm = String(
                  (zh ? pr.name_zh || pr.name_en : pr.name_en || pr.name_zh) ||
                     pr.name ||
                     pr.id,
               );
               entries.push({
                  id: pr.id,
                  name: nm,
                  icon: resolveModeIconName(pr.icon, pr.id),
                  sub: '',
               });
            }

            for (let i = 0; i < entries.length; i++) {
               const it = entries[i];
               const isSelected = pref === String(it.id).toLowerCase();
               const rowBtn = document.createElement('button');
               rowBtn.type = 'button';
               rowBtn.className = 'fnx-pop-btn';
               rowBtn.style.cssText =
                  'width: 100%; height: 30px; padding: 0 9px; justify-content: flex-start; gap: 7px; font-size: 12px; font-weight: ' +
                  (isSelected ? '600' : '450') +
                  '; border-color: ' +
                  (isSelected ? 'var(--fnx-accent)' : 'transparent') +
                  '; background: ' +
                  (isSelected ? 'var(--fnx-accent-bg)' : 'transparent') +
                  '; color: ' +
                  (isSelected ? 'var(--fnx-fg)' : 'var(--fnx-secondary)') +
                  ';';
               const ic = createIcon(it.icon);
               ic.style.cssText = 'width: 13px; height: 13px; flex-shrink: 0;';
               const lbl = document.createElement('span');
               lbl.style.cssText = 'flex: 1; text-align: left; overflow: hidden; text-overflow: ellipsis; white-space: nowrap;';
               lbl.textContent = it.name;
               rowBtn.appendChild(ic);
               rowBtn.appendChild(lbl);
               if (it.sub) {
                  const sub = document.createElement('span');
                  sub.style.cssText = 'font-size: 10.5px; color: var(--fnx-accent); flex-shrink: 0; max-width: 78px; overflow: hidden; text-overflow: ellipsis; white-space: nowrap;';
                  sub.textContent = it.sub;
                  rowBtn.appendChild(sub);
               } else if (isSelected) {
                  const chk = createIcon('check');
                  chk.style.cssText = 'width: 12px; height: 12px; color: var(--fnx-accent); flex-shrink: 0;';
                  rowBtn.appendChild(chk);
               }
               rowBtn.onclick = (ev) => {
                  ev.stopPropagation();
                  rawBaseConfig.active_mode = it.id;
                  config.active_mode = it.id;
                  pageManualFeatureOverrides = Object.create(null);
                  userFpsHudOverride = null;
                  sendIpc({ type: 'PATCH_HUD_PREFS', active_mode: it.id });
                  syncAllStyles();
                  syncMainMenuUI();
                  renderModePopoverUI();
                  closeAllNavPopovers();
                  showHudToast((zh ? '模式已切换：' : 'Mode: ') + it.name);
               };
               modePopList.appendChild(rowBtn);
            }

            modePopFeatsWrap.textContent = '';
            if (activeModeMatch && activeModeMatch.profile) {
               const pr = activeModeMatch.profile;
               const featDefs = [
                  ['adblock', zh ? '广告拦截' : 'Adblock'],
                  ['force_dark', zh ? '智能夜间' : 'Force Dark'],
                  ['ext_side_panel', zh ? '侧边栏' : 'Side Panel'],
                  ['ext_reader_mode', zh ? '阅读模式' : 'Reader'],
                  ['ext_yt_ad_skipper', zh ? 'YT 跳广告' : 'YT Ad Skip'],
                  ['ext_media_booster', zh ? '音量增益' : 'Audio Boost'],
                  ['ext_userscript_engine', 'ScriptForge'],
                  ['ext_mouse_gestures', zh ? '鼠标手势' : 'Gestures'],
                  ['fps_hud', 'FPS'],
               ];
               let shown = 0;
               for (let i = 0; i < featDefs.length; i++) {
                  const [k, label] = featDefs[i];
                  if (!pr[k]) continue;
                  const chip = document.createElement('span');
                  chip.style.cssText =
                     'font-size: 10px; padding: 2px 6px; border-radius: 5px; background: var(--fnx-surface); border: 1px solid var(--fnx-border); color: var(--fnx-secondary);';
                  chip.textContent = label;
                  modePopFeatsWrap.appendChild(chip);
                  shown++;
               }
               modePopFeatsWrap.style.display = shown > 0 ? 'flex' : 'none';
            } else {
               modePopFeatsWrap.style.display = 'none';
            }
         }
         renderModePopoverUI();

         function toggleSiteInfoPopover(anchorBtn) {
            const isOpen =
               siteInfoPopoverEl &&
               (siteInfoPopoverEl.style.display === 'flex' ||
                  activeNavPopoverEl === siteInfoPopoverEl);
            if (isOpen) {
               closeAllNavPopovers();
            } else {
               closeAllNavPopovers();
               syncSiteInfoPopoverUI();
               positionNavPopover(
                  anchorBtn || addrIcon,
                  siteInfoPopoverEl,
                  isMobilePlatform() ? 'right' : 'left',
               );
               siteInfoPopoverEl.style.display = 'flex';
               activeNavPopoverEl = siteInfoPopoverEl;
            }
         }

         function toggleModePopover(anchorBtn) {
            const isOpen =
               modePopoverEl &&
               (modePopoverEl.style.display === 'flex' ||
                  activeNavPopoverEl === modePopoverEl);
            if (isOpen) {
               closeAllNavPopovers();
            } else {
               closeAllNavPopovers();
               renderModePopoverUI();
               const anchor =
                  anchorBtn ||
                  (modeTopPill && modeTopPill.style.display !== 'none'
                     ? modeTopPill
                     : btnMore);
               positionNavPopover(anchor, modePopoverEl, 'right');
               modePopoverEl.style.display = 'flex';
               activeNavPopoverEl = modePopoverEl;
            }
         }

         function toggleShieldPopover(anchorBtn) {
            const isOpen =
               shieldPopoverEl &&
               (shieldPopoverEl.style.display === 'flex' ||
                  activeNavPopoverEl === shieldPopoverEl);
            if (isOpen) {
               closeAllNavPopovers();
            } else {
               closeAllNavPopovers();
               syncShieldPopoverUI();
               const c = shieldPopoverEl.querySelector(
                  '#__fynexo_shield_pop_count__',
               );
               if (c) c.textContent = String(getDetectedAdCount());
               positionNavPopover(anchorBtn || btnShield, shieldPopoverEl);
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
               const anchor =
                  btnDownloads && btnDownloads.style.display !== 'none'
                     ? btnDownloads
                     : btnMore;
               positionNavPopover(anchor, downloadsPopoverEl);
               downloadsPopoverEl.style.display = 'flex';
               activeNavPopoverEl = downloadsPopoverEl;
            }
         }

         shell.appendChild(mainMenuEl);
         shell.appendChild(mediaPopoverEl);
         shell.appendChild(shieldPopoverEl);
         shell.appendChild(siteInfoPopoverEl);
         shell.appendChild(modePopoverEl);
         shell.appendChild(downloadsPopoverEl);

         // Popover 4.2: Live UI Studio & Site Boost Customizer (Arc Boosts + Vivaldi Quick Studio)
         studioPopoverEl = document.createElement('div');
         studioPopoverEl.id = '__fynexo_studio_popover__';
         studioPopoverEl.className = 'fnx-nav-popover fnx-nav-popover--panel';
         studioPopoverEl.style.width = '324px';

         let studioTabMode = 'ui';
         const studioHead = document.createElement('div');
         studioHead.className = 'fnx-pop-head';
         const studioTitleWrap = document.createElement('div');
         studioTitleWrap.className = 'fnx-pop-title';
         studioTitleWrap.appendChild(createIcon('palette'));
         const studioTitleTxt = document.createElement('span');
         studioTitleWrap.appendChild(studioTitleTxt);
         studioHead.appendChild(studioTitleWrap);

         const studioTabsSeg = document.createElement('div');
         studioTabsSeg.style.cssText =
            'display: inline-flex; background: var(--fnx-surface); border: 1px solid var(--fnx-border); border-radius: 7px; padding: 2px; gap: 2px;';
         const btnTabUi = document.createElement('button');
         btnTabUi.type = 'button';
         btnTabUi.style.cssText =
            'border: none; border-radius: 5px; padding: 3px 8px; font-size: 11px; font-weight: 600; cursor: pointer;';
         const btnTabBoost = document.createElement('button');
         btnTabBoost.type = 'button';
         btnTabBoost.style.cssText =
            'border: none; border-radius: 5px; padding: 3px 8px; font-size: 11px; font-weight: 600; cursor: pointer;';
         studioTabsSeg.appendChild(btnTabUi);
         studioTabsSeg.appendChild(btnTabBoost);
         studioHead.appendChild(studioTabsSeg);
         studioPopoverEl.appendChild(studioHead);

         const studioUiPane = document.createElement('div');
         studioUiPane.style.cssText =
            'display: flex; flex-direction: column; gap: 10px; padding: 10px 12px;';
         const studioBoostPane = document.createElement('div');
         studioBoostPane.style.cssText =
            'display: none; flex-direction: column; gap: 8px; padding: 10px 12px;';

         // --- Pane 1: Live UI Customizer ---
         const uiPresetLbl = document.createElement('div');
         uiPresetLbl.className = 'fnx-pop-sub';
         const uiPresetGrid = document.createElement('div');
         uiPresetGrid.style.cssText =
            'display: grid; grid-template-columns: repeat(2, minmax(0, 1fr)); gap: 5px;';
         const uiPresetsList = [
            ['deck', '经典双栏', 'Layered Deck'],
            ['compact', '极简单栏', 'Compact 1-Row'],
            ['pill', '大圆角胶囊', 'Rounded Pill'],
            ['vertical', '左侧竖排栏', 'Left Sidebar'],
            ['glass', '亚克力毛玻璃', 'Frosted Glass'],
         ];
         const uiPresetBtns = [];
         uiPresetsList.forEach(([key, zhNm, enNm]) => {
            const b = document.createElement('button');
            b.type = 'button';
            b.className = 'fnx-pop-btn';
            b.dataset.preset = key;
            b.style.cssText = 'font-size: 11.5px; padding: 5px 8px; justify-content: center;';
            b.onclick = (ev) => {
               ev.stopPropagation();
               if (key === 'compact') {
                  styles.topbar.layout = 'compact';
                  styles.topbar.height = '36px';
                  styles.topbar.background = 'var(--fnx-panel)';
                  styles.topbar.custom_css = '';
                  styles.tabs.height = '26px';
                  styles.tabs.border_radius = '6px';
                  styles.address_bar.border_radius = '6px';
               } else if (key === 'deck') {
                  styles.topbar.layout = 'deck';
                  styles.topbar.height = '40px';
                  styles.topbar.titlebar_height = '42px';
                  styles.topbar.background = 'var(--fnx-panel)';
                  styles.topbar.custom_css = '';
                  styles.tabs.height = '30px';
                  styles.tabs.border_radius = '8px';
                  styles.address_bar.border_radius = '8px';
               } else if (key === 'pill') {
                  styles.topbar.layout = 'deck';
                  styles.topbar.height = '42px';
                  styles.topbar.titlebar_height = '44px';
                  styles.topbar.button_radius = '999px';
                  styles.tabs.height = '32px';
                  styles.tabs.border_radius = '999px';
                  styles.address_bar.border_radius = '999px';
               } else if (key === 'vertical') {
                  styles.topbar.layout = 'vertical';
                  styles.topbar.vertical_width = '216px';
                  styles.tabs.border_radius = '8px';
               } else if (key === 'glass') {
                  styles.topbar.background = 'color-mix(in srgb, var(--fnx-panel) 76%, transparent)';
                  styles.topbar.custom_css = 'backdrop-filter: blur(16px) saturate(140%); -webkit-backdrop-filter: blur(16px) saturate(140%);';
                  styles.tabs.border_radius = '10px';
                  styles.address_bar.border_radius = '10px';
               }
               syncAllStyles();
               sendIpc({ type: 'PATCH_HUD_PREFS', ui_preset: key });
               showHudToast(isZh() ? `已套用风格：${zhNm}` : `Applied preset: ${enNm}`);
            };
            uiPresetBtns.push([b, zhNm, enNm]);
            uiPresetGrid.appendChild(b);
         });
         studioUiPane.appendChild(uiPresetLbl);
         studioUiPane.appendChild(uiPresetGrid);

         const uiAccentLbl = document.createElement('div');
         uiAccentLbl.className = 'fnx-pop-sub';
         const uiAccentRow = document.createElement('div');
         uiAccentRow.style.cssText = 'display: flex; align-items: center; gap: 6px; flex-wrap: wrap;';
         const accentSwatches = ['#7A88F7', '#38BDF8', '#10B981', '#F43F5E', '#F59E0B', '#A855F7', '#EC4899'];
         const applyAccentLive = (hex) => {
            styles.colors = styles.colors || {};
            styles.colors.use_custom_colors = true;
            styles.colors.accent = hex;
            styles.colors.accent_bg = hex + '26';
            applyShadowStyles();
            sendIpc({ type: 'PATCH_HUD_PREFS', accent_color: hex });
         };
         accentSwatches.forEach((hex) => {
            const sw = document.createElement('button');
            sw.type = 'button';
            sw.title = hex;
            sw.style.cssText = `width: 22px; height: 22px; border-radius: 999px; border: 2px solid rgba(255,255,255,0.22); background: ${hex}; cursor: pointer; flex-shrink: 0;`;
            sw.onclick = (ev) => {
               ev.stopPropagation();
               applyAccentLive(hex);
            };
            uiAccentRow.appendChild(sw);
         });
         const customColorPicker = document.createElement('input');
         customColorPicker.type = 'color';
         customColorPicker.value = '#7a88f7';
         customColorPicker.style.cssText = 'width: 24px; height: 24px; border: none; background: transparent; cursor: pointer; padding: 0;';
         customColorPicker.oninput = () => applyAccentLive(customColorPicker.value);
         uiAccentRow.appendChild(customColorPicker);
         studioUiPane.appendChild(uiAccentLbl);
         studioUiPane.appendChild(uiAccentRow);

         const uiBtnsLbl = document.createElement('div');
         uiBtnsLbl.className = 'fnx-pop-sub';
         const uiBtnsRow = document.createElement('div');
         uiBtnsRow.style.cssText = 'display: flex; flex-wrap: wrap; gap: 5px;';
         const tbToggleDefs = [
            ['copy', '复制按钮', 'Copy Btn', () => styles.topbar.show_copy_btn !== false, (v) => { styles.topbar.show_copy_btn = v; }],
            ['star', '收藏星标', 'Star Btn', () => styles.topbar.show_star_btn !== false, (v) => { styles.topbar.show_star_btn = v; }],
            ['shield', '隐私盾牌', 'Shield Btn', () => styles.topbar.show_shield_btn !== false, (v) => { styles.topbar.show_shield_btn = v; }],
            ['downloads', '下载按钮', 'Downloads', () => styles.topbar.show_downloads_btn !== false, (v) => { styles.topbar.show_downloads_btn = v; }],
         ];
         const tbToggleEls = [];
         tbToggleDefs.forEach(([key, zhTxt, enTxt, getter, setter]) => {
            const b = document.createElement('button');
            b.type = 'button';
            b.className = 'fnx-pop-btn';
            b.style.cssText = 'font-size: 11px; padding: 4px 8px; width: auto;';
            b.onclick = (ev) => {
               ev.stopPropagation();
               const next = !getter();
               setter(next);
               b.style.borderColor = next ? 'var(--fnx-accent)' : '';
               applyShadowStyles();
               sendIpc({ type: 'PATCH_HUD_PREFS', toolbar_btn: { key: key, visible: next } });
            };
            tbToggleEls.push([b, zhTxt, enTxt, getter]);
            uiBtnsRow.appendChild(b);
         });
         studioUiPane.appendChild(uiBtnsLbl);
         studioUiPane.appendChild(uiBtnsRow);

         // --- Pane 2: Arc Boosts Style Live Site CSS Studio ---
         const boostHostBadge = document.createElement('div');
         boostHostBadge.className = 'fnx-pop-sub';
         const boostChipsWrap = document.createElement('div');
         boostChipsWrap.style.cssText = 'display: flex; flex-wrap: wrap; gap: 5px;';
         const boostTextarea = document.createElement('textarea');
         boostTextarea.spellcheck = false;
         boostTextarea.placeholder = '/* Live per-site CSS (applies as you type) */\nbody { font-family: "Inter", sans-serif !important; }';
         boostTextarea.style.cssText =
            'width: 100%; height: 112px; resize: vertical; padding: 8px; border-radius: 8px; border: 1px solid var(--fnx-border-hover); background: var(--fnx-bg); color: var(--fnx-fg); font-family: "JetBrains Mono", ui-monospace, Consolas, monospace; font-size: 11.5px; line-height: 1.45; outline: none;';

         function boostRuleIdForHost(rawHost) {
            const dom = String(rawHost || '').trim().toLowerCase().replace(/^www\./, '');
            if (!dom) return '';
            return 'boost-' + dom.replace(/[^a-z0-9-]/g, '-');
         }

         function applyLiveBoostCss(rawCss, persist) {
            const dom = resolveSiteOverrideKey();
            if (!dom || isInternalAny()) return;
            const cleanCss = String(rawCss || '').replace(/[<>]/g, '').trim();
            const ruleId = boostRuleIdForHost(dom);
            const cleanDom = dom.replace(/^www\./, '');
            if (!cleanCss) {
               enabledRules = enabledRules.filter((r) => !r || r.id !== ruleId);
            } else {
               let existing = enabledRules.find((r) => r && r.id === ruleId);
               if (!existing) {
                  existing = {
                     id: ruleId,
                     name: 'Site Boost (' + cleanDom + ')',
                     domain: cleanDom,
                     enabled: true,
                     css: cleanCss,
                     js: '',
                  };
                  enabledRules.push(existing);
               } else {
                  existing.enabled = true;
                  existing.css = cleanCss;
               }
            }
            applySiteRules();
            if (persist) {
               sendIpc({
                  type: 'PATCH_HUD_PREFS',
                  site_boost: { domain: cleanDom, css: cleanCss },
               });
            } else if (IS_HUD_SHELL) {
               sendIpc({
                  type: 'CONTENT_ACTION',
                  action: 'preview_site_boost',
                  payload: { domain: cleanDom, ruleId: ruleId, css: cleanCss },
               });
            }
         }

         const quickBoostSnippets = [
            ['阅读字体', 'Clean Font', 'body, article, p { font-family: "Inter", system-ui, sans-serif !important; line-height: 1.68 !important; }'],
            ['隐藏侧栏/弹窗', 'Hide Clutter', 'aside, [class*="sidebar" i], [class*="popup" i], [class*="banner" i] { display: none !important; }'],
            ['图片圆角', 'Rounded Media', 'img, video { border-radius: 10px !important; }'],
            ['去灰阶', 'Full Color', 'html, body, * { filter: none !important; -webkit-filter: none !important; }'],
         ];
         const boostChipBtns = [];
         quickBoostSnippets.forEach(([zhNm, enNm, snippet]) => {
            const chip = document.createElement('button');
            chip.type = 'button';
            chip.className = 'fnx-pop-btn';
            chip.style.cssText = 'font-size: 11px; padding: 3px 8px; width: auto;';
            chip.onclick = (ev) => {
               ev.stopPropagation();
               const cur = boostTextarea.value.trim();
               boostTextarea.value = cur ? cur + '\n' + snippet : snippet;
               applyLiveBoostCss(boostTextarea.value, true);
            };
            boostChipBtns.push([chip, zhNm, enNm]);
            boostChipsWrap.appendChild(chip);
         });

         boostTextarea.addEventListener('input', () => {
            applyLiveBoostCss(boostTextarea.value, false);
         });

         const boostActionsRow = document.createElement('div');
         boostActionsRow.style.cssText = 'display: flex; align-items: center; gap: 6px;';
         const btnSaveBoost = document.createElement('button');
         btnSaveBoost.type = 'button';
         btnSaveBoost.className = 'fnx-pop-btn';
         btnSaveBoost.style.cssText = 'flex: 1; justify-content: center; border-color: var(--fnx-accent); font-weight: 600;';
         btnSaveBoost.onclick = (ev) => {
            ev.stopPropagation();
            applyLiveBoostCss(boostTextarea.value, true);
            showHudToast(isZh() ? '已保存本站专属 Boost CSS' : 'Saved Site Boost CSS');
         };
         const btnClearBoost = document.createElement('button');
         btnClearBoost.type = 'button';
         btnClearBoost.className = 'fnx-pop-btn';
         btnClearBoost.style.cssText = 'width: auto; padding: 0 10px;';
         btnClearBoost.onclick = (ev) => {
            ev.stopPropagation();
            boostTextarea.value = '';
            applyLiveBoostCss('', true);
            showHudToast(isZh() ? '已清除本站魔改样式' : 'Cleared Site Boost CSS');
         };
         boostActionsRow.appendChild(btnSaveBoost);
         boostActionsRow.appendChild(btnClearBoost);

         studioBoostPane.appendChild(boostHostBadge);
         studioBoostPane.appendChild(boostChipsWrap);
         studioBoostPane.appendChild(boostTextarea);
         studioBoostPane.appendChild(boostActionsRow);

         studioPopoverEl.appendChild(studioUiPane);
         studioPopoverEl.appendChild(studioBoostPane);

         const studioFooter = document.createElement('button');
         studioFooter.type = 'button';
         studioFooter.className = 'fnx-pop-footer-row';
         const studioFooterTxt = document.createElement('span');
         const studioFooterArr = document.createElement('span');
         studioFooterArr.textContent = '→';
         studioFooter.appendChild(studioFooterTxt);
         studioFooter.appendChild(studioFooterArr);
         studioFooter.onclick = () => {
            closeAllNavPopovers();
            sendIpc({ type: 'NAVIGATE', url: 'fynexo://settings#sec-styles' });
         };
         studioPopoverEl.appendChild(studioFooter);
         shell.appendChild(studioPopoverEl);

         function syncStudioPopoverUI() {
            const zh = isZh();
            studioTitleTxt.textContent = zh ? '外观与站点工作台' : 'UI & Site Studio';
            btnTabUi.textContent = zh ? '外观布局' : 'UI Studio';
            btnTabBoost.textContent = zh ? '魔改此站' : 'Site Boost';
            const isBoost = studioTabMode === 'boost' && !isInternalAny();
            btnTabUi.style.background = !isBoost ? 'var(--fnx-accent-bg)' : 'transparent';
            btnTabUi.style.color = !isBoost ? 'var(--fnx-fg)' : 'var(--fnx-secondary)';
            btnTabBoost.style.background = isBoost ? 'var(--fnx-accent-bg)' : 'transparent';
            btnTabBoost.style.color = isBoost ? 'var(--fnx-fg)' : 'var(--fnx-secondary)';
            studioUiPane.style.display = !isBoost ? 'flex' : 'none';
            studioBoostPane.style.display = isBoost ? 'flex' : 'none';

            uiPresetLbl.textContent = zh ? '一键界面风格与毛玻璃材质' : 'Layout & Material Presets';
            uiPresetBtns.forEach(([b, zhNm, enNm]) => {
               b.textContent = zh ? zhNm : enNm;
            });
            uiAccentLbl.textContent = zh ? '实时强调色 (Accent Color)' : 'Live Accent Color';
            uiBtnsLbl.textContent = zh ? '顶栏快捷按钮开关' : 'Toolbar Buttons Visibility';
            tbToggleEls.forEach(([b, zhTxt, enTxt, getter]) => {
               const on = getter();
               b.textContent = (zh ? zhTxt : enTxt) + (on ? ' · ON' : '');
               b.style.borderColor = on ? 'var(--fnx-accent)' : '';
            });
            const dom = resolveSiteOverrideKey();
            boostHostBadge.textContent = dom
               ? (zh ? `当前站点：${dom}（输入即实时预览）` : `Target: ${dom} (live preview)`)
               : (zh ? '系统内部页面不支持站点注入' : 'Internal page exempt');
            boostChipBtns.forEach(([chip, zhNm, enNm]) => {
               chip.textContent = '+ ' + (zh ? zhNm : enNm);
            });
            const ruleId = boostRuleIdForHost(dom);
            const existingBoost = enabledRules.find((r) => r && r.id === ruleId);
            boostTextarea.value = existingBoost ? existingBoost.css || '' : '';
            btnSaveBoost.textContent = zh ? '保存本站 CSS' : 'Save Site CSS';
            btnClearBoost.textContent = zh ? '重置' : 'Reset';
            studioFooterTxt.textContent = zh ? '打开完整组件工作台 & JSON' : 'Open Full Component Studio';
         }

         btnTabUi.onclick = (ev) => {
            ev.stopPropagation();
            studioTabMode = 'ui';
            syncStudioPopoverUI();
         };
         btnTabBoost.onclick = (ev) => {
            ev.stopPropagation();
            if (isInternalAny()) {
               showHudToast(isZh() ? '请在普通外部网页上使用站点魔改' : 'Open an external website to use Site Boost');
               return;
            }
            studioTabMode = 'boost';
            syncStudioPopoverUI();
         };

         function toggleStudioPopover(anchorBtn, mode) {
            const isOpen =
               studioPopoverEl &&
               (studioPopoverEl.style.display === 'flex' ||
                  activeNavPopoverEl === studioPopoverEl);
            if (isOpen && (!mode || mode === studioTabMode)) {
               closeAllNavPopovers();
            } else {
               closeAllNavPopovers();
               studioTabMode = mode === 'boost' && !isInternalAny() ? 'boost' : 'ui';
               syncStudioPopoverUI();
               positionNavPopover(anchorBtn || btnMore, studioPopoverEl);
               studioPopoverEl.style.display = 'flex';
               activeNavPopoverEl = studioPopoverEl;
            }
         }

         // Popover 4.5: Chrome Extensions Hub + Pinned Topbar Icons + Store One-Click Pill
         chromeExtsPopoverEl = document.createElement('div');
         chromeExtsPopoverEl.id = '__fynexo_exts_popover__';
         chromeExtsPopoverEl.className = 'fnx-nav-popover fnx-nav-popover--panel';
         chromeExtsPopoverEl.style.width = '300px';

         const extsHead = document.createElement('div');
         extsHead.className = 'fnx-pop-head';
         const extsTitleWrap = document.createElement('div');
         extsTitleWrap.className = 'fnx-pop-title';
         extsTitleWrap.appendChild(createIcon('puzzle'));
         const extsTitleTxt = document.createElement('span');
         extsTitleTxt.textContent = isZh() ? 'Chrome 扩展' : 'Chrome Extensions';
         extsTitleWrap.appendChild(extsTitleTxt);
         const extsEngineSub = document.createElement('span');
         extsEngineSub.className = 'fnx-pop-sub';
         extsHead.appendChild(extsTitleWrap);
         extsHead.appendChild(extsEngineSub);
         chromeExtsPopoverEl.appendChild(extsHead);

         const extsListBody = document.createElement('div');
         extsListBody.style.cssText =
            'display: flex; flex-direction: column; gap: 4px; padding: 6px 8px; max-height: 280px; overflow-y: auto;';
         chromeExtsPopoverEl.appendChild(extsListBody);

         const extsFooter = document.createElement('div');
         extsFooter.className = 'fnx-pop-footer';
         const btnManageChromeExts = document.createElement('button');
         btnManageChromeExts.type = 'button';
         btnManageChromeExts.className = 'fnx-pop-btn';
         btnManageChromeExts.textContent = isZh() ? '管理 / 导入' : 'Manage / Import';
         btnManageChromeExts.onclick = () => {
            closeAllNavPopovers();
            sendIpc({ type: 'NAVIGATE', url: 'fynexo://settings#sec-extensions' });
         };
         const btnOpenCwsStore = document.createElement('button');
         btnOpenCwsStore.type = 'button';
         btnOpenCwsStore.className = 'fnx-pop-btn';
         btnOpenCwsStore.textContent = isZh() ? 'Chrome 商店' : 'Web Store';
         btnOpenCwsStore.onclick = () => {
            closeAllNavPopovers();
            sendIpc({ type: 'NEW_TAB', url: 'https://chromewebstore.google.com/' });
         };
         extsFooter.appendChild(btnManageChromeExts);
         extsFooter.appendChild(btnOpenCwsStore);
         chromeExtsPopoverEl.appendChild(extsFooter);
         shell.appendChild(chromeExtsPopoverEl);

         function makeChromeExtAvatar(ext, sizePx) {
            const sz = sizePx || 16;
            if (ext.icon_data_url) {
               const img = document.createElement('img');
               img.src = ext.icon_data_url;
               img.alt = '';
               img.style.cssText = `width: ${sz}px; height: ${sz}px; border-radius: 3px; object-fit: contain; flex-shrink: 0;`;
               return img;
            }
            const badge = document.createElement('span');
            badge.style.cssText = `width: ${sz}px; height: ${sz}px; border-radius: 4px; background: var(--fnx-accent-bg); color: var(--fnx-accent); font-size: ${Math.max(9, sz - 5)}px; font-weight: 700; display: inline-flex; align-items: center; justify-content: center; flex-shrink: 0;`;
            badge.textContent = String(ext.name || 'E').trim().charAt(0).toUpperCase();
            return badge;
         }

         const storeBannerEl = document.createElement('div');
         storeBannerEl.id = '__fynexo_store_pill__';
         storeBannerEl.style.cssText =
            'position: fixed; right: 18px; bottom: 62px; z-index: 2147483645; display: none; align-items: center; gap: 8px; padding: 7px 10px 7px 12px; border-radius: 999px; background: var(--fnx-bg, #141417); color: var(--fnx-fg, #f4f4f6); border: 1px solid var(--fnx-accent, #38bdf8); box-shadow: 0 10px 28px rgba(0,0,0,0.42); font: 600 12px/1.3 system-ui, sans-serif; pointer-events: auto;';
         shell.appendChild(storeBannerEl);
         let dismissedStoreId = '';

         function extractStoreExtIdClient(rawHref) {
            const href = String(rawHref || '').split(/[?#]/)[0];
            let m = href.match(/^https?:\/\/(?:chromewebstore\.google\.com|chrome\.google\.com\/webstore)\/detail\/(?:[^/]+\/)?([a-p]{32})\/?$/i);
            if (m) return { id: m[1].toLowerCase(), store: 'chrome' };
            m = href.match(/^https?:\/\/microsoftedge\.microsoft\.com\/addons\/detail\/(?:[^/]+\/)?([a-p]{32})\/?$/i);
            if (m) return { id: m[1].toLowerCase(), store: 'edge' };
            return null;
         }

         function syncChromeStoreBanner(explicitHref) {
            if (isMobilePlatform() || IS_CONTENT_TAB || !storeBannerEl) {
               if (storeBannerEl) storeBannerEl.style.display = 'none';
               if (IS_HUD_SHELL && typeof syncHudLayoutRef === 'function') syncHudLayoutRef();
               return;
            }
            const curHref = explicitHref || canonicalCurrentUrl();
            const parsed = extractStoreExtIdClient(curHref);
            if (!parsed || parsed.id === dismissedStoreId) {
               storeBannerEl.style.display = 'none';
               if (IS_HUD_SHELL && typeof syncHudLayoutRef === 'function') syncHudLayoutRef();
               return;
            }
            const zh = isZh();
            const installed = chromeExtensions.some(
               (e) => e && (e.id === parsed.id || e.runtime_id === parsed.id),
            );
            storeBannerEl.textContent = '';
            const ic = createIcon('puzzle');
            ic.style.color = installed ? '#34D399' : 'var(--fnx-accent, #38bdf8)';
            storeBannerEl.appendChild(ic);

            const actBtn = document.createElement('button');
            actBtn.type = 'button';
            actBtn.style.cssText =
               'background: transparent; border: none; color: inherit; font: inherit; cursor: pointer; padding: 0 2px;';
            if (installed) {
               actBtn.textContent = zh ? '已安装到 Fynexo（点此管理）' : 'Installed in Fynexo (Manage)';
               actBtn.onclick = () => sendIpc({ type: 'NAVIGATE', url: 'fynexo://settings#sec-extensions' });
            } else {
               actBtn.textContent = zh ? '安装此扩展到 Fynexo' : 'Install to Fynexo';
               actBtn.onclick = (ev) => {
                  if (ev && ev.isTrusted === false && !isInternalAny()) return;
                  sendIpc({ type: 'INSTALL_CHROME_EXT_STORE', input: curHref });
               };
            }
            storeBannerEl.appendChild(actBtn);

            const xBtn = document.createElement('button');
            xBtn.type = 'button';
            xBtn.style.cssText =
               'width: 18px; height: 18px; border-radius: 50%; border: none; background: transparent; color: var(--fnx-muted, #9ca3af); cursor: pointer; font-size: 13px; line-height: 1; display: inline-flex; align-items: center; justify-content: center;';
            xBtn.textContent = '×';
            xBtn.onclick = () => {
               dismissedStoreId = parsed.id;
               storeBannerEl.style.display = 'none';
               if (IS_HUD_SHELL && typeof syncHudLayoutRef === 'function') syncHudLayoutRef();
            };
            storeBannerEl.appendChild(xBtn);

            storeBannerEl.style.display = 'inline-flex';
            if (IS_HUD_SHELL && typeof syncHudLayoutRef === 'function') syncHudLayoutRef();
         }
         syncChromeStoreBannerRef = syncChromeStoreBanner;

         function syncChromeExtsUI() {
            const zh = isZh();
            const showOnDesktop = !isMobilePlatform() && (chromeExtEngineActive || chromeExtensions.length > 0);
            btnChromeExts.style.display = showOnDesktop ? 'flex' : 'none';
            btnChromeExts.title = zh ? 'Chrome 扩展程序' : 'Chrome Extensions';
            extsTitleTxt.textContent = zh ? 'Chrome 扩展' : 'Chrome Extensions';
            extsEngineSub.textContent = chromeExtEngineActive
               ? (zh ? 'WebView2 已就绪' : 'Engine active')
               : (zh ? '需开启并重启' : 'Restart needed');
            extsEngineSub.style.color = chromeExtEngineActive ? '#10B981' : '#F59E0B';
            btnManageChromeExts.textContent = zh ? '管理 / 导入' : 'Manage / Import';
            btnOpenCwsStore.textContent = zh ? 'Chrome 商店' : 'Web Store';

            pinnedExtsBar.textContent = '';
            const pinnedList = showOnDesktop
               ? chromeExtensions.filter((e) => e && e.enabled && e.pinned)
               : [];
            pinnedExtsBar.style.display = pinnedList.length > 0 ? 'flex' : 'none';
            pinnedList.forEach((ext) => {
               const pb = document.createElement('button');
               pb.type = 'button';
               pb.className = 'fnx-nav-btn fnx-popover-trigger';
               pb.title = ext.name;
               pb.appendChild(makeChromeExtAvatar(ext, 16));
               pb.onclick = (ev) => {
                  if (ev && ev.stopPropagation) ev.stopPropagation();
                  closeAllNavPopovers();
                  sendIpc({ type: 'OPEN_CHROME_EXT_POPUP', id: ext.id });
               };
               pinnedExtsBar.appendChild(pb);
            });

            extsListBody.textContent = '';
            if (chromeExtensions.length === 0) {
               const empty = document.createElement('div');
               empty.style.cssText =
                  'padding: 16px 10px; text-align: center; font-size: 12px; color: var(--fnx-muted); line-height: 1.5;';
               empty.textContent = zh
                  ? '暂无已安装的 Chrome 扩展；可点击下方逛 Chrome 商店一键安装，或导入本地 .crx / 文件夹。'
                  : 'No Chrome extensions installed. Browse Web Store or import a local .crx / folder.';
               extsListBody.appendChild(empty);
            } else {
               chromeExtensions.forEach((ext) => {
                  if (!ext) return;
                  const row = document.createElement('div');
                  row.className = 'fnx-pop-row';
                  row.style.cssText =
                     'display: flex; align-items: center; justify-content: space-between; gap: 6px; padding: 5px 6px; border-radius: 7px;';

                  const leftBtn = document.createElement('button');
                  leftBtn.type = 'button';
                  leftBtn.style.cssText =
                     'display: flex; align-items: center; gap: 8px; flex: 1; min-width: 0; background: transparent; border: none; color: inherit; cursor: pointer; text-align: left; padding: 2px 0;';
                  leftBtn.appendChild(makeChromeExtAvatar(ext, 16));
                  const nameCol = document.createElement('div');
                  nameCol.style.cssText = 'min-width: 0; display: flex; flex-direction: column;';
                  const nm = document.createElement('span');
                  nm.style.cssText =
                     'font-size: 12px; font-weight: 500; white-space: nowrap; overflow: hidden; text-overflow: ellipsis; color: var(--fnx-fg);';
                  nm.textContent = ext.name;
                  const meta = document.createElement('span');
                  meta.className = 'fnx-pop-sub';
                  meta.style.fontSize = '10px';
                  meta.textContent = 'v' + (ext.version || '1.0') + (ext.enabled ? '' : (zh ? ' · 已停用' : ' · Disabled'));
                  nameCol.appendChild(nm);
                  nameCol.appendChild(meta);
                  leftBtn.appendChild(nameCol);
                  leftBtn.onclick = () => {
                     closeAllNavPopovers();
                     sendIpc({ type: 'OPEN_CHROME_EXT_POPUP', id: ext.id });
                  };
                  row.appendChild(leftBtn);

                  if (ext.options_path) {
                     const optBtn = document.createElement('button');
                     optBtn.type = 'button';
                     optBtn.className = 'fnx-pop-btn';
                     optBtn.style.padding = '2px 6px';
                     optBtn.style.fontSize = '10.5px';
                     optBtn.textContent = zh ? '选项' : 'Opt';
                     optBtn.onclick = (ev) => {
                        if (ev && ev.isTrusted === false && !isInternalAny()) return;
                        closeAllNavPopovers();
                        sendIpc({ type: 'OPEN_CHROME_EXT_OPTIONS', id: ext.id });
                     };
                     row.appendChild(optBtn);
                  }

                  const pinBtn = document.createElement('button');
                  pinBtn.type = 'button';
                  pinBtn.className = 'fnx-pop-btn';
                  pinBtn.style.padding = '2px 6px';
                  pinBtn.style.fontSize = '10.5px';
                  pinBtn.textContent = ext.pinned ? (zh ? '已固定' : 'Pin') : (zh ? '固定' : 'Pin');
                  if (ext.pinned) pinBtn.style.color = 'var(--fnx-accent)';
                  pinBtn.onclick = (ev) => {
                     if (ev && ev.isTrusted === false && !isInternalAny()) return;
                     sendIpc({ type: 'PIN_CHROME_EXT', id: ext.id });
                  };
                  row.appendChild(pinBtn);

                  const togBtn = document.createElement('button');
                  togBtn.type = 'button';
                  togBtn.className = 'fnx-pop-btn';
                  togBtn.style.padding = '2px 7px';
                  togBtn.style.fontSize = '10.5px';
                  togBtn.textContent = ext.enabled ? (zh ? '开' : 'On') : (zh ? '关' : 'Off');
                  if (ext.enabled) togBtn.style.color = '#10B981';
                  togBtn.onclick = (ev) => {
                     if (ev && ev.isTrusted === false && !isInternalAny()) return;
                     sendIpc({ type: 'TOGGLE_CHROME_EXT', id: ext.id, enabled: !ext.enabled });
                  };
                  row.appendChild(togBtn);

                  extsListBody.appendChild(row);
               });
            }
            syncChromeStoreBanner();
         }
         syncChromeExtsUIRef = syncChromeExtsUI;

         function toggleChromeExtsPopover() {
            const isOpen =
               chromeExtsPopoverEl &&
               (chromeExtsPopoverEl.style.display === 'flex' ||
                  activeNavPopoverEl === chromeExtsPopoverEl);
            if (isOpen) {
               closeAllNavPopovers();
            } else {
               closeAllNavPopovers();
               syncChromeExtsUI();
               positionNavPopover(btnChromeExts, chromeExtsPopoverEl);
               chromeExtsPopoverEl.style.display = 'flex';
               activeNavPopoverEl = chromeExtsPopoverEl;
            }
         }
         syncChromeExtsUI();

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
         const tmTitleTxt = document.createElement('span');
         tmTitleTxt.textContent = isZh() ? '内存与资源监视' : 'Resource Monitor';
         tmTitle.appendChild(tmTitleTxt);
         const tmBadge = document.createElement('span');
         tmBadge.className = 'fnx-pop-sub';
         tmBadge.textContent = isZh() ? '站点隔离' : 'Site-Isolated';
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
            row.__fnxLabelEl = l;
            row.__fnxValueEl = v;
            return row;
         }
         const tmHostRow = makeStatRow(
            isZh() ? '总内存 (Host + 渲染)' : 'Total RAM (Host + Tabs)',
            '-- MB',
            '__fynexo_tm_host__',
         );
         const tmCommitRow = makeStatRow(
            isZh() ? '内核 / 子进程提交' : 'Committed / Procs',
            '-- MB',
            '__fynexo_tm_commit__',
         );
         const tmModeRow = makeStatRow(
            isZh() ? 'Edge 能效压缩' : 'Efficiency Pool',
            config.efficiency_mode !== false
               ? (isZh() ? '已启用' : 'Active')
               : (isZh() ? '已暂停' : 'Off'),
            '__fynexo_tm_mode__',
            config.efficiency_mode !== false ? '#10B981' : 'var(--fnx-muted)',
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
         cacheTitle.textContent = isZh() ? '后台标签缓存上限' : 'Cached tab limit';
         cacheWrap.appendChild(cacheTitle);
         const cacheBtnRow = document.createElement('div');
         cacheBtnRow.className = 'fnx-pop-seg';
         cacheBtnRow.style.gridTemplateColumns = 'repeat(5, 1fr)';
         const cacheOptions = isMobilePlatform()
            ? [
                 { id: 'minimal', label: '1' },
                 { id: 'eco', label: '2' },
                 { id: 'balanced', label: '2+' },
                 { id: 'medium', label: '3' },
                 { id: 'smooth', label: '4' },
              ]
            : [
                 { id: 'minimal', label: '1' },
                 { id: 'eco', label: '2' },
                 { id: 'balanced', label: '3' },
                 { id: 'medium', label: '5' },
                 { id: 'smooth', label: '8' },
              ];
         const cacheBtns = [];
         function syncCacheModeButtons() {
            const cur = String(
               config.tab_cache_mode || 'balanced',
            ).toLowerCase();
            const known = ['minimal', 'eco', 'balanced', 'medium', 'smooth'];
            for (let i = 0; i < cacheBtns.length; i++) {
               const b = cacheBtns[i];
               const active =
                  b.dataset.mode === cur ||
                  (!known.includes(cur) && b.dataset.mode === 'balanced');
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
               config.perf_preset = 'custom';
               syncCacheModeButtons();
               sendIpc({ type: 'PATCH_HUD_PREFS', tab_cache_mode: opt.id, perf_preset: 'custom' });
               showHudToast((isZh() ? '后台缓存上限：' : 'Cached limit: ') + opt.label);
            };
            cacheBtns.push(b);
            cacheBtnRow.appendChild(b);
         }
         cacheWrap.appendChild(cacheBtnRow);
         tmBody.appendChild(cacheWrap);
         taskMgrPopoverEl.appendChild(tmBody);

         const tmFooter = document.createElement('div');
         tmFooter.className = 'fnx-pop-footer';
         const btnPerfSet = document.createElement('button');
         btnPerfSet.type = 'button';
         btnPerfSet.className = 'fnx-pop-btn';
         btnPerfSet.textContent = isZh() ? '性能设置' : 'Performance';
         btnPerfSet.onclick = () => {
            closeAllNavPopovers();
            sendIpc({ type: 'NAVIGATE', url: 'fynexo://settings#sec-performance' });
         };
         const btnTrimMem = document.createElement('button');
         btnTrimMem.type = 'button';
         btnTrimMem.className = 'fnx-pop-btn';
         btnTrimMem.textContent = isZh() ? '释放内存' : 'Flush RAM';
         btnTrimMem.onclick = (ev) => {
            if (ev && ev.isTrusted === false && !isInternalAny()) return;
            sendIpc({ type: 'FORCE_TRIM_MEMORY' });
         };
         tmFooter.appendChild(btnPerfSet);
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
            const m = taskMgrPopoverEl.querySelector('#__fynexo_tm_mode__');
            if (h) {
               const tot = stats.total_mb !== undefined ? stats.total_mb : stats.host_mb;
               if (tot !== undefined) {
                  const winPart = stats.window_count > 1 ? ' (' + stats.window_count + (isZh() ? '窗' : 'w') + ')' : '';
                  h.textContent = Math.round(tot) + ' MB' + winPart;
               }
            }
            if (c && stats.commit_mb !== undefined) {
               const tabsPart = stats.live_tabs !== undefined && stats.total_tabs !== undefined
                  ? ' · ' + stats.live_tabs + '/' + stats.total_tabs + (isZh() ? ' 页' : ' tabs')
                  : '';
               c.textContent = Math.round(stats.commit_mb) + ' MB' + tabsPart;
            }
            if (m) {
               const effOn = config.efficiency_mode !== false;
               m.textContent = effOn ? (isZh() ? '已启用' : 'Active') : (isZh() ? '已暂停' : 'Off');
               m.style.color = effOn ? '#10B981' : 'var(--fnx-muted)';
            }
         };

         shell.appendChild(taskMgrPopoverEl);

         // Tier 3: Slim Shortcuts / Bookmarks Bar (#__fynexo_bm_row__)
         const bmRow = document.createElement('div');
         bmRow.id = '__fynexo_bm_row__';

         const bmStartBtn = document.createElement('button');
         bmStartBtn.type = 'button';
         bmStartBtn.id = '__fynexo_bm_start__';
         bmStartBtn.className = 'fnx-bm-pill';
         bmStartBtn.title = isZh() ? '打开起始页 (fynexo://newtab)' : 'Open Start page (fynexo://newtab)';
         const bmStartIcon = document.createElement('span');
         bmStartIcon.style.cssText =
            'display: inline-flex; align-items: center; color: var(--fnx-secondary);';
         bmStartIcon.appendChild(createIcon('grid'));
         const bmStartText = document.createElement('span');
         bmStartText.textContent = isZh() ? '起始页' : 'Start';
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
         findInput.placeholder = isZh() ? '在页面中查找...' : 'Find in page...';
         findInput.spellcheck = false;
         findInput.autocomplete = 'off';
         const findCount = document.createElement('span');
         findCount.id = '__fynexo_find_count__';
         findCount.textContent = '0/0';

         const btnFindPrev = document.createElement('button');
         btnFindPrev.type = 'button';
         btnFindPrev.className = 'fnx-find-btn';
         btnFindPrev.title = isZh() ? '上一个匹配 (Shift+Enter)' : 'Previous match (Shift+Enter)';
         btnFindPrev.appendChild(createIcon('chevUp'));
         btnFindPrev.onclick = () => stepFindMatch(-1);

         const btnFindNext = document.createElement('button');
         btnFindNext.type = 'button';
         btnFindNext.className = 'fnx-find-btn';
         btnFindNext.title = isZh() ? '下一个匹配 (Enter)' : 'Next match (Enter)';
         btnFindNext.appendChild(createIcon('chevDown'));
         btnFindNext.onclick = () => stepFindMatch(1);

         const btnFindClose = document.createElement('button');
         btnFindClose.type = 'button';
         btnFindClose.className = 'fnx-find-btn';
         btnFindClose.title = isZh() ? '关闭 (Esc)' : 'Close (Esc)';
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

         // 1C. Pure-DOM Reader Mode Overlay (Lazy-mounted when ext_reader_mode is active)
         ensureReaderMountedRef = () => {
            if (shadowRoot.getElementById('__fynexo_reader__')) return;
            const readerOverlay = document.createElement('div');
            readerOverlay.id = '__fynexo_reader__';
            const readerBar = document.createElement('div');
            readerBar.className = 'fnx-rd-toolbar';
            const readerMeta = document.createElement('span');
            readerMeta.id = '__fynexo_reader_meta__';
            readerMeta.className = 'fnx-rd-meta';
            readerMeta.textContent = isZh() ? '纯净阅读模式' : 'Reader mode';

            const readerCtrls = document.createElement('div');
            readerCtrls.className = 'fnx-rd-controls';

            const readerSizeSeg = document.createElement('div');
            readerSizeSeg.className = 'fnx-rd-seg';

            const btnRdSmaller = document.createElement('button');
            btnRdSmaller.type = 'button';
            btnRdSmaller.className = 'fnx-pop-seg-btn';
            btnRdSmaller.title = isZh() ? '缩小字号' : 'Decrease font size';
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
            btnRdLarger.title = isZh() ? '放大字号' : 'Increase font size';
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
            btnRdWidth.title = isZh() ? '切换宽版/窄版栏宽' : 'Toggle wide column';
            btnRdWidth.textContent = isZh() ? '宽版' : 'Wide';
            btnRdWidth.onclick = () => {
               readerWide = !readerWide;
               syncReaderTypography();
            };
            readerWidthSeg.appendChild(btnRdWidth);

            const btnRdExit = document.createElement('button');
            btnRdExit.type = 'button';
            btnRdExit.id = '__fynexo_reader_exit__';
            btnRdExit.className = 'fnx-rd-btn';
            btnRdExit.appendChild(createIcon('close'));
            btnRdExit.appendChild(document.createTextNode(isZh() ? '退出阅读' : 'Exit Reader'));
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
         };

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
         const mobPillProgress = document.createElement('div');
         mobPillProgress.className = 'fnx-pill-progress';
         mbPill.appendChild(mobPillProgress);
         progressBars.push(mobPillProgress);

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

         function bindBottomSheetSwipeDismiss(sheetEl, scrollListEl, onClose) {
            if (!sheetEl || typeof onClose !== 'function') return;
            let tracking = false;
            let dragging = false;
            let startX = 0;
            let startY = 0;
            let lastDy = 0;
            let lastTs = 0;
            let velocity = 0;
            let startedInScrollableList = false;

            const isScrollAtTop = () =>
               !scrollListEl || scrollListEl.scrollTop <= 1;

            sheetEl.addEventListener(
               'touchstart',
               (e) => {
                  if (!isMobilePlatform() && window.innerWidth > 768) return;
                  if (!e.touches || e.touches.length !== 1) return;
                  const targetTag = (
                     (e.target && e.target.tagName) ||
                     ''
                  ).toLowerCase();
                  if (
                     targetTag === 'input' ||
                     targetTag === 'textarea' ||
                     targetTag === 'select'
                  ) {
                     return;
                  }
                  startedInScrollableList = !!(
                     scrollListEl && scrollListEl.contains(e.target)
                  );
                  if (startedInScrollableList && !isScrollAtTop()) return;
                  const touch = e.touches[0];
                  tracking = true;
                  dragging = false;
                  startX = touch.clientX;
                  startY = touch.clientY;
                  lastDy = 0;
                  lastTs = Date.now();
                  velocity = 0;
               },
               { passive: true },
            );

            sheetEl.addEventListener(
               'touchmove',
               (e) => {
                  if (!tracking || !e.touches || e.touches.length !== 1) return;
                  const touch = e.touches[0];
                  const dx = touch.clientX - startX;
                  const dy = touch.clientY - startY;
                  if (!dragging) {
                     if (dy < -2 || Math.abs(dx) > Math.abs(dy) * 1.15) {
                        tracking = false;
                        return;
                     }
                     if (dy > 8) {
                        if (startedInScrollableList && !isScrollAtTop()) {
                           tracking = false;
                           return;
                        }
                        dragging = true;
                        sheetEl.style.transition = 'none';
                     } else {
                        return;
                     }
                  }
                  const pull = Math.max(0, dy - 8);
                  const now = Date.now();
                  const dt = Math.max(1, now - lastTs);
                  velocity = (pull - lastDy) / dt;
                  lastDy = pull;
                  lastTs = now;
                  sheetEl.style.transform =
                     'translate3d(0, ' + Math.round(pull) + 'px, 0)';
                  if (e.cancelable) e.preventDefault();
               },
               { passive: false },
            );

            const finishTouch = () => {
               if (!tracking) return;
               tracking = false;
               if (!dragging) return;
               dragging = false;
               const threshold = Math.min(
                  120,
                  (sheetEl.offsetHeight || 360) * 0.28,
               );
               if (lastDy > threshold || (lastDy > 36 && velocity > 0.45)) {
                  sheetEl.style.transition =
                     'transform 0.16s cubic-bezier(0.22, 1, 0.36, 1)';
                  sheetEl.style.transform = 'translate3d(0, 100%, 0)';
                  setTimeout(() => {
                     sheetEl.style.transition = '';
                     sheetEl.style.transform = '';
                     onClose();
                  }, 150);
               } else {
                  sheetEl.style.transition =
                     'transform 0.18s cubic-bezier(0.16, 1, 0.3, 1)';
                  sheetEl.style.transform = 'translate3d(0, 0, 0)';
                  setTimeout(() => {
                     if (!dragging) {
                        sheetEl.style.transition = '';
                        sheetEl.style.transform = '';
                     }
                  }, 190);
               }
            };

            sheetEl.addEventListener('touchend', finishTouch, { passive: true });
            sheetEl.addEventListener('touchcancel', finishTouch, {
               passive: true,
            });
         }

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
         drawerTitle.id = '__fynexo_drawer_title__';
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
         drawerPrefsTxt.id = '__fynexo_drawer_prefs_txt__';
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
         const drawerNewTxt = document.createElement('span');
         drawerNewTxt.id = '__fynexo_drawer_new_txt__';
         drawerNewTxt.textContent = 'New tab';
         btnDrawerNew.appendChild(drawerNewTxt);
         btnDrawerNew.onclick = () => {
            closeMobileDrawer();
            flushScrollSync();
            if (!IS_HUD_SHELL) triggerProgress();
            sendIpc({ type: 'NEW_TAB' });
         };

         const btnDrawerDone = document.createElement('button');
         btnDrawerDone.id = '__fynexo_drawer_done__';
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
         bindBottomSheetSwipeDismiss(drawerSheet, drawerList, closeMobileDrawer);

         // 3.5 Side Drawer / Web Panel (Alt+S, Lazy-mounted only when ext_side_panel is active)
         isSidePanelOpen = false;
         let sidePanelRefs = null;
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

         function ensureSidePanelMounted() {
            if (sidePanelRefs && sidePanelEl) return sidePanelRefs;
            sidePanelEl = document.createElement('div');
            sidePanelEl.id = '__fynexo_side_panel__';

            const spHeader = document.createElement('div');
            spHeader.className = 'fnx-sp-header';

            const spTabsWrap = document.createElement('div');
            spTabsWrap.className = 'fnx-pop-seg fnx-sp-tabs';

            const spTabItems = [];
            function makeSpTab(iconName, zhLabel, enLabel, isActive) {
               const btn = document.createElement('button');
               btn.type = 'button';
               btn.className = isActive
                  ? 'fnx-pop-seg-btn fnx-sp-tab active'
                  : 'fnx-pop-seg-btn fnx-sp-tab';
               btn.appendChild(createIcon(iconName));
               const txt = document.createTextNode(isZh() ? zhLabel : enLabel);
               btn.appendChild(txt);
               spTabItems.push({ btn, iconName, zhLabel, enLabel });
               return btn;
            }

            const zh = isZh();
            const btnSpWeb = makeSpTab('globe', '小窗网页', 'Web Panel', true);
            const btnSpNotes = makeSpTab('fileText', '随手便签', 'Quick Notes', false);
            const btnSpDev = makeSpTab('code', '开发工具', 'Dev Tools', false);
            spTabsWrap.appendChild(btnSpWeb);
            spTabsWrap.appendChild(btnSpNotes);
            spTabsWrap.appendChild(btnSpDev);
            spHeader.appendChild(spTabsWrap);

            const btnSpClose = document.createElement('button');
            btnSpClose.type = 'button';
            btnSpClose.className = 'fnx-sp-close';
            btnSpClose.title = zh ? '关闭侧边栏 (Esc)' : 'Close side panel (Esc)';
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
            spInput.placeholder = zh ? '输入侧边栏小窗网址...' : 'Enter URL for side panel...';
            const spGoBtn = document.createElement('button');
            spGoBtn.type = 'button';
            spGoBtn.className = 'fnx-sp-go';
            spGoBtn.textContent = zh ? '前往' : 'Go';

            const spReloadBtn = document.createElement('button');
            spReloadBtn.type = 'button';
            spReloadBtn.className = 'fnx-sp-icon-btn';
            spReloadBtn.title = zh ? '刷新小窗网页' : 'Reload side panel';
            spReloadBtn.appendChild(createIcon('reload'));

            const spPopBtn = document.createElement('button');
            spPopBtn.type = 'button';
            spPopBtn.className = 'fnx-sp-icon-btn';
            spPopBtn.title = zh ? '在新标签页中打开' : 'Open in new tab (if blocked in frame)';
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
            spTextarea.placeholder = zh
               ? '在这里写随手笔记、代码片段… 浏览器会自动本地保存。'
               : 'Type quick notes, scratchpad, code snippets here... Auto-saved locally.';
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

            const spBodyDev = document.createElement('div');
            spBodyDev.className = 'fnx-sp-body hidden';
            spBodyDev.style.cssText =
               'padding: 10px; gap: 8px; overflow-y: auto;';
            const devIn = document.createElement('textarea');
            devIn.className = 'fnx-sp-textarea';
            devIn.style.cssText =
               'flex: none; height: 112px; min-height: 82px; padding: 8px; font-family: ui-monospace, Consolas, monospace; font-size: 11.5px; border: 1px solid var(--fnx-border); border-radius: 8px; background: var(--fnx-surface);';
            devIn.placeholder = isZh()
               ? '粘贴 JSON / Base64 / URL / Unix 时间戳 (秒或毫秒)...'
               : 'Paste JSON / Base64 / URL / Unix timestamp...';

            const devBtnGrid = document.createElement('div');
            devBtnGrid.style.cssText =
               'display: grid; grid-template-columns: repeat(2, minmax(0, 1fr)); gap: 6px;';
            const devOut = document.createElement('textarea');
            devOut.className = 'fnx-sp-textarea';
            devOut.readOnly = true;
            devOut.style.cssText =
               'flex: 1; min-height: 120px; padding: 8px; font-family: ui-monospace, Consolas, monospace; font-size: 11.5px; border: 1px solid var(--fnx-border); border-radius: 8px; background: var(--fnx-surface);';
            devOut.placeholder = isZh()
               ? '转换 / 格式化输出结果将显示在这里...'
               : 'Formatted output appears here...';

            const devToolButtons = [];
            function makeDevToolBtn(zhLbl, enLbl, fn, iconName) {
               const b = document.createElement('button');
               b.type = 'button';
               b.className = 'fnx-pop-btn';
               b.style.cssText = 'height: 29px; font-size: 11.5px;';
               const lbl = isZh() ? zhLbl : enLbl;
               if (iconName) {
                  setBtnIconAndText(b, iconName, lbl);
               } else {
                  b.textContent = lbl;
               }
               b.onclick = () => {
                  try {
                     fn();
                  } catch (err) {
                     devOut.value = 'Error: ' + (err && err.message ? err.message : String(err));
                  }
               };
               devToolButtons.push({ btn: b, zhLbl, enLbl, iconName });
               return b;
            }

            function hexToRgbStr(hex) {
               const h = String(hex || '').replace('#', '');
               if (h.length !== 6) return '';
               const r = parseInt(h.slice(0, 2), 16);
               const g = parseInt(h.slice(2, 4), 16);
               const bl = parseInt(h.slice(4, 6), 16);
               return `rgb(${r}, ${g}, ${bl})`;
            }

            const fallbackColorInp = document.createElement('input');
            fallbackColorInp.type = 'color';
            fallbackColorInp.style.cssText = 'position: fixed; opacity: 0; pointer-events: none; width: 0; height: 0;';
            fallbackColorInp.oninput = () => {
               const hex = fallbackColorInp.value.toUpperCase();
               devOut.value = `${hex}\n${hexToRgbStr(hex)}`;
            };
            spBodyDev.appendChild(fallbackColorInp);

            devBtnGrid.appendChild(
               makeDevToolBtn('JSON 格式化', 'Format JSON', () => {
                  devOut.value = JSON.stringify(JSON.parse(devIn.value), null, 2);
               }),
            );
            devBtnGrid.appendChild(
               makeDevToolBtn('JSON 压缩', 'Minify JSON', () => {
                  devOut.value = JSON.stringify(JSON.parse(devIn.value));
               }),
            );
            devBtnGrid.appendChild(
               makeDevToolBtn('Base64 编码', 'Base64 Encode', () => {
                  const bytes = new TextEncoder().encode(devIn.value);
                  let bin = '';
                  for (let i = 0; i < bytes.length; i++) bin += String.fromCharCode(bytes[i]);
                  devOut.value = btoa(bin);
               }),
            );
            devBtnGrid.appendChild(
               makeDevToolBtn('Base64 解码', 'Base64 Decode', () => {
                  const bin = atob(devIn.value.trim());
                  const bytes = new Uint8Array(bin.length);
                  for (let i = 0; i < bin.length; i++) bytes[i] = bin.charCodeAt(i);
                  devOut.value = new TextDecoder().decode(bytes);
               }),
            );
            devBtnGrid.appendChild(
               makeDevToolBtn('URL 编码', 'URL Encode', () => {
                  devOut.value = encodeURIComponent(devIn.value);
               }),
            );
            devBtnGrid.appendChild(
               makeDevToolBtn('URL 解码', 'URL Decode', () => {
                  devOut.value = decodeURIComponent(devIn.value);
               }),
            );
            devBtnGrid.appendChild(
               makeDevToolBtn('时间戳转换', 'Timestamp', () => {
                  const raw = devIn.value.trim();
                  let d;
                  if (!raw) {
                     d = new Date();
                  } else if (/^-?\d+(\.\d+)?$/.test(raw)) {
                     const n = Number(raw);
                     d = new Date(Math.abs(n) < 1e11 ? n * 1000 : n);
                  } else {
                     d = new Date(raw);
                  }
                  if (Number.isNaN(d.getTime())) {
                     throw new Error('Invalid date / timestamp');
                  }
                  const sec = Math.floor(d.getTime() / 1000);
                  devOut.value = [
                     `Unix (s):  ${sec}`,
                     `Unix (ms): ${d.getTime()}`,
                     `Local:     ${d.toLocaleString()}`,
                     `ISO-8601:  ${d.toISOString()}`,
                  ].join('\n');
               }, 'history'),
            );
            devBtnGrid.appendChild(
               makeDevToolBtn('屏幕取色器', 'EyeDropper', () => {
                  if (typeof window.EyeDropper === 'function') {
                     new window.EyeDropper()
                        .open()
                        .then((res) => {
                           const hex = String(res.sRGBHex || '').toUpperCase();
                           const out = `${hex}\n${hexToRgbStr(hex)}`;
                           devOut.value = out;
                           if (navigator.clipboard && navigator.clipboard.writeText) {
                              navigator.clipboard.writeText(hex).catch(() => {});
                           }
                           showHudToast((isZh() ? '已取色并复制：' : 'Picked color: ') + hex);
                        })
                        .catch(() => {});
                  } else {
                     fallbackColorInp.click();
                  }
               }, 'eyedropper'),
            );
            devBtnGrid.appendChild(
               makeDevToolBtn('JWT 解码', 'JWT Decode', () => {
                  const raw = devIn.value.trim();
                  const parts = raw.split('.');
                  if (parts.length < 2) {
                     throw new Error('Invalid JWT (expected header.payload.sig)');
                  }
                  const decodeB64Url = (s) => {
                     const b64 = s.replace(/-/g, '+').replace(/_/g, '/');
                     const pad = b64 + '='.repeat((4 - (b64.length % 4)) % 4);
                     const bin = atob(pad);
                     const bytes = new Uint8Array(bin.length);
                     for (let i = 0; i < bin.length; i++) bytes[i] = bin.charCodeAt(i);
                     return new TextDecoder().decode(bytes);
                  };
                  const hdr = JSON.parse(decodeB64Url(parts[0]));
                  const pay = JSON.parse(decodeB64Url(parts[1]));
                  let statusLine = 'Token Status: No exp claim';
                  if (typeof pay.exp === 'number') {
                     const diffSec = Math.round(pay.exp - Date.now() / 1000);
                     const expDate = new Date(pay.exp * 1000).toLocaleString();
                     statusLine =
                        diffSec >= 0
                           ? `[VALID] (${Math.ceil(diffSec / 60)} min left · exp ${expDate})`
                           : `[EXPIRED] (${Math.abs(Math.floor(diffSec / 60))} min ago · exp ${expDate})`;
                  }
                  if (typeof pay.iat === 'number') {
                     statusLine += `\nIssued (iat): ${new Date(pay.iat * 1000).toLocaleString()}`;
                  }
                  devOut.value = [
                     statusLine,
                     '',
                     '--- HEADER ---',
                     JSON.stringify(hdr, null, 2),
                     '',
                     '--- PAYLOAD ---',
                     JSON.stringify(pay, null, 2),
                  ].join('\n');
               }, 'key'),
            );
            devBtnGrid.appendChild(
               makeDevToolBtn('UUID / SHA256', 'UUID / SHA256', () => {
                  const raw = devIn.value;
                  if (!raw.trim()) {
                     const uuid =
                        window.crypto && typeof window.crypto.randomUUID === 'function'
                           ? window.crypto.randomUUID()
                           : 'xxxxxxxx-xxxx-4xxx-yxxx-xxxxxxxxxxxx'.replace(/[xy]/g, (c) => {
                                const r = (Math.random() * 16) | 0;
                                return (c === 'x' ? r : (r & 0x3) | 0x8).toString(16);
                             });
                     devOut.value = `UUID v4:\n${uuid}`;
                     return;
                  }
                  if (!window.crypto || !window.crypto.subtle) {
                     throw new Error('SubtleCrypto SHA-256 requires secure context');
                  }
                  const bytes = new TextEncoder().encode(raw);
                  window.crypto.subtle.digest('SHA-256', bytes).then((buf) => {
                     const arr = Array.from(new Uint8Array(buf));
                     const hex = arr.map((b) => b.toString(16).padStart(2, '0')).join('');
                     devOut.value = `SHA-256 (${bytes.length} bytes):\n${hex}`;
                  });
               }, 'hash'),
            );
            devBtnGrid.appendChild(
               makeDevToolBtn('正则测试', 'Regex Test', () => {
                  const lines = devIn.value.split(/\r?\n/);
                  const first = (lines[0] || '').trim();
                  if (!first) {
                     throw new Error('Line 1: /pattern/flags, remaining lines: test text');
                  }
                  const body = lines.slice(1).join('\n');
                  let pat = first;
                  let flags = 'g';
                  const m = first.match(/^\/(.*)\/([gimsuy]*)$/);
                  if (m) {
                     pat = m[1];
                     flags = m[2].includes('g') ? m[2] : m[2] + 'g';
                  }
                  const re = new RegExp(pat, flags);
                  const hits = [];
                  let mt;
                  while ((mt = re.exec(body)) !== null && hits.length < 100) {
                     hits.push(
                        `#${hits.length + 1} @${mt.index}: ${JSON.stringify(mt[0])}` +
                           (mt.length > 1 ? `  groups=${JSON.stringify(mt.slice(1))}` : ''),
                     );
                     if (mt[0].length === 0) re.lastIndex += 1;
                  }
                  devOut.value = hits.length
                     ? `Matched ${hits.length} time(s):\n${hits.join('\n')}`
                     : 'No matches found (Line 1 is /regex/flags, Line 2+ is test text).';
               }, 'search'),
            );

            const devBottomRow = document.createElement('div');
            devBottomRow.style.cssText = 'display: flex; gap: 6px;';
            const btnDevCopy = makeDevToolBtn('复制输出', 'Copy Output', () => {
               if (!devOut.value) return;
               if (navigator.clipboard && navigator.clipboard.writeText) {
                  navigator.clipboard.writeText(devOut.value).catch(() => {});
               }
               showHudToast(isZh() ? '已复制输出内容' : 'Output copied');
            }, 'copy');
            const btnDevSwap = makeDevToolBtn('移入输入框', 'Move to Input', () => {
               if (!devOut.value) return;
               devIn.value = devOut.value;
               devOut.value = '';
            }, 'undo');
            devBottomRow.appendChild(btnDevCopy);
            devBottomRow.appendChild(btnDevSwap);

            spBodyDev.appendChild(devIn);
            spBodyDev.appendChild(devBtnGrid);
            spBodyDev.appendChild(devOut);
            spBodyDev.appendChild(devBottomRow);

            function selectSpTab(activeKey) {
               const map = [
                  ['web', btnSpWeb, spBodyWeb],
                  ['notes', btnSpNotes, spBodyNotes],
                  ['dev', btnSpDev, spBodyDev],
               ];
               for (let i = 0; i < map.length; i++) {
                  const [k, btnEl, bodyEl] = map[i];
                  const on = k === activeKey;
                  btnEl.classList.toggle('active', on);
                  bodyEl.classList.toggle('hidden', !on);
               }
            }
            btnSpWeb.onclick = () => selectSpTab('web');
            btnSpNotes.onclick = () => selectSpTab('notes');
            btnSpDev.onclick = () => selectSpTab('dev');

            sidePanelEl.appendChild(spBodyWeb);
            sidePanelEl.appendChild(spBodyNotes);
            sidePanelEl.appendChild(spBodyDev);
            shell.appendChild(sidePanelEl);
            syncSidePanelLocalizationRef = () => {
               const z = isZh();
               spTabItems.forEach((it) => {
                  it.btn.textContent = '';
                  it.btn.appendChild(createIcon(it.iconName));
                  it.btn.appendChild(document.createTextNode(z ? it.zhLabel : it.enLabel));
               });
               btnSpClose.title = z ? '关闭侧边栏 (Esc)' : 'Close side panel (Esc)';
               spInput.placeholder = z ? '输入侧边栏小窗网址...' : 'Enter URL for side panel...';
               spGoBtn.textContent = z ? '前往' : 'Go';
               spReloadBtn.title = z ? '刷新小窗网页' : 'Reload side panel';
               spPopBtn.title = z ? '在新标签页中打开' : 'Open in new tab (if blocked in frame)';
               spTextarea.placeholder = z
                  ? '在这里写随手笔记、代码片段… 浏览器会自动本地保存。'
                  : 'Type quick notes, scratchpad, code snippets here... Auto-saved locally.';
               devIn.placeholder = z
                  ? '粘贴 JSON / Base64 / URL / Unix 时间戳 (秒或毫秒)...'
                  : 'Paste JSON / Base64 / URL / Unix timestamp...';
               devOut.placeholder = z
                  ? '转换 / 格式化输出结果将显示在这里...'
                  : 'Formatted output appears here...';
               devToolButtons.forEach((item) => {
                  const txt = z ? item.zhLbl : item.enLbl;
                  if (item.iconName) setBtnIconAndText(item.btn, item.iconName, txt);
                  else item.btn.textContent = txt;
               });
            };
            sidePanelRefs = { spIframe, spBodyWeb, loadSpUrl };
            return sidePanelRefs;
         }

         function destroySidePanel() {
            isSidePanelOpen = false;
            if (sidePanelIdleTimer) {
               clearTimeout(sidePanelIdleTimer);
               sidePanelIdleTimer = null;
            }
            if (sidePanelRefs && sidePanelRefs.spIframe) {
               try {
                  sidePanelRefs.spIframe.src = 'about:blank';
               } catch (_) {}
            }
            if (sidePanelEl) {
               try {
                  sidePanelEl.remove();
               } catch (_) {}
               sidePanelEl = null;
            }
            sidePanelRefs = null;
            if (typeof btnSidePanel !== 'undefined' && btnSidePanel) {
               btnSidePanel.style.color = 'inherit';
            }
            if (IS_HUD_SHELL && typeof syncHudLayoutRef === 'function') {
               syncHudLayoutRef();
            }
         }

         function toggleSidePanel(force) {
            const next = force !== undefined ? !!force : !isSidePanelOpen;
            if (next && !config.ext_side_panel) {
               showHudToast(
                  isZh()
                     ? '分屏侧边栏未启用（请在设置 → 扩展与脚本中开启）'
                     : 'Side Panel extension is off (enable in Settings → Extensions)',
               );
               return;
            }
            if (!next && !sidePanelEl) {
               isSidePanelOpen = false;
               return;
            }
            const refs = ensureSidePanelMounted();
            isSidePanelOpen = next;
            if (sidePanelIdleTimer) {
               clearTimeout(sidePanelIdleTimer);
               sidePanelIdleTimer = null;
            }
            if (IS_HUD_SHELL) {
               sidePanelEl.style.display = next ? 'flex' : 'none';
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
                  !refs.spBodyWeb.classList.contains('hidden') &&
                  refs.spIframe.src === 'about:blank'
               ) {
                  refs.loadSpUrl(sidePanelLastUrl || 'https://translate.google.com');
               }
            } else if (refs.spIframe.src && refs.spIframe.src !== 'about:blank') {
               sidePanelIdleTimer = setTimeout(() => {
                  sidePanelIdleTimer = null;
                  if (!isSidePanelOpen && sidePanelRefs) {
                     sidePanelRefs.spIframe.src = 'about:blank';
                  }
               }, 45000);
            }
            if (IS_HUD_SHELL && typeof syncHudLayoutRef === 'function') {
               syncHudLayoutRef();
            }
         }
         closeSidePanelRef = () => toggleSidePanel(false);
         isSidePanelOpenRef = () => isSidePanelOpen;

         // 3.5b Built-in Fynexo DevDrawer (Console / Network / Storage / DOM + Debug Switches)
         let isDevDrawerOpen = false;
         let devDrawerEl = null;
         let devActiveTab = 'console';
         let devConsoleFilter = 'all';
         let devStorageKind = 'local';
         let devDesignModeOn = false;
         let devWireframeOn = false;
         let devViewportIdx = 0;
         const DEV_VIEWPORTS = [0, 375, 768, 1280];
         let devDrawerRefs = null;
         let devInspectActive = false;
         let devSelectedNetEntry = null;
         let devFpsActive = false;
         let devFpsEl = null;
         let devFpsRafId = null;
         let devFpsLastSec = 0;
         let devFpsFrames = 0;

         function toggleDevFpsHud(force, silent) {
            const next = force !== undefined ? !!force : !devFpsActive;
            if (!silent) {
               userFpsHudOverride = next;
            }
            if (next === devFpsActive) return;
            devFpsActive = next;
            if (!devFpsActive) {
               if (devFpsRafId) {
                  cancelAnimationFrame(devFpsRafId);
                  devFpsRafId = null;
               }
               if (devFpsEl && devFpsEl.isConnected) devFpsEl.style.display = 'none';
               if (devDrawerRefs && devDrawerRefs.syncDevQuickToggles) {
                  devDrawerRefs.syncDevQuickToggles();
               }
               return;
            }
            if (!devFpsEl) {
               devFpsEl = document.createElement('div');
               devFpsEl.id = '__fynexo_fps_hud__';
               devFpsEl.style.cssText =
                  'position: fixed; right: 14px; bottom: 64px; z-index: 2147483644; pointer-events: none; padding: 4px 10px; border-radius: 999px; background: rgba(16,16,20,0.84); border: 1px solid var(--fnx-accent, #38bdf8); color: #f4f4f6; font: 600 11px/1.35 ui-monospace, Consolas, monospace; box-shadow: 0 6px 18px rgba(0,0,0,0.35);';
               shell.appendChild(devFpsEl);
            }
            devFpsEl.style.display = 'block';
            devFpsFrames = 0;
            devFpsLastSec = performance.now ? performance.now() : Date.now();
            const tickFps = (now) => {
               if (!devFpsActive) return;
               devFpsFrames += 1;
               const ts = now || Date.now();
               if (ts - devFpsLastSec >= 1000) {
                  const fps = Math.round((devFpsFrames * 1000) / Math.max(1, ts - devFpsLastSec));
                  const domCnt = document.querySelectorAll('*').length;
                  const heapMb =
                     performance &&
                     performance.memory &&
                     performance.memory.usedJSHeapSize
                        ? ` · ${(performance.memory.usedJSHeapSize / 1048576).toFixed(1)}MB`
                        : '';
                  devFpsEl.textContent = `FPS ${fps} · DOM ${domCnt}${heapMb}`;
                  devFpsFrames = 0;
                  devFpsLastSec = ts;
               }
               devFpsRafId = requestAnimationFrame(tickFps);
            };
            devFpsRafId = requestAnimationFrame(tickFps);
            if (devDrawerRefs && devDrawerRefs.syncDevQuickToggles) {
               devDrawerRefs.syncDevQuickToggles();
            }
         }
         toggleDevFpsHudRef = toggleDevFpsHud;

         function applyDevWireframe(on) {
            devWireframeOn = !!on;
            let st = document.getElementById('__fynexo_dev_wireframe__');
            if (!devWireframeOn) {
               if (st) st.remove();
               return;
            }
            if (!st) {
               st = document.createElement('style');
               st.id = '__fynexo_dev_wireframe__';
               st.textContent =
                  '*:not(#__fynexo_host__):not(#__fynexo_host__ *) { outline: 1px solid rgba(56, 189, 248, 0.45) !important; }\n*:not(#__fynexo_host__):not(#__fynexo_host__ *):hover { outline: 1.5px solid #f43f5e !important; }';
               (document.head || document.documentElement).appendChild(st);
            }
         }

         function applyDevViewportWidth(px) {
            let st = document.getElementById('__fynexo_dev_viewport__');
            if (!px) {
               if (st) st.remove();
               return;
            }
            if (!st) {
               st = document.createElement('style');
               st.id = '__fynexo_dev_viewport__';
               (document.head || document.documentElement).appendChild(st);
            }
            st.textContent =
               `html { max-width: ${px}px !important; margin: 0 auto !important; box-shadow: 0 0 0 1px rgba(56, 189, 248, 0.45), 0 0 40px rgba(0, 0, 0, 0.45) !important; min-height: 100vh !important; }`;
         }

         let devInspectOnHover = null;
         let devInspectOnClick = null;
         let devInspectOnKeydown = null;

         function stopDevInspectPicker() {
            if (!devInspectActive) return;
            devInspectActive = false;
            if (devInspectOnHover) {
               window.removeEventListener('mousemove', devInspectOnHover, true);
               window.removeEventListener('mouseover', devInspectOnHover, true);
            }
            if (devInspectOnClick) {
               window.removeEventListener('click', devInspectOnClick, true);
            }
            if (devInspectOnKeydown) {
               window.removeEventListener('keydown', devInspectOnKeydown, true);
            }
            if (pickerHighlightBox) pickerHighlightBox.style.display = 'none';
            if (devDrawerRefs && devDrawerRefs.btnDomPick) {
               devDrawerRefs.btnDomPick.style.borderColor = '';
               devDrawerRefs.btnDomPick.style.color = '';
            }
         }

         function renderDevConsolePanel() {
            if (!devDrawerRefs || !isDevDrawerOpen) return;
            const list = devDrawerRefs.consoleList;
            list.textContent = '';
            const frag = document.createDocumentFragment();
            const zh = isZh();
            let count = 0;
            for (let i = 0; i < devConsoleLogs.length; i++) {
               const item = devConsoleLogs[i];
               if (
                  devConsoleFilter !== 'all' &&
                  item.level !== devConsoleFilter
               ) {
                  continue;
               }
               count += 1;
               const row = document.createElement('div');
               row.style.cssText =
                  'display: flex; gap: 8px; padding: 4px 8px; border-bottom: 1px solid var(--fnx-border); word-break: break-word; white-space: pre-wrap;';
               const tSpan = document.createElement('span');
               tSpan.style.cssText =
                  'opacity: 0.55; flex-shrink: 0; font-size: 10.5px;';
               tSpan.textContent = item.time;
               const lSpan = document.createElement('span');
               lSpan.style.cssText =
                  'flex-shrink: 0; font-weight: 700; width: 42px; text-transform: uppercase; font-size: 10px;';
               lSpan.textContent = item.level;
               lSpan.style.color =
                  item.level === 'error'
                     ? '#ef4444'
                     : item.level === 'warn'
                       ? '#d97706'
                       : item.level === 'info'
                         ? '#0284c7'
                         : 'var(--fnx-accent, #38bdf8)';
               const txtSpan = document.createElement('span');
               txtSpan.style.cssText = 'flex: 1; min-width: 0;';
               if (item.level === 'error') txtSpan.style.color = '#ef4444';
               else if (item.level === 'warn') txtSpan.style.color = '#d97706';
               txtSpan.textContent = item.text;
               row.appendChild(tSpan);
               row.appendChild(lSpan);
               row.appendChild(txtSpan);
               frag.appendChild(row);
            }
            if (count === 0) {
               const empty = document.createElement('div');
               empty.style.cssText = 'padding: 16px; opacity: 0.6; text-align: center;';
               empty.textContent = zh
                  ? '暂无控制台日志（可在下方输入 JS 表达式运行）'
                  : 'No console entries yet (run a JS expression below)';
               list.appendChild(empty);
            } else {
               list.appendChild(frag);
               list.scrollTop = list.scrollHeight;
            }
         }
         renderDevConsoleRef = renderDevConsolePanel;

         function buildCurlCommand(entry) {
            const sq = (s) => "'" + String(s || '').replace(/'/g, "'\\''") + "'";
            const parts = ['curl', '-X', sq(entry.method || 'GET'), sq(entry.url || '')];
            if (entry.reqHeaders && typeof entry.reqHeaders === 'object') {
               Object.keys(entry.reqHeaders).forEach((k) => {
                  parts.push('-H', sq(k + ': ' + entry.reqHeaders[k]));
               });
            }
            if (entry.reqBody && !String(entry.reqBody).startsWith('[')) {
               parts.push('--data-raw', sq(entry.reqBody));
            }
            return parts.join(' ');
         }

         function renderDevNetworkPanel() {
            if (!devDrawerRefs || !isDevDrawerOpen) return;
            const zh = isZh();
            const summary = devDrawerRefs.netSummary;
            const list = devDrawerRefs.netList;
            const detailBox = devDrawerRefs.netDetail;
            list.textContent = '';

            let nav = null;
            try {
               const navs = performance.getEntriesByType('navigation');
               nav = navs && navs[0];
            } catch (_) {}
            const res = (() => {
               try {
                  return performance.getEntriesByType('resource') || [];
               } catch (_) {
                  return [];
               }
            })();
            const domNodes = document.querySelectorAll('*').length;
            const ttfb = nav
               ? Math.max(0, Math.round(nav.responseStart - nav.requestStart))
               : 0;
            const dcl = nav ? Math.max(0, Math.round(nav.domContentLoadedEventEnd)) : 0;
            const load = nav ? Math.max(0, Math.round(nav.loadEventEnd)) : 0;
            summary.textContent = zh
               ? `TTFB ${ttfb}ms · DOM Ready ${dcl}ms · Load ${load}ms · API抓包 ${devNetworkLogs.length} · 资源 ${res.length} · 节点 ${domNodes}`
               : `TTFB ${ttfb}ms · Ready ${dcl}ms · Load ${load}ms · Fetch/XHR ${devNetworkLogs.length} · Assets ${res.length}`;

            const frag = document.createDocumentFragment();
            const liveRecent = devNetworkLogs.slice().reverse();
            for (let i = 0; i < liveRecent.length; i++) {
               const item = liveRecent[i];
               const row = document.createElement('div');
               row.style.cssText =
                  'display: flex; align-items: center; gap: 6px; padding: 5px 8px; border-bottom: 1px solid var(--fnx-border); font-size: 11px; cursor: pointer;';
               if (devSelectedNetEntry && devSelectedNetEntry.id === item.id) {
                  row.style.background = 'var(--fnx-accent-bg, rgba(56, 189, 248, 0.12))';
               }
               const stBadge = document.createElement('span');
               stBadge.style.cssText =
                  'width: 34px; flex-shrink: 0; font-weight: 700; text-align: center; font-size: 10.5px;';
               stBadge.textContent = item.status > 0 ? String(item.status) : 'ERR';
               stBadge.style.color =
                  item.status >= 200 && item.status < 300
                     ? '#34d399'
                     : item.status >= 300 && item.status < 400
                       ? '#38bdf8'
                       : '#f87171';
               const mBadge = document.createElement('span');
               mBadge.style.cssText =
                  'width: 44px; flex-shrink: 0; padding: 1px 4px; border-radius: 4px; background: var(--fnx-surface); text-align: center; font-size: 10px; font-weight: 600;';
               mBadge.textContent = item.method || 'GET';
               const urlSpan = document.createElement('span');
               urlSpan.style.cssText =
                  'flex: 1; min-width: 0; overflow: hidden; text-overflow: ellipsis; white-space: nowrap;';
               urlSpan.textContent = String(item.url || '').replace(/^https?:\/\/[^/]+/, '') || item.url;
               urlSpan.title = item.url || '';

               const durSpan = document.createElement('span');
               durSpan.style.cssText = 'width: 48px; flex-shrink: 0; text-align: right; color: var(--fnx-accent);';
               durSpan.textContent = `${item.duration || 0}ms`;

               const curlBtn = document.createElement('button');
               curlBtn.type = 'button';
               curlBtn.className = 'fnx-pop-btn';
               curlBtn.style.cssText = 'width: auto; height: 20px; padding: 0 6px; font-size: 10px;';
               curlBtn.textContent = 'cURL';
               curlBtn.onclick = (ev) => {
                  ev.stopPropagation();
                  copyTextToClipboard(
                     buildCurlCommand(item),
                     zh ? '已复制为 cURL 命令' : 'Copied as cURL',
                  );
               };

               const replayBtn = document.createElement('button');
               replayBtn.type = 'button';
               replayBtn.className = 'fnx-pop-btn';
               replayBtn.style.cssText = 'width: auto; height: 20px; padding: 0 6px; font-size: 10px;';
               replayBtn.textContent = zh ? '重发' : 'Replay';
               replayBtn.onclick = (ev) => {
                  ev.stopPropagation();
                  const opts = { method: item.method || 'GET', headers: item.reqHeaders || {} };
                  if (item.reqBody && !String(item.reqBody).startsWith('[') && opts.method !== 'GET' && opts.method !== 'HEAD') {
                     opts.body = item.reqBody;
                  }
                  window.fetch(item.url, opts).catch(() => {});
                  showHudToast(zh ? '已重新发起请求' : 'Replayed request');
               };

               row.onclick = () => {
                  devSelectedNetEntry =
                     devSelectedNetEntry && devSelectedNetEntry.id === item.id ? null : item;
                  renderDevNetworkPanel();
               };
               row.appendChild(stBadge);
               row.appendChild(mBadge);
               row.appendChild(urlSpan);
               row.appendChild(durSpan);
               row.appendChild(curlBtn);
               row.appendChild(replayBtn);
               frag.appendChild(row);
            }

            if (detailBox) {
               if (devSelectedNetEntry) {
                  let prettyRes = devSelectedNetEntry.resBody || '';
                  try {
                     if (prettyRes && (prettyRes.trim().startsWith('{') || prettyRes.trim().startsWith('['))) {
                        prettyRes = JSON.stringify(JSON.parse(prettyRes), null, 2);
                     }
                  } catch (_) {}
                  detailBox.style.display = 'block';
                  detailBox.textContent = [
                     `${devSelectedNetEntry.method} ${devSelectedNetEntry.url}  [${devSelectedNetEntry.status} · ${devSelectedNetEntry.duration}ms]`,
                     `Headers: ${JSON.stringify(devSelectedNetEntry.reqHeaders || {})}`,
                     devSelectedNetEntry.reqBody ? `Payload: ${devSelectedNetEntry.reqBody}` : '',
                     `--- Response ---\n${prettyRes || '(empty)'}`,
                  ]
                     .filter(Boolean)
                     .join('\n');
               } else {
                  detailBox.style.display = 'none';
                  detailBox.textContent = '';
               }
            }

            const recent = res.slice(-60).reverse();
            for (let i = 0; i < recent.length; i++) {
               const r = recent[i];
               const row = document.createElement('div');
               row.style.cssText =
                  'display: flex; align-items: center; gap: 8px; padding: 4px 8px; border-bottom: 1px solid var(--fnx-border); font-size: 10.5px; opacity: 0.85;';
               const badge = document.createElement('span');
               badge.style.cssText =
                  'width: 58px; flex-shrink: 0; padding: 1px 5px; border-radius: 4px; background: var(--fnx-surface); text-align: center; font-size: 9.5px; text-transform: uppercase; overflow: hidden; text-overflow: ellipsis;';
               badge.textContent = r.initiatorType || 'asset';
               const urlSpan = document.createElement('span');
               urlSpan.style.cssText =
                  'flex: 1; min-width: 0; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; cursor: pointer;';
               const shortName = String(r.name || '').replace(/^https?:\/\/[^/]+/, '') || r.name;
               urlSpan.textContent = shortName;
               urlSpan.title = (r.name || '') + (zh ? '\n点击复制完整 URL' : '\nClick to copy full URL');
               urlSpan.onclick = () => {
                  copyTextToClipboard(r.name || '', zh ? '已复制请求地址' : 'Copied URL');
               };
               const sizeBytes = Number(r.transferSize || r.encodedBodySize || 0);
               const sizeSpan = document.createElement('span');
               sizeSpan.style.cssText = 'width: 60px; flex-shrink: 0; text-align: right; opacity: 0.72;';
               sizeSpan.textContent = sizeBytes > 0 ? (sizeBytes / 1024).toFixed(1) + ' KB' : '—';
               const durSpan = document.createElement('span');
               durSpan.style.cssText = 'width: 54px; flex-shrink: 0; text-align: right; color: var(--fnx-accent, #38bdf8);';
               durSpan.textContent = Math.round(r.duration || 0) + ' ms';
               row.appendChild(badge);
               row.appendChild(urlSpan);
               row.appendChild(sizeSpan);
               row.appendChild(durSpan);
               frag.appendChild(row);
            }
            if (liveRecent.length === 0 && recent.length === 0) {
               const empty = document.createElement('div');
               empty.style.cssText = 'padding: 16px; opacity: 0.6; text-align: center;';
               empty.textContent = zh ? '暂无网络请求记录' : 'No network requests captured yet';
               list.appendChild(empty);
            } else {
               list.appendChild(frag);
            }
         }
         renderDevNetworkRef = renderDevNetworkPanel;

         function renderDevStoragePanel() {
            if (!devDrawerRefs) return;
            const zh = isZh();
            const list = devDrawerRefs.storageList;
            list.textContent = '';

            let pairs = [];
            let errMsg = '';
            try {
               if (devStorageKind === 'cookies') {
                  const raw = String(document.cookie || '');
                  if (raw.trim()) {
                     raw.split(/;\s*/).forEach((part) => {
                        const eq = part.indexOf('=');
                        if (eq > 0) {
                           const k = decodeURIComponent(part.slice(0, eq));
                           const v = decodeURIComponent(part.slice(eq + 1));
                           pairs.push([k, v]);
                        }
                     });
                  }
               } else {
                  const store =
                     devStorageKind === 'session'
                        ? window.sessionStorage
                        : window.localStorage;
                  for (let i = 0; i < store.length; i++) {
                     const k = store.key(i);
                     if (k !== null) pairs.push([k, store.getItem(k) || '']);
                  }
               }
            } catch (err) {
               errMsg = err && err.message ? err.message : String(err);
            }

            if (errMsg) {
               const errDiv = document.createElement('div');
               errDiv.style.cssText = 'padding: 16px; color: #fca5a5;';
               errDiv.textContent = errMsg;
               list.appendChild(errDiv);
               return;
            }
            if (pairs.length === 0) {
               const empty = document.createElement('div');
               empty.style.cssText = 'padding: 16px; opacity: 0.6; text-align: center;';
               empty.textContent = zh ? '当前存储为空' : 'Storage is empty';
               list.appendChild(empty);
               return;
            }

            const frag = document.createDocumentFragment();
            for (let i = 0; i < pairs.length; i++) {
               const [k, v] = pairs[i];
               const row = document.createElement('div');
               row.style.cssText =
                  'display: flex; align-items: center; gap: 8px; padding: 5px 8px; border-bottom: 1px solid var(--fnx-border); font-size: 11px;';
               const kSpan = document.createElement('span');
               kSpan.style.cssText =
                  'width: 32%; max-width: 200px; flex-shrink: 0; font-weight: 600; color: var(--fnx-accent, #38bdf8); overflow: hidden; text-overflow: ellipsis; white-space: nowrap;';
               kSpan.textContent = k;
               kSpan.title = k;
               const vSpan = document.createElement('span');
               vSpan.style.cssText =
                  'flex: 1; min-width: 0; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; opacity: 0.85;';
               vSpan.textContent = v;
               vSpan.title = v;
               const cpBtn = document.createElement('button');
               cpBtn.type = 'button';
               cpBtn.className = 'fnx-pop-btn';
               cpBtn.style.cssText = 'width: auto; height: 22px; padding: 0 7px; font-size: 10.5px;';
               cpBtn.textContent = zh ? '复制' : 'Copy';
               cpBtn.onclick = () => {
                  if (navigator.clipboard && navigator.clipboard.writeText) {
                     navigator.clipboard.writeText(v).catch(() => {});
                  }
                  showHudToast(zh ? '已复制值' : 'Copied value');
               };
               const delBtn = document.createElement('button');
               delBtn.type = 'button';
               delBtn.className = 'fnx-pop-btn';
               delBtn.style.cssText = 'width: auto; height: 22px; padding: 0 7px; font-size: 10.5px;';
               delBtn.textContent = zh ? '删除' : 'Del';
               delBtn.onclick = () => {
                  try {
                     if (devStorageKind === 'cookies') {
                        document.cookie = `${encodeURIComponent(k)}=; max-age=0; path=/`;
                     } else {
                        const store =
                           devStorageKind === 'session'
                              ? window.sessionStorage
                              : window.localStorage;
                        store.removeItem(k);
                     }
                     renderDevStoragePanel();
                  } catch (_) {}
               };
               row.appendChild(kSpan);
               row.appendChild(vSpan);
               row.appendChild(cpBtn);
               row.appendChild(delBtn);
               frag.appendChild(row);
            }
            list.appendChild(frag);
         }

         function renderDevDomInfo(targetEl) {
            if (!devDrawerRefs) return;
            const zh = isZh();
            const el =
               targetEl && targetEl.nodeType === 1
                  ? targetEl
                  : document.body || document.documentElement;
            if (!el) return;
            const rect = el.getBoundingClientRect();
            const cs = window.getComputedStyle(el);
            const sel =
               typeof buildUniqueCssSelector === 'function'
                  ? buildUniqueCssSelector(el)
                  : el.tagName.toLowerCase();
            devDrawerRefs.domSummary.textContent = [
               `${zh ? '选择器' : 'Selector'}: ${sel}`,
               `${zh ? '盒模型尺寸' : 'Box'}: ${Math.round(rect.width)} × ${Math.round(rect.height)} px  @ (${Math.round(rect.left)}, ${Math.round(rect.top)})`,
               `display: ${cs.display} | position: ${cs.position} | z-index: ${cs.zIndex}`,
               `margin: ${cs.margin} | padding: ${cs.padding}`,
               `color: ${cs.color} | bg: ${cs.backgroundColor} | font: ${cs.fontSize} ${cs.fontFamily}`,
            ].join('\n');
            const htmlText =
               targetEl && targetEl.nodeType === 1
                  ? el.outerHTML
                  : document.documentElement
                    ? document.documentElement.outerHTML
                    : '';
            devDrawerRefs.domSource.value =
               htmlText.length > 180000
                  ? htmlText.slice(0, 180000) + '\n... (truncated for preview; Copy HTML copies full source)'
                  : htmlText;
         }

         function measureDevDrawerBottomPx() {
            const mobDockBottom =
               isMobilePlatform() &&
               styles.mobile_bar.visible !== false &&
               (styles.mobile_bar.position || 'bottom').toLowerCase() !== 'top';
            if (!mobDockBottom) return 0;
            const mobBarEl = shadowRoot
               ? shadowRoot.getElementById('__fynexo_bottom_bar__')
               : null;
            if (mobBarEl) {
               const br = mobBarEl.getBoundingClientRect();
               if (br.height > 0) return Math.round(br.height);
            }
            return Math.max(44, parseInt(styles.mobile_bar.height || '54', 10) || 54);
         }

         function ensureDevDrawerMounted() {
            if (devDrawerRefs && devDrawerEl) return devDrawerRefs;
            devDrawerEl = document.createElement('div');
            devDrawerEl.id = '__fynexo_dev_drawer__';

            const resizeHandle = document.createElement('div');
            resizeHandle.className = 'fnx-dev-resize';
            const resizePill = document.createElement('span');
            resizePill.className = 'fnx-dev-resize-pill';
            resizeHandle.appendChild(resizePill);
            let dragResizing = false;
            let dragStartY = 0;
            let dragStartH = 340;
            let lastDragH = 340;
            let dragMovedDown = 0;
            resizeHandle.addEventListener('pointerdown', (ev) => {
               dragResizing = true;
               dragStartY = ev.clientY || 0;
               dragStartH = devDrawerEl.getBoundingClientRect().height || 340;
               lastDragH = dragStartH;
               dragMovedDown = 0;
               try { resizeHandle.setPointerCapture(ev.pointerId); } catch (_) {}
               ev.preventDefault();
            });
            resizeHandle.addEventListener('pointermove', (ev) => {
               if (!dragResizing) return;
               const curY = ev.clientY || 0;
               const dy = dragStartY - curY;
               dragMovedDown = Math.max(0, curY - dragStartY);
               const dockB = measureDevDrawerBottomPx();
               const maxH = Math.max(
                  220,
                  Math.round((window.innerHeight || 800) - dockB - 20),
               );
               const nextH = Math.max(88, Math.min(maxH, Math.round(dragStartH + dy)));
               lastDragH = nextH;
               devDrawerEl.style.height = nextH + 'px';
            });
            const endResize = (ev) => {
               if (!dragResizing) return;
               dragResizing = false;
               try { resizeHandle.releasePointerCapture(ev.pointerId); } catch (_) {}
               if (lastDragH < 150 || (isMobilePlatform() && dragMovedDown > 130 && lastDragH < 210)) {
                  devDrawerEl.style.height = '340px';
                  toggleDevDrawer(false);
               } else if (lastDragH < 200) {
                  devDrawerEl.style.height = '200px';
               }
            };
            resizeHandle.addEventListener('pointerup', endResize);
            resizeHandle.addEventListener('pointercancel', endResize);
            devDrawerEl.appendChild(resizeHandle);

            const head = document.createElement('div');
            head.className = 'fnx-dev-head';

            const headTopRow = document.createElement('div');
            headTopRow.className = 'fnx-dev-top-row';

            const tabWrap = document.createElement('div');
            tabWrap.className = 'fnx-dev-tabs';
            const rightWrap = document.createElement('div');
            rightWrap.className = 'fnx-dev-tools';

            const tabDefs = [
               ['console', 'Console', '控制台'],
               ['network', 'Network', '网络抓包'],
               ['api', 'API Client', '接口调试'],
               ['storage', 'Storage', '本地存储'],
               ['dom', 'DOM', '元素与源码'],
            ];
            const tabButtons = Object.create(null);
            const tabPanes = Object.create(null);

            function selectDevTab(key) {
               devActiveTab = key;
               const zh = isZh();
               for (let i = 0; i < tabDefs.length; i++) {
                  const [k, enT, zhT] = tabDefs[i];
                  const on = k === key;
                  tabButtons[k].textContent = zh ? zhT : enT;
                  tabButtons[k].style.borderColor = on ? 'var(--fnx-accent, #38bdf8)' : '';
                  tabButtons[k].style.color = on ? 'var(--fnx-fg)' : '';
                  tabButtons[k].style.fontWeight = on ? '600' : '450';
                  tabPanes[k].style.display = on ? 'flex' : 'none';
               }
               if (key === 'console') renderDevConsolePanel();
               else if (key === 'network') renderDevNetworkPanel();
               else if (key === 'storage') renderDevStoragePanel();
               else if (key === 'dom') renderDevDomInfo(null);
            }

            for (let i = 0; i < tabDefs.length; i++) {
               const [k, enT, zhT] = tabDefs[i];
               const b = document.createElement('button');
               b.type = 'button';
               b.className = 'fnx-pop-btn';
               b.style.cssText = 'width: auto; height: 25px; padding: 0 9px; font-size: 11.5px;';
               b.textContent = isZh() ? zhT : enT;
               b.onclick = () => selectDevTab(k);
               tabButtons[k] = b;
               tabWrap.appendChild(b);
            }

            const btnDesign = document.createElement('button');
            btnDesign.type = 'button';
            btnDesign.className = 'fnx-pop-btn';
            btnDesign.style.cssText = 'width: auto; height: 25px; padding: 0 8px; font-size: 11px;';

            const btnWireframe = document.createElement('button');
            btnWireframe.type = 'button';
            btnWireframe.className = 'fnx-pop-btn';
            btnWireframe.style.cssText = 'width: auto; height: 25px; padding: 0 8px; font-size: 11px;';

            const btnViewport = document.createElement('button');
            btnViewport.type = 'button';
            btnViewport.className = 'fnx-pop-btn';
            btnViewport.style.cssText = 'width: auto; height: 25px; padding: 0 8px; font-size: 11px;';

            const btnFpsHud = document.createElement('button');
            btnFpsHud.type = 'button';
            btnFpsHud.className = 'fnx-pop-btn';
            btnFpsHud.style.cssText = 'width: auto; height: 25px; padding: 0 8px; font-size: 11px;';
            btnFpsHud.onclick = () => toggleDevFpsHud();

            function syncDevQuickToggles() {
               const zh = isZh();
               setBtnIconAndText(btnFpsHud, 'chart', 'FPS');
               btnFpsHud.style.borderColor = devFpsActive ? 'var(--fnx-accent)' : '';
               btnFpsHud.style.color = devFpsActive ? 'var(--fnx-accent)' : '';

               setBtnIconAndText(btnDesign, 'edit', zh ? '自由编辑' : 'DesignMode');
               btnDesign.style.borderColor = devDesignModeOn ? 'var(--fnx-accent)' : '';
               btnDesign.style.color = devDesignModeOn ? 'var(--fnx-accent)' : '';

               setBtnIconAndText(btnWireframe, 'wireframe', zh ? '线框透视' : 'Wireframe');
               btnWireframe.style.borderColor = devWireframeOn ? 'var(--fnx-accent)' : '';
               btnWireframe.style.color = devWireframeOn ? 'var(--fnx-accent)' : '';

               const vpPx = DEV_VIEWPORTS[devViewportIdx] || 0;
               setBtnIconAndText(
                  btnViewport,
                  'device',
                  vpPx ? `${vpPx}px` : (zh ? '视口模拟' : 'Viewport'),
               );
               btnViewport.style.borderColor = vpPx ? 'var(--fnx-accent)' : '';
               btnViewport.style.color = vpPx ? 'var(--fnx-accent)' : '';
            }

            btnDesign.onclick = () => {
               devDesignModeOn = !devDesignModeOn;
               try {
                  document.designMode = devDesignModeOn ? 'on' : 'off';
               } catch (_) {}
               syncDevQuickToggles();
               showHudToast(
                  isZh()
                     ? (devDesignModeOn ? '已开启网页文字自由编辑' : '已关闭网页自由编辑')
                     : (devDesignModeOn ? 'document.designMode ON' : 'document.designMode OFF'),
               );
            };
            btnWireframe.onclick = () => {
               applyDevWireframe(!devWireframeOn);
               syncDevQuickToggles();
            };
            btnViewport.onclick = () => {
               devViewportIdx = (devViewportIdx + 1) % DEV_VIEWPORTS.length;
               applyDevViewportWidth(DEV_VIEWPORTS[devViewportIdx]);
               syncDevQuickToggles();
            };
            const devPortSel = document.createElement('select');
            devPortSel.className = 'fnx-pop-btn';
            devPortSel.style.cssText =
               'width: auto; height: 25px; padding: 0 6px; font-size: 11px; outline: none; background: var(--fnx-bg, #141417); color: var(--fnx-fg, #f4f4f6);';
            const devPorts = ['', '3000', '5173', '8080', '8000', '4200', '4321'];
            const rebuildPortOpts = () => {
               const zh = isZh();
               const curPort = String(window.location.port || '');
               devPortSel.textContent = '';
               devPorts.forEach((pt) => {
                  const opt = document.createElement('option');
                  opt.value = pt;
                  opt.textContent = pt ? `:${pt}` : (zh ? '切端口' : ':port');
                  if (pt && pt === curPort) opt.selected = true;
                  devPortSel.appendChild(opt);
               });
               devPortSel.title = zh
                  ? '一键跳转到本地开发端口（保留当前路径）'
                  : 'Quick jump to localhost port (keeps path)';
            };
            rebuildPortOpts();
            devPortSel.onchange = () => {
               const pt = devPortSel.value;
               if (!pt) return;
               const curHost = String(window.location.hostname || '').toLowerCase();
               const targetHost = isLocalDevHost(curHost) ? curHost : 'localhost';
               const suffix = isLocalDevHost(curHost)
                  ? (window.location.pathname || '/') + (window.location.search || '')
                  : '/';
               const targetUrl = `http://${targetHost}:${pt}${suffix}`;
               triggerProgress();
               sendIpc({ type: 'NAVIGATE', url: targetUrl });
            };

            const btnHardReload = document.createElement('button');
            btnHardReload.type = 'button';
            btnHardReload.className = 'fnx-pop-btn';
            btnHardReload.style.cssText = 'width: auto; height: 25px; padding: 0 8px; font-size: 11px;';
            setBtnIconAndText(btnHardReload, 'reload', isZh() ? '硬刷新' : 'Hard Reload');
            btnHardReload.title = isZh()
               ? '清理 CacheStorage 与 ServiceWorker 并重新加载 (Ctrl+Shift+R)'
               : 'Clear CacheStorage & ServiceWorkers and reload (Ctrl+Shift+R)';
            btnHardReload.onclick = () => performHardCacheBustReload();

            rightWrap.appendChild(devPortSel);
            rightWrap.appendChild(btnHardReload);
            rightWrap.appendChild(btnFpsHud);
            rightWrap.appendChild(btnDesign);
            rightWrap.appendChild(btnWireframe);
            if (!isMobilePlatform()) {
               rightWrap.appendChild(btnViewport);
               const btnF12 = document.createElement('button');
               btnF12.type = 'button';
               btnF12.className = 'fnx-pop-btn';
               btnF12.style.cssText = 'width: auto; height: 25px; padding: 0 8px; font-size: 11px;';
               btnF12.textContent = 'F12';
               btnF12.onclick = () => sendIpc({ type: 'OPEN_DEVTOOLS' });
               rightWrap.appendChild(btnF12);
            }
            const btnClose = document.createElement('button');
            btnClose.type = 'button';
            btnClose.className = 'fnx-sp-close fnx-dev-close';
            btnClose.title = isZh() ? '关闭 (Esc)' : 'Close (Esc)';
            btnClose.setAttribute('aria-label', 'Close developer drawer');
            btnClose.appendChild(createIcon('close'));
            btnClose.onclick = (ev) => {
               if (ev && ev.stopPropagation) ev.stopPropagation();
               toggleDevDrawer(false);
            };

            headTopRow.appendChild(tabWrap);
            headTopRow.appendChild(btnClose);
            head.appendChild(headTopRow);
            head.appendChild(rightWrap);
            devDrawerEl.appendChild(head);

            // Tab 1: Console + JS REPL
            const paneConsole = document.createElement('div');
            paneConsole.style.cssText =
               'display: flex; flex-direction: column; flex: 1; min-height: 0;';
            const cToolbar = document.createElement('div');
            cToolbar.style.cssText =
               'display: flex; align-items: center; gap: 6px; padding: 5px 10px; border-bottom: 1px solid var(--fnx-border); flex-shrink: 0;';
            const filterBtns = Object.create(null);
            ['all', 'error', 'warn', 'log'].forEach((fk) => {
               const fb = document.createElement('button');
               fb.type = 'button';
               fb.className = 'fnx-pop-btn';
               fb.style.cssText = 'width: auto; height: 22px; padding: 0 8px; font-size: 11px; text-transform: capitalize;';
               fb.textContent = fk;
               fb.onclick = () => {
                  devConsoleFilter = fk;
                  Object.keys(filterBtns).forEach((k) => {
                     filterBtns[k].style.borderColor = k === fk ? 'var(--fnx-accent)' : '';
                     filterBtns[k].style.color = k === fk ? 'var(--fnx-accent)' : '';
                  });
                  renderDevConsolePanel();
               };
               filterBtns[fk] = fb;
               cToolbar.appendChild(fb);
            });
            filterBtns.all.style.borderColor = 'var(--fnx-accent)';
            filterBtns.all.style.color = 'var(--fnx-accent)';
            const btnConsoleClear = document.createElement('button');
            btnConsoleClear.type = 'button';
            btnConsoleClear.className = 'fnx-pop-btn';
            btnConsoleClear.style.cssText = 'width: auto; height: 22px; padding: 0 8px; font-size: 11px; margin-left: auto;';
            btnConsoleClear.textContent = isZh() ? '清空' : 'Clear';
            btnConsoleClear.onclick = () => {
               devConsoleLogs.length = 0;
               renderDevConsolePanel();
            };
            cToolbar.appendChild(btnConsoleClear);

            const consoleList = document.createElement('div');
            consoleList.style.cssText =
               'flex: 1; min-height: 0; overflow-y: auto; font-family: ui-monospace, Consolas, monospace; font-size: 11.5px;';

            const replRow = document.createElement('div');
            replRow.style.cssText =
               'display: flex; align-items: center; gap: 6px; padding: 6px 10px; border-top: 1px solid var(--fnx-border); flex-shrink: 0;';
            const replCaret = document.createElement('span');
            replCaret.style.cssText =
               'font-family: ui-monospace, Consolas, monospace; font-weight: 700; color: var(--fnx-accent);';
            replCaret.textContent = '>';
            const replInp = document.createElement('input');
            replInp.type = 'text';
            replInp.style.cssText =
               'flex: 1; min-width: 0; height: 26px; padding: 0 8px; border-radius: 6px; border: 1px solid var(--fnx-border); background: var(--fnx-surface); color: var(--fnx-fg); font-family: ui-monospace, Consolas, monospace; font-size: 11.5px; outline: none;';
            replInp.placeholder = config.allow_page_eval
               ? (isZh()
                    ? '输入 JS 表达式回车执行 (如: document.title)'
                    : 'Evaluate JS on page... (Enter)')
               : (isZh()
                    ? '安全保护：请在「设置 -> 常规」开启控制台 JS 执行'
                    : 'Protected: enable Console JS Eval in Settings -> General');
            const replBtn = document.createElement('button');
            replBtn.type = 'button';
            replBtn.className = 'fnx-pop-btn';
            replBtn.style.cssText = 'width: auto; height: 26px; padding: 0 10px; font-size: 11.5px;';
            replBtn.textContent = isZh() ? '执行' : 'Run';

            const replHistory = [];
            let replHistPos = 0;
            function runRepl() {
               const code = replInp.value.trim();
               if (!code) return;
               if (!config.allow_page_eval) {
                  const msg = isZh()
                     ? '已阻止执行：请先在「设置 -> 常规」开启「允许控制台 JS 执行与原生 DevTools」'
                     : 'Blocked: enable "Allow Console JS Eval & Native DevTools" in Settings -> General';
                  pushDevConsoleEntry('warn', [msg]);
                  showHudToast(msg);
                  return;
               }
               replHistory.push(code);
               replHistPos = replHistory.length;
               replInp.value = '';
               pushDevConsoleEntry('info', ['> ' + code]);
               try {
                  // Evaluate expression cleanly inside active page scope when explicitly permitted
                  const res = window.eval(code);
                  pushDevConsoleEntry('log', ['<-', res]);
               } catch (err) {
                  pushDevConsoleEntry('error', [err]);
               }
            }
            replBtn.onclick = runRepl;
            replInp.onkeydown = (ev) => {
               if (ev.key === 'Enter') {
                  ev.preventDefault();
                  runRepl();
               } else if (ev.key === 'ArrowUp' && replHistory.length > 0) {
                  ev.preventDefault();
                  replHistPos = Math.max(0, replHistPos - 1);
                  replInp.value = replHistory[replHistPos] || '';
               } else if (ev.key === 'ArrowDown' && replHistory.length > 0) {
                  ev.preventDefault();
                  replHistPos = Math.min(replHistory.length, replHistPos + 1);
                  replInp.value = replHistory[replHistPos] || '';
               }
            };
            replRow.appendChild(replCaret);
            replRow.appendChild(replInp);
            replRow.appendChild(replBtn);

            paneConsole.appendChild(cToolbar);
            paneConsole.appendChild(consoleList);
            paneConsole.appendChild(replRow);
            tabPanes.console = paneConsole;

            // Tab 2: Network & Performance
            const paneNet = document.createElement('div');
            paneNet.style.cssText =
               'display: none; flex-direction: column; flex: 1; min-height: 0;';
            const netHead = document.createElement('div');
            netHead.style.cssText =
               'display: flex; align-items: center; gap: 8px; padding: 6px 10px; border-bottom: 1px solid var(--fnx-border); flex-shrink: 0; flex-wrap: wrap;';
            const netSummary = document.createElement('span');
            netSummary.style.cssText = 'font-size: 11px; flex: 1; min-width: 180px;';
            const btnNetRefresh = document.createElement('button');
            btnNetRefresh.type = 'button';
            btnNetRefresh.className = 'fnx-pop-btn';
            btnNetRefresh.style.cssText = 'width: auto; height: 22px; padding: 0 8px; font-size: 11px;';
            btnNetRefresh.textContent = isZh() ? '刷新' : 'Refresh';
            btnNetRefresh.onclick = renderDevNetworkPanel;
            const btnNetClear = document.createElement('button');
            btnNetClear.type = 'button';
            btnNetClear.className = 'fnx-pop-btn';
            btnNetClear.style.cssText = 'width: auto; height: 22px; padding: 0 8px; font-size: 11px;';
            btnNetClear.textContent = isZh() ? '清空' : 'Clear';
            btnNetClear.onclick = () => {
               devNetworkLogs.length = 0;
               devSelectedNetEntry = null;
               try {
                  performance.clearResourceTimings();
               } catch (_) {}
               renderDevNetworkPanel();
            };
            netHead.appendChild(netSummary);
            netHead.appendChild(btnNetRefresh);
            netHead.appendChild(btnNetClear);
            const netList = document.createElement('div');
            netList.style.cssText =
               'flex: 1; min-height: 0; overflow-y: auto; font-family: ui-monospace, Consolas, monospace;';
            const netDetail = document.createElement('pre');
            netDetail.style.cssText =
               'display: none; margin: 0; padding: 7px 10px; max-height: 120px; overflow-y: auto; border-top: 1px solid var(--fnx-border); font-family: ui-monospace, Consolas, monospace; font-size: 11px; white-space: pre-wrap; word-break: break-word; background: var(--fnx-surface); flex-shrink: 0;';
            paneNet.appendChild(netHead);
            paneNet.appendChild(netList);
            paneNet.appendChild(netDetail);
            tabPanes.network = paneNet;

            // Tab 2.5: Built-in API Client (Mini-Postman)
            const paneApi = document.createElement('div');
            paneApi.style.cssText =
               'display: none; flex-direction: column; flex: 1; min-height: 0; padding: 8px 10px; gap: 6px;';
            const apiBar = document.createElement('div');
            apiBar.style.cssText = 'display: flex; align-items: center; gap: 6px; flex-shrink: 0;';
            const apiMethod = document.createElement('select');
            apiMethod.className = 'fnx-pop-btn';
            apiMethod.style.cssText =
               'width: 82px; height: 26px; padding: 0 6px; font-weight: 700; font-size: 11.5px; background: var(--fnx-bg, #141417); color: var(--fnx-fg, #f4f4f6);';
            ['GET', 'POST', 'PUT', 'PATCH', 'DELETE', 'HEAD'].forEach((m) => {
               const o = document.createElement('option');
               o.value = m;
               o.textContent = m;
               apiMethod.appendChild(o);
            });
            const apiUrl = document.createElement('input');
            apiUrl.type = 'text';
            apiUrl.style.cssText =
               'flex: 1; min-width: 0; height: 26px; padding: 0 8px; border-radius: 6px; border: 1px solid var(--fnx-border); background: var(--fnx-surface); color: var(--fnx-fg); font-family: ui-monospace, Consolas, monospace; font-size: 11.5px;';
            apiUrl.placeholder = isZh()
               ? '/api/hello 或 https://...'
               : '/api/hello or https://...';
            const btnApiSend = document.createElement('button');
            btnApiSend.type = 'button';
            btnApiSend.className = 'fnx-pop-btn';
            btnApiSend.style.cssText =
               'width: auto; height: 26px; padding: 0 12px; font-weight: 600; font-size: 11.5px; border-color: var(--fnx-accent); color: var(--fnx-accent);';
            btnApiSend.textContent = isZh() ? '发送' : 'Send';
            apiBar.appendChild(apiMethod);
            apiBar.appendChild(apiUrl);
            apiBar.appendChild(btnApiSend);

            const apiInputsRow = document.createElement('div');
            apiInputsRow.style.cssText = 'display: grid; grid-template-columns: 1fr 1fr; gap: 6px; height: 64px; flex-shrink: 0;';
            const apiHeadersInp = document.createElement('textarea');
            apiHeadersInp.style.cssText =
               'width: 100%; height: 100%; box-sizing: border-box; padding: 5px 7px; border-radius: 6px; border: 1px solid var(--fnx-border); background: var(--fnx-surface); color: var(--fnx-fg); font-family: ui-monospace, Consolas, monospace; font-size: 11px; resize: none;';
            apiHeadersInp.placeholder = '{"Content-Type": "application/json"}';
            const apiBodyInp = document.createElement('textarea');
            apiBodyInp.style.cssText =
               'width: 100%; height: 100%; box-sizing: border-box; padding: 5px 7px; border-radius: 6px; border: 1px solid var(--fnx-border); background: var(--fnx-surface); color: var(--fnx-fg); font-family: ui-monospace, Consolas, monospace; font-size: 11px; resize: none;';
            apiBodyInp.placeholder = isZh() ? 'Request Body (JSON 或文本)' : 'Request Body (JSON / text)';
            apiInputsRow.appendChild(apiHeadersInp);
            apiInputsRow.appendChild(apiBodyInp);

            const apiResOut = document.createElement('textarea');
            apiResOut.readOnly = true;
            apiResOut.style.cssText =
               'flex: 1; min-height: 0; width: 100%; box-sizing: border-box; padding: 6px 8px; border-radius: 6px; border: 1px solid var(--fnx-border); background: var(--fnx-surface); color: var(--fnx-fg); font-family: ui-monospace, Consolas, monospace; font-size: 11px; resize: none;';
            apiResOut.placeholder = isZh() ? '响应状态码与格式化 JSON 将显示在此处...' : 'Response status, headers and JSON appear here...';

            btnApiSend.onclick = () => {
               const rawTarget = apiUrl.value.trim();
               if (!rawTarget) return;
               let target = rawTarget;
               if (target.startsWith('/')) {
                  target = window.location.origin + target;
               }
               let parsedHeaders = {};
               if (apiHeadersInp.value.trim()) {
                  try {
                     parsedHeaders = JSON.parse(apiHeadersInp.value.trim());
                  } catch (err) {
                     apiResOut.value = 'Invalid Headers JSON: ' + err.message;
                     return;
                  }
               }
               const opts = { method: apiMethod.value, headers: parsedHeaders };
               if (apiMethod.value !== 'GET' && apiMethod.value !== 'HEAD' && apiBodyInp.value) {
                  opts.body = apiBodyInp.value;
               }
               apiResOut.value = isZh() ? '正在请求...' : 'Sending request...';
               const t0 = performance.now ? performance.now() : Date.now();
               window.fetch(target, opts)
                  .then((res) =>
                     res.text().then((txt) => {
                        const dur = Math.max(0, Math.round((performance.now ? performance.now() : Date.now()) - t0));
                        let formatted = txt;
                        try {
                           formatted = JSON.stringify(JSON.parse(txt), null, 2);
                        } catch (_) {}
                        const hdrLines = [];
                        res.headers.forEach((v, k) => hdrLines.push(`${k}: ${v}`));
                        apiResOut.value = [
                           `HTTP ${res.status} ${res.statusText} · ${dur} ms`,
                           hdrLines.join('\n'),
                           '--- BODY ---',
                           formatted,
                        ].join('\n');
                     }),
                  )
                  .catch((err) => {
                     apiResOut.value = 'Request failed: ' + (err && err.message ? err.message : String(err));
                  });
            };
            paneApi.appendChild(apiBar);
            paneApi.appendChild(apiInputsRow);
            paneApi.appendChild(apiResOut);
            tabPanes.api = paneApi;

            // Tab 3: Storage & Cookies
            const paneStorage = document.createElement('div');
            paneStorage.style.cssText =
               'display: none; flex-direction: column; flex: 1; min-height: 0;';
            const stHead = document.createElement('div');
            stHead.style.cssText =
               'display: flex; align-items: center; gap: 6px; padding: 6px 10px; border-bottom: 1px solid var(--fnx-border); flex-shrink: 0; flex-wrap: wrap;';
            const stModeBtns = Object.create(null);
            ['local', 'session', 'cookies'].forEach((sk) => {
               const sb = document.createElement('button');
               sb.type = 'button';
               sb.className = 'fnx-pop-btn';
               sb.style.cssText = 'width: auto; height: 22px; padding: 0 8px; font-size: 11px;';
               sb.textContent =
                  sk === 'local'
                     ? 'localStorage'
                     : sk === 'session'
                       ? 'sessionStorage'
                       : 'Cookies';
               sb.onclick = () => {
                  devStorageKind = sk;
                  Object.keys(stModeBtns).forEach((k) => {
                     stModeBtns[k].style.borderColor = k === sk ? 'var(--fnx-accent)' : '';
                     stModeBtns[k].style.color = k === sk ? 'var(--fnx-accent)' : '';
                  });
                  renderDevStoragePanel();
               };
               stModeBtns[sk] = sb;
               stHead.appendChild(sb);
            });
            stModeBtns.local.style.borderColor = 'var(--fnx-accent)';
            stModeBtns.local.style.color = 'var(--fnx-accent)';

            const stKeyInp = document.createElement('input');
            stKeyInp.type = 'text';
            stKeyInp.placeholder = 'key';
            stKeyInp.style.cssText =
               'width: 100px; height: 22px; padding: 0 6px; border-radius: 5px; border: 1px solid var(--fnx-border); background: var(--fnx-surface); color: var(--fnx-fg); font-size: 11px;';
            const stValInp = document.createElement('input');
            stValInp.type = 'text';
            stValInp.placeholder = 'value';
            stValInp.style.cssText =
               'width: 120px; height: 22px; padding: 0 6px; border-radius: 5px; border: 1px solid var(--fnx-border); background: var(--fnx-surface); color: var(--fnx-fg); font-size: 11px;';
            const btnStSet = document.createElement('button');
            btnStSet.type = 'button';
            btnStSet.className = 'fnx-pop-btn';
            btnStSet.style.cssText = 'width: auto; height: 22px; padding: 0 8px; font-size: 11px;';
            btnStSet.textContent = isZh() ? '写入' : 'Set';
            btnStSet.onclick = () => {
               const k = stKeyInp.value.trim();
               if (!k) return;
               try {
                  if (devStorageKind === 'cookies') {
                     document.cookie = `${encodeURIComponent(k)}=${encodeURIComponent(stValInp.value)}; path=/`;
                  } else {
                     const store =
                        devStorageKind === 'session'
                           ? window.sessionStorage
                           : window.localStorage;
                     store.setItem(k, stValInp.value);
                  }
                  stKeyInp.value = '';
                  stValInp.value = '';
                  renderDevStoragePanel();
               } catch (_) {}
            };
            const btnStClear = document.createElement('button');
            btnStClear.type = 'button';
            btnStClear.className = 'fnx-pop-btn';
            btnStClear.style.cssText = 'width: auto; height: 22px; padding: 0 8px; font-size: 11px; margin-left: auto;';
            btnStClear.textContent = isZh() ? '清空当前' : 'Clear';
            btnStClear.onclick = () => {
               try {
                  if (devStorageKind === 'cookies') {
                     String(document.cookie || '')
                        .split(/;\s*/)
                        .forEach((p) => {
                           const eq = p.indexOf('=');
                           if (eq > 0) {
                              document.cookie = `${p.slice(0, eq)}=; max-age=0; path=/`;
                           }
                        });
                  } else {
                     const store =
                        devStorageKind === 'session'
                           ? window.sessionStorage
                           : window.localStorage;
                     store.clear();
                  }
                  renderDevStoragePanel();
               } catch (_) {}
            };
            stHead.appendChild(stKeyInp);
            stHead.appendChild(stValInp);
            stHead.appendChild(btnStSet);
            stHead.appendChild(btnStClear);
            const storageList = document.createElement('div');
            storageList.style.cssText =
               'flex: 1; min-height: 0; overflow-y: auto; font-family: ui-monospace, Consolas, monospace;';
            paneStorage.appendChild(stHead);
            paneStorage.appendChild(storageList);
            tabPanes.storage = paneStorage;

            // Tab 4: DOM Inspector & Page Source
            const paneDom = document.createElement('div');
            paneDom.style.cssText =
               'display: none; flex-direction: column; flex: 1; min-height: 0;';
            const domHead = document.createElement('div');
            domHead.style.cssText =
               'display: flex; align-items: center; gap: 6px; padding: 6px 10px; border-bottom: 1px solid var(--fnx-border); flex-shrink: 0; flex-wrap: wrap;';
            const btnDomPick = document.createElement('button');
            btnDomPick.type = 'button';
            btnDomPick.className = 'fnx-pop-btn';
            btnDomPick.style.cssText = 'width: auto; height: 22px; padding: 0 9px; font-size: 11px;';
            setBtnIconAndText(btnDomPick, 'target', isZh() ? '点选检查元素' : 'Inspect Element');
            const btnDomHtml = document.createElement('button');
            btnDomHtml.type = 'button';
            btnDomHtml.className = 'fnx-pop-btn';
            btnDomHtml.style.cssText = 'width: auto; height: 22px; padding: 0 9px; font-size: 11px;';
            setBtnIconAndText(btnDomHtml, 'code', isZh() ? '查看全页 HTML' : 'Page HTML');
            btnDomHtml.onclick = () => renderDevDomInfo(null);
            const btnDomCopy = document.createElement('button');
            btnDomCopy.type = 'button';
            btnDomCopy.className = 'fnx-pop-btn';
            btnDomCopy.style.cssText = 'width: auto; height: 22px; padding: 0 9px; font-size: 11px;';
            setBtnIconAndText(btnDomCopy, 'copy', isZh() ? '复制完整 HTML' : 'Copy Full HTML');
            btnDomCopy.onclick = () => {
               const html = document.documentElement
                  ? document.documentElement.outerHTML
                  : '';
               if (navigator.clipboard && navigator.clipboard.writeText) {
                  navigator.clipboard.writeText(html).catch(() => {});
               }
               showHudToast(isZh() ? '已复制完整页面源码' : 'Full HTML copied');
            };
            domHead.appendChild(btnDomPick);
            domHead.appendChild(btnDomHtml);
            domHead.appendChild(btnDomCopy);

            const domSummary = document.createElement('pre');
            domSummary.style.cssText =
               'margin: 0; padding: 6px 10px; border-bottom: 1px solid var(--fnx-border); font-family: ui-monospace, Consolas, monospace; font-size: 11px; white-space: pre-wrap; max-height: 92px; overflow-y: auto; flex-shrink: 0;';
            const domSource = document.createElement('textarea');
            domSource.readOnly = true;
            domSource.style.cssText =
               'flex: 1; min-height: 0; width: 100%; box-sizing: border-box; padding: 8px 10px; border: none; outline: none; resize: none; background: transparent; color: var(--fnx-fg); font-family: ui-monospace, Consolas, monospace; font-size: 11px;';

            btnDomPick.onclick = () => {
               if (devInspectActive) {
                  stopDevInspectPicker();
                  return;
               }
               ensurePickerUi();
               devInspectActive = true;
               btnDomPick.style.borderColor = 'var(--fnx-accent)';
               btnDomPick.style.color = 'var(--fnx-accent)';
               devInspectOnHover = (ev) => {
                  if (!devInspectActive || !pickerHighlightBox) return;
                  const t = ev.target;
                  if (
                     !t ||
                     t.nodeType !== 1 ||
                     isHudOwnTarget(t) ||
                     t === document.documentElement ||
                     t === document.body
                  ) {
                     pickerHighlightBox.style.display = 'none';
                     return;
                  }
                  const r = t.getBoundingClientRect();
                  if (!r || r.width < 2 || r.height < 2) return;
                  pickerHighlightBox.style.left = Math.max(0, r.left) + 'px';
                  pickerHighlightBox.style.top = Math.max(0, r.top) + 'px';
                  pickerHighlightBox.style.width = r.width + 'px';
                  pickerHighlightBox.style.height = r.height + 'px';
                  pickerHighlightBox.style.display = 'block';
               };
               devInspectOnClick = (ev) => {
                  if (!devInspectActive) return;
                  if (isHudOwnTarget(ev.target)) return;
                  ev.preventDefault();
                  ev.stopPropagation();
                  if (typeof ev.stopImmediatePropagation === 'function') {
                     ev.stopImmediatePropagation();
                  }
                  const el =
                     ev.target && ev.target.nodeType === 1
                        ? ev.target
                        : document.elementFromPoint(ev.clientX || 0, ev.clientY || 0);
                  stopDevInspectPicker();
                  if (el) renderDevDomInfo(el);
               };
               devInspectOnKeydown = (ev) => {
                  if (ev.key === 'Escape') {
                     ev.preventDefault();
                     ev.stopPropagation();
                     stopDevInspectPicker();
                  }
               };
               window.addEventListener('mousemove', devInspectOnHover, true);
               window.addEventListener('mouseover', devInspectOnHover, true);
               window.addEventListener('click', devInspectOnClick, true);
               window.addEventListener('keydown', devInspectOnKeydown, true);
               showHudToast(
                  isZh() ? '请点击页面任意元素查看样式与盒模型' : 'Click any element to inspect box & CSS',
               );
            };

            paneDom.appendChild(domHead);
            paneDom.appendChild(domSummary);
            paneDom.appendChild(domSource);
            tabPanes.dom = paneDom;

            devDrawerEl.appendChild(paneConsole);
            devDrawerEl.appendChild(paneNet);
            devDrawerEl.appendChild(paneApi);
            devDrawerEl.appendChild(paneStorage);
            devDrawerEl.appendChild(paneDom);
            shell.appendChild(devDrawerEl);

            devDrawerRefs = {
               consoleList,
               netSummary,
               netList,
               netDetail,
               storageList,
               domSummary,
               domSource,
               btnDomPick,
               selectDevTab,
               syncDevQuickToggles,
            };
            syncDevDrawerLocalizationRef = () => {
               const z = isZh();
               for (let i = 0; i < tabDefs.length; i++) {
                  const [k, enT, zhT] = tabDefs[i];
                  if (tabButtons[k]) tabButtons[k].textContent = z ? zhT : enT;
               }
               rebuildPortOpts();
               syncDevQuickToggles();
               setBtnIconAndText(btnHardReload, 'reload', z ? '硬刷新' : 'Hard Reload');
               btnHardReload.title = z
                  ? '清理 CacheStorage 与 ServiceWorker 并重新加载 (Ctrl+Shift+R)'
                  : 'Clear CacheStorage & ServiceWorkers and reload (Ctrl+Shift+R)';
               btnClose.title = z ? '关闭 (Esc)' : 'Close (Esc)';
               btnConsoleClear.textContent = z ? '清空' : 'Clear';
               replInp.placeholder = config.allow_page_eval
                  ? (z
                       ? '输入 JS 表达式回车执行 (如: document.title)'
                       : 'Evaluate JS on page... (Enter)')
                  : (z
                       ? '安全保护：请在「设置 -> 常规」开启控制台 JS 执行'
                       : 'Protected: enable Console JS Eval in Settings -> General');
               replBtn.textContent = z ? '执行' : 'Run';
               btnNetRefresh.textContent = z ? '刷新' : 'Refresh';
               btnNetClear.textContent = z ? '清空' : 'Clear';
               apiUrl.placeholder = z ? '/api/hello 或 https://...' : '/api/hello or https://...';
               btnApiSend.textContent = z ? '发送' : 'Send';
               apiBodyInp.placeholder = z ? 'Request Body (JSON 或文本)' : 'Request Body (JSON / text)';
               apiResOut.placeholder = z
                  ? '响应状态码与格式化 JSON 将显示在此处...'
                  : 'Response status, headers and JSON appear here...';
               btnStSet.textContent = z ? '写入' : 'Set';
               btnStClear.textContent = z ? '清空当前' : 'Clear';
               setBtnIconAndText(btnDomPick, 'target', z ? '点选检查元素' : 'Inspect Element');
               setBtnIconAndText(btnDomHtml, 'code', z ? '查看全页 HTML' : 'Page HTML');
               setBtnIconAndText(btnDomCopy, 'copy', z ? '复制完整 HTML' : 'Copy Full HTML');
               if (isDevDrawerOpen) {
                  if (devActiveTab === 'console') renderDevConsolePanel();
                  else if (devActiveTab === 'network') renderDevNetworkPanel();
                  else if (devActiveTab === 'storage') renderDevStoragePanel();
                  else if (devActiveTab === 'dom') renderDevDomInfo(null);
               }
            };
            syncDevQuickToggles();
            selectDevTab(devActiveTab);
            return devDrawerRefs;
         }

         function toggleDevDrawer(force) {
            if (IS_HUD_SHELL) {
               if (typeof closeAllNavPopoversRef === 'function') {
                  closeAllNavPopoversRef();
               }
               sendIpc({
                  type: 'CONTENT_ACTION',
                  action: 'toggle_dev_drawer',
                  payload: {},
               });
               return;
            }
            const next = force !== undefined ? !!force : !isDevDrawerOpen;
            if (!next && !devDrawerEl) {
               isDevDrawerOpen = false;
               return;
            }
            const refs = ensureDevDrawerMounted();
            isDevDrawerOpen = next;
            if (!next) {
               stopDevInspectPicker();
               devDrawerEl.style.display = 'none';
               if (IS_CONTENT_TAB && typeof reportContentPageMetaRef === 'function') {
                  reportContentPageMetaRef(true);
               }
               return;
            }
            const dockBottomPx = measureDevDrawerBottomPx();
            devDrawerEl.style.bottom = dockBottomPx + 'px';
            const availH = Math.max(
               220,
               Math.round((window.innerHeight || 800) - dockBottomPx - 20),
            );
            devDrawerEl.style.maxHeight = availH + 'px';
            const curH = parseInt(devDrawerEl.style.height, 10) || 340;
            devDrawerEl.style.height = Math.max(220, Math.min(availH, curH)) + 'px';
            devDrawerEl.style.display = 'flex';
            refs.syncDevQuickToggles();
            refs.selectDevTab(devActiveTab);
            if (IS_CONTENT_TAB && typeof reportContentPageMetaRef === 'function') {
               reportContentPageMetaRef(true);
            }
         }
         toggleDevDrawerRef = toggleDevDrawer;
         isDevDrawerOpenRef = () => isDevDrawerOpen;

         // 3.6 History Drawer Modal (Linear / Raycast Style)
         const historyRoot = document.createElement('div');
         historyRoot.id = '__fynexo_history_root__';

         const historyCard = document.createElement('div');
         historyCard.id = '__fynexo_history_card__';
         historyCard.onclick = (e) => e.stopPropagation();

         const histHeader = document.createElement('div');
         histHeader.className = 'fnx-pop-head';
         const histTitle = document.createElement('span');
         histTitle.className = 'fnx-pop-title';
         histTitle.textContent = isZh() ? '浏览历史记录' : 'Browsing History';
         const histCountBadge = document.createElement('span');
         histCountBadge.className = 'fnx-pop-badge';
         const histCloseBtn = document.createElement('button');
         histCloseBtn.type = 'button';
         histCloseBtn.className = 'fnx-sp-close';
         histCloseBtn.title = isZh() ? '关闭 (Esc)' : 'Close (Esc)';
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
         histSearchInput.placeholder = isZh() ? '搜索访问过的网页...' : 'Search visited pages...';
         histSearchWrap.appendChild(histSearchInput);

         const histList = document.createElement('div');
         histList.className = 'fnx-hist-list';

         const histFooter = document.createElement('button');
         histFooter.type = 'button';
         histFooter.className = 'fnx-pop-footer-row';
         const histFooterLabel = document.createElement('span');
         histFooterLabel.textContent = isZh() ? '在偏好设置中管理历史记录' : 'Manage browsing history';
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
            historyCard.style.transition = '';
            historyCard.style.transform = '';
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
            histCountBadge.textContent = isZh()
               ? `${filtered.length} 条记录`
               : (filtered.length === 1 ? '1 item' : `${filtered.length} items`);
            histList.textContent = '';

            if (filtered.length === 0) {
               const empty = document.createElement('div');
               empty.className = 'fnx-hist-empty';
               empty.textContent = q
                  ? (isZh() ? '历史记录中未找到匹配网页。' : 'No matching pages found in history.')
                  : (isZh() ? '暂无浏览历史记录。' : 'No visited pages recorded yet.');
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
               bgBtn.title = isZh() ? '在新标签页中打开' : 'Open in new tab';
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
            historyCard.style.transition = '';
            historyCard.style.transform = '';
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
         bindBottomSheetSwipeDismiss(historyCard, histList, closeHistoryDrawer);

         // 3.7 Custom Prompt Dialog (Replaces native window.prompt)
         const promptRoot = document.createElement('div');
         promptRoot.id = '__fynexo_prompt_root__';

         const promptCard = document.createElement('div');
         promptCard.id = '__fynexo_prompt_card__';
         promptCard.onclick = (e) => e.stopPropagation();

         const promptHeader = document.createElement('div');
         promptHeader.className = 'fnx-pop-head';
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
         promptCancelBtn.textContent = isZh() ? '取消' : 'Cancel';
         const promptConfirmBtn = document.createElement('button');
         promptConfirmBtn.type = 'button';
         promptConfirmBtn.className = 'fnx-prompt-btn primary';
         promptConfirmBtn.textContent = isZh() ? '确定' : 'Confirm';
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

         function syncHudLocalization() {
            const z = isZh();
            btnAddTab.title = (z ? '新建标签页' : 'New tab') + ` (${keybinds.new_tab || 'Ctrl+T'})`;
            btnReload.title = (z ? '重新加载' : 'Reload') + ` (${keybinds.reload || 'Ctrl+R'})`;
            addrBtn.title = z ? '点击编辑网址或搜索 (支持 !bang 快捷指令)' : 'Click to edit URL or search (!bangs supported)';
            btnDeskCopy.title = z ? '复制当前网址 (Ctrl+Shift+C)' : 'Copy current URL (Ctrl+Shift+C)';
            btnMedia.title = z ? '媒体控制与音量增强' : 'Media Control & Volume Booster';
            btnChromeExts.title = z ? 'Chrome 扩展程序' : 'Chrome Extensions';
            btnShield.title = z ? '隐私拦截与站点开关' : 'Adblock & Privacy Shield';
            btnSidePanel.title = (z ? '分屏侧边栏' : 'Side Panel') + ` (${keybinds.toggle_side_panel || 'Alt+S'})`;
            btnMore.title = z ? '主菜单' : 'Menu';
            if (winControls && winControls.children.length >= 3) {
               winControls.children[0].title = z ? '最小化' : 'Minimize';
               winControls.children[1].title = z ? '最大化' : 'Maximize';
               winControls.children[2].title = z ? '关闭窗口' : 'Close';
            }
            btnZoomOut.title = (z ? '缩小' : 'Zoom out') + ` (${keybinds.zoom_out || 'Ctrl+-'})`;
            zoomValSpan.title = z ? '点击重置缩放 (100%)' : 'Click to reset zoom (100%)';
            btnZoomIn.title = (z ? '放大' : 'Zoom in') + ` (${keybinds.zoom_in || 'Ctrl+='})`;
            btnZoomFull.title = (z ? '全屏' : 'Fullscreen') + ` (${keybinds.fullscreen || 'F11'})`;

            bmStartBtn.title = z ? '打开起始页 (fynexo://newtab)' : 'Open Start page (fynexo://newtab)';
            bmStartText.textContent = z ? '起始页' : 'Start';

            findInput.placeholder = z ? '在页面中查找...' : 'Find in page...';
            btnFindPrev.title = z ? '上一个匹配 (Shift+Enter)' : 'Previous match (Shift+Enter)';
            btnFindNext.title = z ? '下一个匹配 (Enter)' : 'Next match (Enter)';
            btnFindClose.title = z ? '关闭 (Esc)' : 'Close (Esc)';

            tmTitleTxt.textContent = z ? '内存与资源监视' : 'Resource Monitor';
            tmBadge.textContent = z ? '站点隔离' : 'Site-Isolated';
            if (tmHostRow.__fnxLabelEl) tmHostRow.__fnxLabelEl.textContent = z ? '总内存 (Host + 渲染)' : 'Total RAM (Host + Tabs)';
            if (tmCommitRow.__fnxLabelEl) tmCommitRow.__fnxLabelEl.textContent = z ? '内核 / 子进程提交' : 'Committed / Procs';
            if (tmModeRow.__fnxLabelEl) tmModeRow.__fnxLabelEl.textContent = z ? 'Edge 能效压缩' : 'Efficiency Pool';
            if (tmModeRow.__fnxValueEl) {
               const effOn = config.efficiency_mode !== false;
               tmModeRow.__fnxValueEl.textContent = effOn ? (z ? '已启用' : 'Active') : (z ? '已暂停' : 'Off');
               tmModeRow.__fnxValueEl.style.color = effOn ? '#10B981' : 'var(--fnx-muted)';
            }
            cacheTitle.textContent = z ? '后台标签缓存上限' : 'Cached tab limit';
            btnPerfSet.textContent = z ? '性能设置' : 'Performance';
            btnTrimMem.textContent = z ? '释放内存' : 'Flush RAM';

            histTitle.textContent = z ? '浏览历史记录' : 'Browsing History';
            histCloseBtn.title = z ? '关闭 (Esc)' : 'Close (Esc)';
            histSearchInput.placeholder = z ? '搜索访问过的网页...' : 'Search visited pages...';
            histFooterLabel.textContent = z ? '在偏好设置中管理历史记录' : 'Manage browsing history';
            if (!isPromptDialogOpen) {
               promptCancelBtn.textContent = z ? '取消' : 'Cancel';
               promptConfirmBtn.textContent = z ? '确定' : 'Confirm';
            }

            mbBack.setAttribute('aria-label', z ? '后退' : 'Back');
            mbReload.setAttribute('aria-label', z ? '重新加载' : 'Reload');
            mbTabs.setAttribute('aria-label', z ? '标签页' : 'Tabs');
            mbMore.setAttribute('aria-label', z ? '菜单与工具' : 'Menu & tools');
            btnDrawerPrefs.setAttribute('aria-label', z ? '偏好设置' : 'Preferences');
            btnSheetFwd.title = z ? '前进' : 'Forward';
            btnSheetStar.title = z ? '收藏本页' : 'Bookmark page';

            const rdMeta = shadowRoot ? shadowRoot.getElementById('__fynexo_reader_meta__') : null;
            if (rdMeta) rdMeta.textContent = z ? '纯净阅读模式' : 'Reader mode';
            const rdW = shadowRoot ? shadowRoot.getElementById('__fynexo_reader_width__') : null;
            if (rdW) {
               rdW.textContent = z ? '宽版' : 'Wide';
               rdW.title = z ? '切换宽版/窄版栏宽' : 'Toggle wide column';
            }
            const rdExit = shadowRoot ? shadowRoot.getElementById('__fynexo_reader_exit__') : null;
            if (rdExit) setBtnIconAndText(rdExit, 'close', z ? '退出阅读' : 'Exit Reader');

            syncNavButtons();
            syncBookmarkButtons();
            syncBackgroundMediaDock();
            syncDownloadsUI();
            syncChromeExtsUI();
            syncShieldPopoverUI();
            syncMediaPopoverUI();
            syncMainMenuUI();
            updateAddressLabels();
            renderTabs(true);
            if (isDrawerOpen) renderDrawerCards();
            if (typeof syncSidePanelLocalizationRef === 'function') syncSidePanelLocalizationRef();
            if (typeof syncDevDrawerLocalizationRef === 'function') syncDevDrawerLocalizationRef();
         }
         syncHudLocalizationRef = syncHudLocalization;
         syncHudLocalization();

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
            inp.placeholder = omniboxPlaceholder(currentEngineObj().name);

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
                  inp.placeholder = omniboxPlaceholder(next.name);
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
            input.placeholder = omniboxPlaceholder(currentEngineObj().name);
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
                  input.placeholder = omniboxPlaceholder(next.name);
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
         let lastRemotePeekAt = 0;
         window.addEventListener(
            'mousemove',
            (e) => {
               if (!isFullscreen) return;
               if (IS_CONTENT_TAB) {
                  if (e.clientY <= 4) {
                     const now = Date.now();
                     if (now - lastRemotePeekAt >= 250) {
                        lastRemotePeekAt = now;
                        sendIpc({ type: 'HUD_ACTION', action: 'fullscreen_peek' });
                     }
                  }
                  return;
               }
               if (e.clientY <= 6 && !isTopbarPeek) {
                  isTopbarPeek = true;
                  applyShadowStyles();
               }
            },
            { passive: true },
         );

         if (IS_CONTENT_TAB) {
            window.addEventListener(
               'pointerdown',
               (e) => {
                  if (!isEventInsideHud(e)) {
                     sendIpc({ type: 'CONTENT_POINTER_DOWN' });
                  }
               },
               { capture: true, passive: true },
            );
         }

         // Mobile Webpage Input Auto-Hide
         // Note: On native Android (__FNX_MOBILE_HOST), MainActivity tracks real OS IME visibility
         // via WindowInsetsCompat.Type.ime() and invokes __fynexo_on_ime_inset__(true/false).
         // Never hide the dock blindly on `focusin`, because pages like wikipedia.org set
         // <input autofocus> on load without opening any soft keyboard.
         let lastWebPointerDownTs = 0;
         document.addEventListener(
            'pointerdown',
            () => {
               lastWebPointerDownTs = Date.now();
            },
            { capture: true, passive: true },
         );
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
                     if (!__FNX_MOBILE_HOST) {
                        if (Date.now() - lastWebPointerDownTs < 1200) {
                           setKeyboardHiddenState(true);
                        }
                     }
                     try {
                        setTimeout(() => {
                           if (
                              isPageKeyboardOpen &&
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
               if (window.innerWidth <= 768 && !__FNX_MOBILE_HOST) {
                  setTimeout(() => {
                     if (!isEditableWebElement(getDeepActiveElement())) {
                        setKeyboardHiddenState(false);
                     }
                  }, 120);
               }
            },
            true,
         );

         if (window.visualViewport && !__FNX_MOBILE_HOST) {
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
                     if (ratio < 0.72) {
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
                  if (IS_CONTENT_TAB) {
                     sendIpc({ type: 'HUD_ACTION', action: 'open_omnibox' });
                  } else {
                     openHud();
                  }
                  return;
               }
               if (matchesKeybind(e, keybinds.reopen_closed_tab)) {
                  e.preventDefault();
                  e.stopPropagation();
                  triggerProgress();
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
                  if (!IS_HUD_SHELL) triggerProgress();
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
               const cycleRelativeTab = (step) => {
                  if (tabState.tabs.length > 1) {
                     const curIdx = tabState.tabs.findIndex(
                        (t) => t.id === tabState.active_id,
                     );
                     const nextIdx =
                        (curIdx + step + tabState.tabs.length) %
                        tabState.tabs.length;
                     flushScrollSync();
                     if (!IS_HUD_SHELL) triggerProgress();
                     sendIpc({
                        type: 'SWITCH_TAB',
                        id: tabState.tabs[nextIdx].id,
                     });
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
                  (String(keybinds.next_tab || '').toLowerCase() ===
                     'ctrl+tab' &&
                     e.ctrlKey &&
                     e.shiftKey &&
                     e.key === 'Tab')
               ) {
                  e.preventDefault();
                  e.stopPropagation();
                  cycleRelativeTab(e.shiftKey ? -1 : 1);
                  return;
               }
               if (
                  ((e.ctrlKey || e.metaKey) &&
                     e.shiftKey &&
                     !e.altKey &&
                     (e.key || '').toLowerCase() === 'r') ||
                  ((e.ctrlKey || e.metaKey) &&
                     !e.shiftKey &&
                     !e.altKey &&
                     e.key === 'F5')
               ) {
                  e.preventDefault();
                  e.stopPropagation();
                  performHardCacheBustReload();
                  return;
               }
               if (matchesKeybind(e, keybinds.reload)) {
                  e.preventDefault();
                  e.stopPropagation();
                  triggerProgress();
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
                  if (IS_CONTENT_TAB) {
                     sendIpc({ type: 'HUD_ACTION', action: 'toggle_side_panel' });
                  } else if (typeof toggleSidePanel === 'function') {
                     toggleSidePanel();
                  }
                  return;
               }
               if (matchesKeybind(e, keybinds.pin_tab)) {
                  e.preventDefault();
                  e.stopPropagation();
                  if (tabState.active_id) {
                     sendIpc({ type: 'TOGGLE_PIN_TAB', id: tabState.active_id });
                  }
                  return;
               }
               if (matchesKeybind(e, keybinds.mute_tab)) {
                  e.preventDefault();
                  e.stopPropagation();
                  if (tabState.active_id) {
                     sendIpc({ type: 'TOGGLE_MUTE_TAB', id: tabState.active_id });
                  }
                  return;
               }
               if (matchesKeybind(e, keybinds.bookmark_page)) {
                  e.preventDefault();
                  e.stopPropagation();
                  if (IS_CONTENT_TAB) {
                     sendIpc({ type: 'HUD_ACTION', action: 'toggle_bookmark' });
                  } else {
                     toggleCurrentBookmark();
                  }
                  return;
               }
               if (
                  (e.ctrlKey || e.metaKey) &&
                  !e.altKey &&
                  !e.shiftKey &&
                  (e.key || '').toLowerCase() === 'p'
               ) {
                  e.preventDefault();
                  e.stopPropagation();
                  if (IS_HUD_SHELL) {
                     sendIpc({
                        type: 'CONTENT_ACTION',
                        action: 'print_page',
                        payload: {},
                     });
                  } else {
                     triggerNativeOrWindowPrint();
                  }
                  return;
               }
               if (
                  (e.ctrlKey || e.metaKey) &&
                  !e.altKey &&
                  !e.shiftKey &&
                  (e.key || '').toLowerCase() === 'h'
               ) {
                  e.preventDefault();
                  e.stopPropagation();
                  if (IS_CONTENT_TAB) {
                     sendIpc({ type: 'HUD_ACTION', action: 'open_history' });
                  } else {
                     sendIpc({ type: 'NEW_TAB', url: 'fynexo://settings#sec-privacy' });
                  }
                  return;
               }
               if (
                  (e.ctrlKey || e.metaKey) &&
                  !e.altKey &&
                  !e.shiftKey &&
                  (e.key || '').toLowerCase() === 'j'
               ) {
                  e.preventDefault();
                  e.stopPropagation();
                  if (IS_CONTENT_TAB) {
                     sendIpc({ type: 'HUD_ACTION', action: 'toggle_downloads' });
                  } else if (typeof toggleDownloadsPopover === 'function') {
                     toggleDownloadsPopover();
                  }
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
                  if (IS_CONTENT_TAB) {
                     sendIpc({ type: 'HUD_ACTION', action: 'toggle_force_dark' });
                     return;
                  }
                  config.force_dark = !config.force_dark;
                  sendIpc({
                     type: 'PATCH_HUD_PREFS',
                     force_dark: config.force_dark,
                  });
                  applyPageRules();
                  syncBookmarkButtons();
                  if ((IS_HUD_SHELL ? !!remotePageMeta.page_is_dark : isPageAlreadyDark()) && config.force_dark) {
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
                  if (IS_CONTENT_TAB) {
                     flushScrollSync();
                     sendIpc({ type: 'HUD_ACTION', action: 'toggle_preferences' });
                  } else {
                     closeAnyOmnibox();
                     flushScrollSync();
                     togglePreferences();
                  }
                  return;
               }
               if (matchesKeybind(e, keybinds.devtools)) {
                  e.preventDefault();
                  e.stopPropagation();
                  if (!isMobilePlatform()) {
                     sendIpc({ type: 'OPEN_DEVTOOLS' });
                  } else if (typeof toggleDevDrawerRef === 'function') {
                     toggleDevDrawerRef();
                  }
                  return;
               }
               if (matchesKeybind(e, keybinds.dev_drawer)) {
                  e.preventDefault();
                  e.stopPropagation();
                  if (typeof toggleDevDrawerRef === 'function') {
                     toggleDevDrawerRef();
                  }
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
                           if (!IS_HUD_SHELL) triggerProgress();
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
                     if (IS_CONTENT_TAB) {
                        sendIpc({ type: 'HUD_ACTION', action: 'copy_url' });
                     } else {
                        copyCurrentUrl();
                     }
                     return;
                  }
               }
               if (e.key === 'Escape') {
                  if (IS_CONTENT_TAB) {
                     if (isDevDrawerOpenRef && isDevDrawerOpenRef()) {
                        if (toggleDevDrawerRef) toggleDevDrawerRef(false);
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
                     if (isFullscreen) {
                        sendIpc({ type: 'TOGGLE_FULLSCREEN', force: false });
                        return;
                     }
                     sendIpc({ type: 'HUD_ACTION', action: 'dismiss_overlays' });
                     return;
                  }
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
                  if (isDevDrawerOpenRef && isDevDrawerOpenRef()) {
                     if (toggleDevDrawerRef) toggleDevDrawerRef(false);
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
               if (IS_HUD_SHELL || isInternalAny() || isReaderOpen) return;
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
               if (IS_HUD_SHELL || isMobilePlatform() || isEventInsideHud(e)) return;
               e.preventDefault();
               showPageContextMenu(e);
            },
            false,
         );
         window.addEventListener(
            'blur',
            () => {
               hideTabContextMenu();
               hideTabHoverCard();
            },
            { passive: true },
         );
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

         if (!IS_HUD_SHELL && !IS_CONTENT_TAB) {
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
         }

         // Viewport & Dimensions Patch for Web Games, Canvas & IO Games (single-WebView only)
         if (!IS_HUD_SHELL && !IS_CONTENT_TAB) {
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
         }

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
            if (IS_HUD_SHELL || IS_CONTENT_TAB) return;
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

         if (!IS_HUD_SHELL && !IS_CONTENT_TAB) {
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
         }

         // Universal Fixed Header & Mobile Bottom-Bar HUD-Avoidance Harmonizer (single-WebView external pages)
         (() => {
            if (IS_HUD_SHELL || IS_CONTENT_TAB || isInternalAny()) return;
            let fixedHdrPatching = false;
            let fixedHdrTimer = null;
            let fixedHdrLastRun = 0;
            let fixedHdrBodyScrollBound = false;
            let fixedHdrObserverAttached = false;

            function saveInlineProp(el, prop, attrName, priAttrName) {
               if (!el.hasAttribute(attrName)) {
                  el.setAttribute(
                     attrName,
                     el.style.getPropertyValue(prop) || '',
                  );
                  el.setAttribute(
                     priAttrName,
                     el.style.getPropertyPriority(prop) || '',
                  );
               }
            }

            function restoreInlineProp(el, prop, attrName, priAttrName) {
               if (!el.hasAttribute(attrName)) return;
               const prev = el.getAttribute(attrName) || '';
               const pri = el.getAttribute(priAttrName) || '';
               if (prev) el.style.setProperty(prop, prev, pri);
               else el.style.removeProperty(prop);
               el.removeAttribute(attrName);
               el.removeAttribute(priAttrName);
            }

            function unpatchFixedHeader(el) {
               if (!el) return;
               restoreInlineProp(
                  el,
                  'top',
                  'data-fnx-fixed-prev-top',
                  'data-fnx-fixed-prev-top-pri',
               );
               restoreInlineProp(
                  el,
                  'inset-block-start',
                  'data-fnx-fixed-prev-ibs',
                  'data-fnx-fixed-prev-ibs-pri',
               );
               restoreInlineProp(
                  el,
                  'left',
                  'data-fnx-fixed-prev-left',
                  'data-fnx-fixed-prev-left-pri',
               );
               restoreInlineProp(
                  el,
                  'max-width',
                  'data-fnx-fixed-prev-mw',
                  'data-fnx-fixed-prev-mw-pri',
               );
               el.removeAttribute('data-fnx-fixed-hdr');
               el.removeAttribute('data-fnx-fixed-orig-top');
               el.removeAttribute('data-fnx-fixed-orig-left');
               el.removeAttribute('data-fnx-fixed-left-patched');
            }

            function resetAllFixedHeaders() {
               const patched = document.querySelectorAll('[data-fnx-fixed-hdr]');
               for (let i = 0; i < patched.length; i++) {
                  unpatchFixedHeader(patched[i]);
               }
            }

            function unpatchFixedBottomBar(el) {
               if (!el) return;
               restoreInlineProp(
                  el,
                  'bottom',
                  'data-fnx-fixed-prev-btm',
                  'data-fnx-fixed-prev-btm-pri',
               );
               restoreInlineProp(
                  el,
                  'inset-block-end',
                  'data-fnx-fixed-prev-ibe',
                  'data-fnx-fixed-prev-ibe-pri',
               );
               el.removeAttribute('data-fnx-fixed-btm');
               el.removeAttribute('data-fnx-fixed-orig-btm');
            }

            function resetAllFixedBottomBars() {
               const patched = document.querySelectorAll('[data-fnx-fixed-btm]');
               for (let i = 0; i < patched.length; i++) {
                  unpatchFixedBottomBar(patched[i]);
               }
            }

            function isExcludedFixedNode(el) {
               if (!el || el.nodeType !== 1) return true;
               if (
                  el === document.documentElement ||
                  el === document.body
               ) {
                  return true;
               }
               const id = el.id || '';
               if (id === '__fynexo_host__' || id.startsWith('__fynexo_')) {
                  return true;
               }
               if (el.closest && el.closest('#__fynexo_host__')) return true;
               if (el.hasAttribute('data-fnx-gh-offset')) return true;
               const tag = el.tagName;
               if (
                  id === 'masthead-container' ||
                  id === 'guide' ||
                  id === 'guide-wrapper' ||
                  tag === 'TP-YT-APP-DRAWER' ||
                  tag === 'YTD-MINI-GUIDE-RENDERER' ||
                  tag === 'YTD-VIDEO-PREVIEW'
               ) {
                  return true;
               }
               const role = (el.getAttribute('role') || '').toLowerCase();
               if (
                  tag === 'DIALOG' ||
                  role === 'dialog' ||
                  el.getAttribute('aria-modal') === 'true'
               ) {
                  return true;
               }
               return false;
            }

            function hasNegativeYTransform(cs) {
               if (
                  !cs ||
                  !cs.transform ||
                  cs.transform === 'none' ||
                  typeof DOMMatrix !== 'function'
               ) {
                  return false;
               }
               try {
                  return new DOMMatrix(cs.transform).f < -2;
               } catch (_) {
                  return false;
               }
            }

            function measureHudOffsets() {
               if (
                  isInternalAny() ||
                  isFullscreen ||
                  window.innerWidth < 120 ||
                  window.innerHeight < 120
               ) {
                  return { topPx: 0, leftPx: 0, bottomPx: 0 };
               }
               if (isMobilePlatform()) {
                  if (styles.mobile_bar.visible === false || isPageKeyboardOpen) {
                     return { topPx: 0, leftPx: 0, bottomPx: 0 };
                  }
                  const mbEl =
                     shadowRoot &&
                     shadowRoot.getElementById('__fynexo_bottom_bar__');
                  const mbRect = mbEl ? mbEl.getBoundingClientRect() : null;
                  const mbH =
                     mbRect &&
                     mbRect.height > 1 &&
                     mbRect.height < window.innerHeight * 0.3
                        ? Math.ceil(mbRect.height)
                        : 0;
                  const mobIsTop =
                     (styles.mobile_bar.position || 'bottom').toLowerCase() ===
                     'top';
                  return mobIsTop
                     ? { topPx: mbH, leftPx: 0, bottomPx: 0 }
                     : { topPx: 0, leftPx: 0, bottomPx: mbH };
               }
               if (styles.topbar.visible_on_desktop === false) {
                  return { topPx: 0, leftPx: 0, bottomPx: 0 };
               }
               const bodyRect = document.body
                  ? document.body.getBoundingClientRect()
                  : null;
               const tb =
                  shadowRoot && shadowRoot.getElementById('__fynexo_topbar__');
               const tbRect = tb ? tb.getBoundingClientRect() : null;
               const bodyTop = bodyRect ? Math.round(bodyRect.top) : 0;
               const tbBottom =
                  tbRect &&
                  tbRect.height > 1 &&
                  tbRect.height < window.innerHeight * 0.4
                     ? Math.ceil(tbRect.bottom)
                     : 0;
               const topPx = Math.max(bodyTop, tbBottom);
               const isVert =
                  (styles.topbar.layout || 'deck').toLowerCase() ===
                     'vertical' && !isMobilePlatform();
               const leftPx =
                  isVert && bodyRect ? Math.max(0, Math.round(bodyRect.left)) : 0;
               return { topPx, leftPx, bottomPx: 0 };
            }

            function applyLeftPatch(el, cs, rect, leftPx) {
               saveInlineProp(
                  el,
                  'left',
                  'data-fnx-fixed-prev-left',
                  'data-fnx-fixed-prev-left-pri',
               );
               saveInlineProp(
                  el,
                  'max-width',
                  'data-fnx-fixed-prev-mw',
                  'data-fnx-fixed-prev-mw-pri',
               );
               const parsedLeft =
                  cs.left && cs.left !== 'auto' ? parseFloat(cs.left) : NaN;
               const origLeft = Number.isFinite(parsedLeft)
                  ? Math.max(0, Math.round(parsedLeft))
                  : Math.max(0, Math.round(rect.left));
               el.setAttribute('data-fnx-fixed-orig-left', String(origLeft));
               el.setAttribute('data-fnx-fixed-left-patched', '1');
               const leftExpr =
                  origLeft > 0
                     ? `calc(${origLeft}px + var(--fnx-host-left-offset, ${leftPx}px))`
                     : `var(--fnx-host-left-offset, ${leftPx}px)`;
               el.style.setProperty('left', leftExpr, 'important');
               el.style.setProperty(
                  'max-width',
                  `calc(100vw - var(--fnx-host-left-offset, ${leftPx}px))`,
                  'important',
               );
            }

            function patchFixedBottomBars(bottomPx) {
               if (!Number.isFinite(bottomPx) || bottomPx <= 1) {
                  resetAllFixedBottomBars();
                  return;
               }
               const vh = window.innerHeight;
               const vw = window.innerWidth;

               const existing = document.querySelectorAll('[data-fnx-fixed-btm]');
               for (let i = 0; i < existing.length; i++) {
                  const el = existing[i];
                  if (isExcludedFixedNode(el)) {
                     unpatchFixedBottomBar(el);
                     continue;
                  }
                  const cs = getComputedStyle(el);
                  if (cs.position !== 'fixed') {
                     unpatchFixedBottomBar(el);
                     continue;
                  }
                  const origBtm =
                     parseInt(
                        el.getAttribute('data-fnx-fixed-orig-btm') || '0',
                        10,
                     ) || 0;
                  const btmExpr = `${origBtm + bottomPx}px`;
                  el.style.setProperty('bottom', btmExpr, 'important');
                  el.style.setProperty('inset-block-end', btmExpr, 'important');
               }

               const seen = typeof Set === 'function' ? new Set() : null;
               const candidates = [];
               const pushCand = (el) => {
                  if (!el || el.nodeType !== 1) return;
                  if (seen) {
                     if (seen.has(el)) return;
                     seen.add(el);
                  } else if (candidates.indexOf(el) !== -1) {
                     return;
                  }
                  candidates.push(el);
               };

               const sem = document.querySelectorAll(
                  'ytm-pivot-bar-renderer, footer, nav, [role="tablist"], [role="navigation"]',
               );
               for (let i = 0; i < sem.length; i++) pushCand(sem[i]);

               if (typeof document.elementsFromPoint === 'function') {
                  const probeYs = [
                     Math.max(4, vh - 4),
                     Math.max(4, vh - Math.min(26, Math.round(bottomPx * 0.5))),
                     Math.max(4, vh - bottomPx + 4),
                  ];
                  const probeXs = [
                     Math.round(vw * 0.16),
                     Math.round(vw * 0.38),
                     Math.round(vw * 0.62),
                     Math.round(vw * 0.84),
                  ];
                  for (let py = 0; py < probeYs.length; py++) {
                     for (let px = 0; px < probeXs.length; px++) {
                        let hits = [];
                        try {
                           hits = document.elementsFromPoint(
                              probeXs[px],
                              probeYs[py],
                           );
                        } catch (_) {}
                        for (let h = 0; h < hits.length; h++) {
                           let cur = hits[h];
                           for (
                              let d = 0;
                              d < 10 &&
                              cur &&
                              cur !== document.body &&
                              cur !== document.documentElement;
                              d++
                           ) {
                              if (cur.id === '__fynexo_host__') break;
                              pushCand(cur);
                              cur = cur.parentElement;
                           }
                        }
                     }
                  }
               }

               for (let i = 0; i < candidates.length; i++) {
                  const el = candidates[i];
                  if (
                     isExcludedFixedNode(el) ||
                     el.hasAttribute('data-fnx-fixed-btm')
                  ) {
                     continue;
                  }
                  const cs = getComputedStyle(el);
                  if (
                     cs.position !== 'fixed' ||
                     cs.display === 'none' ||
                     cs.visibility === 'hidden' ||
                     parseFloat(cs.opacity) === 0
                  ) {
                     continue;
                  }
                  const rect = el.getBoundingClientRect();
                  if (
                     rect.height < 24 ||
                     rect.height > 120 ||
                     rect.height >= vh * 0.25 ||
                     rect.width < vw * 0.6
                  ) {
                     continue;
                  }
                  if (rect.bottom <= vh - bottomPx + 2 || rect.bottom > vh + 8) {
                     continue;
                  }
                  if (cs.bottom === 'auto' && rect.top < vh * 0.65) {
                     continue;
                  }
                  const parsedBtm =
                     cs.bottom && cs.bottom !== 'auto'
                        ? parseFloat(cs.bottom)
                        : NaN;
                  const origBtm = Number.isFinite(parsedBtm)
                     ? Math.max(0, Math.round(parsedBtm))
                     : Math.max(0, Math.round(vh - rect.bottom));

                  saveInlineProp(
                     el,
                     'bottom',
                     'data-fnx-fixed-prev-btm',
                     'data-fnx-fixed-prev-btm-pri',
                  );
                  saveInlineProp(
                     el,
                     'inset-block-end',
                     'data-fnx-fixed-prev-ibe',
                     'data-fnx-fixed-prev-ibe-pri',
                  );
                  el.setAttribute('data-fnx-fixed-btm', '1');
                  el.setAttribute('data-fnx-fixed-orig-btm', String(origBtm));
                  const btmExpr = `${origBtm + bottomPx}px`;
                  el.style.setProperty('bottom', btmExpr, 'important');
                  el.style.setProperty('inset-block-end', btmExpr, 'important');
               }
            }

            function patchFixedHeaders() {
               if (fixedHdrPatching) return;
               fixedHdrLastRun = Date.now();
               fixedHdrPatching = true;
               try {
                  const { topPx, leftPx, bottomPx } = measureHudOffsets();
                  patchFixedBottomBars(bottomPx);
                  if (!Number.isFinite(topPx) || topPx <= 1) {
                     resetAllFixedHeaders();
                     return;
                  }

                  // Pass 1: Maintain or clean up previously patched nodes
                  const existing = document.querySelectorAll(
                     '[data-fnx-fixed-hdr]',
                  );
                  for (let i = 0; i < existing.length; i++) {
                     const el = existing[i];
                     if (isExcludedFixedNode(el)) {
                        unpatchFixedHeader(el);
                        continue;
                     }
                     const cs = getComputedStyle(el);
                     if (cs.position !== 'fixed') {
                        unpatchFixedHeader(el);
                        continue;
                     }
                     const rect = el.getBoundingClientRect();
                     if (
                        rect.height >= 16 &&
                        rect.top >= -4 &&
                        rect.top < topPx - 2 &&
                        !hasNegativeYTransform(cs)
                     ) {
                        const savedOrigTop =
                           parseInt(
                              el.getAttribute('data-fnx-fixed-orig-top') || '0',
                              10,
                           ) || 0;
                        const topExpr =
                           savedOrigTop > 0
                              ? `calc(${savedOrigTop}px + var(--fnx-host-top-offset, ${topPx}px))`
                              : `var(--fnx-host-top-offset, ${topPx}px)`;
                        el.style.setProperty('top', topExpr, 'important');
                        el.style.setProperty(
                           'inset-block-start',
                           topExpr,
                           'important',
                        );
                     }
                     if (leftPx <= 1) {
                        if (el.hasAttribute('data-fnx-fixed-left-patched')) {
                           restoreInlineProp(
                              el,
                              'left',
                              'data-fnx-fixed-prev-left',
                              'data-fnx-fixed-prev-left-pri',
                           );
                           restoreInlineProp(
                              el,
                              'max-width',
                              'data-fnx-fixed-prev-mw',
                              'data-fnx-fixed-prev-mw-pri',
                           );
                           el.removeAttribute('data-fnx-fixed-left-patched');
                           el.removeAttribute('data-fnx-fixed-orig-left');
                        }
                     } else if (
                        !el.hasAttribute('data-fnx-fixed-left-patched') &&
                        rect.left < leftPx - 2
                     ) {
                        applyLeftPatch(el, cs, rect, leftPx);
                     }
                  }

                  // Pass 2: Collect candidates without scanning the entire DOM
                  const seen = typeof Set === 'function' ? new Set() : null;
                  const candidates = [];
                  const pushCandidate = (el) => {
                     if (!el || el.nodeType !== 1) return;
                     if (seen) {
                        if (seen.has(el)) return;
                        seen.add(el);
                     } else if (candidates.indexOf(el) !== -1) {
                        return;
                     }
                     candidates.push(el);
                  };

                  const semanticNodes = document.querySelectorAll(
                     'header, nav, [role="banner"], [role="navigation"], [role="search"], #gb, [id*="header" i], [id*="navbar" i], [id*="topbar" i]',
                  );
                  for (let i = 0; i < semanticNodes.length; i++) {
                     pushCandidate(semanticNodes[i]);
                  }

                  if (document.body) {
                     const bfsQueue = [];
                     const bodyChildren = document.body.children;
                     for (let i = 0; i < bodyChildren.length; i++) {
                        bfsQueue.push({ el: bodyChildren[i], depth: 1 });
                     }
                     let visited = 0;
                     while (bfsQueue.length > 0 && visited < 80) {
                        const item = bfsQueue.shift();
                        const node = item.el;
                        if (!node || node.nodeType !== 1) continue;
                        const tag = node.tagName;
                        if (
                           tag === 'SCRIPT' ||
                           tag === 'STYLE' ||
                           tag === 'LINK' ||
                           tag === 'META' ||
                           tag === 'NOSCRIPT' ||
                           tag === 'SVG' ||
                           tag === 'PATH'
                        ) {
                           continue;
                        }
                        visited++;
                        pushCandidate(node);
                        if (item.depth < 3) {
                           const ch = node.children;
                           for (let c = 0; c < ch.length; c++) {
                              bfsQueue.push({
                                 el: ch[c],
                                 depth: item.depth + 1,
                              });
                           }
                        }
                     }
                  }

                  if (typeof document.elementsFromPoint === 'function') {
                     const vw = window.innerWidth;
                     const availW = Math.max(120, vw - leftPx);
                     const probeYs = [
                        Math.min(6, Math.max(1, topPx - 2)),
                        Math.min(24, Math.max(2, topPx - 2)),
                        Math.min(54, Math.max(4, topPx - 4)),
                     ];
                     const probeXs = [
                        leftPx + 24,
                        leftPx + Math.round(availW * 0.28),
                        leftPx + Math.round(availW * 0.52),
                        Math.max(leftPx + 40, vw - 36),
                     ];
                     for (let py = 0; py < probeYs.length; py++) {
                        for (let px = 0; px < probeXs.length; px++) {
                           let hits = [];
                           try {
                              hits = document.elementsFromPoint(
                                 probeXs[px],
                                 probeYs[py],
                              );
                           } catch (_) {}
                           for (let h = 0; h < hits.length; h++) {
                              const hit = hits[h];
                              if (!hit || hit.id === '__fynexo_host__') {
                                 continue;
                              }
                              const chain = [];
                              let cur = hit;
                              for (
                                 let d = 0;
                                 d < 12 &&
                                 cur &&
                                 cur !== document.body &&
                                 cur !== document.documentElement;
                                 d++
                              ) {
                                 if (cur.id === '__fynexo_host__') break;
                                 chain.push(cur);
                                 if (cur.shadowRoot) {
                                    const shadowHdrs =
                                       cur.shadowRoot.querySelectorAll(
                                          'header, nav, [role="banner"]',
                                       );
                                    for (let s = 0; s < shadowHdrs.length; s++) {
                                       chain.push(shadowHdrs[s]);
                                    }
                                 }
                                 cur = cur.parentElement;
                              }
                              for (let c = chain.length - 1; c >= 0; c--) {
                                 pushCandidate(chain[c]);
                              }
                           }
                        }
                     }
                  }

                  // Pass 3: Qualify and shift occluded fixed top bars
                  const vh = window.innerHeight;
                  const vw = window.innerWidth;
                  for (let i = 0; i < candidates.length; i++) {
                     const el = candidates[i];
                     if (
                        isExcludedFixedNode(el) ||
                        el.hasAttribute('data-fnx-fixed-hdr')
                     ) {
                        continue;
                     }
                     const cs = getComputedStyle(el);
                     if (
                        cs.position !== 'fixed' ||
                        cs.display === 'none' ||
                        cs.visibility === 'hidden' ||
                        parseFloat(cs.opacity) === 0
                     ) {
                        continue;
                     }
                     if (hasNegativeYTransform(cs)) continue;
                     const rect = el.getBoundingClientRect();
                     if (
                        rect.height < 20 ||
                        rect.height > 220 ||
                        rect.height >= vh * 0.38
                     ) {
                        continue;
                     }
                     if (rect.top < -4 || rect.top >= topPx - 2) continue;
                     if (
                        cs.top === 'auto' &&
                        cs.bottom !== 'auto' &&
                        rect.bottom > topPx + 220
                     ) {
                        continue;
                     }
                     const tag = el.tagName;
                     const role = (el.getAttribute('role') || '').toLowerCase();
                     const isSemantic =
                        tag === 'HEADER' ||
                        tag === 'NAV' ||
                        role === 'banner' ||
                        role === 'navigation' ||
                        role === 'search' ||
                        el.id === 'gb';
                     const minW = isSemantic
                        ? 120
                        : Math.min(320, Math.max(180, (vw - leftPx) * 0.45));
                     if (rect.width < minW || rect.width <= rect.height * 1.25) {
                        continue;
                     }

                     const parsedTop =
                        cs.top && cs.top !== 'auto' ? parseFloat(cs.top) : NaN;
                     const origTop = Number.isFinite(parsedTop)
                        ? Math.max(0, Math.round(parsedTop))
                        : Math.max(0, Math.round(rect.top));

                     saveInlineProp(
                        el,
                        'top',
                        'data-fnx-fixed-prev-top',
                        'data-fnx-fixed-prev-top-pri',
                     );
                     saveInlineProp(
                        el,
                        'inset-block-start',
                        'data-fnx-fixed-prev-ibs',
                        'data-fnx-fixed-prev-ibs-pri',
                     );
                     el.setAttribute('data-fnx-fixed-hdr', '1');
                     el.setAttribute('data-fnx-fixed-orig-top', String(origTop));
                     const topExpr =
                        origTop > 0
                           ? `calc(${origTop}px + var(--fnx-host-top-offset, ${topPx}px))`
                           : `var(--fnx-host-top-offset, ${topPx}px)`;
                     el.style.setProperty('top', topExpr, 'important');
                     el.style.setProperty(
                        'inset-block-start',
                        topExpr,
                        'important',
                     );

                     if (leftPx > 1 && rect.left < leftPx - 2) {
                        applyLeftPatch(el, cs, rect, leftPx);
                     }
                  }
               } catch (_) {
               } finally {
                  fixedHdrPatching = false;
               }
            }

            function ensureBodyHooks() {
               if (!document.body) return;
               if (!fixedHdrBodyScrollBound) {
                  fixedHdrBodyScrollBound = true;
                  document.body.addEventListener(
                     'scroll',
                     () => scheduleFixedHeaderSync(false),
                     { passive: true },
                  );
               }
               if (!fixedHdrObserverAttached && typeof MutationObserver === 'function') {
                  fixedHdrObserverAttached = true;
                  try {
                     const mo = new MutationObserver((mutations) => {
                        if (fixedHdrPatching) return;
                        for (let i = 0; i < mutations.length; i++) {
                           const t = mutations[i].target;
                           if (
                              t &&
                              (t.id === '__fynexo_host__' ||
                                 (t.closest && t.closest('#__fynexo_host__')))
                           ) {
                              continue;
                           }
                           scheduleFixedHeaderSync(false);
                           break;
                        }
                     });
                     mo.observe(document.body, {
                        childList: true,
                        subtree: true,
                     });
                  } catch (_) {}
               }
            }

            function scheduleFixedHeaderSync(immediate) {
               if (isInternalAny()) return;
               ensureBodyHooks();
               if (immediate) {
                  if (fixedHdrTimer) {
                     clearTimeout(fixedHdrTimer);
                     fixedHdrTimer = null;
                  }
                  requestAnimationFrame(() => patchFixedHeaders());
                  return;
               }
               const elapsed = Date.now() - fixedHdrLastRun;
               const delay = elapsed >= 140 ? 16 : 140 - elapsed;
               if (fixedHdrTimer) return;
               fixedHdrTimer = setTimeout(() => {
                  fixedHdrTimer = null;
                  requestAnimationFrame(() => patchFixedHeaders());
               }, delay);
            }

            syncFixedHeadersRef = scheduleFixedHeaderSync;
            scheduleFixedHeaderSync(true);
            document.addEventListener(
               'DOMContentLoaded',
               () => scheduleFixedHeaderSync(true),
               { once: true },
            );
            window.addEventListener('load', () => scheduleFixedHeaderSync(true), {
               once: true,
            });
            window.addEventListener(
               'resize',
               () => scheduleFixedHeaderSync(true),
               { passive: true },
            );
            window.addEventListener(
               'scroll',
               () => scheduleFixedHeaderSync(false),
               { passive: true },
            );
            window.addEventListener(
               'popstate',
               () => scheduleFixedHeaderSync(false),
               { passive: true },
            );
            window.addEventListener(
               'hashchange',
               () => scheduleFixedHeaderSync(false),
               { passive: true },
            );
            window.addEventListener(
               'pageshow',
               () => scheduleFixedHeaderSync(true),
               { passive: true },
            );
            document.addEventListener(
               'fullscreenchange',
               () => scheduleFixedHeaderSync(true),
               { passive: true },
            );
            window.addEventListener(
               'turbo:load',
               () => scheduleFixedHeaderSync(true),
               { passive: true },
            );
            window.addEventListener(
               'yt-navigate-finish',
               () => scheduleFixedHeaderSync(true),
               { passive: true },
            );
         })();

         __FNX_EXTENSIONS__;

         syncExtensionsRef = () => {
            if (typeof btnSidePanel !== 'undefined' && btnSidePanel) {
               btnSidePanel.style.display = config.ext_side_panel ? '' : 'none';
            }
            if (typeof mobSidePanelItem !== 'undefined' && mobSidePanelItem) {
               mobSidePanelItem.style.display = config.ext_side_panel ? '' : 'none';
            }
            if (!config.ext_side_panel) {
               destroySidePanel();
            }
            if (typeof readerMenuItem !== 'undefined' && readerMenuItem) {
               readerMenuItem.style.display = config.ext_reader_mode ? '' : 'none';
            }
            if (!config.ext_reader_mode && isReaderOpen) {
               closeReaderMode();
            }
            if (config.ext_reader_mode) {
               scheduleReaderableCheck();
            } else {
               refreshReaderableState();
            }
            if (typeof syncMediaPopoverUI === 'function') {
               syncMediaPopoverUI();
            }
            if (typeof syncShieldPopoverUIRef === 'function') {
               syncShieldPopoverUIRef();
            }
            if (typeof syncYouTubeAdSkipper === 'function') {
               syncYouTubeAdSkipper();
            }
            if (typeof syncGithubDrawerHarmonizer === 'function') {
               syncGithubDrawerHarmonizer();
            }
            if (typeof syncMouseGestures === 'function') {
               syncMouseGestures();
            }
         };

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
            if (IS_HUD_SHELL || isBackgroundParked) return;
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
            if (IS_CONTENT_TAB && typeof reportContentPageMetaRef === 'function') {
               reportContentPageMetaRef(false);
            }
         }

         onForegroundResumeCb = function () {
            mutLastTitle = '';
            mutLastUrl = '';
            syncTitle();
            if (IS_CONTENT_TAB && typeof reportContentPageMetaRef === 'function') {
               reportContentPageMetaRef(true);
            }
         };

         function bindTitleObserver() {
            if (IS_HUD_SHELL) return;
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

         if (!IS_HUD_SHELL) {
            bindTitleObserver();
            scheduleReaderableCheck();
            document.addEventListener(
               'DOMContentLoaded',
               () => {
                  bindTitleObserver();
                  scheduleReaderableCheck();
                  if (IS_CONTENT_TAB && typeof reportContentPageMetaRef === 'function') {
                     reportContentPageMetaRef(true);
                  }
               },
               { once: true },
            );
            window.addEventListener(
               'load',
               () => {
                  bindTitleObserver();
                  scheduleReaderableCheck();
                  if (IS_CONTENT_TAB && typeof reportContentPageMetaRef === 'function') {
                     reportContentPageMetaRef(true);
                  }
               },
               { once: true },
            );
            const onSpaNavChange = () => {
               finishProgress(false);
               requestAnimationFrame(syncTitle);
               sendIpc({ type: 'SYNC_NAV_STATE' });
               scheduleReaderableCheck();
               if (IS_CONTENT_TAB && typeof reportContentPageMetaRef === 'function') {
                  reportContentPageMetaRef(false);
               }
            };
            window.addEventListener('popstate', onSpaNavChange, {
               passive: true,
            });
            window.addEventListener('hashchange', onSpaNavChange, {
               passive: true,
            });
            window.addEventListener(
               'pageshow',
               (e) => {
                  if (e && e.persisted) {
                     mutLastTitle = '';
                     mutLastUrl = '';
                  }
                  onSpaNavChange();
               },
               { passive: true },
            );
            window.addEventListener('turbo:load', scheduleReaderableCheck, {
               passive: true,
            });
            window.addEventListener(
               'yt-navigate-finish',
               () => {
                  onSpaNavChange();
               },
               { passive: true },
            );
            if (IS_CONTENT_TAB && window.history) {
               try {
                  const origPushState = window.history.pushState;
                  if (typeof origPushState === 'function') {
                     window.history.pushState = function () {
                        const res = origPushState.apply(this, arguments);
                        onSpaNavChange();
                        return res;
                     };
                  }
                  const origReplaceState = window.history.replaceState;
                  if (typeof origReplaceState === 'function') {
                     window.history.replaceState = function () {
                        const res = origReplaceState.apply(this, arguments);
                        onSpaNavChange();
                        return res;
                     };
                  }
               } catch (_) {}
            }
         }

         if (IS_HUD_SHELL) {
            let hudLayoutRaf = 0;
            let lastHudLayoutJson = '';

            function measureVisibleRect(id, pad, fallbackRadius, optionalCheckOpacity) {
               if (!shadowRoot) return null;
               const el = shadowRoot.getElementById(id);
               if (!el || !el.isConnected) return null;
               const st = window.getComputedStyle(el);
               if (!st || st.display === 'none' || st.visibility === 'hidden') return null;
               if (optionalCheckOpacity && parseFloat(st.opacity || '0') <= 0.05) {
                  return null;
               }
               const r = el.getBoundingClientRect();
               if (!r || r.width < 1 || r.height < 1) return null;
               const vw = Math.max(0, window.innerWidth || 0);
               const vh = Math.max(0, window.innerHeight || 0);
               let rawL = r.left;
               let rawT = r.top;
               let rawR = r.right;
               let rawB = r.bottom;
               // During entrance animations (e.g. translateY(-3px) scale(0.985)), getBoundingClientRect()
               // reports a shrunk/shifted box; union with the untransformed fixed layout box so Windows
               // SetWindowRgn never clips the bottom or right border of popovers.
               if (st.position === 'fixed' && el.offsetWidth > 0 && el.offsetHeight > 0) {
                  const ow = el.offsetWidth;
                  const oh = el.offsetHeight;
                  const parsePx = (v) => (v && v.endsWith('px') ? parseFloat(v) : NaN);
                  const topPx = parsePx(st.top);
                  const botPx = parsePx(st.bottom);
                  const leftPx = parsePx(st.left);
                  const rightPx = parsePx(st.right);
                  if (Number.isFinite(topPx) && st.bottom === 'auto') {
                     rawT = Math.min(rawT, topPx);
                     rawB = Math.max(rawB, topPx + oh);
                  } else if (Number.isFinite(botPx) && st.top === 'auto') {
                     const endB = vh - botPx;
                     rawT = Math.min(rawT, endB - oh);
                     rawB = Math.max(rawB, endB);
                  } else if (oh > r.height) {
                     rawB = Math.max(rawB, rawT + oh);
                  }
                  if (Number.isFinite(leftPx) && st.right === 'auto') {
                     rawL = Math.min(rawL, leftPx);
                     rawR = Math.max(rawR, leftPx + ow);
                  } else if (Number.isFinite(rightPx) && st.left === 'auto') {
                     const endR = vw - rightPx;
                     rawL = Math.min(rawL, endR - ow);
                     rawR = Math.max(rawR, endR);
                  } else if (ow > r.width) {
                     rawR = Math.max(rawR, rawL + ow);
                  }
               }
               const x0 = Math.max(0, Math.floor(rawL - pad));
               const y0 = Math.max(0, Math.floor(rawT - pad));
               const x1 = Math.min(vw, Math.ceil(rawR + pad));
               const y1 = Math.min(vh, Math.ceil(rawB + pad));
               if (x1 <= x0 || y1 <= y0) return null;
               let radius = fallbackRadius;
               if (radius < 0) {
                  const rawRad = st.borderTopLeftRadius || '';
                  const parsed = rawRad.endsWith('px') ? parseFloat(rawRad) : 0;
                  radius = Number.isFinite(parsed) && parsed > 0 ? Math.round(parsed) : 0;
               }
               return {
                  x: x0,
                  y: y0,
                  w: x1 - x0,
                  h: y1 - y0,
                  r: radius,
               };
            }

            function syncHudLayoutNow() {
               hudLayoutRaf = 0;
               if (!shadowRoot) return;
               const tbRect = measureVisibleRect('__fynexo_topbar__', 0, 0, false);
               const layoutMode = ((styles && styles.topbar && styles.topbar.layout) || 'deck').toLowerCase();
               const vertRect =
                  layoutMode === 'vertical'
                     ? measureVisibleRect('__fynexo_tab_row__', 0, 0, false)
                     : null;
               const validVert =
                  vertRect && vertRect.w > 10 && vertRect.h > 80 && vertRect.x <= 2
                     ? vertRect
                     : null;
               const spRect = isSidePanelOpen
                  ? measureVisibleRect('__fynexo_side_panel__', 0, 0, false)
                  : null;

               const content_inset = {
                  top: isFullscreen ? 0 : tbRect ? tbRect.y + tbRect.h : 0,
                  left: isFullscreen ? 0 : validVert ? validVert.x + validVert.w : 0,
                  right: isFullscreen ? 0 : spRect ? spRect.w : 0,
                  bottom: 0,
               };

               const regions = [];
               if (tbRect) regions.push(tbRect);
               if (validVert) regions.push(validVert);
               if (spRect) regions.push(spRect);
               const progRect = measureVisibleRect('__fynexo_progress__', 0, 0, true);
               if (progRect) regions.push(progRect);

               let has_overlay = false;
               const modalCardIds = [
                  '__fynexo_hud_card__',
                  '__fynexo_history_card__',
                  '__fynexo_prompt_card__',
               ];
               for (let i = 0; i < modalCardIds.length; i++) {
                  const cr = measureVisibleRect(modalCardIds[i], 0, -1, false);
                  if (cr) {
                     regions.push(cr);
                     has_overlay = true;
                  }
               }

               const dismissiblePopoverIds = [
                  '__fynexo_inline_dropdown__',
                  '__fynexo_main_menu__',
                  '__fynexo_media_popover__',
                  '__fynexo_shield_popover__',
                  '__fynexo_site_popover__',
                  '__fynexo_mode_popover__',
                  '__fynexo_downloads_popover__',
                  '__fynexo_studio_popover__',
                  '__fynexo_exts_popover__',
                  '__fynexo_taskmgr_popover__',
                  '__fynexo_tab_context_menu__',
               ];
               for (let i = 0; i < dismissiblePopoverIds.length; i++) {
                  const pr = measureVisibleRect(dismissiblePopoverIds[i], 0, -1, false);
                  if (pr) {
                     regions.push(pr);
                     has_overlay = true;
                  }
               }
               if (
                  isInlineActive ||
                  isModalActive ||
                  (typeof isHistoryOpenRef === 'function' && isHistoryOpenRef()) ||
                  (typeof isPromptOpenRef === 'function' && isPromptOpenRef())
               ) {
                  has_overlay = true;
               }

               const passiveWidgetIds = [
                  '__fynexo_tab_hover_card__',
                  '__fynexo_store_pill__',
               ];
               for (let i = 0; i < passiveWidgetIds.length; i++) {
                  const wr = measureVisibleRect(passiveWidgetIds[i], 0, -1, false);
                  if (wr) regions.push(wr);
               }
               const toastRect = measureVisibleRect('__fynexo_hud_toast__', 0, -1, true);
               if (toastRect) regions.push(toastRect);

               const payload = {
                  type: 'SYNC_HUD_LAYOUT',
                  content_inset: content_inset,
                  regions: regions,
                  has_overlay: has_overlay,
               };
               const json = __fnxJsonStringify(payload);
               if (json === lastHudLayoutJson) return;
               lastHudLayoutJson = json;
               sendIpc(payload);
            }

            function scheduleSyncHudLayout() {
               if (hudLayoutRaf) return;
               if (typeof window.requestAnimationFrame === 'function') {
                  hudLayoutRaf = window.requestAnimationFrame(syncHudLayoutNow);
               } else {
                  hudLayoutRaf = setTimeout(syncHudLayoutNow, 16);
               }
            }
            syncHudLayoutRef = scheduleSyncHudLayout;

            try {
               const hudLayoutObs = new MutationObserver(scheduleSyncHudLayout);
               hudLayoutObs.observe(shadowRoot, {
                  subtree: true,
                  childList: true,
                  attributes: true,
                  attributeFilter: ['style', 'class'],
               });
               shadowRoot.addEventListener('animationend', scheduleSyncHudLayout, true);
               shadowRoot.addEventListener('transitionend', scheduleSyncHudLayout, true);
            } catch (_) {}

            window.addEventListener(
               'resize',
               () => {
                  lastHudLayoutJson = '';
                  scheduleSyncHudLayout();
               },
               { passive: true },
            );

            onHudActionCb = function (action) {
               switch (action) {
                  case 'open_omnibox':
                     openHud();
                     break;
                  case 'open_command_palette':
                     openModalHud();
                     break;
                  case 'open_history':
                     closeAnyOmnibox();
                     sendIpc({ type: 'NEW_TAB', url: 'fynexo://settings#sec-privacy' });
                     break;
                  case 'toggle_downloads':
                     if (typeof toggleDownloadsPopover === 'function') {
                        toggleDownloadsPopover();
                     }
                     break;
                  case 'toggle_side_panel':
                     if (typeof toggleSidePanel === 'function') toggleSidePanel();
                     break;
                  case 'toggle_bookmark':
                     toggleCurrentBookmark();
                     break;
                  case 'copy_url':
                     copyCurrentUrl();
                     break;
                  case 'toggle_preferences':
                     closeAnyOmnibox();
                     togglePreferences();
                     break;
                  case 'toggle_force_dark':
                     config.force_dark = !config.force_dark;
                     sendIpc({
                        type: 'PATCH_HUD_PREFS',
                        force_dark: config.force_dark,
                     });
                     applyPageRules();
                     syncBookmarkButtons();
                     showHudToast(
                        config.force_dark
                           ? isZh()
                              ? '已开启智能深色模式'
                              : 'Smart Force Dark enabled'
                           : isZh()
                             ? '已关闭智能深色模式'
                             : 'Smart Force Dark off',
                     );
                     break;
                  case 'fullscreen_peek':
                     if (isFullscreen && !isTopbarPeek) {
                        isTopbarPeek = true;
                        applyShadowStyles();
                     }
                     break;
                  case 'dismiss_overlays':
                     hideTabContextMenu();
                     hideTabHoverCard();
                     if (typeof closeAllNavPopovers === 'function') {
                        closeAllNavPopovers();
                     }
                     if (isInlineActive) {
                        deactivateInlineAddressBar();
                     }
                     if (isModalActive) {
                        deactivateModalHud();
                     }
                     if (isHistoryOpenRef && isHistoryOpenRef() && closeHistoryDrawerRef) {
                        closeHistoryDrawerRef();
                     }
                     if (isPromptOpenRef && isPromptOpenRef() && closePromptModalRef) {
                        closePromptModalRef();
                     }
                     scheduleSyncHudLayout();
                     break;
               }
            };

            onPageMetaCb = function (meta) {
               const m = meta || {};
               remotePageMeta = Object.assign({}, remotePageMeta, m);
               if (typeof m.media_muted === 'boolean') {
                  mediaIsMuted = m.media_muted;
               }
               if (typeof m.media_gain === 'number' && Number.isFinite(m.media_gain)) {
                  mediaGainValue = Math.max(0, Math.min(300, m.media_gain));
               }
               refreshReaderableState();
               updateMediaButtonState();
               if (mediaPopoverEl && mediaPopoverEl.style.display === 'flex') {
                  syncMediaPopoverUI();
               }
               updateShieldState();
               if (shieldPopoverEl && shieldPopoverEl.style.display === 'flex') {
                  syncShieldPopoverUI();
               }
               if (siteInfoPopoverEl && siteInfoPopoverEl.style.display === 'flex') {
                  syncSiteInfoPopoverUI();
               }
               if (modePopoverEl && modePopoverEl.style.display === 'flex') {
                  renderModePopoverUI();
               }
               if (typeof syncMainMenuUIRef === 'function') {
                  syncMainMenuUIRef();
               }
               scheduleSyncHudLayout();
            };

            scheduleSyncHudLayout();
         }

         if (IS_CONTENT_TAB) {
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
                     const pageH1 = document.querySelector(
                        'h1.ytd-watch-metadata, h1.title, h1',
                     );
                     mediaTitle =
                        (pageH1 && pageH1.textContent.trim()) ||
                        document.title ||
                        '';
                  } catch (_) {}
               }
               const blockedAds = !internal ? getDetectedAdCount() : 0;
               const sfCommands =
                  !internal && Array.isArray(scriptForgeMenuCommands)
                     ? scriptForgeMenuCommands.map((c) => ({
                          id: c.id,
                          caption: c.caption,
                          scriptName: c.scriptName || '',
                       }))
                     : [];
               return {
                  url: curUrl || '',
                  readerable:
                     !!config.ext_reader_mode &&
                     !internal &&
                     !!cachedReaderable,
                  reader_open: !!isReaderOpen,
                  dev_drawer_open: !!(isDevDrawerOpenRef && isDevDrawerOpenRef()),
                  picker_active: !!pickerActive,
                  has_media: hasMedia,
                  media_paused: hasMedia
                     ? !!(activeMedia.paused || activeMedia.ended)
                     : true,
                  media_title: mediaTitle,
                  media_muted: !!mediaIsMuted,
                  media_gain: Number(mediaGainValue) || 100,
                  blocked_ads: blockedAds,
                  script_count: Number(activePageScriptCount) || 0,
                  sf_commands: sfCommands,
                  page_is_dark: !internal && isPageAlreadyDark(),
               };
            }

            function sendContentPageMetaNow(force) {
               pageMetaTimer = null;
               if (isBackgroundParked) return;
               const meta = buildContentPageMeta();
               const payload = { type: 'CONTENT_PAGE_META', meta: meta };
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

            onContentActionCb = function (action, payload) {
               const data = payload || {};
               switch (action) {
                  case 'open_find':
                     openFindBar();
                     break;
                  case 'toggle_reader':
                     toggleReaderMode();
                     sendContentPageMetaNow(true);
                     break;
                  case 'toggle_dev_drawer':
                     if (typeof toggleDevDrawerRef === 'function') {
                        toggleDevDrawerRef();
                     }
                     sendContentPageMetaNow(true);
                     break;
                  case 'start_element_picker':
                     if (typeof startVisualElementPickerRef === 'function') {
                        startVisualElementPickerRef();
                     } else {
                        startVisualElementPicker();
                     }
                     sendContentPageMetaNow(true);
                     break;
                  case 'clear_site_storage':
                     clearCurrentSiteStorageLocal();
                     break;
                  case 'hard_reload':
                     performHardCacheBustReload();
                     break;
                  case 'media_play_pause':
                     toggleLocalMediaPlayPause();
                     sendContentPageMetaNow(true);
                     break;
                  case 'media_mute':
                     mediaIsMuted =
                        typeof data.muted === 'boolean'
                           ? data.muted
                           : !mediaIsMuted;
                     applyMediaVolume(mediaGainValue);
                     sendContentPageMetaNow(true);
                     break;
                  case 'media_volume':
                     if (typeof data.muted === 'boolean') {
                        mediaIsMuted = data.muted;
                     }
                     applyMediaVolume(
                        data.gain !== undefined ? data.gain : mediaGainValue,
                     );
                     sendContentPageMetaNow(true);
                     break;
                  case 'run_sf_command':
                     if (Array.isArray(scriptForgeMenuCommands)) {
                        const cmd = scriptForgeMenuCommands.find(
                           (c) => c && c.id === data.id,
                        );
                        if (cmd && typeof cmd.onClick === 'function') {
                           try {
                              cmd.onClick();
                           } catch (err) {
                              console.warn(
                                 '[ScriptForge Menu Command Error]:',
                                 err,
                              );
                           }
                        }
                     }
                     break;
                  case 'preview_site_boost': {
                     const ruleId = String(data.ruleId || '');
                     const cleanCss = String(data.css || '')
                        .replace(/[<>]/g, '')
                        .trim();
                     const cleanDom = String(data.domain || '');
                     if (!ruleId || !cleanDom) break;
                     if (!cleanCss) {
                        enabledRules = enabledRules.filter(
                           (r) => !r || r.id !== ruleId,
                        );
                     } else {
                        let existing = enabledRules.find(
                           (r) => r && r.id === ruleId,
                        );
                        if (!existing) {
                           enabledRules.push({
                              id: ruleId,
                              name: 'Site Boost (' + cleanDom + ')',
                              domain: cleanDom,
                              enabled: true,
                              css: cleanCss,
                              js: '',
                           });
                        } else {
                           existing.enabled = true;
                           existing.css = cleanCss;
                        }
                     }
                     applySiteRules();
                     break;
                  }
               }
            };

            sendContentPageMetaNow(true);
         }

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
               if (!IS_HUD_SHELL && !IS_CONTENT_TAB) {
                  syncGameMode();
               }
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
      if (IS_HUD_SHELL || paintReadySent) return;
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
