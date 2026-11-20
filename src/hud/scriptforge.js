   function domainMatchesRule(hostname, pattern) {
      const p = String(pattern || '*')
         .trim()
         .toLowerCase();
      if (!p || p === '*') return true;
      if (p.includes(':')) {
         const curHostPort = String(window.location.host || hostname || '').toLowerCase();
         const normP = p.replace(/^www\./, '');
         return curHostPort === normP || curHostPort.replace(/^www\./, '') === normP;
      }
      const clean = p.startsWith('*.') ? p.slice(2) : p.replace(/^www\./, '');
      const h = String(hostname || '').toLowerCase();
      return h === clean || h.endsWith('.' + clean);
   }

   function parseUserscriptMeta(code) {
      const src = String(code || '');
      const match = src.match(
         /\/\/\s*==UserScript==([\s\S]*?)\/\/\s*==\/UserScript==/,
      );
      if (!match) return null;
      const meta = {
         name: '',
         version: '',
         description: '',
         author: '',
         runAt: 'document-end',
         matches: [],
         includes: [],
         excludes: [],
         grants: [],
         connects: [],
      };
      const lines = match[1].split(/\r?\n/);
      for (let i = 0; i < lines.length; i++) {
         const line = lines[i].trim();
         const m = line.match(/^\/\/\s*@([A-Za-z0-9_-]+)\s+(.*)$/);
         if (!m) continue;
         const key = m[1].toLowerCase();
         const val = m[2].trim();
         if (!val) continue;
         if (key === 'name' && !meta.name) meta.name = val;
         else if (key === 'version' && !meta.version) meta.version = val;
         else if (key === 'description' && !meta.description)
            meta.description = val;
         else if (key === 'author' && !meta.author) meta.author = val;
         else if (key === 'run-at') {
            const ra = val.toLowerCase();
            if (
               ra === 'document-start' ||
               ra === 'document-body' ||
               ra === 'document-end' ||
               ra === 'document-idle'
            ) {
               meta.runAt = ra;
            }
         } else if (key === 'match') meta.matches.push(val);
         else if (key === 'include') meta.includes.push(val);
         else if (key === 'exclude') meta.excludes.push(val);
         else if (key === 'grant') meta.grants.push(val);
         else if (key === 'connect') meta.connects.push(val.toLowerCase());
      }
      return meta;
   }

   function escapeRegexExact(s) {
      return String(s).replace(/[.*+?^${}()|[\]\\]/g, '\\$&');
   }

   function escapeRegexGlob(s) {
      return String(s)
         .replace(/[.+?^${}()|[\]\\]/g, '\\$&')
         .replace(/\*/g, '.*');
   }

   function userscriptPatternMatchesUrl(pattern, href) {
      const p = String(pattern || '').trim();
      const target = String(href || '');
      if (!p || !target) return false;
      if (p === '*' || p === '<all_urls>') {
         return /^https?:\/\//i.test(target);
      }
      if (p.length > 2 && p.startsWith('/') && p.endsWith('/')) {
         try {
            return new RegExp(p.slice(1, -1)).test(target);
         } catch (_) {
            return false;
         }
      }
      const schemeMatch = p.match(/^(\*|https?):\/\/(\*\.[^/]+|[^/]+)(\/.*)?$/i);
      if (schemeMatch) {
         const schemeRe =
            schemeMatch[1] === '*' ? 'https?' : escapeRegexExact(schemeMatch[1]);
         const hostPat = schemeMatch[2];
         const hostRe =
            hostPat === '*'
               ? '[^/]+'
               : hostPat.startsWith('*.')
                 ? '(?:[^/]+\\.)?' + escapeRegexGlob(hostPat.slice(2))
                 : escapeRegexGlob(hostPat);
         const pathRe = escapeRegexGlob(schemeMatch[3] || '/*');
         try {
            return new RegExp(
               '^' + schemeRe + '://' + hostRe + pathRe + '$',
               'i',
            ).test(target);
         } catch (_) {
            return false;
         }
      }
      try {
         return new RegExp('^' + escapeRegexGlob(p) + '$', 'i').test(target);
      } catch (_) {
         return false;
      }
   }

   function urlMatchesUserscriptMeta(href, meta) {
      if (!meta) return true;
      for (let i = 0; i < meta.excludes.length; i++) {
         if (userscriptPatternMatchesUrl(meta.excludes[i], href)) return false;
      }
      if (meta.matches.length === 0 && meta.includes.length === 0) return true;
      for (let i = 0; i < meta.matches.length; i++) {
         if (userscriptPatternMatchesUrl(meta.matches[i], href)) return true;
      }
      for (let i = 0; i < meta.includes.length; i++) {
         if (userscriptPatternMatchesUrl(meta.includes[i], href)) return true;
      }
      return false;
   }

   function isUrlAllowedByUserscriptConnect(rawUrl, rule, meta) {
      let parsed;
      try {
         parsed = new URL(String(rawUrl || ''), window.location.href);
      } catch (_) {
         return { allowed: false, reason: 'Invalid URL' };
      }
      const proto = (parsed.protocol || '').toLowerCase();
      if (proto !== 'http:' && proto !== 'https:') {
         return { allowed: false, reason: 'Disallowed protocol (' + proto + ')' };
      }
      const targetHost = (parsed.hostname || '').toLowerCase();
      if (!targetHost || targetHost === 'fynexo.localhost') {
         return { allowed: false, reason: 'Blocked internal origin' };
      }
      if (meta && Array.isArray(meta.grants) && meta.grants.includes('none')) {
         const hasXhrGrant = meta.grants.some(
            (g) => g === 'GM_xmlhttpRequest' || g === 'GM.xmlHttpRequest',
         );
         if (!hasXhrGrant) {
            return { allowed: false, reason: '@grant none' };
         }
      }
      const curHost = (window.location.hostname || '').toLowerCase();
      const isSameHost = Boolean(
         curHost && (targetHost === curHost || targetHost.endsWith('.' + curHost)),
      );
      const ruleDom =
         rule && rule.domain
            ? String(rule.domain || '')
                 .trim()
                 .toLowerCase()
            : '';
      const cleanRuleDom =
         ruleDom && ruleDom !== '*'
            ? ruleDom.startsWith('*.')
               ? ruleDom.slice(2)
               : ruleDom.replace(/^www\./, '')
            : '';
      const matchesRuleDomain = Boolean(
         cleanRuleDom &&
            (targetHost === cleanRuleDom || targetHost.endsWith('.' + cleanRuleDom)),
      );

      if (meta && Array.isArray(meta.connects) && meta.connects.length > 0) {
         let matched = false;
         for (let i = 0; i < meta.connects.length; i++) {
            const c = String(meta.connects[i] || '').trim().toLowerCase();
            if (!c) continue;
            if (c === '*') {
               matched = true;
               break;
            }
            if (c === 'self') {
               if (isSameHost) {
                  matched = true;
                  break;
               }
               continue;
            }
            const cleanC = c.startsWith('*.') ? c.slice(2) : c;
            if (targetHost === cleanC || targetHost.endsWith('.' + cleanC)) {
               matched = true;
               break;
            }
         }
         if (!matched && matchesRuleDomain) {
            matched = true;
         }
         if (!matched) {
            return {
               allowed: false,
               reason: '@connect (' + targetHost + ' not in ' + meta.connects.join(', ') + ')',
            };
         }
      } else if (!isSameHost && !matchesRuleDomain) {
         return {
            allowed: false,
            reason: 'Missing @connect declaration for cross-origin host ' + targetHost,
         };
      }
      return { allowed: true, url: parsed.href, isSameHost };
   }

   function createScriptForgeSandbox(r, meta) {
      const ruleId = String(r.id || 'script');
      const scriptName = (meta && meta.name) || r.name || ruleId;
      const storePrefix = '__fnx_gm_' + ruleId + '_';
      const mem =
         scriptForgeMemStore[ruleId] ||
         (scriptForgeMemStore[ruleId] = Object.create(null));

      const gmInfo = {
         scriptHandler: 'ScriptForge',
         version: '1.0.0',
         script: {
            name: scriptName,
            version: (meta && meta.version) || r.version || '1.0.0',
            description: (meta && meta.description) || '',
            author: (meta && meta.author) || '',
            domain: r.domain || '*',
            runAt: (meta && meta.runAt) || 'document-end',
            matches:
               meta && Array.isArray(meta.matches) ? meta.matches.slice() : [],
            includes:
               meta && Array.isArray(meta.includes) ? meta.includes.slice() : [],
            excludes:
               meta && Array.isArray(meta.excludes) ? meta.excludes.slice() : [],
            grants:
               meta && Array.isArray(meta.grants) ? meta.grants.slice() : [],
            connects:
               meta && Array.isArray(meta.connects) ? meta.connects.slice() : [],
         },
      };

      const notifyValueChange = (key, oldVal, newVal) => {
         const keys = Object.keys(scriptForgeListeners);
         for (let i = 0; i < keys.length; i++) {
            const entry = scriptForgeListeners[keys[i]];
            if (entry && entry.ruleId === ruleId && entry.key === key) {
               try {
                  entry.cb(key, oldVal, newVal, false);
               } catch (_) {}
            }
         }
      };

      const gmGetValue = (key, defaultValue) => {
         const k = String(key);
         try {
            if (window.localStorage) {
               const raw = window.localStorage.getItem(storePrefix + k);
               if (raw === null) return defaultValue;
               return __fnxJsonParse(raw);
            }
         } catch (_) {}
         return Object.prototype.hasOwnProperty.call(mem, k)
            ? mem[k]
            : defaultValue;
      };

      const gmDeleteValue = (key) => {
         const k = String(key);
         const oldVal = gmGetValue(k, undefined);
         try {
            if (window.localStorage) {
               window.localStorage.removeItem(storePrefix + k);
            }
         } catch (_) {}
         delete mem[k];
         if (oldVal !== undefined) notifyValueChange(k, oldVal, undefined);
      };

      const gmSetValue = (key, value) => {
         const k = String(key);
         if (value === undefined) {
            gmDeleteValue(k);
            return;
         }
         const oldVal = gmGetValue(k, undefined);
         let encoded;
         try {
            encoded = __fnxJsonStringify(value);
         } catch (_) {
            encoded = 'null';
         }
         try {
            if (window.localStorage) {
               window.localStorage.setItem(storePrefix + k, encoded);
            }
         } catch (_) {}
         mem[k] = value;
         notifyValueChange(k, oldVal, value);
      };

      const gmListValues = () => {
         const out = [];
         try {
            if (window.localStorage) {
               for (let i = 0; i < window.localStorage.length; i++) {
                  const fullKey = window.localStorage.key(i);
                  if (fullKey && fullKey.startsWith(storePrefix)) {
                     out.push(fullKey.slice(storePrefix.length));
                  }
               }
               return out;
            }
         } catch (_) {}
         return Object.keys(mem);
      };

      const gmAddValueChangeListener = (key, cb) => {
         if (typeof cb !== 'function') return 0;
         const id = scriptForgeListenerSeq++;
         scriptForgeListeners[id] = { ruleId, key: String(key), cb };
         return id;
      };

      const gmRemoveValueChangeListener = (id) => {
         delete scriptForgeListeners[id];
      };

      const gmAddStyle = (cssText) => {
         const st = document.createElement('style');
         st.setAttribute('data-fnx-gm-style', ruleId);
         st.textContent = String(cssText || '');
         const parent =
            document.head || document.documentElement || document.body;
         if (parent) parent.appendChild(st);
         return st;
      };

      const gmSetClipboard = (text) => {
         copyTextToClipboard(String(text ?? ''), 'Copied by ScriptForge');
      };

      const gmNotification = (detailsOrText, title) => {
         const msg =
            detailsOrText && typeof detailsOrText === 'object'
               ? detailsOrText.text || detailsOrText.title || title || 'ScriptForge'
               : detailsOrText || title || 'ScriptForge';
         showHudToast(String(msg));
      };

      const gmOpenInTab = (url, options) => {
         const target = String(url || '').trim();
         if (!target) return null;
         const bg =
            typeof options === 'boolean'
               ? options
               : !!(options && (options.active === false || options.loadInBackground));
         sendIpc({ type: 'NEW_TAB', url: target, background: bg });
         return { close: () => {} };
      };

      const gmRegisterMenuCommand = (caption, onClick) => {
         if (typeof onClick !== 'function') return '';
         const id = 'sf_cmd_' + scriptForgeCmdSeq++;
         scriptForgeMenuCommands.push({
            id,
            ruleId,
            scriptName,
            caption: String(caption || 'Run command'),
            onClick,
         });
         if (typeof syncShieldPopoverUIRef === 'function') {
            syncShieldPopoverUIRef();
         }
         return id;
      };

      const gmUnregisterMenuCommand = (cmdId) => {
         const before = scriptForgeMenuCommands.length;
         scriptForgeMenuCommands = scriptForgeMenuCommands.filter(
            (c) => c.id !== cmdId && c.caption !== cmdId,
         );
         if (
            scriptForgeMenuCommands.length !== before &&
            typeof syncShieldPopoverUIRef === 'function'
         ) {
            syncShieldPopoverUIRef();
         }
      };

      const gmXmlhttpRequest = (details) => {
         const d = details || {};
         const rawUrl = String(d.url || '');
         const connectCheck = isUrlAllowedByUserscriptConnect(rawUrl, r, meta);
         if (!connectCheck.allowed) {
            const msg = 'Blocked by ScriptForge policy: ' + connectCheck.reason;
            setTimeout(() => {
               if (typeof d.onerror === 'function') {
                  try {
                     d.onerror({ error: msg, status: 0, statusText: msg });
                  } catch (_) {}
               }
            }, 0);
            return { abort: () => {} };
         }
         const url = connectCheck.url || rawUrl;
         const method = String(d.method || 'GET').toUpperCase();
         const controller =
            typeof AbortController === 'function' ? new AbortController() : null;
         let timedOut = false;
         let timeoutTimer = null;
         if (d.timeout && d.timeout > 0 && controller) {
            timeoutTimer = setTimeout(() => {
               timedOut = true;
               try {
                  controller.abort();
               } catch (_) {}
            }, Number(d.timeout));
         }
         const hasExplicitXhrGrant = !!(
            meta &&
            Array.isArray(meta.grants) &&
            meta.grants.some(
               (g) => g === 'GM_xmlhttpRequest' || g === 'GM.xmlHttpRequest',
            )
         );
         const credMode = d.anonymous
            ? 'omit'
            : connectCheck.isSameHost
              ? 'same-origin'
              : hasExplicitXhrGrant && d.withCredentials === true
                ? 'include'
                : 'omit';
         const init = {
            method,
            headers: d.headers || {},
            credentials: credMode,
         };
         if (controller) init.signal = controller.signal;
         if (d.data !== undefined && method !== 'GET' && method !== 'HEAD') {
            init.body = d.data;
         }

         fetch(url, init)
            .then(async (res) => {
               if (timeoutTimer) clearTimeout(timeoutTimer);
               let rawHeaders = '';
               try {
                  res.headers.forEach((v, k) => {
                     rawHeaders += k + ': ' + v + '\r\n';
                  });
               } catch (_) {}
               const rt = String(d.responseType || 'text').toLowerCase();
               let responseVal = null;
               let responseText = '';
               if (rt === 'blob') {
                  responseVal = await res.blob();
               } else if (rt === 'arraybuffer') {
                  responseVal = await res.arrayBuffer();
               } else if (rt === 'json') {
                  responseText = await res.text();
                  try {
                     responseVal = __fnxJsonParse(responseText);
                  } catch (_) {
                     responseVal = null;
                  }
               } else {
                  responseText = await res.text();
                  responseVal = responseText;
               }
               if (typeof d.onload === 'function') {
                  d.onload({
                     readyState: 4,
                     status: res.status,
                     statusText: res.statusText || '',
                     responseHeaders: rawHeaders,
                     finalUrl: res.url || url,
                     responseText,
                     response: responseVal,
                  });
               }
            })
            .catch((err) => {
               if (timeoutTimer) clearTimeout(timeoutTimer);
               if (timedOut && typeof d.ontimeout === 'function') {
                  d.ontimeout({ error: 'timeout' });
               } else if (typeof d.onerror === 'function') {
                  d.onerror({ error: String((err && err.message) || err || 'Network error') });
               }
            });

         return {
            abort: () => {
               if (timeoutTimer) clearTimeout(timeoutTimer);
               if (controller) {
                  try {
                     controller.abort();
                  } catch (_) {}
               }
            },
         };
      };

      const gmLog = console.log.bind(console, '[ScriptForge:' + scriptName + ']');

      const gmAsync = {
         info: gmInfo,
         addStyle: (cssText) => Promise.resolve(gmAddStyle(cssText)),
         getValue: (key, defVal) => Promise.resolve(gmGetValue(key, defVal)),
         setValue: (key, val) => Promise.resolve(gmSetValue(key, val)),
         deleteValue: (key) => Promise.resolve(gmDeleteValue(key)),
         listValues: () => Promise.resolve(gmListValues()),
         addValueChangeListener: (key, cb) =>
            Promise.resolve(gmAddValueChangeListener(key, cb)),
         removeValueChangeListener: (id) =>
            Promise.resolve(gmRemoveValueChangeListener(id)),
         setClipboard: (text) => Promise.resolve(gmSetClipboard(text)),
         notification: (detailsOrText, title) =>
            Promise.resolve(gmNotification(detailsOrText, title)),
         openInTab: (url, opts) => Promise.resolve(gmOpenInTab(url, opts)),
         registerMenuCommand: (caption, cb) =>
            Promise.resolve(gmRegisterMenuCommand(caption, cb)),
         unregisterMenuCommand: (id) =>
            Promise.resolve(gmUnregisterMenuCommand(id)),
         xmlHttpRequest: gmXmlhttpRequest,
         log: gmLog,
      };

      return {
         unsafeWindow: window,
         GM: gmAsync,
         GM_info: gmInfo,
         GM_addStyle: gmAddStyle,
         GM_getValue: gmGetValue,
         GM_setValue: gmSetValue,
         GM_deleteValue: gmDeleteValue,
         GM_listValues: gmListValues,
         GM_addValueChangeListener: gmAddValueChangeListener,
         GM_removeValueChangeListener: gmRemoveValueChangeListener,
         GM_setClipboard: gmSetClipboard,
         GM_notification: gmNotification,
         GM_openInTab: gmOpenInTab,
         GM_registerMenuCommand: gmRegisterMenuCommand,
         GM_unregisterMenuCommand: gmUnregisterMenuCommand,
         GM_xmlhttpRequest: gmXmlhttpRequest,
         GM_log: gmLog,
      };
   }