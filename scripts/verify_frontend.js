const fs = require('fs');
const path = require('path');
const vm = require('vm');
const { execFileSync } = require('child_process');

function walkFiles(dir, exts) {
  const out = [];
  for (const entry of fs.readdirSync(dir, { withFileTypes: true })) {
    const fp = path.join(dir, entry.name);
    if (entry.isDirectory()) {
      out.push(...walkFiles(fp, exts));
    } else if (exts.test(entry.name)) {
      out.push(fp);
    }
  }
  return out;
}

const jsTargets = [
  ...walkFiles(path.join('src', 'hud'), /\.js$/i),
  ...walkFiles(path.join('src', 'pages'), /\.js$/i),
].sort();

for (const jsFile of jsTargets) {
  execFileSync(process.execPath, ['--check', jsFile], { stdio: 'inherit' });
}

const allSrcFiles = walkFiles('src', /\.(js|html|css|rs)$/i);
const emojiRegex = /[\u{1F000}-\u{1FAFF}\u2600-\u27BF\u231A-\u23FF\u2B50\uFE0F\u20E3]/gu;

const emojiHits = [];
const innerHtmlHits = [];
const dupIdHits = [];
const dynamicEvalHits = [];

for (const file of allSrcFiles) {
  const norm = file.replace(/\\/g, '/');
  const txt = fs.readFileSync(file, 'utf8');
  const lines = txt.split(/\r?\n/);

  lines.forEach((line, idx) => {
    const emojiMatches = line.match(emojiRegex);
    if (emojiMatches) {
      emojiHits.push(`${norm}:${idx + 1} -> ${JSON.stringify(emojiMatches)}`);
    }

    if (norm.endsWith('.js') && (norm.startsWith('src/hud/') || norm.startsWith('src/pages/'))) {
      if (/\.innerHTML\b/.test(line)) {
        innerHtmlHits.push(`${norm}:${idx + 1} -> ${line.trim()}`);
      }

      if (/\beval\s*\(/.test(line)) {
        const ctx = lines.slice(Math.max(0, idx - 24), idx + 3).join('\n');
        const isAuditedDevRepl =
          (norm === 'src/hud/runtime.js' || norm === 'src/hud/content_agent.js') &&
          line.includes('window.eval(code)') &&
          ctx.includes('config.allow_page_eval');
        if (!isAuditedDevRepl) {
          dynamicEvalHits.push(`${norm}:${idx + 1} (unaudited eval) -> ${line.trim()}`);
        }
      }

      if (/\bnew\s+Function\s*\(/.test(line)) {
        const ctx = lines.slice(Math.max(0, idx - 6), idx + 4).join('\n');
        const isAuditedScriptForge =
          (norm === 'src/hud/runtime.js' || norm === 'src/hud/content_agent.js') &&
          ctx.includes('createScriptForgeSandbox');
        if (!isAuditedScriptForge) {
          dynamicEvalHits.push(`${norm}:${idx + 1} (unaudited new Function) -> ${line.trim()}`);
        }
      }
    }
  });

  if (norm === 'src/pages/settings.html' || norm === 'src/pages/newtab.html') {
    const seen = new Map();
    for (const m of txt.matchAll(/\bid\s*=\s*"([^"]+)"/g)) {
      const id = m[1];
      seen.set(id, (seen.get(id) || 0) + 1);
    }
    for (const [id, count] of seen.entries()) {
      if (count > 1) {
        dupIdHits.push(`${norm}: id="${id}" appears ${count} times`);
      }
    }
  }
}

const requiredTokensFile = path.join('src', 'hud', 'tokens.css');
if (!fs.existsSync(requiredTokensFile)) {
  console.error('MISSING_TOKENS_CSS:', requiredTokensFile);
  process.exit(1);
}
const allowedScalePx = new Set(['0', '0px', '2px', '3px', '4px', '5px', '6px', '7px', '8px', '9px', '10px', '10.5px', '11px', '11.5px', '12px', '12.5px', '13px', '13.5px', '14px', '14.5px', '15px', '16px', '18px', '20px', '42px', '44px', '50%', '999px']);
const cssTokenViolations = [];
for (const cssFile of [path.join('src', 'hud', 'tokens.css')]) {
  const txt = fs.readFileSync(cssFile, 'utf8');
  for (const m of txt.matchAll(/(?:border-radius|font-size)\s*:\s*([^;]+);/g)) {
    const val = m[1].trim();
    if (!val.startsWith('var(') && !allowedScalePx.has(val)) {
      cssTokenViolations.push(`${cssFile}: off-scale value ${val}`);
    }
  }
}

const themeDriftViolations = [];
const leakedDefaultHexRegex = /#(?:0c0e13|0d0f14|1c212d|151923)\b/i;
for (const jsFile of jsTargets) {
  const norm = jsFile.replace(/\\/g, '/');
  const txt = fs.readFileSync(jsFile, 'utf8');
  if ((norm === 'src/pages/settings.js' || norm === 'src/pages/newtab.js') && /\bisDefaultDark\b/.test(txt)) {
    themeDriftViolations.push(`${norm}: forbidden isDefaultDark hardcoded branch`);
  }
  const lines = txt.split(/\r?\n/);
  lines.forEach((line, idx) => {
    if (leakedDefaultHexRegex.test(line)) {
      themeDriftViolations.push(`${norm}:${idx + 1}: hardcoded default Obsidian hex (${line.trim()})`);
    }
  });
}

console.log('SYNTAX_CHECKED_JS=' + jsTargets.length);
console.log('SCANNED_FILES=' + allSrcFiles.length);
console.log('EMOJI_COUNT=' + emojiHits.length);
emojiHits.forEach((h) => console.error('EMOJI:', h));
console.log('INNERHTML_COUNT=' + innerHtmlHits.length);
innerHtmlHits.forEach((h) => console.error('INNERHTML:', h));
console.log('DUP_ID_COUNT=' + dupIdHits.length);
dupIdHits.forEach((h) => console.error('DUP_ID:', h));
console.log('UNAUDITED_DYNAMIC_CODE=' + dynamicEvalHits.length);
dynamicEvalHits.forEach((h) => console.error('UNAUDITED_EVAL:', h));
console.log('TOKEN_VIOLATIONS=' + cssTokenViolations.length);
cssTokenViolations.forEach((v) => console.error('TOKEN_VIOLATION:', v));
console.log('THEME_DRIFT_VIOLATIONS=' + themeDriftViolations.length);
themeDriftViolations.forEach((v) => console.error('THEME_DRIFT:', v));

// Runtime VM Boot Smoke Tests (catches ReferenceErrors / broken init paths in CI)
const vmSmokeErrors = [];
function runVmSmokeTests() {
  const i18nJs = fs.readFileSync(path.join('src', 'hud', 'i18n.js'), 'utf8');
  const iconsJs = fs.readFileSync(path.join('src', 'hud', 'icons.js'), 'utf8');
  const sharedDefaults = `
    const DEFAULT_ENGINES = [{ id: 'google', name: 'Google', url: 'https://www.google.com/search?q={query}' }];
    const DEFAULT_BANGS = [{ trigger: '!gh', name: 'GitHub', url: 'https://github.com/search?q={query}' }];
    const DEFAULT_KEYBINDS = { omnibox: 'Ctrl+L', new_tab: 'Ctrl+T', close_tab: 'Ctrl+W', reopen_closed_tab: 'Ctrl+Shift+T', next_tab: 'Ctrl+Tab', prev_tab: 'Ctrl+Shift+Tab', reload: 'Ctrl+R', go_back: 'Alt+ArrowLeft', go_forward: 'Alt+ArrowRight', toggle_side_panel: 'Alt+S', pin_tab: 'Alt+P', mute_tab: 'Alt+M', devtools: 'F12', dev_drawer: 'Ctrl+Shift+J', new_window: 'Ctrl+N', new_incognito_window: 'Ctrl+Shift+N', bookmark_page: 'Ctrl+D', find_in_page: 'Ctrl+F', reader_mode: 'Alt+R', force_dark: 'Alt+Shift+D', fullscreen: 'F11', preferences: 'Ctrl+,', zoom_in: 'Ctrl+=', zoom_out: 'Ctrl+-', zoom_reset: 'Ctrl+0' };
    const DEFAULT_STYLES = { colors: { bg: '#0b0d12', panel: '#1d222e', surface: '#141822', elevated: '#272d3d', border: '#2c3345', border_hover: '#404a63', fg: '#f3f5f9', secondary: '#b2bacd', muted: '#838ca3', accent: '#818cf8', accent_bg: '#20254b', hover: '#222838', tab_active: '#242a3a', tab_inactive: 'transparent' }, topbar: { layout: 'deck', visible_on_desktop: true, bookmarks_bar: 'always' }, tabs: {}, address_bar: {}, search_modal: {}, mobile_bar: { visible: true, position: 'bottom' }, tab_drawer: {}, settings_page: {}, newtab: {} };
    const DEFAULT_MODE_PROFILES = [];
    const BUILTIN_THEME_COLORS = {};
    function evalSafeMathExpr() { return null; }
    function setRichI18nText(el, z, e, zh) { if (el) el.textContent = zh ? z : e; }
    function applySelectOptionsI18n() {}
    function syncDocLangAndTitle() {}
  `;

  function makeEl(tag, id) {
    const classes = new Set();
    const children = [];
    const dataset = {};
    const attrs = {};
    const listeners = {};
    const style = {
      _props: {},
      setProperty(k, v) { this._props[k] = v; },
      removeProperty(k) { delete this._props[k]; },
    };
    const el = {
      tagName: (tag || 'DIV').toUpperCase(),
      id: id || '',
      value: '',
      textContent: '',
      placeholder: '',
      disabled: false,
      checked: false,
      options: [],
      style,
      dataset,
      classList: {
        add(...c) { c.forEach((x) => classes.add(x)); },
        remove(...c) { c.forEach((x) => classes.delete(x)); },
        toggle(c, f) { const on = f !== undefined ? !!f : !classes.has(c); if (on) classes.add(c); else classes.delete(c); return on; },
        contains(c) { return classes.has(c); },
      },
      set className(v) { classes.clear(); String(v || '').split(/\s+/).filter(Boolean).forEach((x) => classes.add(x)); },
      get className() { return [...classes].join(' '); },
      setAttribute(k, v) { attrs[k] = String(v); if (k.startsWith('data-')) dataset[k.slice(5).replace(/-([a-z])/g, (_, c) => c.toUpperCase())] = String(v); },
      getAttribute(k) { return attrs[k] ?? null; },
      removeAttribute(k) { delete attrs[k]; },
      appendChild(c) { if (c && c.tagName === 'FRAGMENT' && Array.isArray(c.children)) { c.children.forEach((ch) => { ch.parentNode = el; children.push(ch); }); } else if (c) { c.parentNode = el; children.push(c); } return c; },
      insertBefore(c) { return this.appendChild(c); },
      cloneNode() { return makeEl(this.tagName, ''); },
      remove() {},
      contains(other) { return other === el || children.some((c) => c === other || (c.contains && c.contains(other))); },
      addEventListener(ev, fn) { (listeners[ev] = listeners[ev] || []).push(fn); },
      removeEventListener() {},
      dispatchEvent(ev) { (listeners[ev.type] || []).forEach((fn) => fn(ev)); return true; },
      querySelector(sel) {
        if (sel && sel.startsWith('#')) {
          const targetId = sel.slice(1).split(/[\s.:[]/)[0];
          if (el.id === targetId) return el;
          for (const ch of children) {
            if (ch.querySelector) {
              const found = ch.querySelector(sel);
              if (found) return found;
            }
          }
        }
        return null;
      },
      querySelectorAll() { return []; },
      getElementById(targetId) {
        if (el.id === targetId) return el;
        for (const ch of children) {
          if (ch.getElementById) {
            const found = ch.getElementById(targetId);
            if (found) return found;
          }
        }
        return null;
      },
      closest() { return null; },
      getBoundingClientRect() { return { left: 0, top: 0, right: 40, bottom: 40, width: 40, height: 40 }; },
      attachShadow() { el.shadowRoot = makeEl('SHADOW', ''); return el.shadowRoot; },
      scrollIntoView() {},
      focus() {},
      blur() {},
      select() {},
      childNodes: children,
      children,
      isConnected: true,
      nodeType: 1,
    };
    return el;
  }

  // 1. Settings page boot test
  try {
    const html = fs.readFileSync(path.join('src', 'pages', 'settings.html'), 'utf8');
    const js = fs.readFileSync(path.join('src', 'pages', 'settings.js'), 'utf8');
    const byId = new Map();
    for (const m of html.matchAll(/\bid="([^"]+)"/g)) byId.set(m[1], makeEl('DIV', m[1]));
    const uaBtns = ['default', 'desktop', 'mobile', 'custom'].map((ua) => { const b = makeEl('BUTTON', ''); b.setAttribute('data-ua', ua); return b; });
    const wmBtns = ['multi', 'single'].map((wm) => { const b = makeEl('BUTTON', ''); b.setAttribute('data-wm', wm); return b; });
    const doc = {
      readyState: 'complete',
      documentElement: makeEl('HTML', ''),
      body: makeEl('BODY', ''),
      title: '',
      activeElement: null,
      getElementById: (id) => byId.get(id) || null,
      createElement: (t) => makeEl(t, ''),
      createElementNS: (_ns, t) => makeEl(t, ''),
      createTextNode: (t) => ({ nodeType: 3, textContent: String(t) }),
      createDocumentFragment: () => makeEl('FRAGMENT', ''),
      querySelector: (s) => (s.startsWith('#') ? byId.get(s.slice(1).split(/[\s.:[]/)[0]) || null : null),
      querySelectorAll: (s) => (s === '#uaModeGroup .seg-btn' ? uaBtns : s === '#windowModeGroup .seg-btn' ? wmBtns : []),
      addEventListener() {},
    };
    const boot = JSON.stringify({
      config: { theme: 'dark', language: 'en', user_agent_mode: 'default', window_mode: 'multi', homepage: 'fynexo://newtab', default_zoom: 1.0 },
      styles: { colors: { use_custom_colors: false }, topbar: {}, tabs: {}, address_bar: {}, newtab: {}, settings_page: {} },
      active_theme: { bg: '#0b0d12', panel: '#1d222e', surface: '#141822', elevated: '#272d3d', fg: '#f3f5f9', secondary: '#b2bacd', muted: '#838ca3', accent: '#818cf8' },
      themes: [{ id: 'dark', name: 'Obsidian', builtin: true, colors: { bg: '#0b0d12', panel: '#1d222e', surface: '#141822', elevated: '#272d3d', fg: '#f3f5f9', secondary: '#b2bacd', muted: '#838ca3', accent: '#818cf8' } }],
      bookmarks: [], site_rules: [], adblock: { custom_blocked_domains: [], whitelisted_domains: [] }, keybinds: {},
    });
    const code = js.replace('__FNX_DEFAULTS__;', sharedDefaults + '\n' + i18nJs + '\n' + iconsJs).replace('__FNX_SUPABASE_JSON__', '{}').replace('__FNX_BOOT_JSON__', boot);
    const sb = { window: null, document: doc, location: { hash: '', href: 'http://fynexo.localhost/settings' }, history: { replaceState() {} }, navigator: { userAgent: 'Chrome/134' }, localStorage: { getItem: () => null, setItem() {}, removeItem() {} }, console, setTimeout, clearTimeout, URL, encodeURIComponent, decodeURIComponent, requestAnimationFrame: (fn) => fn() };
    sb.window = sb; sb.window.addEventListener = () => {}; sb.window.scrollTo = () => {}; sb.window.ipc = { postMessage() {} };
    vm.runInNewContext(code, sb);
    const activeUa = uaBtns.filter((b) => b.classList.contains('active')).map((b) => b.dataset.ua);
    if (activeUa.length !== 1 || activeUa[0] !== 'default') throw new Error('Expected activeUa=["default"], got ' + JSON.stringify(activeUa));
    if (byId.get('selInterfaceLang').value !== 'en') throw new Error('Expected selInterfaceLang="en", got ' + byId.get('selInterfaceLang').value);
    if (byId.get('inputHomepage').value !== 'fynexo://newtab') throw new Error('Expected inputHomepage="fynexo://newtab"');
  } catch (e) {
    vmSmokeErrors.push('settings.js boot: ' + (e && e.stack ? e.stack.split('\n')[0] : String(e)));
  }

  // 2. NewTab page boot test
  try {
    const html = fs.readFileSync(path.join('src', 'pages', 'newtab.html'), 'utf8');
    const js = fs.readFileSync(path.join('src', 'pages', 'newtab.js'), 'utf8');
    const byId = new Map();
    for (const m of html.matchAll(/\bid="([^"]+)"/g)) byId.set(m[1], makeEl('DIV', m[1]));
    const docEl = makeEl('HTML', '');
    const doc = {
      readyState: 'complete',
      documentElement: docEl,
      body: makeEl('BODY', ''),
      title: '',
      activeElement: null,
      getElementById: (id) => byId.get(id) || null,
      createElement: (t) => makeEl(t, ''),
      createElementNS: (_ns, t) => makeEl(t, ''),
      createTextNode: (t) => ({ nodeType: 3, textContent: String(t) }),
      createDocumentFragment: () => makeEl('FRAGMENT', ''),
      querySelector: (s) => (s.startsWith('#') ? byId.get(s.slice(1).split(/[\s.:[]/)[0]) || null : null),
      querySelectorAll: () => [],
      addEventListener() {},
    };
    const boot = JSON.stringify({
      config: { theme: 'dark', language: 'en', search_engine: 'google' },
      styles: { colors: { use_custom_colors: false }, newtab: { show_clock: true, show_shortcuts: true, show_history: true } },
      active_theme: { bg: '#0b0d12', panel: '#1d222e', surface: '#141822', elevated: '#272d3d', fg: '#f3f5f9', secondary: '#b2bacd', muted: '#838ca3', accent: '#818cf8' },
      bookmarks: [{ title: 'GitHub', url: 'https://github.com' }],
      history: [{ title: 'GitHub', url: 'https://github.com', visited_at: 1700000000 }],
    });
    const code = js.replace('__FNX_DEFAULTS__;', sharedDefaults + '\n' + i18nJs + '\n' + iconsJs).replace('__FNX_BOOT_JSON__', boot);
    const sb = { window: null, document: doc, location: { hash: '', href: 'http://fynexo.localhost/newtab' }, navigator: { userAgent: 'Chrome/134' }, localStorage: { getItem: () => null, setItem() {}, removeItem() {} }, console, setTimeout, clearTimeout, setInterval: () => 1, clearInterval() {}, URL, encodeURIComponent, decodeURIComponent, requestAnimationFrame: (fn) => fn() };
    sb.window = sb; sb.window.addEventListener = () => {}; sb.window.ipc = { postMessage() {} };
    vm.runInNewContext(code, sb);
    if (docEl.style._props['--nt-surface'] !== '#1d222e') throw new Error('Expected --nt-surface mapped to panel #1d222e, got ' + docEl.style._props['--nt-surface']);
    if (docEl.style._props['--nt-on-accent'] !== '#0b0e17') throw new Error('Expected --nt-on-accent=#0b0e17, got ' + docEl.style._props['--nt-on-accent']);
  } catch (e) {
    vmSmokeErrors.push('newtab.js boot: ' + (e && e.stack ? e.stack.split('\n')[0] : String(e)));
  }

  // 3. ContentTab agent boot + Ctrl+L / Ctrl+Shift+N keybind dispatch test
  try {
    const js = fs.readFileSync(path.join('src', 'hud', 'content_agent.js'), 'utf8');
    const sent = [];
    const winListeners = {};
    const docEl = makeEl('HTML', '');
    const headEl = makeEl('HEAD', '');
    const bodyEl = makeEl('BODY', '');
    docEl.appendChild(headEl);
    docEl.appendChild(bodyEl);
    const doc = {
      readyState: 'complete',
      documentElement: docEl,
      head: headEl,
      body: bodyEl,
      title: 'Example',
      activeElement: bodyEl,
      getElementById: (id) => docEl.getElementById(id),
      createElement: (t) => makeEl(t, ''),
      createElementNS: (_ns, t) => makeEl(t, ''),
      createTextNode: (t) => ({ nodeType: 3, textContent: String(t) }),
      querySelector: () => null,
      querySelectorAll: () => [],
      addEventListener() {},
      removeEventListener() {},
    };
    const code = js
      .replace('__FNX_IPC_TOKEN__', 'tok-test')
      .replace('__FNX_BRIDGE_TOKEN__', 'bridge-test')
      .replace('__FNX_ROLE__', 'content')
      .replace('__FNX_MOBILE_HOST__', 'false')
      .replace('__FNX_I18N__;', i18nJs)
      .replace('__FNX_DEFAULTS__', sharedDefaults)
      .replace('__FNX_CONTENT_CSS__', '""')
      .replace('__FNX_SCRIPTFORGE__;', '')
      .replace('__FNX_EXTENSIONS__;', '')
      .replace('__FNX_INITIAL_JSON__', JSON.stringify({ config: { theme: 'dark', language: 'en' }, active_theme: {}, keybinds: {}, tabs: { tabs: [{ id: 1, url: 'https://example.com', is_active: true }], active_id: 1 } }));
    const sb = {
      window: null, self: null, top: null, document: doc,
      location: { href: 'https://example.com/', hostname: 'example.com', protocol: 'https:', port: '', pathname: '/', origin: 'https://example.com' },
      navigator: { userAgent: 'Chrome/134' },
      history: { pushState() {}, replaceState() {} },
      console, setTimeout, clearTimeout, setInterval: () => 1, clearInterval() {}, URL, encodeURIComponent, decodeURIComponent,
      requestAnimationFrame: (fn) => fn(),
      getComputedStyle: () => ({ backgroundColor: 'rgb(255, 255, 255)', colorScheme: 'light' }),
      MutationObserver: class { observe() {} disconnect() {} },
      ipc: { postMessage(raw) { try { sent.push(JSON.parse(raw)); } catch (_) {} } },
    };
    sb.window = sb; sb.self = sb; sb.top = sb;
    sb.window.addEventListener = (ev, fn) => { (winListeners[ev] = winListeners[ev] || []).push(fn); };
    sb.window.removeEventListener = () => {};
    sb.window.getSelection = () => '';
    vm.runInNewContext(code, sb);
    const kdList = winListeners.keydown || [];
    kdList.forEach((fn) => fn({ key: 'l', ctrlKey: true, metaKey: false, altKey: false, shiftKey: false, target: bodyEl, preventDefault() {}, stopPropagation() {} }));
    kdList.forEach((fn) => fn({ key: 'N', ctrlKey: true, metaKey: false, altKey: false, shiftKey: true, target: bodyEl, preventDefault() {}, stopPropagation() {} }));
    if (!sent.some((m) => m.type === 'HUD_ACTION' && m.action === 'open_omnibox')) {
      throw new Error('Ctrl+L in content_agent.js did not emit open_omnibox');
    }
    if (!sent.some((m) => m.type === 'NEW_WINDOW' && m.incognito === true)) {
      throw new Error('Ctrl+Shift+N in content_agent.js did not emit incognito NEW_WINDOW');
    }
  } catch (e) {
    vmSmokeErrors.push('content_agent.js boot: ' + (e && e.stack ? e.stack.split('\n')[0] : String(e)));
  }
}
runVmSmokeTests();
console.log('VM_SMOKE_ERRORS=' + vmSmokeErrors.length);
vmSmokeErrors.forEach((v) => console.error('VM_SMOKE_ERROR:', v));

if (emojiHits.length || innerHtmlHits.length || dupIdHits.length || dynamicEvalHits.length || cssTokenViolations.length || themeDriftViolations.length || vmSmokeErrors.length) {
  process.exit(1);
}
