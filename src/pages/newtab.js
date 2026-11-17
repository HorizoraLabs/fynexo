        __FNX_DEFAULTS__;

        const DEFAULT_NEWTAB_TITLE = 'Fynexo';
        const DEFAULT_NEWTAB_SUBTITLE = 'Single-WebView studio browser · Zero-FX compositing';

        const boot = __FNX_BOOT_JSON__ || {};
        let config = Object.assign({
            theme: 'dark',
            enforce_perf: true,
            adblock: true,
            restore_session: true,
            force_dark: false,
            incognito: false,
            language: 'zh',
            tab_cache_mode: 'balanced',
            ext_side_panel: false,
            ext_reader_mode: false,
            ext_yt_ad_skipper: false,
            ext_gh_harmonizer: false,
            ext_media_booster: false,
            search_engine: 'google',
            homepage: 'fynexo://newtab',
            search_engines: DEFAULT_ENGINES,
            bangs: DEFAULT_BANGS
        }, boot.config || {});
        function isZh() {
            return String(config.language || 'zh').toLowerCase() !== 'en';
        }
        let styles = boot.styles || {};
        let activeTheme = boot.active_theme || {};
        let bookmarks = Array.isArray(boot.bookmarks) ? boot.bookmarks : [];
        let historyItems = Array.isArray(boot.history) ? boot.history : [];
        let blockedAds = Number(boot.blocked_ads) || 0;

        function getEngines() {
            return (Array.isArray(config.search_engines) && config.search_engines.length > 0)
                ? config.search_engines
                : DEFAULT_ENGINES;
        }

        function getBangs() {
            return Array.isArray(config.bangs) ? config.bangs : DEFAULT_BANGS;
        }

        function currentEngineObj(engineId) {
            const list = getEngines();
            const target = engineId || config.search_engine;
            for (let i = 0; i < list.length; i++) {
                if (list[i] && list[i].id === target) return list[i];
            }
            return list[0] || DEFAULT_ENGINES[0];
        }

        function sendIpc(data) {
            const raw = JSON.stringify(data);
            if (window.ipc && typeof window.ipc.postMessage === 'function') {
                window.ipc.postMessage(raw);
            } else if (window.__TAURI_INTERNALS__ && window.__TAURI_INTERNALS__.ipc) {
                window.__TAURI_INTERNALS__.ipc.postMessage(raw);
            }
        }

        function safeParsedUrl(rawUrl) {
            if (!rawUrl || typeof rawUrl !== 'string') return null;
            try {
                const u = new URL(rawUrl);
                if (u.protocol === 'http:' || u.protocol === 'https:' || u.protocol === 'fynexo:') {
                    return u;
                }
            } catch (_) {}
            return null;
        }

        function formatHost(u) {
            try {
                return new URL(u).hostname.replace(/^www\./, '') || u;
            } catch (_) {
                return u || '';
            }
        }

        function monogramOf(url, title) {
            try {
                const h = new URL(url).hostname.replace(/^(www|m)\./, '');
                if (h) return h.charAt(0).toUpperCase();
            } catch (_) {}
            const t = (title || '').trim();
            return t ? t.charAt(0).toUpperCase() : '•';
        }

        function monogramColor(url) {
            const palette = ['#7A88F7', '#38BDF8', '#34D399', '#F472B6', '#FB923C', '#A78BFA'];
            const s = String(url || '');
            let h = 0;
            for (let i = 0; i < s.length; i++) {
                h = ((h << 5) - h + s.charCodeAt(i)) | 0;
            }
            return palette[Math.abs(h) % palette.length];
        }

        function createSvgEl(tag, attrs) {
            const el = document.createElementNS('http://www.w3.org/2000/svg', tag);
            if (attrs) {
                Object.keys(attrs).forEach(k => el.setAttribute(k, String(attrs[k])));
            }
            return el;
        }

        function createBrandSvg(host, urlStr) {
            if (String(urlStr || '').startsWith('fynexo://')) {
                const svg = createSvgEl('svg', { width: 15, height: 15, viewBox: '0 0 16 16', fill: 'none' });
                svg.appendChild(createSvgEl('path', {
                    d: 'M8 2.2l1.35 4.45L13.8 8l-4.45 1.35L8 13.8l-1.35-4.45L2.2 8l4.45-1.35z',
                    fill: 'var(--nt-accent)'
                }));
                return svg;
            }
            const h = (host || '').toLowerCase();
            if (h.includes('youtube.com') || h === 'youtu.be') {
                const svg = createSvgEl('svg', { width: 16, height: 16, viewBox: '0 0 16 16', fill: 'none' });
                svg.appendChild(createSvgEl('rect', { x: 1.5, y: 3.2, width: 13, height: 9.6, rx: 2.8, fill: '#FF0033' }));
                svg.appendChild(createSvgEl('path', { d: 'M6.6 5.9v4.2L10.3 8z', fill: '#FFFFFF' }));
                return svg;
            }
            if (h.includes('github.com')) {
                const svg = createSvgEl('svg', { width: 15, height: 15, viewBox: '0 0 16 16', fill: 'currentColor' });
                svg.style.color = 'var(--nt-fg)';
                svg.appendChild(createSvgEl('path', {
                    d: 'M8 1.6a6.4 6.4 0 0 0-2.02 12.47c.32.06.44-.14.44-.31v-1.1c-1.78.39-2.16-.86-2.16-.86-.29-.74-.71-.94-.71-.94-.58-.4.04-.39.04-.39.64.05.98.66.98.66.57.98 1.5.7 1.87.53.06-.41.22-.7.4-.86-1.42-.16-2.92-.71-2.92-3.16 0-.7.25-1.27.66-1.72-.07-.16-.29-.81.06-1.69 0 0 .54-.17 1.76.66a6.1 6.1 0 0 1 3.2 0c1.22-.83 1.76-.66 1.76-.66.35.88.13 1.53.06 1.69.41.45.66 1.02.66 1.72 0 2.46-1.5 3-2.93 3.15.23.2.43.59.43 1.19v1.77c0 .17.11.37.44.31A6.4 6.4 0 0 0 8 1.6Z'
                }));
                return svg;
            }
            if (h === 'x.com' || h.endsWith('.x.com') || h.includes('twitter.com')) {
                const svg = createSvgEl('svg', { width: 14, height: 14, viewBox: '0 0 16 16', fill: 'currentColor' });
                svg.style.color = 'var(--nt-fg)';
                svg.appendChild(createSvgEl('path', {
                    d: 'M9.5 6.9 14 1.8h-1.1L9 6.2 5.9 1.8H2.2l4.7 6.8L2.2 14h1.1l4.1-4.7 3.3 4.7h3.7L9.5 6.9Zm-1.5 1.7-.5-.7L3.7 2.6h1.6l3.1 4.4.5.7 4 5.6h-1.6L8 8.6Z'
                }));
                return svg;
            }
            if (h === 'google.com' || h.startsWith('google.') || h.includes('.google.')) {
                const svg = createSvgEl('svg', { width: 14, height: 14, viewBox: '0 0 16 16', fill: 'none' });
                svg.appendChild(createSvgEl('path', { d: 'M14.2 8.15c0-.46-.04-.9-.12-1.32H8v2.5h3.48a2.98 2.98 0 0 1-1.29 1.95v1.62h2.08c1.22-1.12 1.93-2.77 1.93-4.75Z', fill: '#4285F4' }));
                svg.appendChild(createSvgEl('path', { d: 'M8 14.5c1.74 0 3.2-.58 4.27-1.56l-2.08-1.62c-.58.39-1.32.62-2.19.62-1.68 0-3.11-1.14-3.62-2.67H2.23v1.67A6.49 6.49 0 0 0 8 14.5Z', fill: '#34A853' }));
                svg.appendChild(createSvgEl('path', { d: 'M4.38 9.27A3.9 3.9 0 0 1 4.18 8c0-.44.08-.87.2-1.27V5.06H2.23A6.49 6.49 0 0 0 1.5 8c0 1.05.25 2.04.73 2.94l2.15-1.67Z', fill: '#FBBC05' }));
                svg.appendChild(createSvgEl('path', { d: 'M8 4.06c.95 0 1.8.33 2.47.97l1.85-1.85C11.19 2.13 9.74 1.5 8 1.5a6.49 6.49 0 0 0-5.77 3.56l2.15 1.67C4.89 5.2 6.32 4.06 8 4.06Z', fill: '#EA4335' }));
                return svg;
            }
            if (h.includes('wikipedia.org')) {
                const svg = createSvgEl('svg', { width: 15, height: 15, viewBox: '0 0 16 16', fill: 'none' });
                svg.appendChild(createSvgEl('rect', { x: 1.5, y: 1.5, width: 13, height: 13, rx: 3.5, fill: '#E4E4E7' }));
                svg.appendChild(createSvgEl('path', { d: 'M3.8 5h1.5l1.6 4.4L8.2 6l1.3 3.4L11.1 5h1.4l-2.2 6.2H9L8 8.3 7 11.2H5.7z', fill: '#18181B' }));
                return svg;
            }
            if (h.includes('bilibili.com')) {
                const svg = createSvgEl('svg', { width: 15, height: 15, viewBox: '0 0 16 16', fill: 'none' });
                svg.appendChild(createSvgEl('path', { d: 'M5 2.5L6.8 4.2M11 2.5L9.2 4.2', stroke: '#00AEEC', 'stroke-width': 1.4, 'stroke-linecap': 'round' }));
                svg.appendChild(createSvgEl('rect', { x: 2.2, y: 4.3, width: 11.6, height: 8.6, rx: 2.2, stroke: '#00AEEC', 'stroke-width': 1.4 }));
                svg.appendChild(createSvgEl('circle', { cx: 6, cy: 8.5, r: 0.9, fill: '#00AEEC' }));
                svg.appendChild(createSvgEl('circle', { cx: 10, cy: 8.5, r: 0.9, fill: '#00AEEC' }));
                return svg;
            }
            return null;
        }

        function createSiteEmblem(rawUrl, title) {
            const wrap = document.createElement('span');
            wrap.className = 'nt-emblem';

            const parsed = safeParsedUrl(rawUrl);
            const host = parsed ? parsed.hostname.replace(/^www\./, '') : '';
            const brandSvg = createBrandSvg(host, rawUrl);

            const fb = document.createElement('span');
            fb.className = 'nt-emblem-fallback' + (brandSvg ? '' : ' nt-emblem-fallback--mono');
            if (brandSvg) {
                fb.appendChild(brandSvg);
            } else {
                fb.textContent = monogramOf(rawUrl, title);
                fb.style.color = monogramColor(rawUrl);
            }
            wrap.appendChild(fb);

            if (!brandSvg && parsed && (parsed.protocol === 'http:' || parsed.protocol === 'https:') && parsed.hostname) {
                const img = document.createElement('img');
                img.className = 'nt-emblem-img';
                img.alt = '';
                img.loading = 'lazy';
                img.decoding = 'async';
                img.referrerPolicy = 'no-referrer';
                img.onload = () => {
                    if (img.naturalWidth > 1) {
                        wrap.classList.add('has-img');
                    }
                };
                img.onerror = () => {
                    img.remove();
                };
                img.src = `https://www.google.com/s2/favicons?domain=${encodeURIComponent(parsed.hostname)}&sz=32`;
                wrap.appendChild(img);
            }
            return wrap;
        }

        function relativeTime(ts) {
            if (!ts) return '';
            const diff = Math.max(0, Math.floor(Date.now() / 1000) - Number(ts));
            if (diff < 60) return 'now';
            if (diff < 3600) return Math.floor(diff / 60) + 'm';
            if (diff < 86400) return Math.floor(diff / 3600) + 'h';
            return Math.floor(diff / 86400) + 'd';
        }

        function fillTpl(tpl, q) {
            const s = String(tpl || 'https://www.google.com/search?q={query}');
            if (s.includes('{query}')) return s.split('{query}').join(q);
            if (s.includes('%s')) return s.split('%s').join(q);
            return s + q;
        }

        function resolveQuery(raw, engine) {
            raw = (raw || '').trim();
            if (!raw) return '';
            if (/^fynexo:\/\//i.test(raw)) return raw;
            if (/^(settings|preferences|about:settings)$/i.test(raw)) return 'fynexo://settings';
            if (raw.startsWith('!')) {
                const sp = raw.indexOf(' ');
                const tok = (sp === -1 ? raw : raw.slice(0, sp)).toLowerCase();
                const rest = sp === -1 ? '' : raw.slice(sp + 1).trim();
                const bangs = getBangs();
                for (let i = 0; i < bangs.length; i++) {
                    const b = bangs[i];
                    if (!b || !b.trigger) continue;
                    const trig = (b.trigger.startsWith('!') ? b.trigger : '!' + b.trigger).toLowerCase();
                    if (trig === tok) {
                        if (!rest) {
                            try { return new URL(fillTpl(b.url, '')).origin; } catch (_) { return fillTpl(b.url, ''); }
                        }
                        return fillTpl(b.url, encodeURIComponent(rest));
                    }
                }
            }
            if (/^https?:\/\//i.test(raw)) return raw;
            if (/^(localhost|127\.0\.0\.1)(:\d+)?/i.test(raw)) return 'http://' + raw;
            if (raw.includes('.') && !raw.includes(' ')) return 'https://' + raw;
            const q = encodeURIComponent(raw);
            const engObj = currentEngineObj(engine);
            return fillTpl(engObj && engObj.url, q);
        }

        function isLightHex(hex) {
            const m = /^#?([0-9a-f]{6})$/i.exec(String(hex || '').trim());
            if (!m) return false;
            const n = parseInt(m[1], 16);
            const r = (n >> 16) & 255;
            const g = (n >> 8) & 255;
            const b = n & 255;
            return (0.2126 * r + 0.7152 * g + 0.0722 * b) > 140;
        }

        function updateClock() {
            const nt = (styles && styles.newtab) || {};
            const clockWrap = document.getElementById('ntClockWrap');
            const clockEl = document.getElementById('ntClock');
            const dateSub = document.getElementById('ntDateSub');
            const d = new Date();
            const longDate = d.toLocaleDateString(isZh() ? 'zh-CN' : 'en-US', { weekday: 'long', month: 'short', day: 'numeric' });

            if (nt.show_clock === false) {
                clockWrap.style.display = 'none';
                return;
            }
            clockWrap.style.display = 'flex';

            let timeStr;
            if (nt.use_24h_clock === false) {
                let h = d.getHours();
                const ampm = h >= 12 ? 'PM' : 'AM';
                h = h % 12 || 12;
                timeStr = `${h}:${String(d.getMinutes()).padStart(2, '0')} ${ampm}`;
            } else {
                timeStr = `${String(d.getHours()).padStart(2, '0')}:${String(d.getMinutes()).padStart(2, '0')}`;
            }
            clockEl.textContent = timeStr;
            dateSub.textContent = longDate;
        }

        function applySurface() {
            const useCustom = !!(styles.colors && styles.colors.use_custom_colors);
            const src = useCustom ? styles.colors : (activeTheme || {});
            const isDefaultDark = !useCustom && (!config.theme || config.theme === 'dark');
            const accent = src.accent || '#7A88F7';
            const root = document.documentElement;

            if (isDefaultDark) {
                root.removeAttribute('data-theme-kind');
                root.style.setProperty('--nt-bg', '#0B0B0E');
                root.style.setProperty('--nt-surface', '#141418');
                root.style.setProperty('--nt-elevated', '#1B1B21');
                root.style.setProperty('--nt-hover', '#18181D');
                root.style.setProperty('--nt-hairline', 'rgba(255, 255, 255, 0.06)');
                root.style.setProperty('--nt-hairline-hover', 'rgba(255, 255, 255, 0.12)');
                root.style.setProperty('--nt-top-highlight', 'rgba(255, 255, 255, 0.15)');
                root.style.setProperty('--nt-fg', '#F4F4F5');
                root.style.setProperty('--nt-dim', '#71717A');
                root.style.setProperty('--nt-faint', '#52525B');
                root.style.setProperty('--nt-accent', accent);
                root.style.setProperty('--nt-accent-soft', `color-mix(in srgb, ${accent} 16%, transparent)`);
            } else {
                const bg = src.bg || '#0B0B0E';
                const light = isLightHex(bg);
                if (light) {
                    root.setAttribute('data-theme-kind', 'light');
                } else {
                    root.removeAttribute('data-theme-kind');
                }
                const fg = src.fg || '#F4F4F5';
                const surface = src.surface || src.panel || '#141418';
                const elevated = src.elevated || src.hover || '#1B1B21';
                root.style.setProperty('--nt-bg', bg);
                root.style.setProperty('--nt-surface', surface);
                root.style.setProperty('--nt-elevated', elevated);
                root.style.setProperty('--nt-hover', src.hover || elevated);
                root.style.setProperty('--nt-hairline', `color-mix(in srgb, ${fg} ${light ? '9%' : '7%'}, transparent)`);
                root.style.setProperty('--nt-hairline-hover', `color-mix(in srgb, ${fg} ${light ? '18%' : '14%'}, transparent)`);
                root.style.setProperty('--nt-top-highlight', light ? 'rgba(255, 255, 255, 0.9)' : `color-mix(in srgb, ${fg} 15%, transparent)`);
                root.style.setProperty('--nt-fg', fg);
                root.style.setProperty('--nt-dim', src.secondary || '#71717A');
                root.style.setProperty('--nt-faint', src.muted || '#52525B');
                root.style.setProperty('--nt-accent', accent);
                root.style.setProperty('--nt-accent-soft', src.accent_bg || src.accentBg || `color-mix(in srgb, ${accent} 16%, transparent)`);
            }

            const nt = (styles && styles.newtab) || {};
            const bannerEl = document.getElementById('ntCustomBanner');
            const titleEl = document.getElementById('ntTitle');
            const subEl = document.getElementById('ntSubtitle');
            const rawTitle = (nt.title !== undefined && nt.title !== null) ? String(nt.title).trim() : '';
            const rawSub = (nt.subtitle !== undefined && nt.subtitle !== null) ? String(nt.subtitle).trim() : '';
            const hasCustomTitle = rawTitle !== '' && rawTitle !== DEFAULT_NEWTAB_TITLE;
            const hasCustomSub = rawSub !== '' && rawSub !== DEFAULT_NEWTAB_SUBTITLE;

            if (hasCustomTitle || hasCustomSub) {
                bannerEl.style.display = 'flex';
                titleEl.textContent = hasCustomTitle ? rawTitle : '';
                titleEl.style.display = hasCustomTitle ? 'block' : 'none';
                subEl.textContent = hasCustomSub ? rawSub : '';
                subEl.style.display = hasCustomSub ? 'block' : 'none';
            } else {
                bannerEl.style.display = 'none';
            }

            const scSec = document.getElementById('shortcutsSection');
            scSec.style.display = nt.show_shortcuts === false ? 'none' : 'flex';

            const grid = document.getElementById('bmGrid');
            const cols = Number(nt.shortcuts_columns) || 0;
            grid.style.gridTemplateColumns = (cols >= 2 && cols <= 6) ? `repeat(${cols}, minmax(0, 1fr))` : '';

            const customCssEl = document.getElementById('__fynexo_newtab_custom_css__');
            if (customCssEl) customCssEl.textContent = nt.custom_css || '';

            const rawBg = String(nt.background_image || '').trim();
            if (rawBg) {
                const isCssFn = /^(?:url|linear-gradient|radial-gradient|conic-gradient|repeating-linear-gradient|repeating-radial-gradient)\s*\(/i.test(rawBg);
                const safeCssBg = isCssFn
                    ? rawBg
                    : (/^(?:https?:\/\/|data:image\/)/i.test(rawBg) ? `url("${rawBg.replace(/"/g, '\\"')}")` : '');
                if (safeCssBg) {
                    const dimPct = nt.background_dim !== undefined ? Number(nt.background_dim) : 45;
                    const dimVal = Math.max(0, Math.min(90, Number.isFinite(dimPct) ? dimPct : 45)) / 100;
                    const blurVal = Math.max(0, Math.min(24, Number(nt.background_blur) || 0));
                    root.style.setProperty('--nt-wallpaper-img', safeCssBg);
                    root.style.setProperty('--nt-wallpaper-dim', String(dimVal));
                    root.style.setProperty('--nt-wallpaper-blur', `${blurVal}px`);
                    root.classList.add('has-wallpaper');
                } else {
                    root.classList.remove('has-wallpaper');
                }
            } else {
                root.classList.remove('has-wallpaper');
            }

            const zh = isZh();
            const engObj = currentEngineObj();
            const engName = (engObj && engObj.name) || 'Google';
            const engBtn = document.getElementById('ntEngineBtn');
            if (engBtn) {
                engBtn.textContent = '';
                engBtn.appendChild(createSiteEmblem(engObj && engObj.url, engName));
                const chev = createSvgEl('svg', { width: 10, height: 10, viewBox: '0 0 16 16', fill: 'none', class: 'nt-engine-chevron' });
                chev.appendChild(createSvgEl('path', { d: 'M4.5 6.5L8 10l3.5-3.5', stroke: 'currentColor', 'stroke-width': 1.6, 'stroke-linecap': 'round', 'stroke-linejoin': 'round' }));
                engBtn.appendChild(chev);
                engBtn.title = zh ? `当前搜索引擎：${engName}（点击切换）` : `Search engine: ${engName} (click to choose)`;
                engBtn.setAttribute('aria-label', engBtn.title);
                if (typeof engineMenuEl !== 'undefined' && engineMenuEl && engineMenuEl.classList.contains('open')) {
                    renderEngineMenu();
                }
            }
            document.getElementById('ntSearchInput').placeholder = zh
                ? `在 ${engName} 中搜索，点下方 !快捷指令，或直接输入网址...`
                : `Search ${engName}, !gh, or enter URL...`;

            const adOn = config.adblock !== false;
            document.getElementById('ntShieldDot').classList.toggle('off', !adOn);
            document.getElementById('ntShieldText').textContent = zh
                ? (adOn ? (blockedAds > 0 ? `已拦截 · ${blockedAds}` : '广告拦截已开启') : '广告拦截已暂停')
                : (adOn ? (blockedAds > 0 ? `Adblock · ${blockedAds}` : 'Adblock active') : 'Adblock paused');

            const incogBtn = document.getElementById('btnIncogPill');
            const incogTxt = document.getElementById('ntIncogText');
            incogBtn.classList.toggle('active', !!config.incognito);
            incogTxt.textContent = zh
                ? (config.incognito ? '无痕: 开启' : '无痕: 关闭')
                : (config.incognito ? 'Private: Active' : 'Private: Off');

            const langBtn = document.getElementById('btnLangPill');
            if (langBtn) {
                langBtn.textContent = zh ? '中文' : 'EN';
                langBtn.title = zh ? 'Switch interface to English' : '切换界面为简体中文';
            }
            const prefsBtn = document.getElementById('btnPrefsPill');
            if (prefsBtn) {
                prefsBtn.title = zh ? '打开浏览器设置' : 'Open browser preferences';
            }
            const scLbl = document.getElementById('ntShortcutsLabel');
            if (scLbl) scLbl.textContent = zh ? '常用网站' : 'Shortcuts';
            const btnToggleAdd = document.getElementById('btnToggleAddBm');
            if (btnToggleAdd) btnToggleAdd.textContent = zh ? '+ 添加网站' : '+ Add';
            const inpAddTitle = document.getElementById('addBmTitle');
            if (inpAddTitle) inpAddTitle.placeholder = zh ? '名称（可选）' : 'Name (optional)';
            const inpAddUrl = document.getElementById('addBmUrl');
            if (inpAddUrl) inpAddUrl.placeholder = zh ? '网址（例如 github.com）' : 'URL (e.g. github.com)';
            const btnSaveBmEl = document.getElementById('btnSaveBm');
            if (btnSaveBmEl) btnSaveBmEl.textContent = zh ? '保存' : 'Save';
            const histLbl = document.getElementById('ntHistLabel');
            if (histLbl) histLbl.textContent = zh ? '最近访问' : 'Recent';
            const btnClearH = document.getElementById('btnClearHist');
            if (btnClearH) btnClearH.textContent = zh ? '清空' : 'Clear';

            const bangBar = document.getElementById('ntBangBar');
            if (bangBar) {
                bangBar.textContent = '';
                const quickBangs = zh
                    ? [
                        ['!b', '!b 搜B站'],
                        ['!gh', '!gh GitHub'],
                        ['!yt', '!yt YouTube'],
                        ['!tr', '!tr 翻译'],
                        ['!w', '!w 维基百科']
                    ]
                    : [
                        ['!b', '!b Bilibili'],
                        ['!gh', '!gh GitHub'],
                        ['!yt', '!yt YouTube'],
                        ['!tr', '!tr Translate'],
                        ['!w', '!w Wikipedia']
                    ];
                quickBangs.forEach(([trig, label]) => {
                    const chip = document.createElement('button');
                    chip.type = 'button';
                    chip.className = 'nt-bang-chip';
                    chip.textContent = label;
                    chip.onclick = () => {
                        const inp = document.getElementById('ntSearchInput');
                        if (!inp) return;
                        inp.value = `${trig} `;
                        if (typeof renderSuggestions === 'function') renderSuggestions();
                        inp.focus();
                    };
                    bangBar.appendChild(chip);
                });
            }

            updateClock();
        }

        function renderBookmarks() {
            const grid = document.getElementById('bmGrid');
            grid.textContent = '';
            const frag = document.createDocumentFragment();

            if (bookmarks.length === 0) {
                const emptyBtn = document.createElement('button');
                emptyBtn.type = 'button';
                emptyBtn.className = 'bm-card bm-card--empty';
                emptyBtn.textContent = isZh() ? '+ 添加常用网站' : '+ Add a shortcut';
                emptyBtn.onclick = () => {
                    document.getElementById('addBmBar').classList.add('open');
                    document.getElementById('addBmUrl').focus();
                };
                frag.appendChild(emptyBtn);
                grid.appendChild(frag);
                return;
            }

            bookmarks.forEach(bm => {
                const card = document.createElement('div');
                card.className = 'bm-card';
                card.title = bm.url || '';
                card.setAttribute('role', 'button');
                card.tabIndex = 0;
                card.onclick = () => sendIpc({ type: 'NAVIGATE', url: bm.url });
                card.onkeydown = (e) => {
                    if (e.key === 'Enter' || e.key === ' ') {
                        e.preventDefault();
                        sendIpc({ type: 'NAVIGATE', url: bm.url });
                    }
                };

                card.appendChild(createSiteEmblem(bm.url, bm.title));

                const t = document.createElement('span');
                t.className = 'bm-title';
                t.textContent = bm.title || formatHost(bm.url);
                card.appendChild(t);

                const del = document.createElement('button');
                del.type = 'button';
                del.className = 'bm-del';
                del.title = 'Remove shortcut';
                del.setAttribute('aria-label', 'Remove shortcut');
                del.textContent = '×';
                del.onclick = (e) => {
                    e.stopPropagation();
                    sendIpc({ type: 'REMOVE_BOOKMARK', url: bm.url });
                };
                card.appendChild(del);

                frag.appendChild(card);
            });

            grid.appendChild(frag);
        }

        function canonicalHistoryUrl(rawUrl) {
            const s = String(rawUrl || '').trim();
            if (!s) return '';
            try {
                const u = new URL(s);
                const host = u.hostname.toLowerCase().replace(/^www\./, '');
                const path = (u.pathname === '/' ? '' : u.pathname.replace(/\/+$/, ''));
                const dropKeys = new Set(['t', 'pp', 'si', 'feature', 'fbclid', 'gclid']);
                const params = new URLSearchParams();
                for (const [k, v] of u.searchParams.entries()) {
                    const lk = k.toLowerCase();
                    if (!dropKeys.has(lk) && !lk.startsWith('utm_')) {
                        params.append(k, v);
                    }
                }
                const qs = params.toString();
                return `${u.protocol.toLowerCase()}//${host}${u.port ? ':' + u.port : ''}${path}${qs ? '?' + qs : ''}`;
            } catch (_) {
                return s.toLowerCase().replace(/\/+$/, '');
            }
        }

        function cleanHistoryTitle(rawTitle, rawUrl) {
            const stripped = String(rawTitle || '').trim().replace(/^\(\d+\+?\)\s*/, '').trim();
            return stripped || formatHost(rawUrl);
        }

        function dedupeHistoryItems(items) {
            const seenUrls = new Set();
            const seenTitles = new Set();
            const out = [];
            for (let i = 0; i < items.length; i++) {
                const item = items[i];
                if (!item || !item.url) continue;
                const urlKey = canonicalHistoryUrl(item.url);
                if (!urlKey || seenUrls.has(urlKey)) continue;
                const cleanTitle = cleanHistoryTitle(item.title, item.url);
                const titleKey = cleanTitle ? `${formatHost(item.url).toLowerCase()}\0${cleanTitle}` : '';
                if (titleKey && seenTitles.has(titleKey)) continue;
                seenUrls.add(urlKey);
                if (titleKey) seenTitles.add(titleKey);
                out.push(item);
            }
            return out;
        }

        let recentTopItems = [];

        function renderHistory() {
            const nt = (styles && styles.newtab) || {};
            const sec = document.getElementById('histSection');
            const list = document.getElementById('histList');
            list.textContent = '';

            if (nt.show_history === false) {
                recentTopItems = [];
                sec.style.display = 'none';
                return;
            }

            recentTopItems = dedupeHistoryItems(historyItems).slice(0, 8);
            if (recentTopItems.length === 0) {
                sec.style.display = 'none';
                return;
            }

            sec.style.display = 'flex';
            const frag = document.createDocumentFragment();
            recentTopItems.forEach((item, idx) => {
                const displayTitle = cleanHistoryTitle(item.title, item.url);
                const row = document.createElement('button');
                row.type = 'button';
                row.className = 'hist-row';
                row.title = `Open ${item.url || ''} (Alt+${idx + 1})`;
                row.onclick = () => sendIpc({ type: 'NAVIGATE', url: item.url });

                const left = document.createElement('span');
                left.className = 'hist-left';
                left.appendChild(createSiteEmblem(item.url, displayTitle));

                const title = document.createElement('span');
                title.className = 'hist-title';
                title.textContent = displayTitle;

                const host = document.createElement('span');
                host.className = 'hist-host';
                host.textContent = formatHost(item.url);

                left.appendChild(title);
                left.appendChild(host);

                const right = document.createElement('span');
                right.className = 'hist-right';

                const time = document.createElement('span');
                time.className = 'hist-time';
                time.textContent = relativeTime(item.visited_at);

                const kbd = document.createElement('span');
                kbd.className = 'hist-kbd';
                kbd.textContent = `Alt+${idx + 1}`;

                right.appendChild(time);
                right.appendChild(kbd);

                row.appendChild(left);
                row.appendChild(right);
                frag.appendChild(row);
            });
            list.appendChild(frag);
        }

        const searchBox = document.getElementById('ntSearchBox');
        const searchInput = document.getElementById('ntSearchInput');
        const clearBtn = document.getElementById('ntClearBtn');
        const suggestEl = document.getElementById('ntSuggestList');
        let ntSuggestItems = [];
        let ntSuggestIdx = -1;

        function renderSuggestions() {
            const rawTrim = searchInput.value.trim();
            const q = rawTrim.toLowerCase();
            const hasVal = searchInput.value.length > 0;
            clearBtn.style.display = hasVal ? 'flex' : 'none';
            if (searchBox) searchBox.classList.toggle('has-value', hasVal);
            suggestEl.textContent = '';

            if (!q) {
                ntSuggestItems = [];
                ntSuggestIdx = -1;
                suggestEl.style.display = 'none';
                return;
            }

            const matches = [];
            const seen = Object.create(null);

            if (q.startsWith('!')) {
                const tok = q.split(' ')[0];
                getBangs().forEach(b => {
                    if (!b || !b.trigger) return;
                    const trig = (b.trigger.startsWith('!') ? b.trigger : '!' + b.trigger).toLowerCase();
                    if (trig.startsWith(tok) || (b.name || '').toLowerCase().includes(tok.slice(1))) {
                        matches.push({ kind: 'bang', trigger: trig, title: `${trig} — ${b.name || trig}`, url: b.url || '', tag: '!Bang' });
                    }
                });
            }

            bookmarks.forEach(b => {
                const key = (b.url || '').toLowerCase();
                if (!seen[key] && ((b.title || '').toLowerCase().includes(q) || key.includes(q))) {
                    seen[key] = true;
                    matches.push({ kind: 'url', title: b.title || formatHost(b.url), url: b.url, tag: 'Shortcut' });
                }
            });

            dedupeHistoryItems(historyItems).forEach(h => {
                const key = canonicalHistoryUrl(h.url);
                const cleanTitle = cleanHistoryTitle(h.title, h.url);
                if (!seen[key] && (cleanTitle.toLowerCase().includes(q) || key.includes(q))) {
                    seen[key] = true;
                    matches.push({ kind: 'url', title: cleanTitle, url: h.url, tag: 'Recent' });
                }
            });

            ntSuggestItems = matches.slice(0, 6);
            ntSuggestIdx = -1;

            if (ntSuggestItems.length === 0) {
                suggestEl.style.display = 'none';
                return;
            }

            suggestEl.style.display = 'flex';
            const frag = document.createDocumentFragment();
            ntSuggestItems.forEach((it, idx) => {
                const btn = document.createElement('button');
                btn.type = 'button';
                btn.className = 'nt-suggest-item';
                btn.dataset.idx = String(idx);
                btn.onclick = () => {
                    if (it.kind === 'bang' && !searchInput.value.trim().includes(' ')) {
                        searchInput.value = it.trigger + ' ';
                        searchInput.focus();
                        renderSuggestions();
                        return;
                    }
                    const target = it.kind === 'bang' ? resolveQuery(searchInput.value, config.search_engine) : it.url;
                    sendIpc({ type: 'NAVIGATE', url: target });
                };

                if (it.kind === 'bang') {
                    const mono = document.createElement('span');
                    mono.className = 'nt-emblem';
                    const fb = document.createElement('span');
                    fb.className = 'nt-emblem-fallback nt-emblem-fallback--mono';
                    fb.textContent = '!';
                    fb.style.color = 'var(--nt-accent)';
                    mono.appendChild(fb);
                    btn.appendChild(mono);
                } else {
                    btn.appendChild(createSiteEmblem(it.url, it.title));
                }

                const meta = document.createElement('span');
                meta.className = 'nt-suggest-meta';
                const t = document.createElement('span');
                t.className = 'nt-suggest-title';
                t.textContent = it.title;
                const u = document.createElement('span');
                u.className = 'nt-suggest-url';
                u.textContent = it.kind === 'bang' ? it.url : formatHost(it.url);
                meta.appendChild(t);
                meta.appendChild(u);

                const tag = document.createElement('span');
                tag.className = 'nt-suggest-tag';
                tag.textContent = it.tag;

                btn.appendChild(meta);
                btn.appendChild(tag);
                frag.appendChild(btn);
            });
            suggestEl.appendChild(frag);
        }

        function syncSuggestSelection() {
            suggestEl.querySelectorAll('.nt-suggest-item').forEach((el, i) => {
                el.classList.toggle('selected', i === ntSuggestIdx);
            });
        }

        function submitHeroSearch() {
            if (ntSuggestIdx >= 0 && ntSuggestItems[ntSuggestIdx]) {
                const it = ntSuggestItems[ntSuggestIdx];
                if (it.kind === 'bang' && !searchInput.value.trim().includes(' ')) {
                    searchInput.value = it.trigger + ' ';
                    renderSuggestions();
                    return;
                }
                const target = it.kind === 'bang' ? resolveQuery(searchInput.value, config.search_engine) : it.url;
                sendIpc({ type: 'NAVIGATE', url: target });
                return;
            }
            const target = resolveQuery(searchInput.value, config.search_engine);
            if (target) {
                sendIpc({ type: 'NAVIGATE', url: target });
            }
        }

        const engineBtnEl = document.getElementById('ntEngineBtn');
        const engineMenuEl = document.getElementById('ntEngineMenu');

        function closeEngineMenu() {
            if (!engineMenuEl || !engineBtnEl) return;
            engineMenuEl.classList.remove('open');
            engineBtnEl.setAttribute('aria-expanded', 'false');
        }

        function renderEngineMenu() {
            if (!engineMenuEl) return;
            engineMenuEl.textContent = '';
            const list = getEngines();
            const cur = currentEngineObj();
            for (const eng of list) {
                if (!eng || !eng.id) continue;
                const active = cur && eng.id === cur.id;
                const opt = document.createElement('button');
                opt.type = 'button';
                opt.className = 'nt-engine-opt' + (active ? ' active' : '');
                opt.setAttribute('role', 'option');
                opt.setAttribute('aria-selected', active ? 'true' : 'false');
                opt.appendChild(createSiteEmblem(eng.url, eng.name || eng.id));
                const nameSpan = document.createElement('span');
                nameSpan.className = 'nt-engine-opt-name';
                nameSpan.textContent = eng.name || eng.id;
                opt.appendChild(nameSpan);
                const chk = createSvgEl('svg', { width: 12, height: 12, viewBox: '0 0 16 16', fill: 'none', class: 'nt-engine-check' });
                chk.appendChild(createSvgEl('path', { d: 'M3.5 8.5l3 3 6-6.5', stroke: 'currentColor', 'stroke-width': 1.7, 'stroke-linecap': 'round', 'stroke-linejoin': 'round' }));
                opt.appendChild(chk);
                opt.onclick = (ev) => {
                    ev.stopPropagation();
                    config.search_engine = eng.id;
                    closeEngineMenu();
                    applySurface();
                    sendIpc({ type: 'SAVE_CONFIG', config: config });
                    searchInput.focus();
                };
                engineMenuEl.appendChild(opt);
            }
        }

        function toggleEngineMenu() {
            if (!engineMenuEl || !engineBtnEl) return;
            const willOpen = !engineMenuEl.classList.contains('open');
            if (willOpen) {
                suggestEl.style.display = 'none';
                ntSuggestIdx = -1;
                renderEngineMenu();
                engineMenuEl.classList.add('open');
                engineBtnEl.setAttribute('aria-expanded', 'true');
            } else {
                closeEngineMenu();
            }
        }

        searchInput.addEventListener('focus', closeEngineMenu);
        searchInput.addEventListener('input', () => {
            closeEngineMenu();
            renderSuggestions();
        });
        searchInput.addEventListener('keydown', (e) => {
            if (e.key === 'ArrowDown' && ntSuggestItems.length > 0) {
                e.preventDefault();
                ntSuggestIdx = (ntSuggestIdx + 1) % ntSuggestItems.length;
                syncSuggestSelection();
                return;
            }
            if (e.key === 'ArrowUp' && ntSuggestItems.length > 0) {
                e.preventDefault();
                ntSuggestIdx = (ntSuggestIdx - 1 + ntSuggestItems.length) % ntSuggestItems.length;
                syncSuggestSelection();
                return;
            }
            if (e.key === 'Escape') {
                closeEngineMenu();
                suggestEl.style.display = 'none';
                ntSuggestIdx = -1;
                searchInput.blur();
                return;
            }
            if (e.key === 'Enter') {
                e.preventDefault();
                submitHeroSearch();
            }
        });

        document.addEventListener('keydown', (e) => {
            if (e.key === 'Escape' && engineMenuEl && engineMenuEl.classList.contains('open')) {
                closeEngineMenu();
                return;
            }
            if (e.key === 'Alt' || e.altKey) {
                document.documentElement.classList.add('nt-alt-held');
            }
            const active = document.activeElement;
            const inInput = active && (active.tagName === 'INPUT' || active.tagName === 'TEXTAREA');

            if (e.key === '/' && !inInput && !e.ctrlKey && !e.metaKey && !e.altKey) {
                e.preventDefault();
                searchInput.focus();
                searchInput.select();
                return;
            }

            if ((e.metaKey || e.altKey) && !e.ctrlKey && !e.shiftKey) {
                const n = Number(e.key);
                if (Number.isInteger(n) && n >= 1 && n <= recentTopItems.length) {
                    const target = recentTopItems[n - 1];
                    if (target && target.url) {
                        e.preventDefault();
                        sendIpc({ type: 'NAVIGATE', url: target.url });
                    }
                }
            }
        });

        document.addEventListener('keyup', (e) => {
            if (e.key === 'Alt' || !e.altKey) {
                document.documentElement.classList.remove('nt-alt-held');
            }
        });

        window.addEventListener('blur', () => {
            document.documentElement.classList.remove('nt-alt-held');
            closeEngineMenu();
        });

        document.addEventListener('click', (e) => {
            if (!e.target.closest || !e.target.closest('.nt-search-wrap')) {
                suggestEl.style.display = 'none';
                closeEngineMenu();
            } else if (!e.target.closest('#ntEngineBtn') && !e.target.closest('#ntEngineMenu')) {
                closeEngineMenu();
            }
        });

        clearBtn.onclick = () => {
            searchInput.value = '';
            renderSuggestions();
            searchInput.focus();
        };

        document.getElementById('ntGoBtn').onclick = submitHeroSearch;

        engineBtnEl.onclick = (e) => {
            e.stopPropagation();
            toggleEngineMenu();
        };

        document.getElementById('btnIncogPill').onclick = () => {
            config.incognito = !config.incognito;
            applySurface();
            sendIpc({ type: 'SAVE_CONFIG', config: config });
        };

        document.getElementById('btnShieldPill').onclick = () => {
            config.adblock = config.adblock === false;
            applySurface();
            sendIpc({ type: 'SAVE_CONFIG', config: config });
        };

        const btnLangPill = document.getElementById('btnLangPill');
        if (btnLangPill) {
            btnLangPill.onclick = () => {
                config.language = isZh() ? 'en' : 'zh';
                applySurface();
                renderBookmarks();
                renderHistory();
                sendIpc({ type: 'PATCH_HUD_PREFS', language: config.language });
            };
        }
        const btnPrefsPill = document.getElementById('btnPrefsPill');
        if (btnPrefsPill) {
            btnPrefsPill.onclick = () => sendIpc({ type: 'NAVIGATE', url: 'fynexo://settings' });
        }

        const addBar = document.getElementById('addBmBar');
        const addTitle = document.getElementById('addBmTitle');
        const addUrl = document.getElementById('addBmUrl');
        document.getElementById('btnToggleAddBm').onclick = () => {
            const open = addBar.classList.toggle('open');
            if (open) addUrl.focus();
        };

        function submitNewBookmark() {
            const rawUrl = addUrl.value.trim();
            if (!rawUrl) return;
            sendIpc({
                type: 'ADD_BOOKMARK',
                title: addTitle.value.trim(),
                url: rawUrl
            });
            addTitle.value = '';
            addUrl.value = '';
            addBar.classList.remove('open');
        }

        document.getElementById('btnSaveBm').onclick = submitNewBookmark;
        addUrl.addEventListener('keydown', (e) => {
            if (e.key === 'Enter') {
                e.preventDefault();
                submitNewBookmark();
            }
        });

        document.getElementById('btnClearHist').onclick = () => {
            sendIpc({ type: 'CLEAR_HISTORY' });
        };

        setInterval(updateClock, 15000);

        window.__fynexo_on_settings_runtime__ = function(payload) {
            if (!payload) return;
            if (payload.config) config = Object.assign(config, payload.config);
            if (payload.styles) styles = payload.styles;
            if (payload.active_theme) activeTheme = payload.active_theme;
            if (Array.isArray(payload.bookmarks)) bookmarks = payload.bookmarks;
            if (Array.isArray(payload.history)) historyItems = payload.history;
            if (typeof payload.blocked_ads === 'number') blockedAds = payload.blocked_ads;
            applySurface();
            renderBookmarks();
            renderHistory();
        };

        applySurface();
        renderBookmarks();
        renderHistory();
        if (window.innerWidth > 768) {
            requestAnimationFrame(() => searchInput.focus());
        }
