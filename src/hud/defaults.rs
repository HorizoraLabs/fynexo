use crate::config::{
    builtin_theme_presets, default_bangs, default_mode_profiles, default_search_engines, ComponentStyles, KeybindsConfig,
};

const SAFE_MATH_HELPER: &str = r#"function evalSafeMathExpr(raw) {
   const src = String(raw || '').trim().replace(/×/g, '*').replace(/÷/g, '/');
   if (!src || src.length > 80 || !/^[0-9+\-*/().%\s^,]+$/.test(src)) return null;
   if (!/[0-9]/.test(src) || !/[+\-*/%^]/.test(src.replace(/^\s*-/, ''))) return null;
   const toks = src.replace(/,/g, '').replace(/\*\*/g, '^').match(/\d+\.?\d*|\.\d+|[+\-*/%^()]/g);
   if (!toks || toks.join('').length < src.replace(/[\s,]/g, '').replace(/\*\*/g, '^').length) return null;
   let pos = 0;
   let guard = 0;
   function peek() { return toks[pos]; }
   function parseExpr() {
      let v = parseTerm();
      while (peek() === '+' || peek() === '-') {
         const op = toks[pos++];
         const r = parseTerm();
         v = op === '+' ? v + r : v - r;
      }
      return v;
   }
   function parseTerm() {
      let v = parsePow();
      while (peek() === '*' || peek() === '/' || peek() === '%') {
         const op = toks[pos++];
         const r = parsePow();
         v = op === '*' ? v * r : op === '/' ? v / r : v % r;
      }
      return v;
   }
   function parsePow() {
      const base = parseUnary();
      if (peek() === '^') {
         pos++;
         return Math.pow(base, parsePow());
      }
      return base;
   }
   function parseUnary() {
      if (++guard > 200) throw new Error('deep');
      if (peek() === '-') { pos++; return -parseUnary(); }
      if (peek() === '+') { pos++; return parseUnary(); }
      if (peek() === '(') {
         pos++;
         const v = parseExpr();
         if (peek() !== ')') throw new Error('paren');
         pos++;
         return v;
      }
      const n = parseFloat(toks[pos++]);
      if (!isFinite(n)) throw new Error('num');
      return n;
   }
   try {
      const v = parseExpr();
      if (pos !== toks.length || !isFinite(v)) return null;
      const rounded = Math.round(v * 1e10) / 1e10;
      return { value: rounded, text: rounded.toLocaleString('en-US', { maximumFractionDigits: 10 }) };
   } catch (_) {
      return null;
   }
}

function serializeRichRaw(el) {
   let out = '';
   if (!el || !el.childNodes) return out;
   el.childNodes.forEach((n) => {
      if (n.nodeType === 3) out += n.textContent || '';
      else if (n.nodeType === 1 && n.tagName === 'CODE') out += '`' + (n.textContent || '') + '`';
      else if (n.nodeType === 1) out += n.textContent || '';
   });
   return out;
}

function setRichI18nText(el, zhText, enText, zh) {
   if (!el) return;
   if (el.dataset.enRaw === undefined) {
      el.dataset.enRaw = (enText !== undefined && enText !== null) ? String(enText) : serializeRichRaw(el);
   }
   const raw = zh ? String(zhText || el.dataset.enRaw) : ((enText !== undefined && enText !== null) ? String(enText) : el.dataset.enRaw);
   el.textContent = '';
   const re = /`([^`]+)`/g;
   let lastIdx = 0;
   let m;
   while ((m = re.exec(raw)) !== null) {
      if (m.index > lastIdx) {
         el.appendChild(document.createTextNode(raw.slice(lastIdx, m.index)));
      }
      const code = document.createElement('code');
      code.textContent = m[1];
      el.appendChild(code);
      lastIdx = m.index + m[0].length;
   }
   if (lastIdx < raw.length) {
      el.appendChild(document.createTextNode(raw.slice(lastIdx)));
   }
}

function applySelectOptionsI18n(selectId, zhMap, enMap, zh) {
   const sel = document.getElementById(selectId);
   if (!sel) return;
   const map = zh ? zhMap : enMap;
   if (!map) return;
   Array.from(sel.options).forEach((o) => {
      if (map[o.value] !== undefined) o.textContent = map[o.value];
   });
}

function syncDocLangAndTitle(zh, zhTitle, enTitle) {
   try {
      if (document.documentElement) {
         document.documentElement.lang = zh ? 'zh-CN' : 'en';
      }
      if (zhTitle && enTitle) {
         document.title = zh ? zhTitle : enTitle;
      }
   } catch (_) {}
}"#;

pub fn defaults_script() -> String {
    let engines =
        serde_json::to_string(&default_search_engines()).unwrap_or_else(|_| "[]".to_string());
    let bangs = serde_json::to_string(&default_bangs()).unwrap_or_else(|_| "[]".to_string());
    let keybinds =
        serde_json::to_string(&KeybindsConfig::default()).unwrap_or_else(|_| "{}".to_string());
    let styles =
        serde_json::to_string(&ComponentStyles::default()).unwrap_or_else(|_| "{}".to_string());
    let mode_profiles =
        serde_json::to_string(&default_mode_profiles()).unwrap_or_else(|_| "[]".to_string());
    let themes: std::collections::BTreeMap<String, crate::config::ThemeColors> = builtin_theme_presets()
        .into_iter()
        .map(|t| (t.id, t.colors))
        .collect();
    let themes = serde_json::to_string(&themes).unwrap_or_else(|_| "{}".to_string());
    format!(
        "const DEFAULT_ENGINES = {engines};\nconst DEFAULT_BANGS = {bangs};\nconst DEFAULT_KEYBINDS = {keybinds};\nconst DEFAULT_STYLES = {styles};\nconst DEFAULT_MODE_PROFILES = {mode_profiles};\nconst BUILTIN_THEME_COLORS = {themes};\n{SAFE_MATH_HELPER}"
    )
}
