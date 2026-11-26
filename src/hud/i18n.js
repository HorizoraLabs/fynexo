/* Shared multi-language i18n helper for Fynexo HUD, Content Agent, Start Page, and Settings */
const FynexoI18n = (() => {
   const SUPPORTED_LOCALES = [
      { code: 'zh', label: '简体中文' },
      { code: 'en', label: 'English' },
   ];
   const PACKS = Object.create(null);

   function normalizeLocale(raw) {
      const s = String(raw == null ? '' : raw).trim().toLowerCase().replace(/_/g, '-');
      if (!s) return 'zh';
      if (PACKS[s]) return s;
      for (let i = 0; i < SUPPORTED_LOCALES.length; i++) {
         if (SUPPORTED_LOCALES[i].code === s) return s;
      }
      if (s === 'zh' || s.startsWith('zh-')) return 'zh';
      if (s === 'en' || s.startsWith('en-')) return 'en';
      const base = s.split('-')[0];
      if (PACKS[base]) return base;
      return 'en';
   }

   function isZhLocale(raw) {
      return normalizeLocale(raw).startsWith('zh');
   }

   function registerPack(locale, dict, label) {
      const code = String(locale || '').trim().toLowerCase().replace(/_/g, '-');
      if (!code || !dict || typeof dict !== 'object') return;
      PACKS[code] = Object.assign(PACKS[code] || Object.create(null), dict);
      if (label && !SUPPORTED_LOCALES.some((item) => item.code === code)) {
         SUPPORTED_LOCALES.push({ code: code, label: String(label) });
      }
   }

   function interpolate(tpl, vars) {
      let out = String(tpl == null ? '' : tpl);
      if (!vars || typeof vars !== 'object') return out;
      const keys = Object.keys(vars);
      for (let i = 0; i < keys.length; i++) {
         const k = keys[i];
         out = out.split('{' + k + '}').join(String(vars[k]));
      }
      return out;
   }

   function t(locale, key, fallback, vars) {
      const norm = normalizeLocale(locale);
      const base = norm.split('-')[0];
      const val =
         (PACKS[norm] && PACKS[norm][key]) ||
         (PACKS[base] && PACKS[base][key]) ||
         (PACKS.en && PACKS.en[key]) ||
         fallback ||
         key;
      return interpolate(val, vars);
   }

   return {
      SUPPORTED_LOCALES,
      normalizeLocale,
      isZhLocale,
      registerPack,
      t,
   };
})();
