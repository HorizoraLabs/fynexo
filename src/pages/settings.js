        __FNX_DEFAULTS__;
        const KEYBIND_DEFS = [
            ['omnibox', 'Open omnibox & quick bar', 'Focus address bar from anywhere', '打开地址栏与快速搜索框', '在任意页面随时输入网址或搜索'],
            ['new_tab', 'Open new tab', 'Launch a fresh Start Page tab', '新建标签页', '打开一个全新的起始页'],
            ['close_tab', 'Close active tab', 'Close current tab or return home', '关闭当前标签页', '关闭正在浏览的网页'],
            ['reopen_closed_tab', 'Reopen closed tab', 'Restore the most recently closed tab', '恢复刚关闭的标签页', '不小心关错网页时一键找回'],
            ['next_tab', 'Cycle open tabs', 'Switch sequentially to the next tab', '切换到下一个标签页', '顺时针切换已打开的标签页'],
            ['prev_tab', 'Previous open tab', 'Switch sequentially to the previous tab', '切换到上一个标签页', '逆时针切换已打开的标签页'],
            ['reload', 'Reload current page', 'Refresh the active tab', '刷新当前网页', '重新加载正在看的网页'],
            ['go_back', 'Navigate back', 'Step back in tab browsing history', '后退上一页', '返回刚才浏览的页面'],
            ['go_forward', 'Navigate forward', 'Step forward in tab browsing history', '前进下一页', '前往之前回退的页面'],
            ['toggle_side_panel', 'Toggle Side Panel', 'Open or close split web & notes drawer', '开关分屏侧边栏', '呼出或收起右侧小网页与便签'],
            ['pin_tab', 'Pin / unpin active tab', 'Toggle compact pinned state on active tab', '固定 / 取消固定当前标签页', '固定常逛网站并防止误点关闭'],
            ['mute_tab', 'Mute / unmute active tab', 'Silence or restore audio on active tab', '静音 / 取消静音当前标签页', '一键关闭或恢复网页里的声音'],
            ['bookmark_page', 'Save / remove shortcut', 'Toggle active site in shortcuts.json', '收藏 / 取消收藏当前网站', '快速保存到常用网站栏'],
            ['find_in_page', 'Find in page', 'Open HUD page text search', '页内查找文字', '在当前网页里搜索关键词'],
            ['reader_mode', 'Toggle Reader mode', 'Extract clean distraction-free article', '开关纯净阅读模式', '过滤杂乱广告，专注阅读长文章'],
            ['force_dark', 'Toggle Smart Force Dark', 'Invert bright sites while keeping media', '开关智能夜间深色', '把刺眼的白底网页转成护眼深色'],
            ['fullscreen', 'Immersive fullscreen', 'Toggle borderless window fullscreen', '进入 / 退出全屏浏览', '隐藏系统边框沉浸浏览'],
            ['preferences', 'Toggle Preferences', 'Jump to or leave Preferences', '打开偏好设置', '一键进入浏览器设置界面'],
            ['zoom_in', 'Zoom in', 'Increase page scale by 10%', '放大网页文字与画面', '整体缩放比例 +10%'],
            ['zoom_out', 'Zoom out', 'Decrease page scale by 10%', '缩小网页文字与画面', '整体缩放比例 -10%'],
            ['zoom_reset', 'Reset zoom (100%)', 'Restore default 100% scale', '恢复默认缩放 (100%)', '重置回原始网页大小']
        ];
        const COLOR_TOKEN_DEFS = [
            ['bg', 'Canvas bg', '全局底色'],
            ['panel', 'Topbar / panel', '顶栏与面板背景'],
            ['surface', 'Input surface', '输入框底色'],
            ['elevated', 'Elevated row', '悬停与高亮卡片'],
            ['border', 'Border', '常规边框线'],
            ['border_hover', 'Border hover', '悬停边框线'],
            ['fg', 'Primary text', '主要文字颜色'],
            ['secondary', 'Secondary text', '次要文字颜色'],
            ['muted', 'Muted text', '弱化提示文字'],
            ['accent', 'Accent color', '主题强调色'],
            ['accent_bg', 'Accent tint', '强调浅底色'],
            ['hover', 'Hover state', '按钮悬停色']
        ];

        // =========================================================================
        // Supabase Cloud Extensions (Injected via build.rs / .env.local, never in Git)
        // =========================================================================
        const SUPABASE_CONFIG = __FNX_SUPABASE_JSON__ || {};
        const SUPABASE_STORAGE_KEY = '__fnx_supabase_ext_cfg__';

        const boot = __FNX_BOOT_JSON__ || {};
        let config = Object.assign({
            theme: 'dark',
            enforce_perf: true,
            adblock: true,
            restore_session: true,
            force_dark: false,
            incognito: false,
            language: 'zh',
            default_zoom: 1.0,
            user_agent_mode: 'default',
            custom_user_agent: '',
            tab_cache_mode: 'balanced',
            ext_side_panel: false,
            ext_reader_mode: false,
            ext_yt_ad_skipper: false,
            ext_gh_harmonizer: false,
            ext_media_booster: false,
            ext_userscript_engine: true,
            ext_mouse_gestures: true,
            site_overrides: {},
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
        let themes = Array.isArray(boot.themes) ? boot.themes : [];
        let bookmarks = Array.isArray(boot.bookmarks) ? boot.bookmarks : [];
        let siteRules = Array.isArray(boot.site_rules) ? boot.site_rules : [];
        let adblockCfg = Object.assign({ custom_blocked_domains: [], whitelisted_domains: [] }, boot.adblock || {});
        let keybinds = Object.assign({}, DEFAULT_KEYBINDS, boot.keybinds || {});

        let themesDir = boot.themes_dir || 'themes/';
        let stylesPath = boot.styles_path || 'config/styles.json';
        let dataDir = boot.data_dir || '';
        function resolveTotalBlockedAds(rustCount) {
            const androidBlocked =
                window.__fynexo_android__ && typeof window.__fynexo_android__.getSessionBlockedAds === 'function'
                    ? (Number(window.__fynexo_android__.getSessionBlockedAds()) || 0)
                    : 0;
            return (Number(rustCount) || 0) + androidBlocked;
        }
        let rustBlockedAds = Number(boot.blocked_ads) || 0;
        let blockedAds = resolveTotalBlockedAds(rustBlockedAds);
        let historyCount = Array.isArray(boot.history) ? boot.history.length : 0;

        let editingRuleId = null;
        let recordingActionKey = null;

        function getEngines() {
            return (Array.isArray(config.search_engines) && config.search_engines.length > 0)
                ? config.search_engines
                : DEFAULT_ENGINES;
        }
        function getBangs() {
            return Array.isArray(config.bangs) ? config.bangs : DEFAULT_BANGS;
        }

        function sendIpc(data) {
            const raw = JSON.stringify(data);
            if (window.ipc && typeof window.ipc.postMessage === 'function') {
                window.ipc.postMessage(raw);
            } else if (window.__TAURI_INTERNALS__ && window.__TAURI_INTERNALS__.ipc) {
                window.__TAURI_INTERNALS__.ipc.postMessage(raw);
            }
        }

        let toastTimer = null;
        function showToast(text) {
            const t = document.getElementById('toast');
            const m = document.getElementById('toastMsg');
            m.textContent = text;
            t.style.display = 'inline-flex';
            if (toastTimer) clearTimeout(toastTimer);
            toastTimer = setTimeout(() => { t.style.display = 'none'; }, 1800);
        }

        function setSwitchState(btnId, on, labelId, onText, offText) {
            const btn = document.getElementById(btnId);
            if (btn) {
                btn.classList.toggle('on', !!on);
                btn.setAttribute('aria-checked', on ? 'true' : 'false');
            }
            if (labelId) {
                const lbl = document.getElementById(labelId);
                if (lbl) lbl.textContent = on ? onText : offText;
            }
        }

        function createSvgEl(tag, attrs) {
            const el = document.createElementNS('http://www.w3.org/2000/svg', tag);
            for (const k in attrs) el.setAttribute(k, String(attrs[k]));
            return el;
        }

        function createSiteEmblem(url) {
            const wrap = document.createElement('span');
            wrap.className = 'rc-emblem';
            const fallback = document.createElement('span');
            fallback.className = 'rc-emblem-fallback';
            const gSvg = createSvgEl('svg', { width: 13, height: 13, viewBox: '0 0 16 16', fill: 'none' });
            gSvg.appendChild(createSvgEl('circle', { cx: 8, cy: 8, r: 5.8, stroke: 'currentColor', 'stroke-width': 1.3 }));
            gSvg.appendChild(createSvgEl('path', { d: 'M2.6 8h10.8M8 2.2c1.8 1.6 2.6 3.7 2.6 5.8s-.8 4.2-2.6 5.8M8 2.2C6.2 3.8 5.4 5.9 5.4 8s.8 4.2 2.6 5.8', stroke: 'currentColor', 'stroke-width': 1.1 }));
            fallback.appendChild(gSvg);
            wrap.appendChild(fallback);

            try {
                const raw = String(url || '').replace('{query}', 'test');
                const parsed = new URL(/^https?:\/\//i.test(raw) ? raw : ('https://' + raw));
                if (/^https?:$/.test(parsed.protocol) && parsed.hostname) {
                    const img = document.createElement('img');
                    img.className = 'rc-emblem-img';
                    img.alt = '';
                    img.loading = 'lazy';
                    img.decoding = 'async';
                    img.style.display = 'none';
                    img.src = 'https://www.google.com/s2/favicons?sz=32&domain=' + encodeURIComponent(parsed.hostname);
                    img.onload = () => {
                        if (img.naturalWidth > 0) {
                            fallback.style.display = 'none';
                            img.style.display = 'block';
                        }
                    };
                    img.onerror = () => { img.remove(); };
                    wrap.appendChild(img);
                }
            } catch (_) {}
            return wrap;
        }

        function toSixDigitHex(raw) {
            const s = String(raw || '').trim();
            if (/^#[0-9a-fA-F]{6}$/.test(s)) return s;
            if (/^#[0-9a-fA-F]{3}$/.test(s)) {
                return '#' + s[1] + s[1] + s[2] + s[2] + s[3] + s[3];
            }
            return '#0B0B0E';
        }

        function isLightHex(hex) {
            const m = /^#?([0-9a-f]{6})$/i.exec(String(hex || '').trim());
            if (!m) return false;
            const n = parseInt(m[1], 16);
            return (0.2126 * ((n >> 16) & 255) + 0.7152 * ((n >> 8) & 255) + 0.0722 * (n & 255)) > 140;
        }

        function resolveThemePalette() {
            const useCustom = !!(styles.colors && styles.colors.use_custom_colors);
            const matched = themes.find(item => item && item.id === config.theme);
            const src = useCustom ? (styles.colors || {}) : ((matched && matched.colors) ? matched.colors : (activeTheme || {}));
            const isDefaultDark = !useCustom && (!config.theme || config.theme === 'dark');

            if (isDefaultDark) {
                return {
                    isDefaultDark: true,
                    bg: '#0B0B0E',
                    panel: '#0E0E12',
                    surface: '#141418',
                    elevated: '#1B1B21',
                    border: 'rgba(255, 255, 255, 0.08)',
                    borderHover: 'rgba(255, 255, 255, 0.14)',
                    fg: '#F4F4F5',
                    secondary: '#A1A1AA',
                    muted: '#71717A',
                    accent: src.accent || '#7A88F7',
                    accentBg: '#1B1D36',
                    hover: '#18181D'
                };
            }
            return {
                isDefaultDark: false,
                bg: src.bg || '#0B0B0E',
                panel: src.panel || '#0E0E12',
                surface: src.surface || src.panel || '#141418',
                elevated: src.elevated || '#1B1B21',
                border: src.border || 'rgba(255, 255, 255, 0.08)',
                borderHover: src.border_hover || src.borderHover || 'rgba(255, 255, 255, 0.14)',
                fg: src.fg || '#F4F4F5',
                secondary: src.secondary || '#A1A1AA',
                muted: src.muted || '#71717A',
                accent: src.accent || '#7A88F7',
                accentBg: src.accent_bg || src.accentBg || '#1B1D36',
                hover: src.hover || '#18181D'
            };
        }

        function renderColorTokensStudio() {
            const studio = document.getElementById('paletteStudio');
            const grid = document.getElementById('colorTokensGrid');
            const useCustom = !!(styles.colors && styles.colors.use_custom_colors);
            studio.classList.toggle('open', useCustom);
            if (!useCustom) return;

            grid.textContent = '';
            styles.colors = styles.colors || {};
            const frag = document.createDocumentFragment();

            COLOR_TOKEN_DEFS.forEach(([key, enLabel, zhLabel]) => {
                const label = isZh() ? (zhLabel || enLabel) : enLabel;
                const curVal = styles.colors[key] || '#0B0B0E';
                const box = document.createElement('div');
                box.className = 'color-swatch-box';

                const picker = document.createElement('input');
                picker.type = 'color';
                picker.className = 'color-picker-native';
                picker.value = toSixDigitHex(curVal);

                const meta = document.createElement('div');
                meta.className = 'color-token-meta';
                const lbl = document.createElement('span');
                lbl.className = 'color-token-lbl';
                lbl.textContent = label;
                const hexInp = document.createElement('input');
                hexInp.type = 'text';
                hexInp.className = 'color-token-hex';
                hexInp.value = curVal;

                picker.oninput = () => {
                    styles.colors[key] = picker.value;
                    hexInp.value = picker.value;
                    applySettingsSurface();
                };
                picker.onchange = () => { pushStyles(`Updated ${key}`); };
                hexInp.onchange = () => {
                    const v = hexInp.value.trim();
                    if (v) {
                        styles.colors[key] = v;
                        picker.value = toSixDigitHex(v);
                        pushStyles(`Updated ${key}`);
                    }
                };

                meta.appendChild(lbl);
                meta.appendChild(hexInp);
                box.appendChild(picker);
                box.appendChild(meta);
                frag.appendChild(box);
            });

            grid.appendChild(frag);
        }

        function applySettingsSurface() {
            const useCustom = !!(styles.colors && styles.colors.use_custom_colors);
            const t = resolveThemePalette();
            const root = document.documentElement;
            const light = !t.isDefaultDark && isLightHex(t.bg);

            if (light) root.setAttribute('data-theme-kind', 'light');
            else root.removeAttribute('data-theme-kind');

            root.style.setProperty('--fnx-bg', t.bg);
            root.style.setProperty('--fnx-panel', t.panel);
            root.style.setProperty('--fnx-surface', t.surface);
            root.style.setProperty('--fnx-elevated', t.elevated);
            root.style.setProperty('--fnx-hover', t.hover);
            root.style.setProperty('--fnx-fg', t.fg);
            root.style.setProperty('--fnx-secondary', t.secondary);
            root.style.setProperty('--fnx-muted', t.muted);
            root.style.setProperty('--fnx-accent', t.accent);

            if (t.isDefaultDark) {
                root.style.setProperty('--fnx-divider', 'rgba(255, 255, 255, 0.05)');
                root.style.setProperty('--fnx-hairline', 'rgba(255, 255, 255, 0.08)');
                root.style.setProperty('--fnx-hairline-hover', 'rgba(255, 255, 255, 0.14)');
                root.style.setProperty('--fnx-btn-bg', 'rgba(255, 255, 255, 0.05)');
                root.style.setProperty('--fnx-btn-bg-hover', 'rgba(255, 255, 255, 0.09)');
                root.style.setProperty('--fnx-faint', '#52525B');
            } else {
                root.style.setProperty('--fnx-divider', `color-mix(in srgb, ${t.fg} ${light ? '8%' : '5%'}, transparent)`);
                root.style.setProperty('--fnx-hairline', `color-mix(in srgb, ${t.fg} ${light ? '12%' : '8%'}, transparent)`);
                root.style.setProperty('--fnx-hairline-hover', `color-mix(in srgb, ${t.fg} ${light ? '20%' : '15%'}, transparent)`);
                root.style.setProperty('--fnx-btn-bg', `color-mix(in srgb, ${t.fg} ${light ? '5%' : '5%'}, transparent)`);
                root.style.setProperty('--fnx-btn-bg-hover', `color-mix(in srgb, ${t.fg} ${light ? '10%' : '9%'}, transparent)`);
                root.style.setProperty('--fnx-faint', t.muted);
            }

            const sp = styles.settings_page || {};
            const sw = sp.sidebar_width && sp.sidebar_width !== '244px' ? sp.sidebar_width : '224px';
            root.style.setProperty('--cfg-sidebar-w', sw);
            if (sp.content_padding && sp.content_padding !== '28px 36px') {
                root.style.setProperty('--cfg-content-pad', sp.content_padding);
            } else {
                root.style.setProperty('--cfg-content-pad', '32px 36px 56px');
            }

            const appEl = document.getElementById('__fynexo_settings_app__');
            appEl.classList.toggle('compact-width', sp.full_width === false);

            const customCssEl = document.getElementById('__fynexo_settings_custom_css__');
            if (customCssEl) customCssEl.textContent = sp.custom_css || '';

            setSwitchState('btnCustomColorsToggle', useCustom);

            document.querySelectorAll('.theme-card').forEach(card => {
                card.classList.toggle('active', !useCustom && card.dataset.theme === config.theme);
            });
        }

        function renderThemeGrid() {
            const themeGrid = document.getElementById('themeGrid');
            themeGrid.textContent = '';
            const useCustom = !!(styles.colors && styles.colors.use_custom_colors);
            const frag = document.createDocumentFragment();

            themes.forEach(item => {
                if (!item || !item.id) return;
                const c = item.colors || item;
                const borderHover = c.border_hover || c.borderHover || '#34343e';
                const accentBg = c.accent_bg || c.accentBg || '#1b1d36';

                const card = document.createElement('div');
                card.className = (!useCustom && item.id === config.theme) ? 'theme-card active' : 'theme-card';
                card.dataset.theme = item.id;
                card.setAttribute('role', 'button');
                card.tabIndex = 0;

                const preview = document.createElement('div');
                preview.className = 'mini-preview';
                preview.style.background = c.bg;

                const bar = document.createElement('div');
                bar.className = 'mini-bar';
                bar.style.background = c.panel;

                const tabsWrap = document.createElement('div');
                tabsWrap.className = 'mini-tabs';

                const tab1 = document.createElement('div');
                tab1.className = 'mini-tab';
                tab1.style.background = c.elevated;
                tab1.style.border = `1px solid ${c.border}`;
                const dot1 = document.createElement('span');
                dot1.className = 'mini-dot';
                dot1.style.background = c.accent;
                const line1 = document.createElement('span');
                line1.className = 'mini-line';
                line1.style.background = c.fg;
                tab1.appendChild(dot1);
                tab1.appendChild(line1);

                const tab2 = document.createElement('div');
                tab2.className = 'mini-tab';
                const line2 = document.createElement('span');
                line2.className = 'mini-line';
                line2.style.width = '14px';
                line2.style.background = c.muted;
                tab2.appendChild(line2);

                tabsWrap.appendChild(tab1);
                tabsWrap.appendChild(tab2);

                const omni = document.createElement('div');
                omni.className = 'mini-omnibox';
                omni.style.background = c.surface || c.panel;
                omni.style.border = `1px solid ${c.border}`;

                bar.appendChild(tabsWrap);
                bar.appendChild(omni);

                const body = document.createElement('div');
                body.className = 'mini-body';
                const col = document.createElement('div');
                col.className = 'mini-canvas-col';
                const b1 = document.createElement('div');
                b1.style.cssText = `width: 52%; height: 4px; border-radius: 2px; background: ${borderHover};`;
                const b2 = document.createElement('div');
                b2.style.cssText = `width: 34%; height: 3px; border-radius: 2px; background: ${c.border};`;
                col.appendChild(b1);
                col.appendChild(b2);

                const pill = document.createElement('div');
                pill.style.cssText = `width: 22px; height: 11px; border-radius: 3px; background: ${accentBg}; border: 1px solid ${c.accent};`;

                body.appendChild(col);
                body.appendChild(pill);

                preview.appendChild(bar);
                preview.appendChild(body);

                const info = document.createElement('div');
                info.className = 'theme-info';

                const nameWrap = document.createElement('span');
                nameWrap.className = 'theme-name-wrap';
                const nm = document.createElement('span');
                nm.className = 'theme-name';
                nm.textContent = item.name || item.id;
                const activeTag = document.createElement('span');
                activeTag.className = 'theme-active-tag';
                activeTag.textContent = 'Active';
                nameWrap.appendChild(nm);
                nameWrap.appendChild(activeTag);

                const rightWrap = document.createElement('span');
                rightWrap.className = 'theme-meta-right';
                const hx = document.createElement('span');
                hx.className = 'theme-hex';
                hx.textContent = `${item.id}.json`;
                rightWrap.appendChild(hx);

                if (!item.builtin) {
                    const delBtn = document.createElement('button');
                    delBtn.type = 'button';
                    delBtn.className = 'theme-del-btn';
                    delBtn.title = `Delete custom theme ${item.id}.json`;
                    delBtn.textContent = '×';
                    delBtn.onclick = (e) => {
                        e.stopPropagation();
                        sendIpc({ type: 'DELETE_THEME', id: item.id });
                    };
                    rightWrap.appendChild(delBtn);
                }

                info.appendChild(nameWrap);
                info.appendChild(rightWrap);

                const sub = document.createElement('div');
                sub.className = 'theme-sub';
                sub.textContent = item.description || 'Custom JSON palette';

                card.appendChild(preview);
                card.appendChild(info);
                card.appendChild(sub);

                const selectThisTheme = () => {
                    config.theme = item.id;
                    activeTheme = c;
                    if (styles.colors && styles.colors.use_custom_colors) {
                        styles.colors.use_custom_colors = false;
                        sendIpc({ type: 'SAVE_STYLES', styles: styles });
                        document.getElementById('jsonEditor').value = JSON.stringify(styles, null, 2);
                    }
                    renderColorTokensStudio();
                    applySettingsSurface();
                    saveConfig(`Theme set to ${item.name || item.id}`);
                };

                card.onclick = selectThisTheme;
                card.onkeydown = (e) => {
                    if (e.key === 'Enter' || e.key === ' ') {
                        e.preventDefault();
                        selectThisTheme();
                    }
                };

                frag.appendChild(card);
            });

            themeGrid.appendChild(frag);
        }

        function createCompactDeleteBtn(title, onDelete) {
            const del = document.createElement('button');
            del.type = 'button';
            del.className = 'rc-del';
            del.textContent = '×';
            del.title = title;
            del.setAttribute('aria-label', title);
            del.onclick = (e) => {
                e.stopPropagation();
                onDelete();
            };
            return del;
        }

        function renderEngineTable() {
            const tbl = document.getElementById('engineTable');
            tbl.textContent = '';
            const list = getEngines();
            const builtinIds = ['google', 'duckduckgo', 'bing', 'baidu'];
            const frag = document.createDocumentFragment();

            list.forEach(eng => {
                const isDef = eng.id === config.search_engine;
                const row = document.createElement('div');
                row.className = 'rc-row clickable' + (isDef ? ' is-default' : '');

                const left = document.createElement('div');
                left.className = 'rc-left';
                const radio = document.createElement('span');
                radio.className = 'rc-radio';
                left.appendChild(radio);
                left.appendChild(createSiteEmblem(eng.url));

                const mid = document.createElement('div');
                mid.className = 'rc-mid';
                const nm = document.createElement('span');
                nm.className = 'rc-name';
                nm.textContent = eng.name || eng.id;
                const tpl = document.createElement('span');
                tpl.className = 'rc-tpl';
                tpl.textContent = eng.url || '';
                mid.appendChild(nm);
                mid.appendChild(tpl);

                const right = document.createElement('div');
                right.className = 'rc-right';
                if (isDef) {
                    const pill = document.createElement('span');
                    pill.className = 'rc-active-pill';
                    pill.textContent = 'Default';
                    right.appendChild(pill);
                }
                if (!builtinIds.includes(eng.id) && list.length > 1) {
                    right.appendChild(createCompactDeleteBtn('Remove custom search engine', () => {
                        config.search_engines = list.filter(x => x.id !== eng.id);
                        if (config.search_engine === eng.id) {
                            config.search_engine = config.search_engines[0].id;
                        }
                        renderEngineTable();
                        saveConfig('Removed search engine');
                    }));
                }

                row.onclick = () => {
                    config.search_engine = eng.id;
                    renderEngineTable();
                    saveConfig(`Search engine: ${eng.name || eng.id}`);
                };

                row.appendChild(left);
                row.appendChild(mid);
                row.appendChild(right);
                frag.appendChild(row);
            });
            tbl.appendChild(frag);
        }

        function renderBangsTable() {
            const tbl = document.getElementById('bangsTable');
            tbl.textContent = '';
            const bangs = getBangs();
            if (bangs.length === 0) {
                const empty = document.createElement('div');
                empty.className = 'rc-row-empty';
                empty.textContent = 'No !bang triggers yet — add one below.';
                tbl.appendChild(empty);
                return;
            }
            const frag = document.createDocumentFragment();
            bangs.forEach((b, idx) => {
                if (!b || !b.trigger) return;
                const trigStr = b.trigger.startsWith('!') ? b.trigger : ('!' + b.trigger);
                const row = document.createElement('div');
                row.className = 'rc-row';

                const left = document.createElement('div');
                left.className = 'rc-left';
                const badge = document.createElement('span');
                badge.className = 'rc-badge';
                badge.textContent = trigStr;
                left.appendChild(badge);

                const mid = document.createElement('div');
                mid.className = 'rc-mid';
                const nm = document.createElement('span');
                nm.className = 'rc-name';
                nm.textContent = b.name || trigStr.slice(1);
                const tpl = document.createElement('span');
                tpl.className = 'rc-tpl';
                tpl.textContent = b.url || '';
                mid.appendChild(nm);
                mid.appendChild(tpl);

                const right = document.createElement('div');
                right.className = 'rc-right';
                right.appendChild(createCompactDeleteBtn(`Remove ${trigStr}`, () => {
                    config.bangs = bangs.filter((_, i) => i !== idx);
                    renderBangsTable();
                    saveConfig(`Removed ${trigStr}`);
                }));

                row.appendChild(left);
                row.appendChild(mid);
                row.appendChild(right);
                frag.appendChild(row);
            });
            tbl.appendChild(frag);
        }

        function renderShortcutsTable() {
            const tbl = document.getElementById('shortcutsTable');
            tbl.textContent = '';
            if (bookmarks.length === 0) {
                const empty = document.createElement('div');
                empty.className = 'rc-row-empty';
                empty.textContent = 'No shortcuts saved — add one below.';
                tbl.appendChild(empty);
                return;
            }
            const frag = document.createDocumentFragment();
            bookmarks.forEach(bm => {
                const row = document.createElement('div');
                row.className = 'rc-row';

                const left = document.createElement('div');
                left.className = 'rc-left';
                left.appendChild(createSiteEmblem(bm.url));

                const mid = document.createElement('div');
                mid.className = 'rc-mid';
                const nm = document.createElement('span');
                nm.className = 'rc-name';
                nm.textContent = bm.title || bm.url;
                const tpl = document.createElement('span');
                tpl.className = 'rc-tpl';
                tpl.textContent = bm.url || '';
                mid.appendChild(nm);
                mid.appendChild(tpl);

                const right = document.createElement('div');
                right.className = 'rc-right';
                right.appendChild(createCompactDeleteBtn('Remove shortcut', () => {
                    sendIpc({ type: 'REMOVE_BOOKMARK', url: bm.url });
                }));

                row.appendChild(left);
                row.appendChild(mid);
                row.appendChild(right);
                frag.appendChild(row);
            });
            tbl.appendChild(frag);
        }

        function renderDomainTable(targetId, domains, emptyText, onRemove) {
            const tbl = document.getElementById(targetId);
            tbl.textContent = '';
            if (domains.length === 0) {
                const empty = document.createElement('div');
                empty.className = 'rc-row-empty';
                empty.textContent = emptyText;
                tbl.appendChild(empty);
                return;
            }
            const frag = document.createDocumentFragment();
            domains.forEach(dom => {
                const row = document.createElement('div');
                row.className = 'rc-row';

                const mid = document.createElement('div');
                mid.className = 'rc-mid';
                const txt = document.createElement('span');
                txt.className = 'rc-tpl';
                txt.style.color = 'var(--fnx-fg)';
                txt.textContent = dom;
                mid.appendChild(txt);

                const right = document.createElement('div');
                right.className = 'rc-right';
                right.appendChild(createCompactDeleteBtn(`Remove ${dom}`, () => onRemove(dom)));

                row.appendChild(mid);
                row.appendChild(right);
                frag.appendChild(row);
            });
            tbl.appendChild(frag);
        }

        function renderAdblockDomainLists() {
            const bList = Array.isArray(adblockCfg.custom_blocked_domains) ? adblockCfg.custom_blocked_domains : [];
            const wList = Array.isArray(adblockCfg.whitelisted_domains) ? adblockCfg.whitelisted_domains : [];

            renderDomainTable('blockedDomainsList', bList, 'No custom blocked domains.', (dom) => {
                adblockCfg.custom_blocked_domains = bList.filter(x => x !== dom);
                renderAdblockDomainLists();
                sendIpc({ type: 'SAVE_ADBLOCK', adblock: adblockCfg });
                showToast(`Unblocked ${dom}`);
            });

            renderDomainTable('whitelistDomainsList', wList, 'No whitelisted domains.', (dom) => {
                adblockCfg.whitelisted_domains = wList.filter(x => x !== dom);
                renderAdblockDomainLists();
                sendIpc({ type: 'SAVE_ADBLOCK', adblock: adblockCfg });
                showToast(`Removed ${dom} from whitelist`);
            });
        }

        const JS_SECURITY_RULES = [
            ['Dynamic code execution (eval / Function)', /\b(?:eval|Function)\s*\(|\b(?:setTimeout|setInterval)\s*\(\s*['"`]/i],
            ['Network request API (fetch / XHR / WS)', /\b(?:fetch|XMLHttpRequest|sendBeacon|WebSocket|EventSource)\b/i],
            ['Cookie or web storage access', /document\s*\.\s*cookie|\b(?:localStorage|sessionStorage|indexedDB)\b/i],
            ['Dynamic <script> / <iframe> injection', /createElement\s*\(\s*['"`](?:script|iframe)['"`]|<(?:script|iframe)\b/i],
            ['Encoded / obfuscated code (atob / escapes)', /\b(?:atob|fromCharCode)\b|(?:\\x[0-9a-f]{2}){4,}|(?:\\u[0-9a-f]{4}){4,}/i],
            ['Password / form submission interception', /type\s*=\s*['"]?password|addEventListener\s*\(\s*['"]submit['"]/i],
            ['Browser IPC / bridge probing', /\bwindow\s*\.\s*ipc\b|__fynexo_|__TAURI_/i]
        ];
        const CSS_SECURITY_RULES = [
            ['External URL or @import in CSS', /@import\b|url\s*\(\s*['"]?\s*(?:https?:|\/\/)/i],
            ['Legacy CSS script expression', /expression\s*\(|-moz-binding\s*:/i]
        ];

        function auditExtensionPayload(css, js, domain) {
            const warnings = [];
            const cssText = String(css || '');
            const jsText = String(js || '');
            const domText = String(domain || '*').trim() || '*';

            if (jsText.trim()) {
                JS_SECURITY_RULES.forEach(([label, re]) => {
                    if (re.test(jsText)) warnings.push(label);
                });
                if (domText === '*') {
                    warnings.push('Wildcard domain (*) combined with JS');
                }
                const lines = jsText.split(/\r?\n/);
                if (lines.some(line => line.length > 600)) {
                    warnings.push('Long minified/single-line JS (>600 chars)');
                }
            }
            if (cssText.trim()) {
                CSS_SECURITY_RULES.forEach(([label, re]) => {
                    if (re.test(cssText)) warnings.push(label);
                });
            }

            return {
                hasWarnings: warnings.length > 0,
                warnings,
                summary: warnings.join('; '),
                statusValue: warnings.length > 0 ? 'warning' : 'unverified'
            };
        }

        function renderAuditBanner(el, audit, mode) {
            if (!el) return;
            el.classList.remove('warn', 'clean');
            if (audit && audit.hasWarnings) {
                el.classList.add('warn');
                const prefix = mode === 'upload'
                    ? 'Warning — upload will be flagged as "warning" in database: '
                    : 'Security Warning — review source carefully before enabling: ';
                el.textContent = prefix + audit.warnings.join(' · ');
            } else {
                el.classList.add('clean');
                el.textContent = mode === 'upload'
                    ? 'Security check: Clean — upload enters queue as "unverified".'
                    : 'Security check: Clean — no suspicious patterns detected.';
            }
        }

        function renderSiteRulesList() {
            const listEl = document.getElementById('rulesList');
            listEl.textContent = '';

            if (siteRules.length === 0) {
                const empty = document.createElement('div');
                empty.className = 'rc-row-empty';
                empty.style.borderBottom = 'none';
                empty.textContent = 'No custom CSS/JS extensions configured. Click "+ Add Extension" or install from Cloud Store.';
                listEl.appendChild(empty);
                updateExtensionNavCount();
                return;
            }

            const frag = document.createDocumentFragment();
            siteRules.forEach((rule, idx) => {
                const audit = auditExtensionPayload(rule.css, rule.js, rule.domain);
                const row = document.createElement('div');
                row.className = 'rc-row';
                if (idx === siteRules.length - 1) row.style.borderBottom = 'none';

                const left = document.createElement('div');
                left.className = 'rc-left';
                const domBadge = document.createElement('span');
                domBadge.className = 'rc-badge';
                domBadge.textContent = rule.domain || '*';
                left.appendChild(domBadge);
                const isUserscript = !!(rule.js && /\/\/\s*==UserScript==|\bGM(?:_|\.[a-z])/.test(rule.js));
                if (isUserscript) {
                    const sfBadge = document.createElement('span');
                    sfBadge.className = 'rc-badge';
                    sfBadge.textContent = 'ScriptForge';
                    left.appendChild(sfBadge);
                }
                if (audit.hasWarnings) {
                    const warnTag = document.createElement('span');
                    warnTag.className = 'sec-audit-badge warn';
                    warnTag.textContent = 'Warning';
                    warnTag.title = audit.summary;
                    left.appendChild(warnTag);
                }

                const mid = document.createElement('div');
                mid.className = 'rc-mid';
                const nm = document.createElement('span');
                nm.className = 'rc-name';
                nm.textContent = rule.name || rule.id;
                mid.appendChild(nm);

                const tags = [];
                if (rule.css && rule.css.trim()) tags.push('CSS');
                if (rule.js && rule.js.trim()) tags.push('JS');
                const verStr = rule.version ? String(rule.version).trim() : '';
                if (tags.length > 0 || verStr) {
                    const tagTxt = document.createElement('span');
                    tagTxt.className = 'rc-tpl';
                    const kindLabel = tags.join(' + ');
                    const verLabel = verStr ? (/^v/i.test(verStr) ? verStr : `v${verStr}`) : '';
                    tagTxt.textContent = [kindLabel, verLabel].filter(Boolean).join(' · ');
                    mid.appendChild(tagTxt);
                }

                const right = document.createElement('div');
                right.className = 'rc-right';

                const sw = document.createElement('button');
                sw.type = 'button';
                sw.className = rule.enabled ? 'fnx-switch on' : 'fnx-switch';
                sw.setAttribute('role', 'switch');
                sw.setAttribute('aria-checked', rule.enabled ? 'true' : 'false');
                sw.title = rule.enabled ? 'Disable rule' : 'Enable rule';
                sw.onclick = () => {
                    rule.enabled = !rule.enabled;
                    renderSiteRulesList();
                    if (cloudLoadedOnce) renderCloudExtensions();
                    sendIpc({ type: 'SAVE_RULES', rules: siteRules });
                    showToast(`${rule.name || 'Rule'}: ${rule.enabled ? 'Enabled' : 'Disabled'}`);
                };

                const pubBtn = document.createElement('button');
                pubBtn.type = 'button';
                pubBtn.className = 'action-btn compact';
                pubBtn.textContent = 'Publish';
                pubBtn.title = 'Upload extension to Supabase Cloud Store (starts as unverified)';
                pubBtn.onclick = () => openPublishPanel(rule);

                const editBtn = document.createElement('button');
                editBtn.type = 'button';
                editBtn.className = 'action-btn compact';
                editBtn.textContent = 'Edit';
                editBtn.onclick = () => openRuleEditor(rule);

                right.appendChild(pubBtn);
                right.appendChild(editBtn);
                right.appendChild(sw);
                right.appendChild(createCompactDeleteBtn('Delete rule', () => {
                    siteRules = siteRules.filter(x => x.id !== rule.id);
                    renderSiteRulesList();
                    if (cloudLoadedOnce) renderCloudExtensions();
                    sendIpc({ type: 'SAVE_RULES', rules: siteRules });
                    showToast('Deleted rule');
                }));

                row.appendChild(left);
                row.appendChild(mid);
                row.appendChild(right);
                frag.appendChild(row);
            });

            listEl.appendChild(frag);
            updateExtensionNavCount();
        }

        function updateExtensionNavCount() {
            const navBadge = document.getElementById('extNavCount');
            if (!navBadge) return;
            let active = 0;
            BUILTIN_EXTENSION_GROUPS.forEach((group) => {
                group.items.forEach((item) => {
                    if (item.get()) active += 1;
                });
            });
            const customEnabled = Array.isArray(siteRules)
                ? siteRules.filter(r => r && r.enabled).length
                : 0;
            navBadge.textContent = String(active + customEnabled);
        }

        // =========================================================================
        // Supabase Cloud Extensions (On-Demand Catalog + Single-Item Download)
        // =========================================================================
        let cloudCatalog = [];
        let cloudLoadedOnce = false;
        let cloudLoading = false;

        function getSupabaseCreds() {
            let stored = {};
            try {
                stored = JSON.parse(localStorage.getItem(SUPABASE_STORAGE_KEY) || '{}') || {};
            } catch (_) {}
            const url = (SUPABASE_CONFIG.url || stored.url || '').trim().replace(/\/+$/, '');
            const anonKey = (SUPABASE_CONFIG.anonKey || stored.anonKey || '').trim();
            const schema = (SUPABASE_CONFIG.schema || stored.schema || 'public').trim() || 'public';
            const table = (SUPABASE_CONFIG.table || stored.table || 'extensions').trim() || 'extensions';
            return { url, anonKey, schema, table, isConfigured: Boolean(url && anonKey) };
        }

        async function supabaseRequest(path, options) {
            const creds = getSupabaseCreds();
            if (!creds.isConfigured) {
                throw new Error('Supabase API URL & Anon Key are not configured yet.');
            }
            const opts = options || {};
            const method = String(opts.method || 'GET').toUpperCase();
            const profileHeaders = {};
            if (creds.schema && creds.schema !== 'public') {
                profileHeaders['Accept-Profile'] = creds.schema;
                if (method !== 'GET' && method !== 'HEAD') {
                    profileHeaders['Content-Profile'] = creds.schema;
                }
            }
            const headers = Object.assign({
                apikey: creds.anonKey,
                Authorization: 'Bearer ' + creds.anonKey,
                'Content-Type': 'application/json'
            }, profileHeaders, opts.headers || {});
            const res = await fetch(`${creds.url}/rest/v1/${creds.table}${path}`, Object.assign({}, opts, { headers }));
            if (!res.ok) {
                const text = await res.text().catch(() => '');
                throw new Error(`HTTP ${res.status}${text ? ': ' + text.slice(0, 140) : ''}`);
            }
            const raw = await res.text();
            return raw ? JSON.parse(raw) : [];
        }

        function setCloudHint(msg) {
            const el = document.getElementById('cloudStatusHint');
            if (el) el.textContent = msg;
        }

        function syncCloudConfigInputs() {
            const creds = getSupabaseCreds();
            const urlEl = document.getElementById('inputSupabaseUrl');
            const keyEl = document.getElementById('inputSupabaseKey');
            const schemaEl = document.getElementById('inputSupabaseSchema');
            const tableEl = document.getElementById('inputSupabaseTable');
            const card = document.getElementById('cloudConfigCard');
            if (urlEl && !urlEl.value) urlEl.value = creds.url;
            if (keyEl && !keyEl.value) keyEl.value = creds.anonKey;
            if (schemaEl && !schemaEl.value) schemaEl.value = creds.schema;
            if (tableEl && !tableEl.value) tableEl.value = creds.table;
            if (card && !creds.isConfigured) card.classList.add('open');
        }

        function renderCloudExtensions() {
            const listEl = document.getElementById('cloudExtList');
            if (!listEl) return;
            listEl.textContent = '';

            const creds = getSupabaseCreds();
            if (!creds.isConfigured) {
                const empty = document.createElement('div');
                empty.className = 'rc-row-empty';
                empty.style.borderBottom = 'none';
                empty.textContent = 'Supabase API not connected. Configure .env.local before build or enter your URL & Anon Key in API Config above.';
                listEl.appendChild(empty);
                return;
            }

            if (cloudLoading) {
                const loadingRow = document.createElement('div');
                loadingRow.className = 'rc-row-empty';
                loadingRow.style.borderBottom = 'none';
                loadingRow.textContent = 'Fetching extension catalog metadata from Supabase...';
                listEl.appendChild(loadingRow);
                return;
            }

            const q = (document.getElementById('inputCloudSearch')?.value || '').trim().toLowerCase();
            const cat = (document.getElementById('selectCloudCategory')?.value || 'all').toLowerCase();
            const filtered = cloudCatalog.filter((item) => {
                if (cat !== 'all' && (item.category || 'custom').toLowerCase() !== cat) return false;
                if (!q) return true;
                const hay = `${item.name || ''} ${item.domain || ''} ${item.description || ''} ${item.author || ''}`.toLowerCase();
                return hay.includes(q);
            });

            if (filtered.length === 0) {
                const empty = document.createElement('div');
                empty.className = 'rc-row-empty';
                empty.style.borderBottom = 'none';
                empty.textContent = cloudCatalog.length === 0
                    ? 'No verified extensions found in Supabase yet. Click Refresh or upload one above.'
                    : 'No matching extensions for the current filter.';
                listEl.appendChild(empty);
                return;
            }

            const frag = document.createDocumentFragment();
            filtered.forEach((item, idx) => {
                const ruleId = 'cloud-' + (item.slug || item.id);
                const installedRule = siteRules.find(r => r && r.id === ruleId);

                const row = document.createElement('div');
                row.className = 'rc-row';
                if (idx === filtered.length - 1) row.style.borderBottom = 'none';

                const left = document.createElement('div');
                left.className = 'rc-left';
                const domBadge = document.createElement('span');
                domBadge.className = 'rc-badge';
                domBadge.textContent = item.domain || '*';
                left.appendChild(domBadge);

                const mid = document.createElement('div');
                mid.className = 'rc-mid';
                const nm = document.createElement('span');
                nm.className = 'rc-name';
                nm.textContent = item.name || item.slug || 'Extension';
                mid.appendChild(nm);

                const sizeKb = item.size_bytes ? `${Math.max(1, Math.round(item.size_bytes / 1024))} KB` : '';
                const metaParts = [item.category || 'custom', item.version ? `v${item.version}` : '', sizeKb, item.author ? `@${item.author}` : ''].filter(Boolean);
                if (item.description || metaParts.length) {
                    const descEl = document.createElement('span');
                    descEl.className = 'cloud-desc';
                    descEl.textContent = [metaParts.join(' · '), item.description || ''].filter(Boolean).join(' — ');
                    descEl.title = descEl.textContent;
                    mid.appendChild(descEl);
                }

                const right = document.createElement('div');
                right.className = 'rc-right';

                const inspBtn = document.createElement('button');
                inspBtn.type = 'button';
                inspBtn.className = 'action-btn compact';
                inspBtn.textContent = 'Inspect';
                inspBtn.title = 'Inspect CSS/JS source and run local security check';
                inspBtn.onclick = () => inspectCloudExtension(item, inspBtn);
                right.appendChild(inspBtn);

                if (installedRule) {
                    const cloudVer = item.version ? String(item.version) : '';
                    const localVer = installedRule.version ? String(installedRule.version) : '';
                    if (cloudVer && localVer !== cloudVer) {
                        const updBtn = document.createElement('button');
                        updBtn.type = 'button';
                        updBtn.className = 'action-btn compact';
                        updBtn.textContent = 'Update';
                        updBtn.title = `Update from ${localVer || 'legacy'} to v${cloudVer}`;
                        updBtn.onclick = () => installCloudExtension(item, updBtn, false);
                        right.appendChild(updBtn);
                    }

                    const sw = document.createElement('button');
                    sw.type = 'button';
                    sw.className = installedRule.enabled ? 'fnx-switch on' : 'fnx-switch';
                    sw.setAttribute('role', 'switch');
                    sw.setAttribute('aria-checked', installedRule.enabled ? 'true' : 'false');
                    sw.title = installedRule.enabled ? 'Disable extension' : 'Enable extension';
                    sw.onclick = () => {
                        installedRule.enabled = !installedRule.enabled;
                        renderSiteRulesList();
                        renderCloudExtensions();
                        sendIpc({ type: 'SAVE_RULES', rules: siteRules });
                        showToast(`${installedRule.name}: ${installedRule.enabled ? 'Enabled' : 'Disabled'}`);
                    };

                    const rmBtn = document.createElement('button');
                    rmBtn.type = 'button';
                    rmBtn.className = 'action-btn compact';
                    rmBtn.textContent = 'Remove';
                    rmBtn.onclick = () => {
                        siteRules = siteRules.filter(x => x.id !== ruleId);
                        renderSiteRulesList();
                        renderCloudExtensions();
                        sendIpc({ type: 'SAVE_RULES', rules: siteRules });
                        showToast(`Removed ${item.name}`);
                    };
                    right.appendChild(rmBtn);
                    right.appendChild(sw);
                } else {
                    const dlBtn = document.createElement('button');
                    dlBtn.type = 'button';
                    dlBtn.className = 'action-btn compact';
                    dlBtn.textContent = 'Install';
                    dlBtn.onclick = () => installCloudExtension(item, dlBtn, false);
                    right.appendChild(dlBtn);
                }

                row.appendChild(left);
                row.appendChild(mid);
                row.appendChild(right);
                frag.appendChild(row);
            });

            listEl.appendChild(frag);
        }

        function refreshCloudCategorySelect() {
            const sel = document.getElementById('selectCloudCategory');
            if (!sel) return;
            const prev = sel.value || 'all';
            const cats = Array.from(new Set(cloudCatalog.map(x => (x.category || 'custom').trim()).filter(Boolean))).sort();
            sel.textContent = '';
            const allOpt = document.createElement('option');
            allOpt.value = 'all';
            allOpt.textContent = 'All categories';
            sel.appendChild(allOpt);
            cats.forEach((c) => {
                const o = document.createElement('option');
                o.value = c;
                o.textContent = c;
                sel.appendChild(o);
            });
            sel.value = cats.includes(prev) ? prev : 'all';
        }

        async function fetchCloudCatalog(forceToast) {
            const creds = getSupabaseCreds();
            if (!creds.isConfigured) {
                syncCloudConfigInputs();
                renderCloudExtensions();
                if (forceToast) showToast('Enter Supabase URL & Anon Key first');
                return;
            }
            cloudLoading = true;
            renderCloudExtensions();
            try {
                // Metadata only, strictly verified entries (status=eq.verified)
                const rows = await supabaseRequest(
                    '?select=id,slug,name,category,domain,description,author,version,size_bytes&status=eq.verified&order=downloads.desc'
                );
                cloudCatalog = Array.isArray(rows) ? rows : [];
                cloudLoadedOnce = true;
                refreshCloudCategorySelect();
                setCloudHint(`Connected · ${cloudCatalog.length} verified extension${cloudCatalog.length === 1 ? '' : 's'} (CSS/JS downloaded only on Install).`);
                if (forceToast) showToast('Cloud catalog refreshed');
            } catch (err) {
                setCloudHint(`Supabase fetch failed: ${err.message || err}`);
                if (forceToast) showToast('Failed to load cloud catalog');
            } finally {
                cloudLoading = false;
                renderCloudExtensions();
            }
        }

        async function fetchExtensionPayload(meta) {
            const rows = await supabaseRequest(
                `?id=eq.${encodeURIComponent(meta.id)}&select=id,slug,name,domain,css,js,version&limit=1`
            );
            const full = Array.isArray(rows) ? rows[0] : null;
            if (!full) throw new Error('Extension payload not found');
            return full;
        }

        function commitCloudExtensionRule(meta, full) {
            const ruleId = 'cloud-' + (full.slug || full.id);
            const existingIdx = siteRules.findIndex(r => r && r.id === ruleId);
            const prevEnabled = existingIdx >= 0 ? !!siteRules[existingIdx].enabled : true;
            const ruleVer = full.version || meta.version || '1.0.0';
            const ruleObj = {
                id: ruleId,
                name: full.name || meta.name || 'Cloud Extension',
                domain: full.domain || meta.domain || '*',
                enabled: prevEnabled,
                version: String(ruleVer),
                css: full.css || '',
                js: full.js || ''
            };
            if (existingIdx >= 0) siteRules[existingIdx] = ruleObj;
            else siteRules.push(ruleObj);

            renderSiteRulesList();
            renderCloudExtensions();
            sendIpc({ type: 'SAVE_RULES', rules: siteRules });
            showToast(`${existingIdx >= 0 ? 'Updated' : 'Installed'}: ${ruleObj.name} (v${ruleObj.version})`);
        }

        function openCloudInspector(meta, full, audit) {
            const panel = document.getElementById('cloudInspectorPanel');
            if (!panel) return;
            document.getElementById('inspExtTitle').textContent = full.name || meta.name || 'Extension Source';
            const sizeKb = `${Math.max(1, Math.round(new Blob([full.css || '', full.js || '']).size / 1024))} KB`;
            document.getElementById('inspExtMeta').textContent = [
                full.domain || meta.domain || '*',
                full.version ? `v${full.version}` : '',
                meta.author ? `@${meta.author}` : '',
                sizeKb
            ].filter(Boolean).join(' · ');

            const badge = document.getElementById('inspAuditBadge');
            badge.classList.remove('warn', 'clean');
            badge.classList.add(audit.hasWarnings ? 'warn' : 'clean');
            badge.textContent = audit.hasWarnings ? `Warning (${audit.warnings.length})` : 'Clean';
            renderAuditBanner(document.getElementById('inspAuditBanner'), audit, 'inspect');

            document.getElementById('inspExtCss').value = (full.css && full.css.trim()) ? full.css : '/* No CSS */';
            document.getElementById('inspExtJs').value = (full.js && full.js.trim()) ? full.js : '// No JS';

            const ruleId = 'cloud-' + (full.slug || full.id);
            const installedRule = siteRules.find(r => r && r.id === ruleId);
            const instBtn = document.getElementById('btnInspInstall');
            const cloudVer = full.version || meta.version || '';
            const needsUpdate = installedRule && cloudVer && String(installedRule.version || '') !== String(cloudVer);
            if (audit.hasWarnings) {
                instBtn.textContent = needsUpdate ? 'Trust & Update' : 'Trust & Install';
            } else if (needsUpdate) {
                instBtn.textContent = 'Update';
            } else if (installedRule) {
                instBtn.textContent = 'Reinstall';
            } else {
                instBtn.textContent = 'Install';
            }
            instBtn.disabled = false;
            instBtn.onclick = () => {
                commitCloudExtensionRule(meta, full);
                panel.classList.remove('open');
            };

            panel.classList.add('open');
            panel.scrollIntoView({ behavior: 'smooth', block: 'nearest' });
        }

        async function inspectCloudExtension(meta, btn) {
            if (!meta || !meta.id) return;
            const prevLabel = btn ? btn.textContent : 'Inspect';
            if (btn) {
                btn.disabled = true;
                btn.textContent = 'Loading...';
            }
            try {
                const full = await fetchExtensionPayload(meta);
                const audit = auditExtensionPayload(full.css, full.js, full.domain);
                openCloudInspector(meta, full, audit);
            } catch (err) {
                showToast(`Inspect failed: ${err.message || err}`);
            } finally {
                if (btn) {
                    btn.disabled = false;
                    btn.textContent = prevLabel;
                }
            }
        }

        async function installCloudExtension(meta, btn, forceTrust) {
            if (!meta || !meta.id) return;
            const prevLabel = btn ? btn.textContent : 'Install';
            if (btn) {
                btn.disabled = true;
                btn.textContent = 'Downloading...';
            }
            try {
                const full = await fetchExtensionPayload(meta);
                const audit = auditExtensionPayload(full.css, full.js, full.domain);
                if (audit.hasWarnings && !forceTrust) {
                    openCloudInspector(meta, full, audit);
                    showToast('Security Warning detected — review source before installing');
                    if (btn) {
                        btn.disabled = false;
                        btn.textContent = prevLabel;
                    }
                    return;
                }
                commitCloudExtensionRule(meta, full);
            } catch (err) {
                if (btn) {
                    btn.disabled = false;
                    btn.textContent = prevLabel;
                }
                showToast(`Install failed: ${err.message || err}`);
            }
        }

        function updatePublishAuditPreview() {
            const domain = document.getElementById('pubExtDomain')?.value || '*';
            const css = document.getElementById('pubExtCss')?.value || '';
            const js = document.getElementById('pubExtJs')?.value || '';
            const audit = auditExtensionPayload(css, js, domain);
            renderAuditBanner(document.getElementById('publishAuditPreview'), audit, 'upload');
            return audit;
        }

        function openPublishPanel(prefillRule) {
            const cloudTab = document.querySelector('#extSubTabs .seg-btn[data-subpane="pane-ext-cloud"]');
            if (cloudTab) cloudTab.click();
            syncCloudConfigInputs();
            const panel = document.getElementById('publishExtPanel');
            if (!panel) return;
            panel.classList.add('open');
            document.getElementById('pubExtName').value = (prefillRule && prefillRule.name) || '';
            document.getElementById('pubExtDomain').value = (prefillRule && prefillRule.domain) || '*';
            document.getElementById('pubExtAuthor').value = document.getElementById('pubExtAuthor').value || '';
            document.getElementById('pubExtCategory').value = document.getElementById('pubExtCategory').value || 'community';
            document.getElementById('pubExtDesc').value = (prefillRule && prefillRule.name)
                ? `${prefillRule.name} (${prefillRule.domain || '*'})`
                : '';
            document.getElementById('pubExtCss').value = (prefillRule && prefillRule.css) || '';
            document.getElementById('pubExtJs').value = (prefillRule && prefillRule.js) || '';
            updatePublishAuditPreview();
            document.getElementById('pubExtName').focus();
        }

        async function submitPublishedExtension() {
            const creds = getSupabaseCreds();
            if (!creds.isConfigured) {
                syncCloudConfigInputs();
                showToast('Set Supabase URL & Anon Key in API Config first');
                return;
            }
            const name = document.getElementById('pubExtName').value.trim();
            const domain = document.getElementById('pubExtDomain').value.trim() || '*';
            const author = document.getElementById('pubExtAuthor').value.trim() || 'community';
            const category = document.getElementById('pubExtCategory').value.trim() || 'community';
            const description = document.getElementById('pubExtDesc').value.trim() || `Submitted from Fynexo (${domain})`;
            const css = document.getElementById('pubExtCss').value;
            const js = document.getElementById('pubExtJs').value;

            if (!name) return showToast('Enter an extension name');
            if (!css.trim() && !js.trim()) return showToast('Provide CSS or JS to upload');
            const payloadBytes = new Blob([css, js]).size;
            if (payloadBytes > 100 * 1024) {
                return showToast(`CSS + JS exceeds 100 KB limit (${Math.ceil(payloadBytes / 1024)} KB)`);
            }

            const audit = auditExtensionPayload(css, js, domain);
            const btn = document.getElementById('btnSubmitPublish');
            const prevText = btn ? btn.textContent : 'Submit for verification';
            if (btn) {
                btn.disabled = true;
                btn.textContent = 'Uploading...';
            }
            try {
                const slugBase = name
                    .toLowerCase()
                    .replace(/[^a-z0-9_-]+/g, '-')
                    .replace(/^-|-$/g, '') || 'custom-ext';
                const corePayload = {
                    slug: `${slugBase}-${Date.now().toString(36)}`,
                    name,
                    category,
                    domain,
                    description,
                    author,
                    version: '1.0.0',
                    css,
                    js,
                    status: audit.statusValue
                };
                const fullPayload = Object.assign({}, corePayload, {
                    size_bytes: payloadBytes,
                    security_warning: audit.summary
                });
                try {
                    await supabaseRequest('', {
                        method: 'POST',
                        headers: { Prefer: 'return=minimal' },
                        body: JSON.stringify(fullPayload)
                    });
                } catch (postErr) {
                    const msg = String(postErr && postErr.message ? postErr.message : postErr);
                    if (/PGRST204|column/i.test(msg)) {
                        await supabaseRequest('', {
                            method: 'POST',
                            headers: { Prefer: 'return=minimal' },
                            body: JSON.stringify(corePayload)
                        });
                    } else {
                        throw postErr;
                    }
                }
                document.getElementById('publishExtPanel').classList.remove('open');
                if (audit.hasWarnings) {
                    showToast(`已上传 "${name}"（数据库已标为 Warning，需人工重点审核）`);
                } else {
                    showToast(`已上传 "${name}"（状态：unverified），验证通过后将在 Cloud Store 显示`);
                }
            } catch (err) {
                showToast(`Upload failed: ${err.message || err}`);
            } finally {
                if (btn) {
                    btn.disabled = false;
                    btn.textContent = prevText;
                }
            }
        }

        const BUILTIN_EXTENSION_GROUPS = [
            {
                title: 'Workspace & Reading',
                titleZh: '工作区与阅读工具',
                items: [
                    {
                        key: 'ext_side_panel',
                        name: 'Side Panel & Web Dock',
                        nameZh: '分屏侧边栏 & 便签',
                        mem: 'WebView + Notes',
                        memZh: '分屏网页',
                        desc: 'Split-screen side drawer iframe & scratchpad (Alt+S). Unmounted until enabled.',
                        descZh: '按 Alt+S 在屏幕右侧分屏打开第二个网页或随手记便签，关闭时零内存占用。',
                        get: () => !!config.ext_side_panel,
                        set: (next) => { config.ext_side_panel = next; }
                    },
                    {
                        key: 'ext_reader_mode',
                        name: 'Reader Mode',
                        nameZh: '纯净阅读模式',
                        mem: 'DOM Scanner',
                        memZh: '文章精排',
                        desc: 'Automatic article extraction & clean reading view (Alt+R). Skips page scans when off.',
                        descZh: '按 Alt+R 自动提取长文章正文、调大字号去广告，让你像看电子书一样安静阅读。',
                        get: () => !!config.ext_reader_mode,
                        set: (next) => { config.ext_reader_mode = next; }
                    },
                    {
                        key: 'ext_mouse_gestures',
                        name: 'Mouse Gestures (Right-Click Drag)',
                        nameZh: '右键鼠标手势',
                        mem: 'HUD Canvas',
                        memZh: '效率手势',
                        desc: 'Hold right mouse button & drag on desktop pages (← Back, → Forward, ↓→ Close tab, ↑↓ Reload).',
                        descZh: '电脑端按住鼠标右键轻滑即可控制网页：← 后退、→ 前进、↓→ 关标签、↑↓ 刷新。',
                        get: () => config.ext_mouse_gestures !== false,
                        set: (next) => { config.ext_mouse_gestures = next; }
                    }
                ]
            },
            {
                title: 'Media & Site Enhancers',
                titleZh: '视频增强与自定义脚本',
                items: [
                    {
                        key: 'ext_userscript_engine',
                        name: 'ScriptForge · Userscript Engine',
                        nameZh: 'ScriptForge · 内置用户脚本引擎',
                        mem: 'GM_* Sandbox',
                        memZh: '类油猴引擎',
                        desc: 'Runs // ==UserScript== scripts (@match/@include/@run-at) with GM_* APIs & Shield menu commands.',
                        descZh: '直接兼容 .user.js 用户脚本与 GM_* 扩展接口，可在右下角或顶部盾牌弹窗快速控制。',
                        get: () => config.ext_userscript_engine !== false,
                        set: (next) => { config.ext_userscript_engine = next; }
                    },
                    {
                        key: 'ext_yt_ad_skipper',
                        name: 'YouTube Ad Skipper',
                        nameZh: 'YouTube 视频广告自动跳过',
                        mem: 'Observer + Timer',
                        memZh: '自动跳过',
                        desc: 'Detects YouTube promo slots, clicks skip buttons, and hides ad overlays automatically.',
                        descZh: '自动检测 YouTube 广告位并帮忙点击“跳过广告”按钮，同时隐藏播放条贴片。',
                        get: () => !!config.ext_yt_ad_skipper,
                        set: (next) => { config.ext_yt_ad_skipper = next; }
                    },
                    {
                        key: 'ext_gh_harmonizer',
                        name: 'GitHub Drawer Harmonizer',
                        nameZh: 'GitHub 侧边栏高度适配',
                        mem: 'Subtree Observer',
                        memZh: '界面自适应',
                        desc: 'Aligns GitHub sticky sidebars & modal drawers beneath Fynexo topbar height.',
                        descZh: '让 GitHub 弹出的左侧导航抽屉自动避开 Fynexo 顶栏高度，防止顶部遮挡。',
                        get: () => !!config.ext_gh_harmonizer,
                        set: (next) => { config.ext_gh_harmonizer = next; }
                    },
                    {
                        key: 'ext_media_booster',
                        name: 'Media Volume Booster (300%)',
                        nameZh: '网页音量超级放大 (最高 300%)',
                        mem: 'WebAudio DSP',
                        memZh: '音量放大',
                        desc: 'Mounts WebAudio GainNode + DynamicsCompressor for >100% media amplification.',
                        descZh: '在媒体控制弹窗中开启 100%~300% 声音增益，看小声视频或电影时听得更清楚。',
                        get: () => !!config.ext_media_booster,
                        set: (next) => { config.ext_media_booster = next; }
                    }
                ]
            },
            {
                title: 'Visual & Memory Pool',
                titleZh: '夜间视觉与内存后台保活',
                items: [
                    {
                        key: 'force_dark',
                        name: 'Smart Force Dark',
                        nameZh: '智能夜间深色模式',
                        mem: 'CSS Filter',
                        memZh: '护眼夜间',
                        desc: 'Inverts bright webpages to dark while preserving images & video (Alt+Shift+D).',
                        descZh: '把刺眼的白底网页自动转为深色背景，同时保留图片和视频原始色彩（快捷键 Alt+Shift+D）。',
                        get: () => !!config.force_dark,
                        set: (next) => { config.force_dark = next; }
                    },
                    {
                        key: 'tab_cache_smooth',
                        name: 'Multi-Tab Live Keep-Alive',
                        nameZh: '多标签后台流畅保活',
                        mem: 'Smooth RAM Pool',
                        memZh: '后台保活',
                        desc: 'Keeps up to 6 background tabs warm in memory instead of discarding idle WebViews.',
                        descZh: '允许最多 6 个常用标签页留在后台保持运行（内存较充裕的电脑开启后切标签瞬间响应）。',
                        get: () => (config.tab_cache_mode || 'balanced').toLowerCase() === 'smooth',
                        set: (next) => { config.tab_cache_mode = next ? 'smooth' : 'balanced'; }
                    }
                ]
            }
        ];

        function renderBuiltinExtensions() {
            const container = document.getElementById('builtinExtensionsList');
            if (!container) return;
            container.textContent = '';

            const zh = isZh();
            const frag = document.createDocumentFragment();

            BUILTIN_EXTENSION_GROUPS.forEach((group) => {
                const heading = document.createElement('div');
                heading.className = 'set-group-title';
                heading.textContent = zh ? (group.titleZh || group.title) : group.title;
                frag.appendChild(heading);

                group.items.forEach((item, idx) => {
                    const isOn = item.get();
                    const itemName = zh ? (item.nameZh || item.name) : item.name;
                    const itemMem = zh ? (item.memZh || item.mem) : item.mem;
                    const itemDesc = zh ? (item.descZh || item.desc) : item.desc;

                    const row = document.createElement('div');
                    row.className = 'set-row';
                    if (idx === group.items.length - 1) row.style.borderBottom = 'none';

                    const meta = document.createElement('div');
                    meta.className = 'set-meta';

                    const label = document.createElement('div');
                    label.className = 'set-label';
                    label.textContent = itemName;

                    const tag = document.createElement('span');
                    tag.className = isOn ? 'ext-mem-tag on' : 'ext-mem-tag';
                    tag.textContent = zh
                        ? (isOn ? `已开启 · ${itemMem}` : `未开启 · ${itemMem}`)
                        : (isOn ? `Active · ${itemMem}` : `Off · ${itemMem}`);
                    label.appendChild(tag);

                    const hint = document.createElement('div');
                    hint.className = 'set-hint';
                    hint.textContent = itemDesc;

                    meta.appendChild(label);
                    meta.appendChild(hint);

                    const ctrl = document.createElement('div');
                    ctrl.className = 'set-ctrl';

                    const sw = document.createElement('button');
                    sw.type = 'button';
                    sw.className = isOn ? 'fnx-switch on' : 'fnx-switch';
                    sw.setAttribute('role', 'switch');
                    sw.setAttribute('aria-checked', isOn ? 'true' : 'false');
                    sw.setAttribute('aria-label', itemName);
                    sw.onclick = () => {
                        const next = !item.get();
                        item.set(next);
                        renderBuiltinExtensions();
                        saveConfig(zh
                            ? `${itemName}：${next ? '已开启' : '已关闭'}`
                            : `${itemName}: ${next ? 'Enabled' : 'Disabled'}`);
                    };

                    ctrl.appendChild(sw);
                    row.appendChild(meta);
                    row.appendChild(ctrl);
                    frag.appendChild(row);
                });
            });

            container.appendChild(frag);
            updateExtensionNavCount();
        }

        function parseUserscriptHeader(jsCode) {
            const src = String(jsCode || '');
            const blockMatch = src.match(/\/\/\s*==UserScript==([\s\S]*?)\/\/\s*==\/UserScript==/);
            if (!blockMatch) return null;

            const meta = {
                name: '',
                version: '',
                description: '',
                author: '',
                runAt: 'document-end',
                matches: [],
                includes: [],
                domain: '*'
            };

            const lines = blockMatch[1].split(/\r?\n/);
            for (const rawLine of lines) {
                const lineMatch = rawLine.match(/^\s*\/\/\s*@([A-Za-z0-9_-]+)\s+(.+?)\s*$/);
                if (!lineMatch) continue;
                const key = lineMatch[1].toLowerCase();
                const val = lineMatch[2].trim();
                if (!val) continue;
                if (key === 'name' && !meta.name) meta.name = val;
                else if (key === 'version' && !meta.version) meta.version = val;
                else if (key === 'description' && !meta.description) meta.description = val;
                else if (key === 'author' && !meta.author) meta.author = val;
                else if (key === 'run-at') meta.runAt = val;
                else if (key === 'match') meta.matches.push(val);
                else if (key === 'include') meta.includes.push(val);
            }

            const patterns = meta.matches.concat(meta.includes);
            if (patterns.length > 0) {
                const hosts = new Set();
                let allSingleHost = true;
                for (const pat of patterns) {
                    if (pat === '*' || (pat.startsWith('/') && pat.endsWith('/') && pat.length > 2)) {
                        allSingleHost = false;
                        break;
                    }
                    const hostMatch = pat.match(/^(?:\*|https?\*?):\/\/([^\/?#]+)/i);
                    if (!hostMatch) {
                        allSingleHost = false;
                        break;
                    }
                    const host = hostMatch[1].replace(/:\d+$/, '').trim().toLowerCase();
                    if (!host || host === '*' || !/^(\*\.)?[a-z0-9.-]+$/.test(host) || (!host.includes('.') && host !== 'localhost')) {
                        allSingleHost = false;
                        break;
                    }
                    hosts.add(host);
                }
                if (allSingleHost && hosts.size === 1) {
                    meta.domain = Array.from(hosts)[0];
                }
            }

            return meta;
        }

        function autofillFromUserscript(jsCode, fields) {
            const meta = parseUserscriptHeader(jsCode);
            if (!meta) return null;
            if (fields.nameEl && !fields.nameEl.value.trim() && meta.name) {
                fields.nameEl.value = meta.name;
            }
            if (fields.domainEl && (!fields.domainEl.value.trim() || fields.domainEl.value.trim() === '*') && meta.domain && meta.domain !== '*') {
                fields.domainEl.value = meta.domain;
                if (typeof fields.onDomainChange === 'function') fields.onDomainChange();
            }
            if (fields.authorEl && !fields.authorEl.value.trim() && meta.author) {
                fields.authorEl.value = meta.author;
            }
            if (fields.descEl && !fields.descEl.value.trim() && meta.description) {
                fields.descEl.value = meta.description;
            }
            return meta;
        }

        function openRuleEditor(existingRule) {
            const panel = document.getElementById('ruleEditorPanel');
            panel.classList.add('open');
            if (existingRule) {
                editingRuleId = existingRule.id;
                document.getElementById('ruleNameInput').value = existingRule.name || '';
                document.getElementById('ruleDomainInput').value = existingRule.domain || '*';
                document.getElementById('ruleCssInput').value = existingRule.css || '';
                document.getElementById('ruleJsInput').value = existingRule.js || '';
            } else {
                editingRuleId = null;
                document.getElementById('ruleNameInput').value = '';
                document.getElementById('ruleDomainInput').value = '*';
                document.getElementById('ruleCssInput').value = '';
                document.getElementById('ruleJsInput').value = '';
            }
            document.getElementById('ruleNameInput').focus();
        }

        function renderKeybindsGrid() {
            const grid = document.getElementById('keybindsGrid');
            grid.textContent = '';
            const zh = isZh();
            const frag = document.createDocumentFragment();

            KEYBIND_DEFS.forEach(([actionKey, enLabel, enHint, zhLabel, zhHint], idx) => {
                const row = document.createElement('div');
                row.className = 'set-row';
                if (idx === KEYBIND_DEFS.length - 1) row.style.borderBottom = 'none';

                const meta = document.createElement('div');
                meta.className = 'set-meta';
                const lbl = document.createElement('div');
                lbl.className = 'set-label';
                lbl.textContent = zh ? (zhLabel || enLabel) : enLabel;
                const desc = document.createElement('div');
                desc.className = 'set-hint';
                desc.textContent = zh ? (zhHint || enHint) : enHint;
                meta.appendChild(lbl);
                meta.appendChild(desc);

                const ctrl = document.createElement('div');
                ctrl.className = 'set-ctrl';
                const btn = document.createElement('button');
                btn.type = 'button';
                const isRec = recordingActionKey === actionKey;
                btn.className = isRec ? 'kb-record-btn recording' : 'kb-record-btn';
                btn.textContent = isRec
                    ? (zh ? '请按下快捷键… (Esc 取消)' : 'Press keys… (Esc)')
                    : (keybinds[actionKey] || DEFAULT_KEYBINDS[actionKey] || (zh ? '未绑定' : 'Unbound'));
                btn.title = zh ? '点击后直接在键盘上按下新的组合键' : 'Click to record a new shortcut';
                btn.onclick = () => {
                    recordingActionKey = actionKey;
                    window.__fynexo_recording_keybind__ = true;
                    renderKeybindsGrid();
                };
                ctrl.appendChild(btn);

                row.appendChild(meta);
                row.appendChild(ctrl);
                frag.appendChild(row);
            });

            grid.appendChild(frag);
        }

        window.addEventListener('keydown', (e) => {
            if (!recordingActionKey) return;
            e.preventDefault();
            e.stopPropagation();

            if (e.key === 'Escape') {
                recordingActionKey = null;
                window.__fynexo_recording_keybind__ = false;
                renderKeybindsGrid();
                return;
            }
            if (['Control', 'Shift', 'Alt', 'Meta'].includes(e.key)) return;

            const parts = [];
            if (e.ctrlKey || e.metaKey) parts.push('Ctrl');
            if (e.altKey) parts.push('Alt');
            if (e.shiftKey && e.key !== '+' && e.key !== '=') parts.push('Shift');

            let k = e.key;
            if (k === ' ') k = 'Space';
            else if (k === '+') k = '=';
            else if (k.length === 1) k = k.toUpperCase();

            parts.push(k);
            const combo = parts.join('+');
            keybinds[recordingActionKey] = combo;
            recordingActionKey = null;
            window.__fynexo_recording_keybind__ = false;
            renderKeybindsGrid();
            sendIpc({ type: 'SAVE_KEYBINDS', keybinds: keybinds });
            showToast(`Bound shortcut: ${combo}`);
        }, true);

        const SECTION_I18N = {
            'sec-appearance': {
                navZh: '主题与配色',
                navEn: 'Appearance',
                titleZh: '主题与配色',
                titleEn: 'Appearance',
                descZh: '直接挑选一个喜欢的皮肤主题，或开启“自定义调色盘”自由调整颜色。',
                descEn: 'Pick a built-in palette or import *.json themes.'
            },
            'sec-styles': {
                navZh: '界面与起始页',
                navEn: 'Component studio',
                titleZh: '界面布局与新标签页样式',
                titleEn: 'Component studio',
                prefixZh: '可视化调整顶栏/侧边栏与新标签页外观，或在右侧直接编辑 ',
                prefixEn: 'Tune chrome geometry visually or inspect raw '
            },
            'sec-general': {
                navZh: '浏览与习惯',
                navEn: 'Browsing & tabs',
                titleZh: '浏览与基础习惯',
                titleEn: 'Browsing & tabs',
                descZh: '设置新标签页打开地址、网页默认大小、设备伪装，以及重启是否恢复网页。',
                descEn: 'Homepage, User-Agent spoofing, page zoom, Zero-FX rendering, and session restore.'
            },
            'sec-search': {
                navZh: '搜索与 !指令',
                navEn: 'Search & !bangs',
                titleZh: '搜索引擎与 !快捷指令',
                titleEn: 'Search & !bangs',
                descZh: '切换默认搜索引擎，或在地址栏用 !b 搜 B 站、!tr 翻译等快速搜索指令。',
                descEn: 'Pick a default search engine or define custom !bang quick-search triggers.'
            },
            'sec-bookmarks': {
                navZh: '常用网站',
                navEn: 'Shortcuts',
                titleZh: '常用网站快捷卡片',
                titleEn: 'Shortcuts'
            },
            'sec-shortcuts': {
                navZh: '键盘快捷键',
                navEn: 'Keybinds',
                titleZh: '键盘快捷键',
                titleEn: 'Keybinds',
                descZh: '点击右侧任意按键框，直接在键盘上按下新的组合键即可完成改键。',
                descEn: 'Click any keycap on the right and press your combination to rebind in config/keybinds.json.'
            },
            'sec-rules': {
                navZh: '广告拦截盾',
                navEn: 'Adblock shield',
                titleZh: '广告与追踪拦截盾',
                titleEn: 'Adblock shield',
                descZh: '自动过滤网页横幅广告与跟踪器；如个别网站显示异常可添加到放行名单。',
                descEn: 'Core network ad & tracker interception synced to adblock.json (independent of Built-in Extensions).'
            },
            'sec-privacy': {
                navZh: '隐私与备份',
                navEn: 'Privacy & data',
                titleZh: '隐私、清理与整包备份',
                titleEn: 'Privacy & data',
                descZh: '开启无痕模式、清理浏览历史与缓存，或一键导出所有个性化设置。',
                descEn: 'Incognito mode, browsing history, cache/cookies cleanup, profile paths, and setup backup.'
            },
            'sec-extensions': {
                navZh: '内建扩展与脚本',
                navEn: '内建 Extension',
                titleZh: '内建扩展与 ScriptForge 用户脚本',
                titleEn: '内建 Extension',
                descZh: '所有扩展按需开启（关闭时零内存占用），并内置用户脚本引擎与社区分享商店。',
                descEn: 'Opt-in memory-heavy modules (off by default), custom scripts, & on-demand cloud store.'
            }
        };

        const VISUAL_ROW_I18N = {
            qsDeskTopToggle: ['显示桌面端顶部栏', 'Show desktop topbar', '控制电脑端页面最上方的浏览栏是否显示（关闭即进入无边框极简模式）。'],
            qsTopbarLayout: ['顶栏 / 侧边栏布局方式', 'Header layout & shortcuts bar', '选双栏、单行紧凑或左侧竖排标签栏，并设置收藏网站栏何时显示。'],
            qsTopbarHeight: ['顶栏与边栏尺寸', 'Topbar, titlebar & vertical sidebar dimensions', '依次设置导航条高度、顶部标签行高度与左侧竖排边栏宽度。'],
            qsAddrMode: ['地址栏显示位置与宽度', 'Address bar style, alignment & width', '选择直接在顶部输入或居中弹窗搜索，支持靠左 / 居中 / 靠右对齐。'],
            qsBtnCopyToggle: ['顶栏快捷图标开关', 'Toolbar buttons visibility', '不想让顶栏太挤时，可以随时隐藏复制网址、收藏星标、盾牌或下载图标。'],
            qsTabHeight: ['标签页大小与圆角', 'Tab geometry (height · width · radius · gap)', '自定义标签卡片的高度、宽度、圆角弧度和标签之间的间隔。'],
            qsTabDotToggle: ['网站图标与设置页宽屏模式', 'Domain monogram badge & workspace width', '开关标签上的网站小图标，以及是否让设置界面铺满较宽窗口。'],
            qsMobilePos: ['手机端底栏停靠位置', 'Mobile dock position & height', '在手机上可将控制胶囊放在底部单手操作，或放到屏幕顶部。'],
            qsNtTitle: ['起始页标题与快捷卡片列数', 'Hero title & grid columns', '保持默认“Fynexo”即显示极简大时钟；改成其他标题则会显示自定义欢迎语。'],
            qsNtSub: ['起始页副标题', 'Hero subtitle', '显示在起始页主标题下方的小字说明。'],
            qsNtClockToggle: ['起始页大时钟与小时制', 'Clock & 24-hour format', '控制是否在新标签页显示时间，以及切换 24 小时制或 12 小时制。'],
            qsNtScToggle: ['起始页显示模块', 'Start page sections', '自由控制是否在新标签页展示“常用网站卡片”和“最近访问记录”。'],
            qsNtWallpaper: ['起始页壁纸与背景遮罩', 'Start page wallpaper & overlay', '点下方色块一键换肤，也可以上传本地图片或调节背景暗度与模糊程度。']
        };

        const NO_CODE_BLOCKS = {
            unblock_copy: {
                chipZh: '🔓 解除禁止复制与右键',
                chipEn: '🔓 Unblock Copy & Right-Click',
                nameZh: '解除禁止复制与右键',
                nameEn: 'Unblock Copy & Right-Click',
                css: [
                    '/* Unblock text selection & copy */',
                    '*, *::before, *::after {',
                    '  -webkit-user-select: text !important;',
                    '  user-select: text !important;',
                    '}'
                ].join('\n'),
                jsBody: [
                    '(function () {',
                    "  'use strict';",
                    "  const events = ['copy', 'cut', 'selectstart', 'contextmenu'];",
                    '  events.forEach((evt) => {',
                    '    document.addEventListener(evt, (e) => e.stopPropagation(), true);',
                    '  });',
                    '})();'
                ].join('\n')
            },
            links_newtab: {
                chipZh: '🔗 外链强制新标签打开',
                chipEn: '🔗 Open External Links in New Tab',
                nameZh: '外链强制新标签页打开',
                nameEn: 'Open External Links in New Tab',
                css: '',
                jsBody: [
                    '(function () {',
                    "  'use strict';",
                    '  const retarget = () => {',
                    "    document.querySelectorAll('a[href]').forEach((a) => {",
                    "      if (!/^https?:/i.test(a.href)) return;",
                    '      try {',
                    '        const u = new URL(a.href);',
                    '        if (u.origin !== window.location.origin) {',
                    "          a.target = '_blank';",
                    "          a.rel = 'noopener noreferrer';",
                    '        }',
                    '      } catch (_) {}',
                    '    });',
                    '  };',
                    '  retarget();',
                    '  new MutationObserver(retarget).observe(document.documentElement, { childList: true, subtree: true });',
                    '})();'
                ].join('\n')
            },
            clean_font: {
                chipZh: '📖 舒适阅读字体与行距',
                chipEn: '📖 Clean Reading Typography',
                nameZh: '舒适阅读排版增强',
                nameEn: 'Clean Reading Typography',
                css: [
                    '/* Clean comfortable reading line-height */',
                    'article, main, p, li {',
                    '  line-height: 1.75 !important;',
                    '  letter-spacing: 0.01em !important;',
                    '}'
                ].join('\n'),
                jsBody: ''
            },
            scroll_top_btn: {
                chipZh: '⬆️ 悬浮回到顶部按钮',
                chipEn: '⬆️ Floating Back-to-Top Button',
                nameZh: '悬浮一键回到顶部',
                nameEn: 'Floating Back-to-Top Button',
                css: '',
                jsBody: [
                    '(function () {',
                    "  'use strict';",
                    "  if (document.getElementById('sf-scroll-top-btn')) return;",
                    "  const btn = document.createElement('button');",
                    "  btn.id = 'sf-scroll-top-btn';",
                    "  btn.type = 'button';",
                    "  btn.textContent = '↑';",
                    "  btn.title = 'Back to top';",
                    '  btn.style.cssText =',
                    "    'position:fixed;right:20px;bottom:76px;z-index:2147483000;width:38px;height:38px;' +",
                    "    'border-radius:999px;border:1px solid rgba(255,255,255,0.2);background:rgba(20,20,24,0.82);' +",
                    "    'color:#fff;font-size:16px;font-weight:700;cursor:pointer;display:none;box-shadow:0 6px 18px rgba(0,0,0,0.3);';",
                    "  btn.onclick = () => window.scrollTo({ top: 0, behavior: 'smooth' });",
                    "  const sync = () => { btn.style.display = window.scrollY > 360 ? 'block' : 'none'; };",
                    "  window.addEventListener('scroll', sync, { passive: true });",
                    '  (document.body || document.documentElement).appendChild(btn);',
                    "  if (typeof GM_registerMenuCommand === 'function') {",
                    "    GM_registerMenuCommand('Back to top', () => window.scrollTo({ top: 0, behavior: 'smooth' }));",
                    '  }',
                    '  sync();',
                    '})();'
                ].join('\n'),
                grants: ['GM_registerMenuCommand']
            },
            hide_popups: {
                chipZh: '🧹 屏蔽常见悬浮弹窗遮罩',
                chipEn: '🧹 Hide Sticky Popups & App Banners',
                nameZh: '屏蔽悬浮弹窗与 App 引导横幅',
                nameEn: 'Hide Sticky Popups & App Banners',
                css: [
                    '/* Hide common floating App download / login overlay banners */',
                    '[class*="open-in-app"], [class*="app-banner"], [class*="download-banner"],',
                    '[class*="login-modal"], [id*="app-banner"], [id*="openApp"] {',
                    '  display: none !important;',
                    '}',
                    'html, body { overflow: auto !important; }'
                ].join('\n'),
                jsBody: ''
            },
            auto_click_skip: {
                chipZh: '⚡ 自动点击跳过/展开按钮',
                chipEn: '⚡ Auto-Expand / Skip Prompts',
                nameZh: '自动展开全文与跳过弹窗',
                nameEn: 'Auto-Expand / Skip Prompts',
                css: '',
                jsBody: [
                    '(function () {',
                    "  'use strict';",
                    '  const labelRe = /^(?:展开全文|阅读全文|查看全部|跳过|跳过广告|关闭广告|Read more|Show more|Skip ad)$/i;',
                    '  const scan = () => {',
                    "    document.querySelectorAll('button, a, [role=\"button\"]').forEach((el) => {",
                    "      if (el.getAttribute('data-sf-clicked')) return;",
                    "      const txt = (el.textContent || '').trim();",
                    '      if (labelRe.test(txt)) {',
                    "        el.setAttribute('data-sf-clicked', '1');",
                    '        el.click();',
                    '      }',
                    '    });',
                    '  };',
                    '  scan();',
                    '  new MutationObserver(scan).observe(document.documentElement, { childList: true, subtree: true });',
                    '})();'
                ].join('\n')
            }
        };

        function buildUserscriptHeader(nm, domRaw, grants) {
            const cleanHost = String(domRaw || '*').replace(/^https?:\/\//i, '').replace(/\/.*$/, '').trim();
            const matchPat = (!cleanHost || cleanHost === '*') ? '*://*/*' : `*://${cleanHost}/*`;
            const lines = [
                '// ==UserScript==',
                `// @name         ${nm || 'My ScriptForge Script'}`,
                '// @version      1.0.0',
                '// @description  Custom page enhancement powered by Fynexo ScriptForge',
                '// @author       You',
                `// @match        ${matchPat}`,
                '// @run-at       document-end'
            ];
            (grants || []).forEach((g) => {
                lines.push(`// @grant        ${g}`);
            });
            lines.push('// ==/UserScript==');
            return lines.join('\n');
        }

        function applyNoCodeBlockToInputs(blockKey, opts) {
            const block = NO_CODE_BLOCKS[blockKey];
            if (!block || !opts) return;
            const zh = isZh();
            if (opts.nameInput && !opts.nameInput.value.trim()) {
                opts.nameInput.value = zh ? block.nameZh : block.nameEn;
            }
            if (opts.domainInput && !opts.domainInput.value.trim()) {
                opts.domainInput.value = '*';
            }
            if (block.css && opts.cssInput) {
                const curCss = opts.cssInput.value || '';
                if (!curCss.includes(block.css.trim())) {
                    opts.cssInput.value = curCss.trim()
                        ? `${curCss.trimEnd()}\n\n${block.css}`
                        : block.css;
                }
            }
            if (block.jsBody && opts.jsInput) {
                const curJs = opts.jsInput.value || '';
                if (!curJs.includes(block.jsBody.trim())) {
                    if (/\/\/\s*==UserScript==/.test(curJs)) {
                        opts.jsInput.value = `${curJs.trimEnd()}\n\n${block.jsBody}\n`;
                    } else {
                        const header = buildUserscriptHeader(
                            opts.nameInput ? opts.nameInput.value.trim() : (zh ? block.nameZh : block.nameEn),
                            opts.domainInput ? opts.domainInput.value.trim() : '*',
                            block.grants
                        );
                        opts.jsInput.value = curJs.trim()
                            ? `${header}\n\n${curJs.trimEnd()}\n\n${block.jsBody}\n`
                            : `${header}\n\n${block.jsBody}\n`;
                    }
                }
            }
            if (typeof opts.onChanged === 'function') opts.onChanged();
            showToast(zh ? `已填入积木：${block.nameZh}` : `Added block: ${block.nameEn}`);
        }

        function setPlainRowI18n(el, zhText, enText) {
            if (!el) return;
            if (!el.dataset.enText) el.dataset.enText = el.textContent;
            el.textContent = isZh() ? zhText : (enText || el.dataset.enText);
        }

        function applySettingsLocalization() {
            const zh = isZh();
            const brandTitle = document.querySelector('.brand-title');
            if (brandTitle) brandTitle.textContent = zh ? '偏好设置' : 'Preferences';

            const langBtn = document.getElementById('btnSettingsLang');
            if (langBtn) {
                langBtn.textContent = zh ? '中文 · 切换 EN' : 'EN · 切换中文';
            }

            const navGroups = document.querySelectorAll('.nav-group-label');
            const zhGroups = ['界面与外观', '日常浏览', '隐私与拦截'];
            const enGroups = ['Interface', 'Browsing', 'Privacy & Shield'];
            navGroups.forEach((g, idx) => {
                if (zhGroups[idx]) g.textContent = zh ? zhGroups[idx] : enGroups[idx];
            });

            Object.keys(SECTION_I18N).forEach((secId) => {
                const info = SECTION_I18N[secId];
                const navBtn = document.querySelector(`.nav-link[data-sec="${secId}"] span:not(.nav-pill-count)`);
                if (navBtn) navBtn.textContent = zh ? info.navZh : info.navEn;

                const secEl = document.getElementById(secId);
                if (!secEl) return;
                const titleEl = secEl.querySelector('.sec-title');
                if (titleEl) titleEl.textContent = zh ? info.titleZh : info.titleEn;

                const descEl = secEl.querySelector('.sec-desc');
                if (descEl) {
                    if (secId === 'sec-styles' && descEl.firstChild && descEl.firstChild.nodeType === 3) {
                        descEl.firstChild.textContent = zh ? info.prefixZh : info.prefixEn;
                    } else if (info.descZh && info.descEn && descEl.id !== 'shortcutsHint') {
                        descEl.textContent = zh ? info.descZh : info.descEn;
                    }
                }
            });

            const studioGroups = document.querySelectorAll('#pane-studio-visual .set-group-title');
            const zhStudioGroups = ['桌面顶栏与工具按钮', '标签栏与手机端底栏', '新标签起始页 (fynexo://newtab)'];
            const enStudioGroups = ['Desktop Topbar & Omnibox', 'Tabs & Mobile Dock', 'Start Page (fynexo://newtab)'];
            studioGroups.forEach((el, idx) => {
                if (zhStudioGroups[idx]) el.textContent = zh ? zhStudioGroups[idx] : enStudioGroups[idx];
            });

            Object.keys(VISUAL_ROW_I18N).forEach((ctrlId) => {
                const ctrlEl = document.getElementById(ctrlId);
                const row = ctrlEl && ctrlEl.closest('.set-row');
                if (!row) return;
                const [lblZh, lblEn, hintZh] = VISUAL_ROW_I18N[ctrlId];
                setPlainRowI18n(row.querySelector('.set-label'), lblZh, lblEn);
                setPlainRowI18n(row.querySelector('.set-hint'), hintZh, null);
            });

            setPlainRowI18n(document.getElementById('lblRuleNoCode'), '零代码积木工坊（点击一键填入）', 'No-code building blocks (click to add)');
            setPlainRowI18n(document.getElementById('tagRuleNoCode'), '零代码', 'Zero-code');
            setPlainRowI18n(document.getElementById('lblPubNoCode'), '零代码积木工坊（点击一键填入）', 'No-code building blocks (click to add)');
            setPlainRowI18n(document.getElementById('tagPubNoCode'), '零代码', 'Zero-code');
            setPlainRowI18n(document.getElementById('lblRuleCss'), '自定义页面样式 (CSS)', 'Custom CSS');
            setPlainRowI18n(document.getElementById('lblRuleJs'), '用户脚本 / JS（ScriptForge）', 'Userscript / Custom JS (ScriptForge)');
            setPlainRowI18n(document.getElementById('btnSaveRule'), '保存扩展', 'Save extension');
            setPlainRowI18n(document.getElementById('btnInsertUserscriptTpl'), '插入 GM_* 油猴模板', 'Insert GM_* template');
            setPlainRowI18n(document.getElementById('btnCancelRule'), '取消', 'Cancel');
            setPlainRowI18n(document.getElementById('btnNewRule'), '+ 新建扩展', '+ Add Extension');
            setPlainRowI18n(document.getElementById('btnImportUserscript'), '+ 导入 .user.js', '+ Import .user.js');

            document.querySelectorAll('#ruleNoCodeBar [data-nocode], #pubNoCodeBar [data-pub-nocode]').forEach((chip) => {
                const key = chip.dataset.nocode || chip.dataset.pubNocode;
                const b = key && NO_CODE_BLOCKS[key];
                if (b) chip.textContent = zh ? b.chipZh : b.chipEn;
            });
        }

        function syncFormControls() {
            const zh = isZh();
            document.getElementById('themesDirLabel').textContent = themesDir;
            document.getElementById('stylesPathLabel').textContent = stylesPath;
            const pathDisplay = document.getElementById('stylesPathDisplay');
            if (pathDisplay) pathDisplay.textContent = stylesPath;
            document.getElementById('dataDirLabel').textContent = dataDir || stylesPath;
            document.getElementById('shortcutsHint').textContent = zh
                ? `已同步 ${bookmarks.length} 个常用网站卡片（保存在 shortcuts.json）。`
                : `${bookmarks.length} saved shortcut${bookmarks.length === 1 ? '' : 's'} synced to shortcuts.json.`;

            document.getElementById('inputHomepage').value = config.homepage || 'fynexo://newtab';

            const uaMode = (config.user_agent_mode || 'default').toLowerCase();
            document.querySelectorAll('#uaModeGroup .seg-btn').forEach(b => {
                b.classList.toggle('active', b.dataset.ua === uaMode);
            });
            const customUaInp = document.getElementById('inputCustomUa');
            customUaInp.style.display = uaMode === 'custom' ? 'block' : 'none';
            customUaInp.value = config.custom_user_agent || '';

            const zoomPct = Math.round((Number(config.default_zoom) || 1) * 100);
            document.getElementById('btnZoomReset').textContent = `${zoomPct}%`;

            setSwitchState('btnIncognitoToggle', !!config.incognito);

            const perfOn = !!config.enforce_perf;
            setSwitchState('btnPerfToggle', perfOn);

            const adOn = config.adblock !== false;
            setSwitchState('btnAdblockToggle', adOn);
            document.getElementById('adblockHint').textContent = zh
                ? (adOn
                    ? `自动拦截 65+ 常见广告/跟踪域名与自定义黑名单（本次浏览已拦截 ${blockedAds} 次）。`
                    : '网络级广告与追踪拦截目前已暂停。')
                : (adOn
                    ? `Blocks 65+ built-in ad/telemetry domains plus custom rules (${blockedAds} blocked this session).`
                    : 'Network ad & tracker interception is currently paused.');

            setSwitchState('btnRestoreToggle', config.restore_session !== false);

            document.getElementById('btnClearHistory').textContent = historyCount > 0
                ? (zh ? `清空历史记录 (${historyCount})` : `Clear history (${historyCount})`)
                : (zh ? '清空历史记录' : 'Clear history');

            const tb = styles.topbar || {};
            setSwitchState('qsDeskTopToggle', tb.visible_on_desktop !== false);
            const tbLayout = (tb.layout || 'deck').toLowerCase();
            document.getElementById('qsTopbarLayout').value =
                (tbLayout === 'compact' || tbLayout === 'vertical') ? tbLayout : 'deck';
            document.getElementById('qsBookmarksBar').value = ['always', 'newtab', 'hidden'].includes((tb.bookmarks_bar || '').toLowerCase()) ? tb.bookmarks_bar.toLowerCase() : 'always';
            document.getElementById('qsTopbarHeight').value = tb.height || '40px';
            const qsTitlebarInput = document.getElementById('qsTitlebarHeight');
            if (qsTitlebarInput) qsTitlebarInput.value = tb.titlebar_height || '42px';
            const qsVertW = document.getElementById('qsVertWidth');
            if (qsVertW) qsVertW.value = tb.vertical_width || '216px';
            setSwitchState('qsBtnCopyToggle', tb.show_copy_btn !== false);
            setSwitchState('qsBtnStarToggle', tb.show_star_btn !== false);
            setSwitchState('qsBtnShieldToggle', tb.show_shield_btn !== false);
            setSwitchState('qsBtnDlToggle', tb.show_downloads_btn !== false);

            if (styles.address_bar) {
                document.getElementById('qsAddrMode').value = (styles.address_bar.mode || 'inline').toLowerCase() === 'modal' ? 'modal' : 'inline';
                document.getElementById('qsAddrPos').value = styles.address_bar.position || 'center';
                document.getElementById('qsAddrWidth').value = styles.address_bar.width || '340px';
            }
            if (styles.tabs) {
                const tabH = document.getElementById('qsTabHeight');
                if (tabH) tabH.value = styles.tabs.height || '36px';
                document.getElementById('qsTabWidth').value = styles.tabs.width || '152px';
                document.getElementById('qsTabRadius').value = styles.tabs.border_radius || '7px';
                const tabGap = document.getElementById('qsTabGap');
                if (tabGap) tabGap.value = styles.tabs.gap || '0px';
                setSwitchState('qsTabDotToggle', styles.tabs.show_indicator_dot !== false);
            }
            if (styles.mobile_bar) {
                document.getElementById('qsMobilePos').value = styles.mobile_bar.position || 'bottom';
                document.getElementById('qsMobileHeight').value = styles.mobile_bar.height || '54px';
            }
            const fullW = !styles.settings_page || styles.settings_page.full_width !== false;
            setSwitchState('qsFullWidthToggle', fullW);

            const nt = styles.newtab || {};
            document.getElementById('qsNtTitle').value = nt.title !== undefined ? nt.title : 'Fynexo';
            document.getElementById('qsNtSub').value = nt.subtitle !== undefined ? nt.subtitle : 'Single-WebView studio browser · Zero-FX compositing';
            document.getElementById('qsNtCols').value = String(Number(nt.shortcuts_columns) || 0);
            setSwitchState('qsNtClockToggle', nt.show_clock !== false);
            setSwitchState('qsNt24hToggle', nt.use_24h_clock !== false);
            setSwitchState('qsNtScToggle', nt.show_shortcuts !== false);
            setSwitchState('qsNtHistToggle', nt.show_history !== false);
            const qsNtWp = document.getElementById('qsNtWallpaper');
            if (qsNtWp) qsNtWp.value = nt.background_image || '';
            const qsNtDim = document.getElementById('qsNtDim');
            if (qsNtDim) qsNtDim.value = String(nt.background_dim !== undefined ? nt.background_dim : 45);
            const qsNtBlur = document.getElementById('qsNtBlur');
            if (qsNtBlur) qsNtBlur.value = String(nt.background_blur || 0);

            const editor = document.getElementById('jsonEditor');
            if (document.activeElement !== editor) {
                editor.value = JSON.stringify(styles, null, 2);
            }
            setJsonStatus(false, isZh()
                ? '就绪 — 可直接在右侧修改 JSON（Ctrl+S 生效）。'
                : 'Ready — edit JSON directly (Ctrl+S to apply).');
            applySettingsLocalization();
        }

        function setJsonStatus(isError, msg) {
            const box = document.getElementById('jsonStatus');
            const txt = document.getElementById('jsonStatusText');
            box.classList.toggle('error', isError);
            txt.textContent = msg;
        }

        function saveConfig(msg) {
            sendIpc({ type: 'SAVE_CONFIG', config: config });
            showToast(msg || 'Preferences saved');
        }

        function pushStyles(msg) {
            document.getElementById('jsonEditor').value = JSON.stringify(styles, null, 2);
            applySettingsSurface();
            syncFormControls();
            sendIpc({ type: 'SAVE_STYLES', styles: styles });
            setJsonStatus(false, 'Applied live and persisted to styles.json.');
            showToast(msg || 'Styles updated');
        }

        function bindJsonFilePicker(btnId, inputId, ipcType, extraProps) {
            const inp = document.getElementById(inputId);
            document.getElementById(btnId).onclick = () => inp.click();
            inp.addEventListener('change', (e) => {
                const files = Array.from(e.target.files || []);
                files.forEach(file => {
                    const reader = new FileReader();
                    reader.onload = () => {
                        sendIpc(Object.assign({
                            type: ipcType,
                            json: String(reader.result || ''),
                            filename: file.name || 'custom.json'
                        }, extraProps || {}));
                    };
                    reader.readAsText(file, 'utf-8');
                });
                e.target.value = '';
            });
        }

        document.getElementById('btnExportBundle').onclick = () => sendIpc({ type: 'EXPORT_FULL_SETUP' });
        bindJsonFilePicker('btnImportBundle', 'fileImportBundle', 'IMPORT_FULL_SETUP_JSON');

        bindJsonFilePicker('btnImportTheme', 'fileImportTheme', 'IMPORT_THEME_JSON');
        document.getElementById('btnExportTheme').onclick = () => sendIpc({ type: 'EXPORT_THEME', id: config.theme });
        document.getElementById('btnOpenThemesDir').onclick = () => sendIpc({ type: 'OPEN_THEMES_DIR' });
        document.getElementById('btnCopyThemesDir').onclick = () => {
            if (navigator.clipboard && typeof navigator.clipboard.writeText === 'function') {
                navigator.clipboard.writeText(themesDir).then(() => {
                    showToast('Copied themes folder path');
                }).catch(() => sendIpc({ type: 'OPEN_THEMES_DIR' }));
            } else {
                sendIpc({ type: 'OPEN_THEMES_DIR' });
            }
        };

        document.getElementById('btnCustomColorsToggle').onclick = () => {
            styles.colors = styles.colors || {};
            if (!styles.colors.use_custom_colors) {
                const cur = resolveThemePalette();
                styles.colors = Object.assign({
                    bg: cur.bg, panel: cur.panel, surface: cur.surface, elevated: cur.elevated,
                    border: '#222228', border_hover: '#34343e', fg: cur.fg,
                    secondary: cur.secondary, muted: cur.muted, accent: cur.accent,
                    accent_bg: cur.accentBg, hover: cur.hover
                }, styles.colors, { use_custom_colors: true });
            } else {
                styles.colors.use_custom_colors = false;
            }
            renderColorTokensStudio();
            pushStyles(styles.colors.use_custom_colors ? 'Visual color designer enabled' : 'Using JSON theme preset');
        };

        document.getElementById('btnSaveAsTheme').onclick = () => {
            const rawName = document.getElementById('inputNewThemeName').value.trim() || 'My Studio Theme';
            const slug = rawName.toLowerCase().replace(/[^a-z0-9_-]+/g, '-').replace(/^-|-$/g, '') || 'custom-theme';
            const presetObj = {
                id: slug,
                name: rawName,
                description: 'Created in Fynexo Visual Palette Designer',
                colors: styles.colors || resolveThemePalette()
            };
            sendIpc({
                type: 'IMPORT_THEME_JSON',
                json: JSON.stringify(presetObj, null, 2),
                filename: slug + '.json'
            });
            document.getElementById('inputNewThemeName').value = '';
        };

        document.getElementById('btnAddEngine').onclick = () => {
            const nm = document.getElementById('inputEngName').value.trim();
            const u = document.getElementById('inputEngUrl').value.trim();
            if (!nm || !u) return showToast('Enter engine name and URL');
            const id = nm.toLowerCase().replace(/[^a-z0-9_-]+/g, '-') || ('eng-' + Date.now());
            const list = getEngines().slice();
            const fullUrl = /^https?:\/\//i.test(u) ? u : ('https://' + u);
            const existingIdx = list.findIndex(x => x.id === id);
            if (existingIdx >= 0) list[existingIdx] = { id, name: nm, url: fullUrl };
            else list.push({ id, name: nm, url: fullUrl });
            config.search_engines = list;
            config.search_engine = id;
            document.getElementById('inputEngName').value = '';
            document.getElementById('inputEngUrl').value = '';
            renderEngineTable();
            saveConfig(`Added search engine: ${nm}`);
        };

        document.getElementById('btnAddBang').onclick = () => {
            let trig = document.getElementById('inputBangTrig').value.trim().toLowerCase();
            const nm = document.getElementById('inputBangName').value.trim();
            const u = document.getElementById('inputBangUrl').value.trim();
            if (!trig || !u) return showToast('Enter !trigger and search URL');
            if (!trig.startsWith('!')) trig = '!' + trig;
            const fullUrl = /^https?:\/\//i.test(u) ? u : ('https://' + u);
            const list = getBangs().filter(x => (x.trigger || '').toLowerCase() !== trig);
            list.push({ trigger: trig, name: nm || trig.slice(1), url: fullUrl });
            config.bangs = list;
            document.getElementById('inputBangTrig').value = '';
            document.getElementById('inputBangName').value = '';
            document.getElementById('inputBangUrl').value = '';
            renderBangsTable();
            saveConfig(`Saved bang ${trig}`);
        };

        const inputHp = document.getElementById('inputHomepage');
        inputHp.onchange = () => {
            const val = inputHp.value.trim();
            if (val) { config.homepage = val; saveConfig('Homepage updated'); }
        };
        document.querySelectorAll('.preset-chip').forEach(chip => {
            chip.onclick = () => {
                const hp = chip.dataset.hp;
                if (hp) { config.homepage = hp; inputHp.value = hp; saveConfig(`Homepage: ${hp}`); }
            };
        });

        document.querySelectorAll('#uaModeGroup .seg-btn').forEach(btn => {
            btn.onclick = () => {
                config.user_agent_mode = btn.dataset.ua;
                syncFormControls();
                saveConfig(`User-Agent mode: ${btn.textContent}`);
            };
        });
        document.getElementById('inputCustomUa').onchange = (e) => {
            config.custom_user_agent = e.target.value.trim();
            saveConfig('Custom User-Agent updated');
        };
        document.getElementById('btnZoomOut').onclick = () => sendIpc({ type: 'SET_ZOOM', delta: -0.1 });
        document.getElementById('btnZoomReset').onclick = () => sendIpc({ type: 'SET_ZOOM', zoom: 1.0 });
        document.getElementById('btnZoomIn').onclick = () => sendIpc({ type: 'SET_ZOOM', delta: 0.1 });

        bindJsonFilePicker('btnImportShortcuts', 'fileImportShortcuts', 'IMPORT_SHORTCUTS_JSON', { merge: true });
        document.getElementById('btnExportShortcuts').onclick = () => sendIpc({ type: 'EXPORT_SHORTCUTS' });
        document.getElementById('btnResetShortcuts').onclick = () => sendIpc({ type: 'RESET_SHORTCUTS' });
        document.getElementById('btnAddShortcut').onclick = () => {
            const u = document.getElementById('inputScUrl').value.trim();
            const t = document.getElementById('inputScTitle').value.trim();
            if (!u) return showToast('Enter shortcut URL');
            sendIpc({ type: 'ADD_BOOKMARK', title: t, url: u });
            document.getElementById('inputScTitle').value = '';
            document.getElementById('inputScUrl').value = '';
        };

        document.getElementById('btnIncognitoToggle').onclick = () => {
            config.incognito = !config.incognito;
            syncFormControls();
            saveConfig(config.incognito ? 'Private mode active' : 'Private mode off');
        };
        document.getElementById('btnPerfToggle').onclick = () => {
            config.enforce_perf = !config.enforce_perf;
            syncFormControls();
            saveConfig(config.enforce_perf ? 'Zero-FX filter enabled' : 'Zero-FX filter disabled');
        };
        document.getElementById('btnRestoreToggle').onclick = () => {
            config.restore_session = !(config.restore_session !== false);
            syncFormControls();
            saveConfig(config.restore_session ? 'Session restore enabled' : 'Session restore disabled');
        };

        document.getElementById('btnClearHistory').onclick = () => {
            sendIpc({ type: 'CLEAR_HISTORY' });
            historyCount = 0;
            syncFormControls();
        };
        document.getElementById('btnClearSiteData').onclick = () => { sendIpc({ type: 'CLEAR_SITE_DATA' }); };
        document.getElementById('btnOpenDataDir').onclick = () => sendIpc({ type: 'OPEN_DATA_DIR' });
        document.getElementById('btnOpenDownloadsDir').onclick = () => sendIpc({ type: 'OPEN_DOWNLOADS_DIR' });

        bindJsonFilePicker('btnImportAdblock', 'fileImportAdblock', 'IMPORT_ADBLOCK_JSON');
        document.getElementById('btnExportAdblock').onclick = () => sendIpc({ type: 'EXPORT_ADBLOCK' });
        document.getElementById('btnAdblockToggle').onclick = () => {
            config.adblock = !(config.adblock !== false);
            syncFormControls();
            saveConfig(config.adblock ? 'Network adblock enabled' : 'Network adblock paused');
        };
        document.getElementById('btnAddBlockDomain').onclick = () => {
            const inp = document.getElementById('inputBlockDomain');
            const v = inp.value.trim().toLowerCase().replace(/^https?:\/\//, '').split('/')[0];
            if (!v || !v.includes('.')) return showToast('Enter a valid domain (e.g. ads.example.com)');
            adblockCfg.custom_blocked_domains = adblockCfg.custom_blocked_domains || [];
            if (!adblockCfg.custom_blocked_domains.includes(v)) adblockCfg.custom_blocked_domains.push(v);
            inp.value = '';
            renderAdblockDomainLists();
            sendIpc({ type: 'SAVE_ADBLOCK', adblock: adblockCfg });
            showToast(`Blocked ${v}`);
        };
        document.getElementById('btnAddWhiteDomain').onclick = () => {
            const inp = document.getElementById('inputWhiteDomain');
            const v = inp.value.trim().toLowerCase().replace(/^https?:\/\//, '').split('/')[0];
            if (!v || !v.includes('.')) return showToast('Enter a valid domain (e.g. example.org)');
            adblockCfg.whitelisted_domains = adblockCfg.whitelisted_domains || [];
            if (!adblockCfg.whitelisted_domains.includes(v)) adblockCfg.whitelisted_domains.push(v);
            inp.value = '';
            renderAdblockDomainLists();
            sendIpc({ type: 'SAVE_ADBLOCK', adblock: adblockCfg });
            showToast(`Whitelisted ${v}`);
        };

        bindJsonFilePicker('btnImportRules', 'fileImportRules', 'IMPORT_RULES_JSON');
        document.getElementById('btnExportRules').onclick = () => sendIpc({ type: 'EXPORT_RULES' });
        document.getElementById('btnResetRules').onclick = () => sendIpc({ type: 'RESET_RULES' });
        document.getElementById('btnNewRule').onclick = () => openRuleEditor(null);

        const btnImportUs = document.getElementById('btnImportUserscript');
        const inpImportUs = document.getElementById('fileImportUserscript');
        if (btnImportUs && inpImportUs) {
            btnImportUs.onclick = () => inpImportUs.click();
            inpImportUs.addEventListener('change', (e) => {
                const files = Array.from(e.target.files || []);
                if (files.length === 0) return;
                let remaining = files.length;
                let importedCount = 0;
                let lastName = '';

                files.forEach((file, idx) => {
                    const reader = new FileReader();
                    reader.onload = () => {
                        const js = String(reader.result || '');
                        if (js.trim()) {
                            const meta = parseUserscriptHeader(js);
                            const baseFileName = (file.name || 'script.user.js')
                                .replace(/\.user\.js$/i, '')
                                .replace(/\.js$/i, '')
                                .trim();
                            const nm = (meta && meta.name) ? meta.name : (baseFileName || 'Userscript');
                            const dom = (meta && meta.domain) ? meta.domain : '*';
                            const slug = nm.toLowerCase().replace(/[^a-z0-9_-]+/g, '-').replace(/^-+|-+$/g, '') || 'userscript';
                            const id = `${slug}-${Date.now() % 10000}-${idx}`;
                            const entry = {
                                id: id,
                                name: nm,
                                domain: dom,
                                enabled: true,
                                css: '',
                                js: js
                            };
                            if (meta && meta.version) entry.version = meta.version;
                            siteRules.push(entry);
                            importedCount += 1;
                            lastName = nm;
                        }
                        remaining -= 1;
                        if (remaining === 0) {
                            e.target.value = '';
                            if (importedCount > 0) {
                                renderSiteRulesList();
                                sendIpc({ type: 'SAVE_RULES', rules: siteRules });
                                showToast(importedCount === 1
                                    ? `Imported ScriptForge script: ${lastName}`
                                    : `Imported ${importedCount} ScriptForge scripts`);
                            } else {
                                showToast('Selected file was empty');
                            }
                        }
                    };
                    reader.onerror = () => {
                        remaining -= 1;
                        if (remaining === 0) e.target.value = '';
                    };
                    reader.readAsText(file, 'utf-8');
                });
            });
        }

        const btnInsertUsTpl = document.getElementById('btnInsertUserscriptTpl');
        if (btnInsertUsTpl) {
            btnInsertUsTpl.onclick = () => {
                const jsEl = document.getElementById('ruleJsInput');
                if (!jsEl) return;
                const existing = jsEl.value;
                if (/\/\/\s*==UserScript==/.test(existing)) {
                    showToast(isZh() ? '脚本中已包含油猴头部' : 'Userscript header already present');
                    return;
                }
                const nm = document.getElementById('ruleNameInput')?.value.trim() || 'My ScriptForge Script';
                const domRaw = document.getElementById('ruleDomainInput')?.value.trim() || '*';
                const header = buildUserscriptHeader(nm, domRaw, ['GM_addStyle', 'GM_getValue', 'GM_setValue', 'GM_registerMenuCommand', 'GM_notification']);

                if (existing.trim()) {
                    jsEl.value = `${header}\n\n${existing}`;
                } else {
                    jsEl.value = [
                        header,
                        '',
                        '(function () {',
                        "  'use strict';",
                        '',
                        '  GM_addStyle(',
                        "    '/* Optional custom CSS injected by ScriptForge */'",
                        '  );',
                        '',
                        "  const runs = Number(GM_getValue('runs', 0)) + 1;",
                        "  GM_setValue('runs', runs);",
                        '',
                        "  GM_registerMenuCommand('Show run count', () => {",
                        "    GM_notification({ title: GM_info.script.name, text: 'Runs on this script: ' + runs });",
                        '  });',
                        '})();',
                        ''
                    ].join('\n');
                }
                jsEl.focus();
            };
        }

        document.querySelectorAll('#ruleNoCodeBar [data-nocode]').forEach((chip) => {
            chip.onclick = () => {
                applyNoCodeBlockToInputs(chip.dataset.nocode, {
                    nameInput: document.getElementById('ruleNameInput'),
                    domainInput: document.getElementById('ruleDomainInput'),
                    cssInput: document.getElementById('ruleCssInput'),
                    jsInput: document.getElementById('ruleJsInput')
                });
            };
        });
        document.querySelectorAll('#pubNoCodeBar [data-pub-nocode]').forEach((chip) => {
            chip.onclick = () => {
                applyNoCodeBlockToInputs(chip.dataset.pubNocode, {
                    nameInput: document.getElementById('pubExtName'),
                    domainInput: document.getElementById('pubExtDomain'),
                    cssInput: document.getElementById('pubExtCss'),
                    jsInput: document.getElementById('pubExtJs'),
                    onChanged: updatePublishAuditPreview
                });
            };
        });

        const ruleJsEl = document.getElementById('ruleJsInput');
        if (ruleJsEl) {
            ruleJsEl.addEventListener('input', () => {
                autofillFromUserscript(ruleJsEl.value, {
                    nameEl: document.getElementById('ruleNameInput'),
                    domainEl: document.getElementById('ruleDomainInput')
                });
            });
        }

        document.getElementById('btnCancelRule').onclick = () => {
            document.getElementById('ruleEditorPanel').classList.remove('open');
            editingRuleId = null;
        };
        document.getElementById('btnSaveRule').onclick = () => {
            const css = document.getElementById('ruleCssInput').value;
            const js = document.getElementById('ruleJsInput').value;
            if (!css.trim() && !js.trim()) return showToast('Provide CSS or JS for this rule');

            const parsedMeta = parseUserscriptHeader(js);
            const nm = document.getElementById('ruleNameInput').value.trim() || (parsedMeta && parsedMeta.name) || 'Custom rule';
            const dom = document.getElementById('ruleDomainInput').value.trim() || (parsedMeta && parsedMeta.domain) || '*';

            const nextPatch = { name: nm, domain: dom, css: css, js: js };
            if (parsedMeta && parsedMeta.version) {
                nextPatch.version = parsedMeta.version;
            }

            if (editingRuleId) {
                const idx = siteRules.findIndex(r => r.id === editingRuleId);
                if (idx >= 0) {
                    siteRules[idx] = Object.assign({}, siteRules[idx], nextPatch);
                }
            } else {
                const id = nm.toLowerCase().replace(/[^a-z0-9_-]+/g, '-') + '-' + (Date.now() % 10000);
                siteRules.push(Object.assign({ id: id, enabled: true }, nextPatch));
            }
            editingRuleId = null;
            document.getElementById('ruleEditorPanel').classList.remove('open');
            renderSiteRulesList();
            sendIpc({ type: 'SAVE_RULES', rules: siteRules });
            showToast(`Saved rule: ${nm}`);
        };

        bindJsonFilePicker('btnImportKeybinds', 'fileImportKeybinds', 'IMPORT_KEYBINDS_JSON');
        document.getElementById('btnExportKeybinds').onclick = () => sendIpc({ type: 'EXPORT_KEYBINDS' });
        document.getElementById('btnResetKeybinds').onclick = () => sendIpc({ type: 'RESET_KEYBINDS' });

        document.getElementById('qsDeskTopToggle').onclick = () => {
            styles.topbar = styles.topbar || {};
            styles.topbar.visible_on_desktop = styles.topbar.visible_on_desktop === false ? true : false;
            pushStyles(styles.topbar.visible_on_desktop ? 'Desktop topbar shown' : 'Desktop topbar hidden');
        };
        document.getElementById('qsTopbarLayout').onchange = (e) => {
            styles.topbar = styles.topbar || {};
            styles.topbar.layout = e.target.value;
            const msg = e.target.value === 'vertical'
                ? 'Switched to left vertical sidebar'
                : e.target.value === 'deck'
                    ? 'Switched to 2-row layered deck'
                    : 'Switched to single-row compact bar';
            pushStyles(msg);
        };
        document.getElementById('qsBookmarksBar').onchange = (e) => {
            styles.topbar = styles.topbar || {};
            styles.topbar.bookmarks_bar = e.target.value;
            pushStyles('Shortcuts bar: ' + e.target.value);
        };
        document.getElementById('qsTopbarHeight').onchange = (e) => {
            styles.topbar = styles.topbar || {};
            styles.topbar.height = e.target.value.trim() || '40px';
            pushStyles('Nav bar height updated');
        };
        const qsTitlebarInput = document.getElementById('qsTitlebarHeight');
        if (qsTitlebarInput) {
            qsTitlebarInput.onchange = (e) => {
                styles.topbar = styles.topbar || {};
                styles.topbar.titlebar_height = e.target.value.trim() || '42px';
                pushStyles('Titlebar height updated');
            };
        }
        const qsVertWInput = document.getElementById('qsVertWidth');
        if (qsVertWInput) {
            qsVertWInput.onchange = (e) => {
                styles.topbar = styles.topbar || {};
                styles.topbar.vertical_width = e.target.value.trim() || '216px';
                pushStyles('Vertical tab sidebar width updated');
            };
        }
        [
            ['qsBtnCopyToggle', 'show_copy_btn', 'Copy URL button'],
            ['qsBtnStarToggle', 'show_star_btn', 'Bookmark button'],
            ['qsBtnShieldToggle', 'show_shield_btn', 'Shield button'],
            ['qsBtnDlToggle', 'show_downloads_btn', 'Downloads button']
        ].forEach(([elId, field, label]) => {
            const btn = document.getElementById(elId);
            if (!btn) return;
            btn.onclick = () => {
                styles.topbar = styles.topbar || {};
                const next = styles.topbar[field] === false;
                styles.topbar[field] = next;
                pushStyles(`${label} ${next ? 'shown' : 'hidden'}`);
            };
        });
        document.getElementById('qsAddrMode').onchange = (e) => {
            styles.address_bar = styles.address_bar || {};
            styles.address_bar.mode = e.target.value;
            pushStyles('Address bar mode: ' + e.target.value);
        };
        document.getElementById('qsAddrPos').onchange = (e) => {
            styles.address_bar = styles.address_bar || {};
            styles.address_bar.position = e.target.value;
            pushStyles('Omnibox aligned ' + e.target.value);
        };
        document.getElementById('qsAddrWidth').onchange = (e) => {
            styles.address_bar = styles.address_bar || {};
            styles.address_bar.width = e.target.value.trim() || '320px';
            pushStyles('Address capsule width updated');
        };
        const tabHInput = document.getElementById('qsTabHeight');
        if (tabHInput) {
            tabHInput.onchange = (e) => {
                styles.tabs = styles.tabs || {};
                styles.tabs.height = e.target.value.trim() || '36px';
                pushStyles('Tab item height updated');
            };
        }
        document.getElementById('qsTabWidth').onchange = (e) => {
            styles.tabs = styles.tabs || {};
            styles.tabs.width = e.target.value.trim() || '152px';
            pushStyles('Tab width updated');
        };
        document.getElementById('qsTabRadius').onchange = (e) => {
            styles.tabs = styles.tabs || {};
            styles.tabs.border_radius = e.target.value.trim() || '7px';
            pushStyles('Tab corner radius updated');
        };
        const tabGapInput = document.getElementById('qsTabGap');
        if (tabGapInput) {
            tabGapInput.onchange = (e) => {
                styles.tabs = styles.tabs || {};
                styles.tabs.gap = e.target.value.trim() || '0px';
                pushStyles('Tab spacing (gap) updated');
            };
        }
        document.getElementById('qsTabDotToggle').onclick = () => {
            styles.tabs = styles.tabs || {};
            styles.tabs.show_indicator_dot = styles.tabs.show_indicator_dot === false ? true : false;
            pushStyles('Updated tab badge');
        };
        document.getElementById('qsMobilePos').onchange = (e) => {
            styles.mobile_bar = styles.mobile_bar || {};
            styles.mobile_bar.position = e.target.value;
            pushStyles('Mobile bar docked to ' + e.target.value);
        };
        document.getElementById('qsMobileHeight').onchange = (e) => {
            styles.mobile_bar = styles.mobile_bar || {};
            styles.mobile_bar.height = e.target.value.trim() || '54px';
            pushStyles('Mobile bar height updated');
        };
        document.getElementById('qsFullWidthToggle').onclick = () => {
            styles.settings_page = styles.settings_page || {};
            styles.settings_page.full_width = styles.settings_page.full_width === false ? true : false;
            pushStyles('Workspace width updated');
        };

        document.getElementById('qsNtTitle').onchange = (e) => {
            styles.newtab = styles.newtab || {};
            styles.newtab.title = e.target.value;
            pushStyles('Start page title updated');
        };
        document.getElementById('qsNtSub').onchange = (e) => {
            styles.newtab = styles.newtab || {};
            styles.newtab.subtitle = e.target.value;
            pushStyles('Start page subtitle updated');
        };
        document.getElementById('qsNtCols').onchange = (e) => {
            styles.newtab = styles.newtab || {};
            styles.newtab.shortcuts_columns = Number(e.target.value) || 0;
            pushStyles('Shortcuts grid columns updated');
        };
        document.getElementById('qsNtClockToggle').onclick = () => {
            styles.newtab = styles.newtab || {};
            styles.newtab.show_clock = styles.newtab.show_clock === false ? true : false;
            pushStyles('Start page clock updated');
        };
        document.getElementById('qsNt24hToggle').onclick = () => {
            styles.newtab = styles.newtab || {};
            styles.newtab.use_24h_clock = styles.newtab.use_24h_clock === false ? true : false;
            pushStyles('Clock format updated');
        };
        document.getElementById('qsNtScToggle').onclick = () => {
            styles.newtab = styles.newtab || {};
            styles.newtab.show_shortcuts = styles.newtab.show_shortcuts === false ? true : false;
            pushStyles('Start page shortcuts visibility updated');
        };
        document.getElementById('qsNtHistToggle').onclick = () => {
            styles.newtab = styles.newtab || {};
            styles.newtab.show_history = styles.newtab.show_history === false ? true : false;
            pushStyles('Start page history visibility updated');
        };
        const qsNtWpEl = document.getElementById('qsNtWallpaper');
        if (qsNtWpEl) {
            qsNtWpEl.onchange = (e) => {
                styles.newtab = styles.newtab || {};
                styles.newtab.background_image = e.target.value.trim();
                pushStyles(styles.newtab.background_image ? 'Start page wallpaper updated' : 'Start page wallpaper cleared');
            };
        }
        const btnWpUp = document.getElementById('btnNtWallpaperUpload');
        const inpWpFile = document.getElementById('fileNtWallpaper');
        if (btnWpUp && inpWpFile) {
            btnWpUp.onclick = () => inpWpFile.click();
            inpWpFile.addEventListener('change', (e) => {
                const file = e.target.files && e.target.files[0];
                e.target.value = '';
                if (!file) return;
                if (file.size > 4 * 1024 * 1024) {
                    showToast('Please choose an image under 4 MB');
                    return;
                }
                const reader = new FileReader();
                reader.onload = () => {
                    styles.newtab = styles.newtab || {};
                    styles.newtab.background_image = String(reader.result || '');
                    pushStyles('Start page wallpaper uploaded');
                };
                reader.readAsDataURL(file);
            });
        }
        const btnWpClr = document.getElementById('btnNtWallpaperClear');
        if (btnWpClr) {
            btnWpClr.onclick = () => {
                styles.newtab = styles.newtab || {};
                styles.newtab.background_image = '';
                pushStyles('Start page wallpaper cleared');
            };
        }
        const qsNtDimEl = document.getElementById('qsNtDim');
        if (qsNtDimEl) {
            qsNtDimEl.onchange = (e) => {
                styles.newtab = styles.newtab || {};
                const n = Math.max(0, Math.min(90, Number(e.target.value) || 0));
                styles.newtab.background_dim = Math.round(n);
                pushStyles(`Wallpaper dim set to ${styles.newtab.background_dim}%`);
            };
        }
        const qsNtBlurEl = document.getElementById('qsNtBlur');
        if (qsNtBlurEl) {
            qsNtBlurEl.onchange = (e) => {
                styles.newtab = styles.newtab || {};
                const n = Math.max(0, Math.min(24, Number(e.target.value) || 0));
                styles.newtab.background_blur = Math.round(n);
                pushStyles(`Wallpaper blur set to ${styles.newtab.background_blur}px`);
            };
        }

        const btnSettingsLangEl = document.getElementById('btnSettingsLang');
        if (btnSettingsLangEl) {
            btnSettingsLangEl.onclick = () => {
                config.language = isZh() ? 'en' : 'zh';
                renderAllSections();
                saveConfig(isZh() ? '已切换为简体中文界面' : 'Switched interface to English');
            };
        }

        const UI_STYLE_PRESETS = {
            compact: {
                msgZh: '已切换为「极简紧凑」布局',
                msgEn: 'Applied Compact UI preset',
                topbar: { layout: 'compact', height: '38px', titlebar_height: '40px' },
                tabs: { height: '28px', width: '184px', border_radius: '7px', gap: '4px' },
                address_bar: { border_radius: '8px', height: '28px' }
            },
            deck: {
                msgZh: '已切换为「经典双栏」布局',
                msgEn: 'Applied Deck UI preset',
                topbar: { layout: 'deck', height: '40px', titlebar_height: '42px' },
                tabs: { height: '30px', width: '204px', border_radius: '8px', gap: '4px' },
                address_bar: { border_radius: '8px', height: '29px' }
            },
            pill: {
                msgZh: '已切换为「大圆角胶囊」风格',
                msgEn: 'Applied Pill UI preset',
                topbar: { layout: 'deck', height: '42px', titlebar_height: '44px' },
                tabs: { height: '32px', width: '210px', border_radius: '999px', gap: '6px' },
                address_bar: { border_radius: '999px', height: '30px' }
            },
            vertical: {
                msgZh: '已切换为「左侧竖排标签栏」风格',
                msgEn: 'Applied Vertical Sidebar preset',
                topbar: { layout: 'vertical', vertical_width: '216px', height: '40px', titlebar_height: '42px' },
                tabs: { height: '32px', width: '204px', border_radius: '8px', gap: '4px' },
                address_bar: { border_radius: '8px', height: '29px' }
            }
        };

        document.querySelectorAll('[data-ui-preset]').forEach((btn) => {
            btn.onclick = () => {
                const key = btn.dataset.uiPreset;
                const p = UI_STYLE_PRESETS[key];
                if (!p) return;
                styles.topbar = Object.assign({}, styles.topbar || {}, p.topbar);
                styles.tabs = Object.assign({}, styles.tabs || {}, p.tabs);
                styles.address_bar = Object.assign({}, styles.address_bar || {}, p.address_bar);
                pushStyles(isZh() ? p.msgZh : p.msgEn);
            };
        });

        document.querySelectorAll('[data-wp]').forEach((btn) => {
            btn.onclick = () => {
                styles.newtab = styles.newtab || {};
                styles.newtab.background_image = btn.dataset.wp || '';
                pushStyles(styles.newtab.background_image
                    ? (isZh() ? `已切换壁纸：${btn.textContent}` : `Applied wallpaper: ${btn.textContent}`)
                    : (isZh() ? '已恢复默认无壁纸' : 'Cleared Start page wallpaper'));
            };
        });

        const jsonEditorEl = document.getElementById('jsonEditor');
        let validateTimer = null;
        jsonEditorEl.addEventListener('input', () => {
            if (validateTimer) clearTimeout(validateTimer);
            validateTimer = setTimeout(() => {
                try {
                    JSON.parse(jsonEditorEl.value);
                    setJsonStatus(false, 'Syntax valid — click Apply & save or press Ctrl+S.');
                } catch (err) {
                    setJsonStatus(true, 'Syntax error: ' + err.message);
                }
            }, 180);
        });

        document.getElementById('btnSaveJson').onclick = () => {
            try {
                const parsed = JSON.parse(jsonEditorEl.value);
                styles = parsed;
                renderColorTokensStudio();
                applySettingsSurface();
                syncFormControls();
                jsonEditorEl.value = JSON.stringify(styles, null, 2);
                sendIpc({ type: 'SAVE_STYLES', styles: styles });
                setJsonStatus(false, 'Valid JSON — applied live and saved to disk.');
                showToast('Saved styles.json');
            } catch (err) {
                setJsonStatus(true, 'Invalid JSON: ' + err.message);
            }
        };

        document.getElementById('btnReloadJson').onclick = () => {
            sendIpc({ type: 'RELOAD_STYLES' });
            showToast('Reloaded JSON files from disk');
        };
        document.getElementById('btnResetJson').onclick = () => {
            sendIpc({ type: 'RESET_STYLES' });
            showToast('Reset component styles to defaults');
        };

        jsonEditorEl.addEventListener('keydown', (e) => {
            if (e.key === 'Tab') {
                e.preventDefault();
                const start = jsonEditorEl.selectionStart;
                const end = jsonEditorEl.selectionEnd;
                jsonEditorEl.value = jsonEditorEl.value.substring(0, start) + '  ' + jsonEditorEl.value.substring(end);
                jsonEditorEl.selectionStart = jsonEditorEl.selectionEnd = start + 2;
            } else if ((e.ctrlKey || e.metaKey) && e.key.toLowerCase() === 's') {
                e.preventDefault();
                document.getElementById('btnSaveJson').click();
            }
        });

        // Cursor-style Real Single-View Switching
        const navLinks = Array.from(document.querySelectorAll('.nav-link'));
        const viewBlocks = Array.from(document.querySelectorAll('.sec-block'));

        function switchSettingsView(secId, updateHash) {
            const target = viewBlocks.find(v => v.id === secId) || viewBlocks[0];
            if (!target) return;
            viewBlocks.forEach(v => v.classList.toggle('active', v === target));
            navLinks.forEach(l => l.classList.toggle('active', l.dataset.sec === target.id));
            window.scrollTo(0, 0);
            if (updateHash && history && history.replaceState) {
                history.replaceState(null, '', '#' + target.id);
            }
        }

        navLinks.forEach(link => {
            link.addEventListener('click', () => {
                if (link.dataset.sec) switchSettingsView(link.dataset.sec, true);
            });
        });

        const rawHash = (location.hash || '').replace(/^#/, '');
        const initialHash = rawHash === 'pane-siterules' ? 'sec-extensions' : rawHash;
        if (initialHash && viewBlocks.some(v => v.id === initialHash)) {
            switchSettingsView(initialHash, false);
        } else {
            switchSettingsView('sec-appearance', false);
        }

        function bindSubTabSwitcher(groupId) {
            const grp = document.getElementById(groupId);
            if (!grp) return;
            const btns = Array.from(grp.querySelectorAll('.seg-btn'));
            btns.forEach(btn => {
                btn.onclick = () => {
                    btns.forEach(b => b.classList.toggle('active', b === btn));
                    btns.forEach(b => {
                        const pane = document.getElementById(b.dataset.subpane);
                        if (pane) pane.classList.toggle('active', b === btn);
                    });
                };
            });
        }
        bindSubTabSwitcher('extSubTabs');
        bindSubTabSwitcher('studioSubTabs');

        const cloudTabBtn = document.querySelector('#extSubTabs .seg-btn[data-subpane="pane-ext-cloud"]');
        if (cloudTabBtn) {
            cloudTabBtn.addEventListener('click', () => {
                syncCloudConfigInputs();
                if (!cloudLoadedOnce) fetchCloudCatalog(false);
            });
        }
        document.getElementById('btnToggleCloudCfg').onclick = () => {
            syncCloudConfigInputs();
            document.getElementById('cloudConfigCard').classList.toggle('open');
        };
        document.getElementById('btnSaveCloudCfg').onclick = () => {
            const url = document.getElementById('inputSupabaseUrl').value.trim().replace(/\/+$/, '');
            const anonKey = document.getElementById('inputSupabaseKey').value.trim();
            const schema = (document.getElementById('inputSupabaseSchema')?.value || '').trim() || 'public';
            const table = (document.getElementById('inputSupabaseTable')?.value || '').trim() || 'extensions';
            try {
                localStorage.setItem(SUPABASE_STORAGE_KEY, JSON.stringify({ url, anonKey, schema, table }));
            } catch (_) {}
            document.getElementById('cloudConfigCard').classList.remove('open');
            fetchCloudCatalog(true);
        };
        document.getElementById('btnCloudUpload').onclick = () => openPublishPanel(null);
        document.getElementById('btnSubmitPublish').onclick = () => submitPublishedExtension();
        document.getElementById('btnCancelPublish').onclick = () => {
            document.getElementById('publishExtPanel').classList.remove('open');
        };
        ['pubExtDomain', 'pubExtCss', 'pubExtJs'].forEach((id) => {
            const el = document.getElementById(id);
            if (el) {
                el.addEventListener('input', () => {
                    if (id === 'pubExtJs') {
                        autofillFromUserscript(el.value, {
                            nameEl: document.getElementById('pubExtName'),
                            domainEl: document.getElementById('pubExtDomain'),
                            authorEl: document.getElementById('pubExtAuthor'),
                            descEl: document.getElementById('pubExtDesc')
                        });
                    }
                    updatePublishAuditPreview();
                });
            }
        });
        document.getElementById('btnInspClose').onclick = () => {
            document.getElementById('cloudInspectorPanel').classList.remove('open');
        };
        document.getElementById('btnRefreshCloud').onclick = () => fetchCloudCatalog(true);
        document.getElementById('inputCloudSearch').oninput = () => renderCloudExtensions();
        document.getElementById('selectCloudCategory').onchange = () => renderCloudExtensions();

        function renderAllSections() {
            renderThemeGrid();
            renderColorTokensStudio();
            renderEngineTable();
            renderBangsTable();
            renderShortcutsTable();
            renderAdblockDomainLists();
            renderBuiltinExtensions();
            renderSiteRulesList();
            renderCloudExtensions();
            renderKeybindsGrid();
            applySettingsSurface();
            syncFormControls();
        }

        window.__fynexo_on_settings_runtime__ = function(payload) {
            if (!payload) return;
            if (payload.config) config = Object.assign(config, payload.config);
            if (payload.styles) {
                styles = payload.styles;
                if (document.activeElement !== jsonEditorEl) {
                    jsonEditorEl.value = JSON.stringify(styles, null, 2);
                }
            }
            if (payload.active_theme) activeTheme = payload.active_theme;
            if (Array.isArray(payload.themes)) themes = payload.themes;
            if (Array.isArray(payload.bookmarks)) bookmarks = payload.bookmarks;
            if (Array.isArray(payload.site_rules)) siteRules = payload.site_rules;
            if (payload.adblock) adblockCfg = payload.adblock;
            if (payload.keybinds) keybinds = Object.assign({}, DEFAULT_KEYBINDS, payload.keybinds);
            if (payload.themes_dir) themesDir = payload.themes_dir;
            if (payload.styles_path) stylesPath = payload.styles_path;
            if (payload.data_dir) dataDir = payload.data_dir;
            if (typeof payload.blocked_ads === 'number') rustBlockedAds = payload.blocked_ads;
            blockedAds = resolveTotalBlockedAds(rustBlockedAds);
            if (Array.isArray(payload.history)) historyCount = payload.history.length;
            renderAllSections();
        };

        renderAllSections();
