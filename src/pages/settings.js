        __FNX_DEFAULTS__;
        const KEYBIND_DEFS = [
            ['omnibox', 'Open omnibox & quick bar', 'Focus address bar from anywhere', '打开地址栏与快速搜索框', '在任意页面随时输入网址或搜索'],
            ['new_tab', 'Open new tab', 'Launch a fresh Start Page tab', '新建标签页', '打开一个全新的起始页'],
            ['new_window', 'Open new window', 'Launch an independent browser window (Multi-Window mode)', '新建浏览窗口', '在多窗口模式下新开一个独立窗口'],
            ['new_incognito_window', 'New InPrivate window', 'Open an isolated Private window alongside normal windows', '新建独立无痕窗口', '开启隔离隐私会话的新窗口'],
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
            ['zoom_reset', 'Reset zoom (100%)', 'Restore default 100% scale', '恢复默认缩放 (100%)', '重置回原始网页大小'],
            ['devtools', 'Open native DevTools', 'Open browser F12 inspector window', '打开原生开发者工具 (F12)', '呼出系统 WebView 完整调试窗口'],
            ['dev_drawer', 'Toggle Fynexo DevDrawer', 'Open built-in Console, Network, Storage & DOM drawer', '开关内置开发者抽屉', '呼出内置控制台、网络耗时、存储与源码调试面板']
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
            enforce_perf: false,
            adblock: true,
            restore_session: true,
            force_dark: false,
            incognito: false,
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
        let chromeExtensions = Array.isArray(boot.chrome_extensions) ? boot.chrome_extensions : [];
        let chromeExtEngineActive = !!boot.chrome_ext_engine_active;
        let historyEntries = Array.isArray(boot.history) ? boot.history : [];
        let historyCount = historyEntries.length;

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
                activeTag.textContent = isZh() ? '使用中' : 'Active';
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
                    delBtn.title = isZh() ? `删除自定义主题 ${item.id}.json` : `Delete custom theme ${item.id}.json`;
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
                sub.textContent = item.description || (isZh() ? '自定义 JSON 主题' : 'Custom JSON palette');

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
                    saveConfig(isZh() ? `已切换主题：${item.name || item.id}` : `Theme set to ${item.name || item.id}`);
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
                    pill.textContent = isZh() ? '默认' : 'Default';
                    right.appendChild(pill);
                }
                if (!builtinIds.includes(eng.id) && list.length > 1) {
                    right.appendChild(createCompactDeleteBtn(isZh() ? '删除自定义搜索引擎' : 'Remove custom search engine', () => {
                        config.search_engines = list.filter(x => x.id !== eng.id);
                        if (config.search_engine === eng.id) {
                            config.search_engine = config.search_engines[0].id;
                        }
                        renderEngineTable();
                        saveConfig(isZh() ? '已删除搜索引擎' : 'Removed search engine');
                    }));
                }

                row.onclick = () => {
                    config.search_engine = eng.id;
                    renderEngineTable();
                    saveConfig((isZh() ? '默认搜索引擎：' : 'Search engine: ') + (eng.name || eng.id));
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
                empty.textContent = isZh() ? '暂无 !bang 快捷搜索指令 — 可在下方添加。' : 'No !bang triggers yet — add one below.';
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
                right.appendChild(createCompactDeleteBtn(isZh() ? `删除 ${trigStr}` : `Remove ${trigStr}`, () => {
                    config.bangs = bangs.filter((_, i) => i !== idx);
                    renderBangsTable();
                    saveConfig(isZh() ? `已删除指令 ${trigStr}` : `Removed ${trigStr}`);
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
                empty.textContent = isZh() ? '暂无常用网站 — 可在下方添加。' : 'No shortcuts saved — add one below.';
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
                right.appendChild(createCompactDeleteBtn(isZh() ? '移除常用网站' : 'Remove shortcut', () => {
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
            const zh = isZh();
            const bList = Array.isArray(adblockCfg.custom_blocked_domains) ? adblockCfg.custom_blocked_domains : [];
            const wList = Array.isArray(adblockCfg.whitelisted_domains) ? adblockCfg.whitelisted_domains : [];

            renderDomainTable('blockedDomainsList', bList, zh ? '暂无自定义屏蔽域名。' : 'No custom blocked domains.', (dom) => {
                adblockCfg.custom_blocked_domains = bList.filter(x => x !== dom);
                renderAdblockDomainLists();
                sendIpc({ type: 'SAVE_ADBLOCK', adblock: adblockCfg });
                showToast(zh ? `已移出屏蔽列表：${dom}` : `Unblocked ${dom}`);
            });

            renderDomainTable('whitelistDomainsList', wList, zh ? '暂无白名单域名。' : 'No whitelisted domains.', (dom) => {
                adblockCfg.whitelisted_domains = wList.filter(x => x !== dom);
                renderAdblockDomainLists();
                sendIpc({ type: 'SAVE_ADBLOCK', adblock: adblockCfg });
                showToast(zh ? `已移出白名单：${dom}` : `Removed ${dom} from whitelist`);
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
            const zh = isZh();
            el.classList.remove('warn', 'clean');
            if (audit && audit.hasWarnings) {
                el.classList.add('warn');
                const prefix = mode === 'upload'
                    ? (zh ? '警告 — 上传后数据库将标为 warning：' : 'Warning — upload will be flagged as "warning" in database: ')
                    : (zh ? '安全提示 — 启用前请核对代码：' : 'Security Warning — review source carefully before enabling: ');
                el.textContent = prefix + audit.warnings.join(' · ');
            } else {
                el.classList.add('clean');
                el.textContent = mode === 'upload'
                    ? (zh ? '安全检测通过 — 上传后将进入 unverified 待审队列。' : 'Security check: Clean — upload enters queue as "unverified".')
                    : (zh ? '安全检测通过 — 未发现高风险接口调用。' : 'Security check: Clean — no suspicious patterns detected.');
            }
        }

        function renderSiteRulesList() {
            const zh = isZh();
            const listEl = document.getElementById('rulesList');
            listEl.textContent = '';

            if (siteRules.length === 0) {
                const empty = document.createElement('div');
                empty.className = 'rc-row-empty';
                empty.style.borderBottom = 'none';
                empty.textContent = zh
                    ? '暂无自定义 CSS/JS 扩展规则。点击「+ 新建扩展」或前往「Cloud Store」安装。'
                    : 'No custom CSS/JS extensions configured. Click "+ Add Extension" or install from Cloud Store.';
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
                sw.title = rule.enabled ? (zh ? '停用规则' : 'Disable rule') : (zh ? '启用规则' : 'Enable rule');
                sw.onclick = () => {
                    rule.enabled = !rule.enabled;
                    renderSiteRulesList();
                    if (cloudLoadedOnce) renderCloudExtensions();
                    sendIpc({ type: 'SAVE_RULES', rules: siteRules });
                    showToast(`${rule.name || (zh ? '规则' : 'Rule')}: ${rule.enabled ? (zh ? '已启用' : 'Enabled') : (zh ? '已停用' : 'Disabled')}`);
                };

                const pubBtn = document.createElement('button');
                pubBtn.type = 'button';
                pubBtn.className = 'action-btn compact';
                pubBtn.textContent = zh ? '发布' : 'Publish';
                pubBtn.title = zh ? '上传扩展到 Supabase 云商店（需审核通过后公开）' : 'Upload extension to Supabase Cloud Store (starts as unverified)';
                pubBtn.onclick = () => openPublishPanel(rule);

                const editBtn = document.createElement('button');
                editBtn.type = 'button';
                editBtn.className = 'action-btn compact';
                editBtn.textContent = zh ? '编辑' : 'Edit';
                editBtn.onclick = () => openRuleEditor(rule);

                right.appendChild(pubBtn);
                right.appendChild(editBtn);
                right.appendChild(sw);
                right.appendChild(createCompactDeleteBtn(zh ? '删除规则' : 'Delete rule', () => {
                    siteRules = siteRules.filter(x => x.id !== rule.id);
                    renderSiteRulesList();
                    if (cloudLoadedOnce) renderCloudExtensions();
                    sendIpc({ type: 'SAVE_RULES', rules: siteRules });
                    showToast(zh ? '已删除扩展规则' : 'Deleted rule');
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
            const chromeEnabled = Array.isArray(chromeExtensions)
                ? chromeExtensions.filter(e => e && e.enabled).length
                : 0;
            navBadge.textContent = String(active + customEnabled + chromeEnabled);
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
            const zh = isZh();
            const listEl = document.getElementById('cloudExtList');
            if (!listEl) return;
            listEl.textContent = '';

            const creds = getSupabaseCreds();
            if (!creds.isConfigured) {
                const empty = document.createElement('div');
                empty.className = 'rc-row-empty';
                empty.style.borderBottom = 'none';
                empty.textContent = zh
                    ? '尚未连接 Supabase API。请构建前配置 .env.local，或在上方「API 配置」填入 URL 与 Anon Key。'
                    : 'Supabase API not connected. Configure .env.local before build or enter your URL & Anon Key in API Config above.';
                listEl.appendChild(empty);
                return;
            }

            if (cloudLoading) {
                const loadingRow = document.createElement('div');
                loadingRow.className = 'rc-row-empty';
                loadingRow.style.borderBottom = 'none';
                loadingRow.textContent = zh ? '正在从 Supabase 获取云端扩展列表…' : 'Fetching extension catalog metadata from Supabase...';
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
                    ? (zh ? '云商店暂无已验证扩展，请点击「刷新」或上传分享一个。' : 'No verified extensions found in Supabase yet. Click Refresh or upload one above.')
                    : (zh ? '没有匹配当前筛选条件的扩展。' : 'No matching extensions for the current filter.');
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
                inspBtn.textContent = zh ? '查源码' : 'Inspect';
                inspBtn.title = zh ? '查看 CSS/JS 源码并执行本地安全审计' : 'Inspect CSS/JS source and run local security check';
                inspBtn.onclick = () => inspectCloudExtension(item, inspBtn);
                right.appendChild(inspBtn);

                if (installedRule) {
                    const cloudVer = item.version ? String(item.version) : '';
                    const localVer = installedRule.version ? String(installedRule.version) : '';
                    if (cloudVer && localVer !== cloudVer) {
                        const updBtn = document.createElement('button');
                        updBtn.type = 'button';
                        updBtn.className = 'action-btn compact';
                        updBtn.textContent = zh ? '更新' : 'Update';
                        updBtn.title = zh ? `从 ${localVer || '旧版'} 更新到 v${cloudVer}` : `Update from ${localVer || 'legacy'} to v${cloudVer}`;
                        updBtn.onclick = () => installCloudExtension(item, updBtn, false);
                        right.appendChild(updBtn);
                    }

                    const sw = document.createElement('button');
                    sw.type = 'button';
                    sw.className = installedRule.enabled ? 'fnx-switch on' : 'fnx-switch';
                    sw.setAttribute('role', 'switch');
                    sw.setAttribute('aria-checked', installedRule.enabled ? 'true' : 'false');
                    sw.title = installedRule.enabled ? (zh ? '停用扩展' : 'Disable extension') : (zh ? '启用扩展' : 'Enable extension');
                    sw.onclick = () => {
                        installedRule.enabled = !installedRule.enabled;
                        renderSiteRulesList();
                        renderCloudExtensions();
                        sendIpc({ type: 'SAVE_RULES', rules: siteRules });
                        showToast(`${installedRule.name}: ${installedRule.enabled ? (zh ? '已启用' : 'Enabled') : (zh ? '已停用' : 'Disabled')}`);
                    };

                    const rmBtn = document.createElement('button');
                    rmBtn.type = 'button';
                    rmBtn.className = 'action-btn compact';
                    rmBtn.textContent = zh ? '卸载' : 'Remove';
                    rmBtn.onclick = () => {
                        siteRules = siteRules.filter(x => x.id !== ruleId);
                        renderSiteRulesList();
                        renderCloudExtensions();
                        sendIpc({ type: 'SAVE_RULES', rules: siteRules });
                        showToast(zh ? `已移除 ${item.name}` : `Removed ${item.name}`);
                    };
                    right.appendChild(rmBtn);
                    right.appendChild(sw);
                } else {
                    const dlBtn = document.createElement('button');
                    dlBtn.type = 'button';
                    dlBtn.className = 'action-btn compact';
                    dlBtn.textContent = zh ? '安装' : 'Install';
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
            allOpt.textContent = isZh() ? '全部分类' : 'All categories';
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
            const zh = isZh();
            const creds = getSupabaseCreds();
            if (!creds.isConfigured) {
                syncCloudConfigInputs();
                renderCloudExtensions();
                if (forceToast) showToast(zh ? '请先在 API 配置填写 Supabase URL 与 Anon Key' : 'Enter Supabase URL & Anon Key first');
                return;
            }
            cloudLoading = true;
            renderCloudExtensions();
            try {
                const rows = await supabaseRequest(
                    '?select=id,slug,name,category,domain,description,author,version,size_bytes&status=eq.verified&order=downloads.desc'
                );
                cloudCatalog = Array.isArray(rows) ? rows : [];
                cloudLoadedOnce = true;
                refreshCloudCategorySelect();
                setCloudHint(zh
                    ? `已连接 · 共 ${cloudCatalog.length} 个已审核扩展（点击安装时才按需下载源码）。`
                    : `Connected · ${cloudCatalog.length} verified extension${cloudCatalog.length === 1 ? '' : 's'} (CSS/JS downloaded only on Install).`);
                if (forceToast) showToast(zh ? '云商店目录已刷新' : 'Cloud catalog refreshed');
            } catch (err) {
                setCloudHint((zh ? '云端获取失败：' : 'Supabase fetch failed: ') + (err.message || err));
                if (forceToast) showToast(zh ? '无法加载云商店目录' : 'Failed to load cloud catalog');
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
            showToast(isZh()
                ? `${existingIdx >= 0 ? '已更新' : '已安装'}：${ruleObj.name} (v${ruleObj.version})`
                : `${existingIdx >= 0 ? 'Updated' : 'Installed'}: ${ruleObj.name} (v${ruleObj.version})`);
        }

        function openCloudInspector(meta, full, audit) {
            const zh = isZh();
            const panel = document.getElementById('cloudInspectorPanel');
            if (!panel) return;
            document.getElementById('inspExtTitle').textContent = full.name || meta.name || (zh ? '扩展源码审查' : 'Extension Source');
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
            badge.textContent = audit.hasWarnings ? (zh ? `警告 (${audit.warnings.length})` : `Warning (${audit.warnings.length})`) : (zh ? '安全' : 'Clean');
            renderAuditBanner(document.getElementById('inspAuditBanner'), audit, 'inspect');

            document.getElementById('inspExtCss').value = (full.css && full.css.trim()) ? full.css : '/* No CSS */';
            document.getElementById('inspExtJs').value = (full.js && full.js.trim()) ? full.js : '// No JS';

            const ruleId = 'cloud-' + (full.slug || full.id);
            const installedRule = siteRules.find(r => r && r.id === ruleId);
            const instBtn = document.getElementById('btnInspInstall');
            const cloudVer = full.version || meta.version || '';
            const needsUpdate = installedRule && cloudVer && String(installedRule.version || '') !== String(cloudVer);
            if (audit.hasWarnings) {
                instBtn.textContent = needsUpdate ? (zh ? '信任并更新' : 'Trust & Update') : (zh ? '信任并安装' : 'Trust & Install');
            } else if (needsUpdate) {
                instBtn.textContent = zh ? '更新' : 'Update';
            } else if (installedRule) {
                instBtn.textContent = zh ? '重新安装' : 'Reinstall';
            } else {
                instBtn.textContent = zh ? '安装' : 'Install';
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
            const zh = isZh();
            const prevLabel = btn ? btn.textContent : (zh ? '查源码' : 'Inspect');
            if (btn) {
                btn.disabled = true;
                btn.textContent = zh ? '加载中…' : 'Loading...';
            }
            try {
                const full = await fetchExtensionPayload(meta);
                const audit = auditExtensionPayload(full.css, full.js, full.domain);
                openCloudInspector(meta, full, audit);
            } catch (err) {
                showToast((zh ? '源码查看失败：' : 'Inspect failed: ') + (err.message || err));
            } finally {
                if (btn) {
                    btn.disabled = false;
                    btn.textContent = prevLabel;
                }
            }
        }

        async function installCloudExtension(meta, btn, forceTrust) {
            if (!meta || !meta.id) return;
            const zh = isZh();
            const prevLabel = btn ? btn.textContent : (zh ? '安装' : 'Install');
            if (btn) {
                btn.disabled = true;
                btn.textContent = zh ? '下载中…' : 'Downloading...';
            }
            try {
                const full = await fetchExtensionPayload(meta);
                const audit = auditExtensionPayload(full.css, full.js, full.domain);
                if (audit.hasWarnings && !forceTrust) {
                    openCloudInspector(meta, full, audit);
                    showToast(zh ? '检测到高风险脚本调用 — 请核对源码后再确认安装' : 'Security Warning detected — review source before installing');
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
                showToast((zh ? '安装失败：' : 'Install failed: ') + (err.message || err));
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
            const zh = isZh();
            const creds = getSupabaseCreds();
            if (!creds.isConfigured) {
                syncCloudConfigInputs();
                showToast(zh ? '请先在 API 配置填写 Supabase URL 与 Anon Key' : 'Set Supabase URL & Anon Key in API Config first');
                return;
            }
            const name = document.getElementById('pubExtName').value.trim();
            const domain = document.getElementById('pubExtDomain').value.trim() || '*';
            const author = document.getElementById('pubExtAuthor').value.trim() || 'community';
            const category = document.getElementById('pubExtCategory').value.trim() || 'community';
            const description = document.getElementById('pubExtDesc').value.trim() || `Submitted from Fynexo (${domain})`;
            const css = document.getElementById('pubExtCss').value;
            const js = document.getElementById('pubExtJs').value;

            if (!name) return showToast(zh ? '请输入扩展名称' : 'Enter an extension name');
            if (!css.trim() && !js.trim()) return showToast(zh ? '请填写至少一段 CSS 或 JS 代码' : 'Provide CSS or JS to upload');
            const payloadBytes = new Blob([css, js]).size;
            if (payloadBytes > 100 * 1024) {
                return showToast(zh
                    ? `CSS + JS 体积超出 100 KB 上限 (${Math.ceil(payloadBytes / 1024)} KB)`
                    : `CSS + JS exceeds 100 KB limit (${Math.ceil(payloadBytes / 1024)} KB)`);
            }

            const audit = auditExtensionPayload(css, js, domain);
            const btn = document.getElementById('btnSubmitPublish');
            const prevText = btn ? btn.textContent : (zh ? '提交审核' : 'Submit for verification');
            if (btn) {
                btn.disabled = true;
                btn.textContent = zh ? '上传中…' : 'Uploading...';
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
                    showToast(zh
                        ? `已上传 "${name}"（数据库已标为 Warning，需人工重点审核）`
                        : `Uploaded "${name}" (flagged Warning for review)`);
                } else {
                    showToast(zh
                        ? `已上传 "${name}"（状态：unverified），验证通过后将在 Cloud Store 显示`
                        : `Uploaded "${name}" (status: unverified), visible in Cloud Store after review`);
                }
            } catch (err) {
                showToast((zh ? '上传失败：' : 'Upload failed: ') + (err.message || err));
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
                        desc: 'Keeps up to 8 background tabs warm in memory instead of discarding idle WebViews.',
                        descZh: '允许最多 8 个常用标签页留在后台保持运行（内存较充裕的电脑开启后切标签瞬间响应）。',
                        get: () => (config.tab_cache_mode || 'balanced').toLowerCase() === 'smooth',
                        set: (next) => {
                            config.tab_cache_mode = next ? 'smooth' : 'balanced';
                            config.perf_preset = 'custom';
                        }
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

        function bytesToBase64Chunked(uint8) {
            let bin = '';
            const step = 0x4000;
            for (let i = 0; i < uint8.length; i += step) {
                bin += String.fromCharCode.apply(null, uint8.subarray(i, i + step));
            }
            return btoa(bin);
        }

        function renderChromeExtensionsPane() {
            const zh = isZh();
            const tabBtn = document.getElementById('tabBtnExtChrome');
            if (tabBtn) tabBtn.textContent = zh ? 'Chrome 扩展' : 'Chrome Extensions';
            const banner = document.getElementById('chromeExtEngineBanner');
            if (banner) {
                const wantEn = config.chrome_extensions_enabled !== false;
                if (chromeExtEngineActive) {
                    banner.className = 'sec-audit-banner clean';
                    banner.textContent = zh
                        ? 'WebView2 原生 Chrome 扩展引擎已启动：支持从 Chrome / Edge 商店直装或导入 .crx / .zip / 本地已解压目录（注意：依赖 chrome.tabs / bookmarks / sidePanel 等壳层 API 的扩展可能部分受限）。'
                        : 'WebView2 native Chrome extension engine is active: supports Web Store direct install or local .crx / .zip / unpacked folders.';
                } else {
                    banner.className = 'sec-audit-banner warn';
                    banner.textContent = wantEn
                        ? (zh ? '已开启扩展开关，请重启浏览器后加载 WebView2 扩展引擎。' : 'Extension engine enabled — restart Fynexo to activate.')
                        : (zh ? 'Chrome 扩展引擎当前在「性能与内存」中关闭（极低功耗）。可在下方预装扩展，或前往「性能与内存」开启后重启生效。' : 'Chrome extension engine is currently off in Performance settings; enable & restart to run installed extensions.');
                }
            }

            setPlainRowI18n(document.getElementById('lblChromeStoreInstall'), '从 Chrome / Edge 应用商店下载安装', 'Install from Chrome / Edge Web Store');
            setPlainRowI18n(document.getElementById('hintChromeStoreInstall'), '粘贴 Chrome 商店 / Edge 外接程序链接，或直接填入 32 位扩展 ID 一键下载解压安装。', 'Paste a Chrome Web Store / Edge Add-ons URL or 32-character extension ID to download & install directly.');
            setPlainRowI18n(document.getElementById('lblChromeLocalInstall'), '导入本地 .crx / .zip 或已解压扩展文件夹', 'Import local .crx / .zip or unpacked folder');
            setPlainRowI18n(document.getElementById('hintChromeLocalInstall'), '支持本地离线 .crx3 / .zip 包，或输入包含 manifest.json 的开发者目录路径。', 'Supports offline .crx3 / .zip archives or developer unpacked folder paths containing manifest.json.');

            const stInp = document.getElementById('inputChromeStoreUrl');
            if (stInp) stInp.placeholder = zh ? '粘贴 Chrome 商店网址或 32 位 ID（如 cjpalhdlnbpafiamejdnhcphjbkeiagm）' : 'https://chromewebstore.google.com/detail/... or 32-char ID';
            const btnInst = document.getElementById('btnInstallChromeStore');
            if (btnInst) btnInst.textContent = zh ? '下载安装' : 'Install';
            const btnStore = document.getElementById('btnOpenChromeWebStore');
            if (btnStore) btnStore.textContent = zh ? '逛 Chrome 商店' : 'Web Store';
            const btnCrx = document.getElementById('btnImportCrx');
            if (btnCrx) btnCrx.textContent = zh ? '导入 .crx / .zip' : 'Import .crx / .zip';
            const btnOpenDir = document.getElementById('btnOpenChromeExtDir');
            if (btnOpenDir) btnOpenDir.textContent = zh ? '打开扩展目录' : 'Open folder';
            const dirInp = document.getElementById('inputChromeExtDir');
            if (dirInp) dirInp.placeholder = zh ? '例如 C:\\Extensions\\my-extension（含 manifest.json）' : 'C:\\path\\to\\unpacked-extension';
            const btnDir = document.getElementById('btnInstallChromeDir');
            if (btnDir) btnDir.textContent = zh ? '加载目录' : 'Load folder';

            const listEl = document.getElementById('chromeExtensionsList');
            if (!listEl) return;
            listEl.textContent = '';
            if (!Array.isArray(chromeExtensions) || chromeExtensions.length === 0) {
                const empty = document.createElement('div');
                empty.className = 'set-hint';
                empty.style.padding = '14px';
                empty.textContent = zh
                    ? '暂未安装任何 Chrome 扩展。逛 Chrome 商店时顶部会自动出现一键安装胶囊，也可在此粘贴商店链接或导入 .crx 文件。'
                    : 'No Chrome extensions installed yet. Paste a Web Store URL above or browse Chrome Web Store for 1-click install.';
                listEl.appendChild(empty);
                updateExtensionNavCount();
                return;
            }

            const frag = document.createDocumentFragment();
            chromeExtensions.forEach((ext) => {
                if (!ext) return;
                const row = document.createElement('div');
                row.className = 'rc-row';
                row.style.gap = '10px';

                if (ext.icon_data_url) {
                    const img = document.createElement('img');
                    img.src = ext.icon_data_url;
                    img.alt = '';
                    img.style.cssText = 'width: 22px; height: 22px; border-radius: 5px; object-fit: contain; flex-shrink: 0;';
                    row.appendChild(img);
                } else {
                    const ico = document.createElement('span');
                    ico.style.cssText = 'width: 22px; height: 22px; border-radius: 5px; background: var(--fnx-accent-bg); color: var(--fnx-accent); font-size: 11px; font-weight: 700; display: inline-flex; align-items: center; justify-content: center; flex-shrink: 0;';
                    ico.textContent = String(ext.name || 'C').trim().charAt(0).toUpperCase();
                    row.appendChild(ico);
                }

                const mid = document.createElement('div');
                mid.className = 'rc-mid';
                const titleLine = document.createElement('div');
                titleLine.style.cssText = 'display: flex; align-items: center; gap: 6px; flex-wrap: wrap;';
                const nm = document.createElement('span');
                nm.className = 'rc-name';
                nm.textContent = ext.name;
                titleLine.appendChild(nm);

                const verTag = document.createElement('span');
                verTag.className = 'ext-mem-tag' + (ext.enabled ? ' on' : '');
                verTag.textContent = `v${ext.version || '1.0'} · ~25 MB`;
                titleLine.appendChild(verTag);

                const sub = document.createElement('span');
                sub.className = 'rc-url';
                sub.textContent = ext.description || (ext.runtime_id || ext.id);
                mid.appendChild(titleLine);
                mid.appendChild(sub);
                row.appendChild(mid);

                if (ext.popup_path) {
                    const pBtn = document.createElement('button');
                    pBtn.type = 'button';
                    pBtn.className = 'action-btn compact';
                    pBtn.textContent = zh ? '弹窗' : 'Popup';
                    pBtn.onclick = () => sendIpc({ type: 'OPEN_CHROME_EXT_POPUP', id: ext.id });
                    row.appendChild(pBtn);
                }
                if (ext.options_path) {
                    const oBtn = document.createElement('button');
                    oBtn.type = 'button';
                    oBtn.className = 'action-btn compact';
                    oBtn.textContent = zh ? '选项' : 'Options';
                    oBtn.onclick = () => sendIpc({ type: 'OPEN_CHROME_EXT_OPTIONS', id: ext.id });
                    row.appendChild(oBtn);
                }

                const pinBtn = document.createElement('button');
                pinBtn.type = 'button';
                pinBtn.className = 'action-btn compact';
                pinBtn.textContent = ext.pinned ? (zh ? '已固定' : 'Pinned') : (zh ? '固定顶栏' : 'Pin');
                pinBtn.onclick = () => {
                    ext.pinned = !ext.pinned;
                    renderChromeExtensionsPane();
                    sendIpc({ type: 'PIN_CHROME_EXT', id: ext.id });
                };
                row.appendChild(pinBtn);

                const sw = document.createElement('button');
                sw.type = 'button';
                sw.className = ext.enabled ? 'fnx-switch on' : 'fnx-switch';
                sw.setAttribute('role', 'switch');
                sw.setAttribute('aria-checked', ext.enabled ? 'true' : 'false');
                sw.onclick = () => {
                    ext.enabled = !ext.enabled;
                    renderChromeExtensionsPane();
                    syncPerformanceSection();
                    sendIpc({ type: 'TOGGLE_CHROME_EXT', id: ext.id, enabled: ext.enabled });
                };
                row.appendChild(sw);

                const delBtn = document.createElement('button');
                delBtn.type = 'button';
                delBtn.className = 'rc-del';
                delBtn.title = zh ? '卸载扩展' : 'Uninstall extension';
                delBtn.textContent = '×';
                delBtn.onclick = () => {
                    chromeExtensions = chromeExtensions.filter((x) => x.id !== ext.id);
                    renderChromeExtensionsPane();
                    syncPerformanceSection();
                    sendIpc({ type: 'REMOVE_CHROME_EXT', id: ext.id });
                };
                row.appendChild(delBtn);

                frag.appendChild(row);
            });
            listEl.appendChild(frag);
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
                grants: [],
                connects: [],
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
                else if (key === 'grant') meta.grants.push(val);
                else if (key === 'connect') meta.connects.push(val);
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
            showToast(isZh() ? `已绑定快捷键：${combo}` : `Bound shortcut: ${combo}`);
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
                descZh: '设置新标签页打开地址、网页默认大小、窗口架构，以及重启是否恢复网页。',
                descEn: 'Homepage, User-Agent spoofing, page zoom, window architecture, and session restore.'
            },
            'sec-performance': {
                navZh: '性能与内存',
                navEn: 'Performance',
                titleZh: '性能调优与内存管理',
                titleEn: 'Performance & RAM',
                descZh: '像游戏画质一样一键选择「极低 / 低 / 一般 / 高」，或精细微调每一项耗能参数并实时预览预估 RAM。',
                descEn: 'Pick a game-style resource preset or fine-tune memory, sleeping tabs, and GPU effects with live RAM estimates.'
            },
            'sec-modes': {
                navZh: '场景模式与自动切换',
                navEn: 'Contextual modes',
                titleZh: '场景模式与网站自动切换',
                titleEn: 'Contextual Modes & Auto-Triggers',
                descZh: '为开发、学习、影音等不同场景定制专属功能开关，并设定进入哪些网站（如 localhost）时自动切换到该模式。',
                descEn: 'Configure per-mode feature switches and auto-trigger domain patterns (e.g. auto-enter Developer Mode on localhost).'
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
                navZh: '扩展与脚本',
                navEn: 'Extensions',
                titleZh: '扩展程序与 ScriptForge 脚本',
                titleEn: 'Extensions & ScriptForge',
                descZh: '所有扩展按需开启（关闭时零内存占用），并支持 Chrome 扩展、用户脚本引擎与社区云商店。',
                descEn: 'Opt-in memory-heavy modules (off by default), Chrome extensions, custom scripts, & on-demand cloud store.'
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
                icon: 'unlock',
                chipZh: '解除禁止复制与右键',
                chipEn: 'Unblock Copy & Right-Click',
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
                icon: 'externalLink',
                chipZh: '外链强制新标签打开',
                chipEn: 'Open External Links in New Tab',
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
                icon: 'reader',
                chipZh: '舒适阅读字体与行距',
                chipEn: 'Clean Reading Typography',
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
                icon: 'chevUp',
                chipZh: '悬浮回到顶部按钮',
                chipEn: 'Floating Back-to-Top Button',
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
                icon: 'broom',
                chipZh: '屏蔽常见悬浮弹窗遮罩',
                chipEn: 'Hide Sticky Popups & App Banners',
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
                icon: 'bolt',
                chipZh: '自动点击跳过/展开按钮',
                chipEn: 'Auto-Expand / Skip Prompts',
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

        const MODE_FEATURE_DEFS = [
            ['adblock', 'shield', '广告拦截', 'Adblock Shield'],
            ['force_dark', 'moon', '智能夜间', 'Force Dark'],
            ['ext_side_panel', 'sidepanel', '侧边栏工具', 'Side Panel'],
            ['ext_reader_mode', 'reader', '纯净阅读', 'Reader Mode'],
            ['ext_gh_harmonizer', 'siteGitHub', 'GitHub 美化', 'GitHub Fix'],
            ['ext_yt_ad_skipper', 'play', 'YT 去广告', 'YT Ad Skip'],
            ['ext_media_booster', 'volumeUp', '音量增强', 'Audio Boost'],
            ['ext_userscript_engine', 'code', '油猴脚本', 'ScriptForge'],
            ['ext_mouse_gestures', 'mouse', '鼠标手势', 'Gestures'],
            ['dev_drawer_access', 'terminal', '开发者抽屉菜单', 'DevDrawer Menu'],
            ['fps_hud', 'chart', 'FPS 帧率监视', 'FPS Overlay']
        ];

        function ensureModeProfilesList() {
            if (!Array.isArray(config.mode_profiles) || config.mode_profiles.length === 0) {
                const defs = typeof DEFAULT_MODE_PROFILES !== 'undefined' ? DEFAULT_MODE_PROFILES : [];
                config.mode_profiles = JSON.parse(JSON.stringify(defs));
            }
            return config.mode_profiles;
        }

        function renderModeProfilesSection() {
            const zh = isZh();
            const stratEl = document.getElementById('activeModeSelectGroup');
            const listEl = document.getElementById('modeProfilesContainer');
            if (!stratEl || !listEl) return;

            setPlainRowI18n(document.getElementById('lblActiveModeStrat'), '当前模式触发策略', 'Mode Activation Strategy');
            setPlainRowI18n(document.getElementById('hintActiveModeStrat'), '选“自动切换”后，进入对应模式配置的域名时将自动启用；也可以全局强制锁定在单一模式。', 'Auto switches modes automatically when entering matching sites; or lock onto a single mode globally.');
            setPlainRowI18n(document.getElementById('btnAddModeProfile'), '+ 新建自定义模式', '+ New Mode');
            setPlainRowI18n(document.getElementById('btnResetModeProfiles'), '恢复默认模式', 'Reset defaults');

            const profiles = ensureModeProfilesList();
            const curPref = String(config.active_mode || 'auto').toLowerCase();

            stratEl.textContent = '';
            const stratItems = [
                { id: 'auto', icon: 'spark', label: zh ? '自动按网站切换 (推荐)' : 'Auto by Site' },
                { id: 'default', icon: '', label: zh ? '日常标准 (无特殊模式)' : 'Standard (No Mode)' }
            ];
            profiles.forEach((p) => {
                if (!p || !p.id) return;
                const nm = zh ? (p.name_zh || p.name_en) : (p.name_en || p.name_zh);
                stratItems.push({ id: p.id, icon: resolveModeIconName(p.icon, p.id), label: nm });
            });
            stratItems.forEach((it) => {
                const btn = document.createElement('button');
                btn.type = 'button';
                btn.className = curPref === String(it.id).toLowerCase() ? 'preset-chip active' : 'preset-chip';
                if (it.icon) {
                    setBtnIconAndText(btn, it.icon, it.label);
                } else {
                    btn.textContent = it.label;
                }
                btn.style.fontWeight = curPref === String(it.id).toLowerCase() ? '600' : '500';
                if (curPref === String(it.id).toLowerCase()) {
                    btn.style.borderColor = 'var(--fnx-accent)';
                    btn.style.color = 'var(--fnx-fg)';
                }
                btn.onclick = () => {
                    config.active_mode = it.id;
                    renderModeProfilesSection();
                    saveConfig((zh ? '模式切换：' : 'Active mode: ') + it.label);
                };
                stratEl.appendChild(btn);
            });

            listEl.textContent = '';
            const frag = document.createDocumentFragment();
            profiles.forEach((p, idx) => {
                const card = document.createElement('div');
                card.className = 'set-block';
                card.style.cssText = 'border: 1px solid var(--fnx-hairline); border-radius: 12px; padding: 14px; background: var(--fnx-surface); gap: 12px;';

                const headRow = document.createElement('div');
                headRow.className = 'set-block-head';
                headRow.style.gap = '10px';
                headRow.style.flexWrap = 'wrap';

                const titleWrap = document.createElement('div');
                titleWrap.style.cssText = 'display: flex; align-items: center; gap: 8px; flex: 1; min-width: 200px;';
                const iconBtn = document.createElement('button');
                iconBtn.type = 'button';
                iconBtn.className = 'action-btn compact';
                iconBtn.style.cssText = 'width: 34px; height: 30px; padding: 0;';
                const curIconKey = resolveModeIconName(p.icon, p.id);
                iconBtn.appendChild(createIcon(curIconKey));
                iconBtn.title = zh ? '点击切换模式图标' : 'Click to cycle mode icon';
                iconBtn.onclick = () => {
                    const iIdx = MODE_ICON_CYCLE.indexOf(curIconKey);
                    p.icon = MODE_ICON_CYCLE[(iIdx + 1) % MODE_ICON_CYCLE.length];
                    renderModeProfilesSection();
                    saveConfig(zh ? '已更新模式图标' : 'Updated mode icon');
                };
                const nameInp = document.createElement('input');
                nameInp.type = 'text';
                nameInp.className = 'text-input';
                nameInp.style.cssText = 'max-width: 220px; font-weight: 600;';
                nameInp.value = zh ? (p.name_zh || p.name_en) : (p.name_en || p.name_zh);
                nameInp.onchange = () => {
                    const v = nameInp.value.trim();
                    if (!v) return;
                    if (zh) p.name_zh = v;
                    else p.name_en = v;
                    renderModeProfilesSection();
                    saveConfig((zh ? '已重命名模式：' : 'Renamed mode: ') + v);
                };
                const idTag = document.createElement('span');
                idTag.className = 'field-tag';
                idTag.textContent = p.id;
                titleWrap.appendChild(iconBtn);
                titleWrap.appendChild(nameInp);
                titleWrap.appendChild(idTag);
                headRow.appendChild(titleWrap);

                if (profiles.length > 1) {
                    const delModeBtn = document.createElement('button');
                    delModeBtn.type = 'button';
                    delModeBtn.className = 'action-btn compact';
                    delModeBtn.textContent = zh ? '删除模式' : 'Remove';
                    delModeBtn.onclick = () => {
                        if (String(config.active_mode || '').toLowerCase() === String(p.id).toLowerCase()) {
                            config.active_mode = 'auto';
                        }
                        config.mode_profiles.splice(idx, 1);
                        renderModeProfilesSection();
                        saveConfig(zh ? '已删除场景模式' : 'Removed mode profile');
                    };
                    headRow.appendChild(delModeBtn);
                }
                card.appendChild(headRow);

                // Feature toggle chips (11 toggles)
                const featBox = document.createElement('div');
                featBox.style.cssText = 'display: flex; flex-direction: column; gap: 6px;';
                const featLbl = document.createElement('div');
                featLbl.className = 'set-hint';
                featLbl.textContent = zh ? '该模式下启用的功能（点击随时自由调整开启/关闭）：' : 'Features active in this mode (click to customize):';
                const featChips = document.createElement('div');
                featChips.className = 'preset-chips';
                MODE_FEATURE_DEFS.forEach(([key, iconKey, zhL, enL]) => {
                    const on = !!p[key];
                    const b = document.createElement('button');
                    b.type = 'button';
                    b.className = 'preset-chip';
                    setBtnIconAndText(b, iconKey, zh ? zhL : enL);
                    b.style.borderColor = on ? 'var(--fnx-accent)' : 'var(--fnx-hairline)';
                    b.style.background = on ? 'var(--fnx-accent-bg, rgba(122,136,247,0.14))' : 'transparent';
                    b.style.color = on ? 'var(--fnx-fg)' : 'var(--fnx-secondary)';
                    b.style.opacity = on ? '1' : '0.58';
                    b.onclick = () => {
                        p[key] = !on;
                        renderModeProfilesSection();
                        saveConfig(`${zh ? zhL : enL}: ${p[key] ? (zh ? '已开启' : 'ON') : (zh ? '已关闭' : 'OFF')}`);
                    };
                    featChips.appendChild(b);
                });
                featBox.appendChild(featLbl);
                featBox.appendChild(featChips);
                card.appendChild(featBox);

                // Contextual theme & topbar layout binding
                const skinRow = document.createElement('div');
                skinRow.style.cssText = 'display: flex; flex-wrap: wrap; gap: 14px; align-items: center; padding-top: 6px; border-top: 1px solid var(--fnx-hairline);';
                const mkSelect = (labelZh, labelEn, options, cur, onPick) => {
                    const wrap = document.createElement('label');
                    wrap.style.cssText = 'display: inline-flex; align-items: center; gap: 8px;';
                    const sp = document.createElement('span');
                    sp.className = 'set-hint';
                    sp.textContent = zh ? labelZh : labelEn;
                    const sel = document.createElement('select');
                    sel.className = 'text-input';
                    sel.style.cssText = 'max-width: 190px;';
                    options.forEach(([val, zhT, enT]) => {
                        const o = document.createElement('option');
                        o.value = val;
                        o.textContent = zh ? zhT : enT;
                        if (val === (cur || '')) o.selected = true;
                        sel.appendChild(o);
                    });
                    sel.onchange = () => onPick(sel.value);
                    wrap.appendChild(sp);
                    wrap.appendChild(sel);
                    return wrap;
                };
                const themeOpts = [['', '跟随全局', 'Follow global']];
                themes.forEach((th) => {
                    if (!th || !th.id) return;
                    const baseName = th.name || th.id;
                    const labelZh = th.builtin ? baseName : `${baseName}（自定义）`;
                    const labelEn = th.builtin ? baseName : `${baseName} (Custom)`;
                    themeOpts.push([th.id, labelZh, labelEn]);
                });
                skinRow.appendChild(mkSelect('专属主题', 'Theme', themeOpts, p.theme, (v) => {
                    p.theme = v;
                    saveConfig(zh ? '已更新模式专属主题' : 'Updated mode theme');
                }));
                skinRow.appendChild(mkSelect('专属顶栏布局', 'Topbar layout', [
                    ['', '跟随全局', 'Follow global'],
                    ['deck', '双层', 'Deck'],
                    ['compact', '单行紧凑', 'Compact'],
                    ['vertical', '左侧竖排', 'Vertical']
                ], p.layout, (v) => {
                    p.layout = v;
                    saveConfig(zh ? '已更新模式专属顶栏布局' : 'Updated mode layout');
                }));
                card.appendChild(skinRow);
                // Auto-trigger domains chips + add input
                const domBox = document.createElement('div');
                domBox.style.cssText = 'display: flex; flex-direction: column; gap: 6px; padding-top: 6px; border-top: 1px solid var(--fnx-hairline);';
                const domLbl = document.createElement('div');
                domLbl.className = 'set-hint';
                domLbl.textContent = zh ? '进入以下网站时自动切换到该模式（点击 × 移除，支持 localhost / *.github.com 等）：' : 'Auto-activate when visiting domains (supports localhost, *.github.com, etc.):';
                const domChips = document.createElement('div');
                domChips.className = 'preset-chips';
                const domList = Array.isArray(p.auto_domains) ? p.auto_domains : (p.auto_domains = []);
                domList.forEach((d, dIdx) => {
                    const dc = document.createElement('button');
                    dc.type = 'button';
                    dc.className = 'preset-chip mono';
                    dc.textContent = `${d} ×`;
                    dc.title = zh ? '点击移除该网站匹配规则' : 'Click to remove domain';
                    dc.onclick = () => {
                        domList.splice(dIdx, 1);
                        renderModeProfilesSection();
                        saveConfig(zh ? `已移除自动触发域名：${d}` : `Removed ${d}`);
                    };
                    domChips.appendChild(dc);
                });
                const domAddRow = document.createElement('div');
                domAddRow.className = 'rc-add-row';
                domAddRow.style.padding = '4px 0 0 0';
                const domInp = document.createElement('input');
                domInp.type = 'text';
                domInp.className = 'text-input mono';
                domInp.style.maxWidth = '240px';
                domInp.placeholder = zh ? '例如 localhost 或 *.bilibili.com' : 'e.g. localhost or *.github.com';
                const domAddBtn = document.createElement('button');
                domAddBtn.type = 'button';
                domAddBtn.className = 'action-btn compact';
                domAddBtn.textContent = zh ? '+ 添加匹配网站' : '+ Add Site';
                const addDomFn = () => {
                    const clean = domInp.value.trim().toLowerCase().replace(/^https?:\/\//, '').split('/')[0];
                    if (!clean) return;
                    if (!domList.includes(clean)) domList.push(clean);
                    domInp.value = '';
                    renderModeProfilesSection();
                    saveConfig((zh ? '已添加到模式触发域名：' : 'Added trigger domain: ') + clean);
                };
                domAddBtn.onclick = addDomFn;
                domInp.onkeydown = (ev) => {
                    if (ev.key === 'Enter') {
                        ev.preventDefault();
                        addDomFn();
                    }
                };
                domAddRow.appendChild(domInp);
                domAddRow.appendChild(domAddBtn);
                domBox.appendChild(domLbl);
                domBox.appendChild(domChips);
                domBox.appendChild(domAddRow);
                card.appendChild(domBox);

                frag.appendChild(card);
            });
            listEl.appendChild(frag);
        }

        function setPlainRowI18n(el, zhText, enText) {
            setRichI18nText(el, zhText, enText, isZh());
        }

        function setRowPairI18n(ctrlId, lblZh, lblEn, hintZh, hintEn) {
            const el = document.getElementById(ctrlId);
            const row = el && el.closest('.set-row');
            if (!row) return;
            const zh = isZh();
            const lblEl = row.querySelector('.set-label > span:first-child') || row.querySelector('.set-label');
            setRichI18nText(lblEl, lblZh, lblEn, zh);
            if (hintZh) {
                setRichI18nText(row.querySelector('.set-hint'), hintZh, hintEn === undefined ? null : hintEn, zh);
            }
        }

        function applySettingsLocalization() {
            const zh = isZh();
            syncDocLangAndTitle(zh, '偏好设置 · Fynexo', 'Preferences · Fynexo');

            const brandTitle = document.querySelector('.brand-title');
            if (brandTitle) brandTitle.textContent = zh ? '偏好设置' : 'Preferences';

            const langBtn = document.getElementById('btnSettingsLang');
            if (langBtn) {
                langBtn.textContent = zh ? '中文 / EN' : 'EN / 中文';
            }
            const searchInp = document.getElementById('inputSettingsSearch');
            if (searchInp) {
                searchInp.placeholder = zh
                    ? '搜索设置 (深色 / 端口 / 手势)...'
                    : 'Search settings...';
            }
            const footerEl = document.getElementById('sidebarFooterText');
            if (footerEl) footerEl.textContent = zh ? 'Fynexo Studio 工作室 · v0.1.0' : 'Fynexo Studio · v0.1.0';

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
                        setRichI18nText(descEl, info.descZh, info.descEn, zh);
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
                setRichI18nText(row.querySelector('.set-label'), lblZh, lblEn, zh);
                setRichI18nText(row.querySelector('.set-hint'), hintZh, null, zh);
            });

            [
                ['langModeGroup', '界面语言', 'Interface language', '实时切换浏览器全部菜单、起始页、偏好设置与提示文案（无需刷新页面）'],
                ['inputHomepage', '新标签起始页网址', 'New tab homepage', '打开新标签页时默认加载的页面地址'],
                ['uaModeGroup', '浏览器身份标识 (User-Agent)', 'User-Agent', '实时切换系统默认、桌面 Chrome、手机或自定义 UA 请求头'],
                ['btnZoomOut', '默认网页缩放比例', 'Default page zoom', '所有浏览标签页的基础缩放大小'],
                ['btnRestoreToggle', '重启恢复上次标签与滚动位置', 'Restore tabs & scroll position', '重新启动浏览器时自动恢复上次未关闭的网页及浏览位置'],
                ['btnAllowPageEvalToggle', '允许控制台 JS 执行与原生 DevTools', 'Allow Console JS Eval & Native DevTools', '开启后可在 DevDrawer 控制台执行交互式 JS 表达式，并在 Release 构建中打开 F12 原生开发者工具（默认关闭以防恶意脚本滥用）'],
                ['btnCustomColorsToggle', '自定义调色盘', 'Custom color palette', '可视化调节界面色彩 Token 并导出为独立主题 `.json` 文件'],
                ['btnAdblockToggle', '网络广告与追踪拦截盾', 'Network ad & tracker shield', '内置 65+ 常见广告与追踪器域名黑名单及自定义过滤规则'],
                ['btnIncognitoToggle', '无痕隐私模式', 'Private / Incognito mode', '开启后暂停本地历史记录与标签会话写入'],
                ['btnClearHistory', '配置文件与缓存目录', 'Profile storage & cache', '包含 JSON 配置文件、浏览历史、Cookie 与 WebView 本地缓存'],
                ['btnPerfToggle', 'Zero-FX 渲染过滤', 'Zero-FX rendering filter', '移除外部网页高耗阴影与高斯模糊（默认关闭以保留网页原本光影特效，低配设备可开启）'],
                ['btnEfficiencyToggle', 'Edge 风格后台能效压缩', 'Edge-style Efficiency Mode', '自动将后台非当前标签降级为低内存水位（释放显存与位图缓存，切回页面无需重载）'],
                ['perfCacheModeGroup', '后台缓存标签上限', 'Background cached tab limit', '保留多少个近期打开的普通标签在后台秒切；超出后自动回收最久未用的标签'],
                ['selTabSleepMinutes', '闲置标签自动休眠', 'Put idle background tabs to sleep', '后台标签超过设定时间未操作后自动休眠彻底释放内存（点击标签页时自动唤醒加载）'],
                ['btnAutoTrimToggle', '闲置工作集自动整理', 'Auto working-set RAM trim', '最小化窗口或休眠标签后自动整理主机工作集内存'],
                ['btnReduceAnimToggle', '减弱界面与网页动画', 'Reduce UI & page animations', '禁用过渡动画与非必要动效，降低合成器开销与掉帧'],
                ['btnHwAccelToggle', 'GPU 硬件加速', 'GPU hardware acceleration', '启用显卡光栅化与零拷贝合成以保证视频与滚动流畅（关闭可省独显/集显显存，需重启生效）'],
                ['btnPerfAdblockToggle', '网络广告与追踪拦截盾', 'Network Adblock Shield', '拦截臃肿广告脚本与追踪器；开启后反而能大幅加快加载并降低每标签 RAM'],
                ['btnPerfManageExt', '内建功能模块与扩展开销', 'Built-in opt-in modules & extensions', '当前启用的高耗扩展都会增加每页内存，未启用模块为 0 内存占用'],
                ['btnChromeExtEngineToggle', 'Chrome 原生扩展引擎 (WebView2)', 'Chrome native extension engine', '启用 Windows WebView2 Chrome/Edge 扩展支持；「极低」预设会在下次启动时关闭以节省基础开销'],
                ['btnPerfToggleWinMode', '窗口架构对内存的影响', 'Window architecture mode', '多窗口支持独立进程/无痕隔离（每个额外窗口约 +80~150 MB）；单窗口可控制在单一窗口内'],
                ['btnHttpsFirstToggle', 'HTTPS 优先安全连接 (HTTPS-First)', 'Always use HTTPS (HTTPS-First)', '访问外部 `http://` 网站时自动升级为加密 `https://`（本地开发域名仍保留 `http://`）'],
                ['inputDownloadsDir', '下载文件保存目录', 'Downloads folder', '自定义下载保存目录（留空则使用系统默认“下载”文件夹）'],
                ['windowModeGroup', '窗口架构模式', 'Window architecture', '多窗口（默认）支持 Ctrl+N 新开独立窗口及隔离无痕窗口；单窗口则全部收口在同一窗口内'],
                ['btnImportRules', '自定义与云端扩展规则 (`rules.json`)', 'Custom & downloaded extensions (`rules.json`)', '支持 CSS 与 ScriptForge (`// ==UserScript==` 与 `GM_*` API)，关闭状态 0 内存占用。']
            ].forEach(([id, lblZh, lblEn, hintZh]) => setRowPairI18n(id, lblZh, lblEn, hintZh));

            document.querySelectorAll('#windowModeGroup .seg-btn').forEach((b) => {
                if (b.dataset.wm === 'multi') b.textContent = zh ? '多窗口（默认）' : 'Multi-Window';
                else if (b.dataset.wm === 'single') b.textContent = zh ? '单窗口' : 'Single-Window';
            });
            document.querySelectorAll('#uaModeGroup .seg-btn').forEach((b) => {
                if (b.dataset.ua === 'default') b.textContent = zh ? '系统默认' : 'System';
                else if (b.dataset.ua === 'desktop') b.textContent = zh ? '桌面端' : 'Desktop';
                else if (b.dataset.ua === 'mobile') b.textContent = zh ? '手机端' : 'Mobile';
                else if (b.dataset.ua === 'custom') b.textContent = zh ? '自定义' : 'Custom';
            });

            // Appearance section buttons, Palette Studio & Themes
            setPlainRowI18n(document.getElementById('btnImportTheme'), '导入 .json', 'Import .json');
            setPlainRowI18n(document.getElementById('btnExportTheme'), '导出 .json', 'Export .json');
            setPlainRowI18n(document.getElementById('btnOpenThemesDir'), '打开 themes/', 'Open themes/');
            setPlainRowI18n(document.getElementById('btnSaveAsTheme'), '保存主题 .json', 'Save theme .json');
            const newThemeInp = document.getElementById('inputNewThemeName');
            if (newThemeInp) newThemeInp.placeholder = zh ? '主题名称（例如 Solarized）' : 'Theme name (e.g. Solarized)';
            const palStudioHead = document.querySelector('#paletteStudio .set-block-head');
            if (palStudioHead) {
                setRichI18nText(palStudioHead.querySelector('.set-label'), '可视化调色盘', 'Visual Color Designer', zh);
                setRichI18nText(palStudioHead.querySelector('.set-hint'), '实时拾取色板并导出为独立 `themes/*.json` 主题文件。', 'Pick swatches live — export as standalone `themes/*.json`.', zh);
            }
            const themeBlockHead = document.querySelector('#sec-appearance > .set-block .set-block-head');
            if (themeBlockHead) {
                setRichI18nText(themeBlockHead.querySelector('.set-label'), '主题皮肤', 'Themes', zh);
            }

            // General homepage preset chips
            document.querySelectorAll('#sec-general [data-hp="fynexo://newtab"]').forEach((chip) => {
                chip.textContent = zh ? '起始页' : 'Start page';
            });

            // Performance section headers & actions
            setPlainRowI18n(document.getElementById('btnPerfRefreshRam'), '刷新实时 RAM', 'Refresh RAM');
            setPlainRowI18n(document.getElementById('btnPerfFlushRam'), '释放闲置内存', 'Flush idle RAM');
            setPlainRowI18n(document.getElementById('lblPerfEstKicker'), '当前配置预估多进程内存', 'Estimated Typical RAM Footprint');
            setPlainRowI18n(document.getElementById('lblPerfLiveKicker'), '实时多进程内存监视', 'Live Multi-Process Memory');
            setPlainRowI18n(document.getElementById('lblPerfPresets'), '性能与功耗预设档位', 'Performance Quality Preset');
            setPlainRowI18n(document.getElementById('hintPerfPresets'), '像游戏画质预设一样 1 键切换，无需理解底层参数', '1-click tuning for beginners — like game graphics presets');
            setPlainRowI18n(document.getElementById('grpPerfMem'), '内存控制与休眠策略', 'Memory & Sleeping Tabs');
            setPlainRowI18n(document.getElementById('grpPerfGpu'), '渲染过滤与硬件加速', 'Rendering & Hardware');
            setPlainRowI18n(document.getElementById('grpPerfModules'), '各模块对内存的影响', 'Features Affecting Memory');

            // Search & !Bangs
            const searchBlocks = document.querySelectorAll('#sec-search .set-block');
            if (searchBlocks[0]) {
                setRichI18nText(searchBlocks[0].querySelector('.set-label'), '搜索引擎列表', 'Search engines', zh);
                setRichI18nText(searchBlocks[0].querySelector('.set-hint'), '点击任意一行设为默认搜索引擎，模板支持 `{query}` 占位符。', 'Click any row to make it default. Templates use `{query}`.', zh);
            }
            if (searchBlocks[1]) {
                setRichI18nText(searchBlocks[1].querySelector('.set-label'), '!Bang 快捷搜索指令', '!Bang triggers', zh);
                setRichI18nText(searchBlocks[1].querySelector('.set-hint'), '在地址栏或起始页输入如 `!gh wry`、`!yt music` 直接跳转对应站内搜索。', 'Type e.g. `!gh wry` or `!yt music` in the address bar or start page.', zh);
            }
            const engNameInp = document.getElementById('inputEngName');
            if (engNameInp) engNameInp.placeholder = zh ? '名称（如 Kagi）' : 'Name (e.g. Kagi)';
            setPlainRowI18n(document.getElementById('btnAddEngine'), '+ 添加', '+ Add');
            const bangNameInp = document.getElementById('inputBangName');
            if (bangNameInp) bangNameInp.placeholder = zh ? '名称' : 'Name';
            setPlainRowI18n(document.getElementById('btnAddBang'), '+ 添加', '+ Add');

            // Shortcuts add row
            const scTitleInp = document.getElementById('inputScTitle');
            if (scTitleInp) scTitleInp.placeholder = zh ? '名称（如 Hacker News）' : 'Name (e.g. Hacker News)';
            setPlainRowI18n(document.getElementById('btnAddShortcut'), '+ 添加快捷网站', '+ Add shortcut');

            // Adblock shield
            setPlainRowI18n(document.getElementById('btnImportAdblock'), '导入 .json', 'Import .json');
            setPlainRowI18n(document.getElementById('btnExportAdblock'), '导出 .json', 'Export .json');
            const adCols = document.querySelectorAll('#sec-rules .domain-col-head > span:first-child');
            if (adCols[0]) adCols[0].textContent = zh ? '自定义黑名单域名' : 'Blocked domains';
            if (adCols[1]) adCols[1].textContent = zh ? '白名单放行域名' : 'Whitelisted domains';
            setPlainRowI18n(document.getElementById('btnAddBlockDomain'), '+ 屏蔽', '+ Block');
            setPlainRowI18n(document.getElementById('btnAddWhiteDomain'), '+ 放行', '+ Allow');

            // Privacy & Data
            setPlainRowI18n(document.getElementById('btnImportBundle'), '导入全量备份 .json', 'Import setup .json');
            setPlainRowI18n(document.getElementById('btnExportBundle'), '导出全量备份 .json', 'Export setup .json');
            setPlainRowI18n(document.getElementById('btnClearSiteData'), '清空缓存与 Cookie', 'Clear cache & cookies');
            setPlainRowI18n(document.getElementById('btnOpenDataDir'), '打开配置目录', 'Profile folder');
            setPlainRowI18n(document.getElementById('btnOpenDownloadsDir'), '下载文件夹', 'Downloads');

            // Extensions sub-tabs, custom editor & cloud store
            const extTabLabels = {
                'pane-ext-builtin': ['内建模块', 'Built-in'],
                'pane-ext-chrome': ['Chrome 扩展', 'Chrome Extensions'],
                'pane-ext-custom': ['自定义与已安装', 'Installed'],
                'pane-ext-cloud': ['云端扩展商店', 'Cloud Store']
            };
            document.querySelectorAll('#extSubTabs .seg-btn').forEach((btn) => {
                const pair = extTabLabels[btn.dataset.subpane];
                if (pair) btn.textContent = zh ? pair[0] : pair[1];
            });
            setPlainRowI18n(document.getElementById('btnImportRules'), '导入 .json', 'Import .json');
            setPlainRowI18n(document.getElementById('btnExportRules'), '导出 .json', 'Export .json');
            setPlainRowI18n(document.getElementById('btnResetRules'), '恢复默认', 'Reset');
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
            const ruleNameLbl = document.querySelector('#ruleEditorPanel label[for="ruleNameInput"] > span');
            if (ruleNameLbl) ruleNameLbl.textContent = zh ? '扩展名称' : 'Extension name';
            const ruleNameInp = document.getElementById('ruleNameInput');
            if (ruleNameInp) ruleNameInp.placeholder = zh ? '干净阅读去干扰' : 'Clean Distractions';
            const ruleDomLbl = document.querySelector('#ruleEditorPanel label[for="ruleDomainInput"] > span:first-child');
            if (ruleDomLbl) ruleDomLbl.textContent = zh ? '匹配域名' : 'Domain pattern';
            const ruleDomTag = document.querySelector('#ruleEditorPanel label[for="ruleDomainInput"] .field-tag');
            if (ruleDomTag) ruleDomTag.textContent = zh ? '* 或 example.com' : '* or example.com';

            // Cloud Store controls
            const cloudSearchInp = document.getElementById('inputCloudSearch');
            if (cloudSearchInp) cloudSearchInp.placeholder = zh ? '搜索云端扩展…' : 'Search cloud extensions...';
            setPlainRowI18n(document.getElementById('btnCloudUpload'), '+ 上传扩展', '+ Upload Extension');
            setPlainRowI18n(document.getElementById('btnToggleCloudCfg'), 'API 配置', 'API Config');
            setPlainRowI18n(document.getElementById('btnRefreshCloud'), '刷新', 'Refresh');
            setPlainRowI18n(document.getElementById('btnSaveCloudCfg'), '保存并连接', 'Save & Connect');
            setPlainRowI18n(document.getElementById('btnSubmitPublish'), '提交审核', 'Submit for verification');
            setPlainRowI18n(document.getElementById('btnCancelPublish'), '取消', 'Cancel');
            setPlainRowI18n(document.getElementById('btnInspClose'), '关闭', 'Close');
            const pubLabels = [
                ['label[for="pubExtName"] > span:first-child', '扩展名称', 'Extension name'],
                ['label[for="pubExtDomain"] > span:first-child', '匹配域名', 'Domain pattern'],
                ['label[for="pubExtAuthor"] > span:first-child', '作者昵称', 'Author'],
                ['label[for="pubExtCategory"] > span:first-child', '分类', 'Category'],
                ['label[for="pubExtDesc"] > span:first-child', '扩展简介', 'Description'],
                ['label[for="pubExtCss"] > span:first-child', '自定义 CSS', 'Custom CSS'],
                ['label[for="pubExtJs"] > span:first-child', '自定义 JS', 'Custom JS'],
                ['label[for="inspExtCss"] > span:first-child', 'CSS 源码（只读）', 'CSS Source (read-only)'],
                ['label[for="inspExtJs"] > span:first-child', 'JS 源码（只读）', 'JS Source (read-only)']
            ];
            pubLabels.forEach(([sel, zTxt, eTxt]) => {
                const el = document.querySelector(`#pane-ext-cloud ${sel}`);
                if (el) el.textContent = zh ? zTxt : eTxt;
            });
            refreshCloudCategorySelect();

            // Component Studio sub-tabs, select options, switches, presets & JSON toolbar
            document.querySelectorAll('#studioSubTabs .seg-btn').forEach((btn) => {
                if (btn.dataset.subpane === 'pane-studio-visual') btn.textContent = zh ? '可视化调整' : 'Visual controls';
                else if (btn.dataset.subpane === 'pane-studio-json') btn.textContent = 'styles.json';
            });
            const uiPresetLabels = {
                compact: ['极简紧凑', 'Compact (1-row)'],
                deck: ['经典双栏', 'Deck (2-row)'],
                pill: ['大圆角胶囊', 'Pill Capsule'],
                vertical: ['左侧竖排边栏', 'Left Sidebar']
            };
            document.querySelectorAll('[data-ui-preset]').forEach((btn) => {
                const pair = uiPresetLabels[btn.dataset.uiPreset];
                if (pair) btn.textContent = zh ? pair[0] : pair[1];
            });
            setRowPairI18n(
                'qsTopbarLayout',
                '顶栏 / 侧边栏布局方式',
                'Header layout & shortcuts bar',
                '`topbar.layout` · `topbar.bookmarks_bar`',
                '`topbar.layout` · `topbar.bookmarks_bar`'
            );
            const firstPresetRow = document.querySelector('#pane-studio-visual .set-row');
            if (firstPresetRow) {
                setRichI18nText(firstPresetRow.querySelector('.set-label'), '一键界面风格预设', 'One-click UI layout preset', zh);
                setRichI18nText(firstPresetRow.querySelector('.set-hint'), '不懂像素参数也没关系，点一下即可套用整套顶栏比例与圆角风格。', 'Apply complete chrome proportions and corner radii in one click.', zh);
            }
            applySelectOptionsI18n('qsTopbarLayout', { deck: '双层顶栏 (2行)', compact: '单行紧凑', vertical: '左侧竖排边栏' }, { deck: 'Layered deck (2-row)', compact: 'Compact (1-row)', vertical: 'Vertical sidebar (left)' }, zh);
            applySelectOptionsI18n('qsBookmarksBar', { always: '始终显示', newtab: '仅起始页显示', hidden: '隐藏' }, { always: 'Always show', newtab: 'Start page only', hidden: 'Hidden' }, zh);
            applySelectOptionsI18n('qsAddrMode', { inline: '顶部直输', modal: '居中弹窗' }, { inline: 'Inline input', modal: 'Modal' }, zh);
            applySelectOptionsI18n('qsAddrPos', { center: '居中', right: '靠右', left: '靠左' }, { center: 'Center', right: 'Right', left: 'Left' }, zh);
            applySelectOptionsI18n('qsMobilePos', { bottom: '底部', top: '顶部' }, { bottom: 'Bottom', top: 'Top' }, zh);
            applySelectOptionsI18n('qsNtCols', { '0': '自动 (6 列)', '2': '2 列', '3': '3 列', '4': '4 列', '5': '5 列', '6': '6 列' }, { '0': 'Auto (6 cols)', '2': '2 columns', '3': '3 columns', '4': '4 columns', '5': '5 columns', '6': '6 columns' }, zh);

            const switchLabelMap = [
                ['#qsBtnCopyToggle', '复制', 'Copy'],
                ['#qsBtnStarToggle', '收藏', 'Star'],
                ['#qsBtnShieldToggle', '拦截盾', 'Shield'],
                ['#qsBtnDlToggle', '下载', 'Downloads'],
                ['#qsTabDotToggle', '网站标', 'Badge'],
                ['#qsFullWidthToggle', '宽屏', 'Wide'],
                ['#qsNtClockToggle', '大时钟', 'Clock'],
                ['#qsNt24hToggle', '24小时制', '24h'],
                ['#qsNtScToggle', '常用网站', 'Shortcuts'],
                ['#qsNtHistToggle', '历史记录', 'History']
            ];
            switchLabelMap.forEach(([idSel, zTxt, eTxt]) => {
                const btn = document.querySelector(idSel);
                const sp = btn && btn.previousElementSibling;
                if (sp && sp.classList.contains('switch-status')) sp.textContent = zh ? zTxt : eTxt;
            });

            setPlainRowI18n(document.getElementById('btnNtWallpaperUpload'), '上传图片', 'Upload');
            setPlainRowI18n(document.getElementById('btnNtWallpaperClear'), '清除', 'Clear');
            const wpLabels = [
                ['', '默认无壁纸', 'None'],
                ['linear-gradient(135deg, #0f172a 0%, #1e1b4b 45%, #311042 100%)', '深空极光', 'Aurora Dark'],
                ['linear-gradient(135deg, #082f49 0%, #0f172a 55%, #022c22 100%)', '静谧深海', 'Abyssal Ocean'],
                ['linear-gradient(135deg, #2e1065 0%, #4c1d95 45%, #831843 100%)', '暮光紫霞', 'Twilight Dusk'],
                ['linear-gradient(135deg, #052e16 0%, #14532d 50%, #0f172a 100%)', '森系暗绿', 'Deep Forest'],
                ['linear-gradient(135deg, #431407 0%, #7c2d12 48%, #3b0764 100%)', '暖阳落日', 'Warm Sunset']
            ];
            document.querySelectorAll('[data-wp]').forEach((btn, i) => {
                const pair = wpLabels[i];
                if (pair) btn.textContent = zh ? pair[1] : pair[2];
            });
            setPlainRowI18n(document.getElementById('btnReloadJson'), '重新加载', 'Reload');
            setPlainRowI18n(document.getElementById('btnResetJson'), '重置默认', 'Reset');
            setPlainRowI18n(document.getElementById('btnSaveJson'), '应用并保存', 'Apply & save');
            const jsonAutoHint = document.querySelector('#jsonStatus > span:last-child');
            if (jsonAutoHint) jsonAutoHint.textContent = zh ? '窗口聚焦时自动刷新同步' : 'Auto-reloads on window focus';

            // Keybinds header buttons
            setPlainRowI18n(document.getElementById('btnImportKeybinds'), '导入 .json', 'Import .json');
            setPlainRowI18n(document.getElementById('btnExportKeybinds'), '导出 .json', 'Export .json');
            setPlainRowI18n(document.getElementById('btnResetKeybinds'), '恢复默认', 'Reset defaults');

            document.querySelectorAll('#ruleNoCodeBar [data-nocode], #pubNoCodeBar [data-pub-nocode]').forEach((chip) => {
                const key = chip.dataset.nocode || chip.dataset.pubNocode;
                const b = key && NO_CODE_BLOCKS[key];
                if (b) setBtnIconAndText(chip, b.icon, zh ? b.chipZh : b.chipEn);
            });
        }

        function syncFormControls() {
            const zh = isZh();
            document.getElementById('themesDirLabel').textContent = themesDir;
            document.getElementById('stylesPathLabel').textContent = stylesPath;
            const pathDisplay = document.getElementById('stylesPathDisplay');
            if (pathDisplay) pathDisplay.textContent = stylesPath;
            document.getElementById('dataDirLabel').textContent = dataDir || stylesPath;
            setPlainRowI18n(document.getElementById('btnImportShortcuts'), '导入 (Chrome HTML / JSON)', 'Import HTML / JSON');
            setPlainRowI18n(document.getElementById('btnExportShortcuts'), '导出 .json', 'Export .json');
            setPlainRowI18n(document.getElementById('btnResetShortcuts'), '恢复默认', 'Reset defaults');
            document.getElementById('shortcutsHint').textContent = zh
                ? `已同步 ${bookmarks.length} 个常用网站卡片（支持直接导入 Chrome / Edge / Firefox 导出的书签 HTML 或 JSON）。`
                : `${bookmarks.length} saved shortcut${bookmarks.length === 1 ? '' : 's'} (supports importing Chrome / Edge / Firefox bookmarks.html or JSON).`;

            document.getElementById('inputHomepage').value = config.homepage || 'fynexo://newtab';
            const inpDlDir = document.getElementById('inputDownloadsDir');
            const btnResetDlDir = document.getElementById('btnResetDownloadsDir');
            if (inpDlDir) {
                if (document.activeElement !== inpDlDir) {
                    inpDlDir.value = config.downloads_dir || '';
                }
                inpDlDir.placeholder = zh ? '系统默认下载文件夹（Downloads）' : 'Default system Downloads folder';
            }
            if (btnResetDlDir) btnResetDlDir.textContent = zh ? '恢复默认' : 'Default';

            const curLang = isZh() ? 'zh' : 'en';
            document.querySelectorAll('#langModeGroup .seg-btn').forEach(b => {
                b.classList.toggle('active', b.dataset.lang === curLang);
            });

            const uaMode = (config.user_agent_mode || 'default').toLowerCase();
            document.querySelectorAll('#uaModeGroup .seg-btn').forEach(b => {
                b.classList.toggle('active', b.dataset.ua === uaMode);
            });
            const customUaInp = document.getElementById('inputCustomUa');
            customUaInp.style.display = uaMode === 'custom' ? 'block' : 'none';
            customUaInp.value = config.custom_user_agent || '';

            const zoomPct = Math.round((Number(config.default_zoom) || 1) * 100);
            document.getElementById('btnZoomReset').textContent = `${zoomPct}%`;

            const curWMode = (config.window_mode || 'multi').toLowerCase() === 'single' ? 'single' : 'multi';
            document.querySelectorAll('#windowModeGroup .seg-btn').forEach(b => {
                b.classList.toggle('active', b.dataset.wm === curWMode);
            });

            setSwitchState('btnIncognitoToggle', !!config.incognito);
            setSwitchState('btnHttpsFirstToggle', config.https_first !== false);

            const perfOn = !!config.enforce_perf;
            setSwitchState('btnPerfToggle', perfOn);
            syncPerformanceSection();

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
            setSwitchState('btnAllowPageEvalToggle', !!config.allow_page_eval);

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
            showToast(msg || (isZh() ? '设置已保存' : 'Preferences saved'));
        }

        function pushStyles(msg) {
            document.getElementById('jsonEditor').value = JSON.stringify(styles, null, 2);
            applySettingsSurface();
            syncFormControls();
            sendIpc({ type: 'SAVE_STYLES', styles: styles });
            setJsonStatus(false, isZh() ? '已实时应用并保存到 styles.json。' : 'Applied live and persisted to styles.json.');
            showToast(msg || (isZh() ? '界面样式已更新' : 'Styles updated'));
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
                    showToast(isZh() ? '已复制主题文件夹路径' : 'Copied themes folder path');
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
            pushStyles(styles.colors.use_custom_colors
                ? (isZh() ? '已启用自定义调色盘' : 'Visual color designer enabled')
                : (isZh() ? '已恢复使用主题预设配色' : 'Using JSON theme preset'));
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
            const zh = isZh();
            const nm = document.getElementById('inputEngName').value.trim();
            const u = document.getElementById('inputEngUrl').value.trim();
            if (!nm || !u) return showToast(zh ? '请输入搜索引擎名称与网址' : 'Enter engine name and URL');
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
            saveConfig(zh ? `已添加并切换搜索引擎：${nm}` : `Added search engine: ${nm}`);
        };

        document.getElementById('btnAddBang').onclick = () => {
            const zh = isZh();
            let trig = document.getElementById('inputBangTrig').value.trim().toLowerCase();
            const nm = document.getElementById('inputBangName').value.trim();
            const u = document.getElementById('inputBangUrl').value.trim();
            if (!trig || !u) return showToast(zh ? '请输入 !指令 与搜索网址模板' : 'Enter !trigger and search URL');
            if (!trig.startsWith('!')) trig = '!' + trig;
            const fullUrl = /^https?:\/\//i.test(u) ? u : ('https://' + u);
            const list = getBangs().filter(x => (x.trigger || '').toLowerCase() !== trig);
            list.push({ trigger: trig, name: nm || trig.slice(1), url: fullUrl });
            config.bangs = list;
            document.getElementById('inputBangTrig').value = '';
            document.getElementById('inputBangName').value = '';
            document.getElementById('inputBangUrl').value = '';
            renderBangsTable();
            saveConfig(zh ? `已保存 !Bang 指令：${trig}` : `Saved bang ${trig}`);
        };

        const inputHp = document.getElementById('inputHomepage');
        inputHp.onchange = () => {
            const val = inputHp.value.trim();
            if (val) { config.homepage = val; saveConfig(isZh() ? '已更新新标签页地址' : 'Homepage updated'); }
        };
        const inputDlDir = document.getElementById('inputDownloadsDir');
        if (inputDlDir) {
            inputDlDir.onchange = () => {
                config.downloads_dir = inputDlDir.value.trim();
                saveConfig(config.downloads_dir
                    ? (isZh() ? '已更新下载保存目录' : 'Downloads folder updated')
                    : (isZh() ? '已恢复默认下载目录' : 'Restored default Downloads folder'));
            };
        }
        const btnResetDlDir = document.getElementById('btnResetDownloadsDir');
        if (btnResetDlDir) {
            btnResetDlDir.onclick = () => {
                config.downloads_dir = '';
                syncFormControls();
                saveConfig(isZh() ? '已恢复系统默认下载目录' : 'Restored default Downloads folder');
            };
        }
        document.querySelectorAll('.preset-chip').forEach(chip => {
            chip.onclick = () => {
                const hp = chip.dataset.hp;
                if (hp) {
                    config.homepage = hp;
                    inputHp.value = hp;
                    saveConfig((isZh() ? '已设置主页：' : 'Homepage: ') + hp);
                }
            };
        });

        document.querySelectorAll('#langModeGroup .seg-btn').forEach(btn => {
            btn.onclick = () => {
                const nextLang = btn.dataset.lang === 'en' ? 'en' : 'zh';
                config.language = nextLang;
                renderAllSections();
                saveConfig(nextLang === 'en' ? 'Switched interface to English' : '已切换为简体中文界面');
            };
        });

        document.querySelectorAll('#uaModeGroup .seg-btn').forEach(btn => {
            btn.onclick = () => {
                config.user_agent_mode = btn.dataset.ua;
                syncFormControls();
                saveConfig((isZh() ? 'UA 模式：' : 'User-Agent mode: ') + btn.textContent);
            };
        });
        document.getElementById('inputCustomUa').onchange = (e) => {
            config.custom_user_agent = e.target.value.trim();
            saveConfig(isZh() ? '已更新自定义 User-Agent' : 'Custom User-Agent updated');
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
            if (!u) return showToast(isZh() ? '请输入快捷网站网址' : 'Enter shortcut URL');
            sendIpc({ type: 'ADD_BOOKMARK', title: t, url: u });
            document.getElementById('inputScTitle').value = '';
            document.getElementById('inputScUrl').value = '';
        };

        document.querySelectorAll('#windowModeGroup .seg-btn').forEach(btn => {
            btn.onclick = () => {
                config.window_mode = btn.dataset.wm === 'single' ? 'single' : 'multi';
                syncFormControls();
                saveConfig(config.window_mode === 'single'
                    ? (isZh() ? '已切换为单窗口模式' : 'Switched to Single-Window mode')
                    : (isZh() ? '已切换为多窗口模式（默认）' : 'Switched to Multi-Window mode'));
            };
        });
        document.getElementById('btnIncognitoToggle').onclick = () => {
            config.incognito = !config.incognito;
            syncFormControls();
            saveConfig(config.incognito
                ? (isZh() ? '已开启无痕模式' : 'Private mode active')
                : (isZh() ? '已关闭无痕模式' : 'Private mode off'));
        };
        document.getElementById('btnHttpsFirstToggle').onclick = () => {
            config.https_first = !(config.https_first !== false);
            syncFormControls();
            saveConfig(config.https_first
                ? (isZh() ? '已开启 HTTPS 优先加密连接' : 'HTTPS-First enabled')
                : (isZh() ? '已关闭 HTTPS 优先自动升级' : 'HTTPS-First disabled'));
        };
        const PERF_PRESET_DEFS = {
            ultra_low: {
                titleZh: '极低', titleEn: 'Ultra Low',
                tagZh: '4GB / 极限省电', tagEn: 'Battery / 4GB',
                ramZh: '~90 – 130 MB RAM', ramEn: '~90 – 130 MB RAM',
                descZh: '只留当前活动标签、5 分钟休眠、开启 Zero-FX 与激进内存整理。',
                descEn: 'Active tab only, 5m sleep, Zero-FX visuals, trims RAM aggressively.'
            },
            low: {
                titleZh: '低（Edge 推荐）', titleEn: 'Low (Edge Eco)',
                tagZh: '默认推荐', tagEn: 'Recommended',
                ramZh: '~135 – 210 MB RAM', ramEn: '~135 – 210 MB RAM',
                descZh: '普通浏览全体验，开启 Edge 同款能效压缩将后台功耗压至极限。',
                descEn: 'Normal browser experience with Edge-style background memory compression.'
            },
            medium: {
                titleZh: '一般', titleEn: 'Medium',
                tagZh: '均衡多开', tagEn: 'Balanced',
                ramZh: '~230 – 370 MB RAM', ramEn: '~230 – 370 MB RAM',
                descZh: '保留最多 5 个常用网页热切换不重载，30 分钟闲置后自动休眠。',
                descEn: 'Keeps up to 5 recent tabs warm in memory, 30m idle sleep.'
            },
            high: {
                titleZh: '高（流畅性能）', titleEn: 'High',
                tagZh: '零冻结全开', tagEn: 'Max Smooth',
                ramZh: '~440 – 720+ MB RAM', ramEn: '~440 – 720+ MB RAM',
                descZh: '最多 8 个后台标签与 3 个热缓冲常驻，永不休眠且关闭能效限制。',
                descEn: 'Up to 8 live tabs with 3 hot buffers and auto-sleep disabled.'
            }
        };

        function cacheSlotLimitFor(mode) {
            switch (String(mode || 'balanced').toLowerCase()) {
                case 'minimal': return 1;
                case 'eco': return 2;
                case 'medium': return 5;
                case 'smooth': return 8;
                default: return 3;
            }
        }

        function countHeavyExtensions(cfg) {
            return [
                cfg.ext_side_panel,
                cfg.ext_reader_mode,
                cfg.ext_yt_ad_skipper,
                cfg.ext_gh_harmonizer,
                cfg.ext_media_booster
            ].filter(Boolean).length;
        }

        function estimateConfigRam(cfg) {
            const slots = Math.max(0, cacheSlotLimitFor(cfg.tab_cache_mode) - 1);
            const hw = cfg.hw_accel !== false;
            const eff = cfg.efficiency_mode !== false;
            const trim = cfg.auto_trim_ram !== false;
            const zeroFx = !!cfg.enforce_perf;
            const adOn = cfg.adblock !== false;
            const extCount = countHeavyExtensions(cfg);
            const chromeEnabled = cfg.chrome_extensions_enabled !== false
                ? (Array.isArray(chromeExtensions) ? chromeExtensions.filter((e) => e && e.enabled).length : 0)
                : 0;

            const baseMin = hw ? 105 : 90;
            const baseMax = hw ? 145 : 125;
            const perTabMin = eff ? 22 : 42;
            const perTabMax = eff ? 36 : 68;
            const extOver = extCount * 14;
            const adPen = adOn ? 0 : 35;
            const trimBonus = trim ? 0.90 : 1.0;
            const fxBonus = zeroFx ? 0.92 : 1.0;

            const minMb = Math.round((baseMin + slots * perTabMin + extOver + adPen) * trimBonus * fxBonus + chromeEnabled * 18);
            const maxMb = Math.max(minMb + 20, Math.round((baseMax + slots * perTabMax + extOver * 1.6 + adPen * 1.8) * trimBonus * fxBonus + chromeEnabled * 32));
            return { minMb, maxMb, extCount, chromeEnabled };
        }

        function applyPerfPresetClient(preset) {
            config.perf_preset = preset;
            if (preset === 'ultra_low') {
                config.tab_cache_mode = 'minimal';
                config.efficiency_mode = true;
                config.auto_trim_ram = true;
                config.tab_sleep_minutes = 5;
                config.enforce_perf = true;
                config.reduce_animations = true;
                config.chrome_extensions_enabled = false;
                config.ext_side_panel = false;
                config.ext_reader_mode = false;
                config.ext_yt_ad_skipper = false;
                config.ext_gh_harmonizer = false;
                config.ext_media_booster = false;
            } else if (preset === 'low') {
                config.tab_cache_mode = 'balanced';
                config.efficiency_mode = true;
                config.auto_trim_ram = true;
                config.tab_sleep_minutes = 15;
                config.enforce_perf = false;
                config.reduce_animations = false;
                config.chrome_extensions_enabled = true;
            } else if (preset === 'medium') {
                config.tab_cache_mode = 'medium';
                config.efficiency_mode = true;
                config.auto_trim_ram = false;
                config.tab_sleep_minutes = 30;
                config.enforce_perf = false;
                config.reduce_animations = false;
                config.chrome_extensions_enabled = true;
            } else if (preset === 'high') {
                config.tab_cache_mode = 'smooth';
                config.efficiency_mode = false;
                config.auto_trim_ram = false;
                config.tab_sleep_minutes = 0;
                config.enforce_perf = false;
                config.reduce_animations = false;
                config.chrome_extensions_enabled = true;
            }
        }

        function renderLivePerfTelemetry(stats) {
            const zh = isZh();
            const totEl = document.getElementById('perfLiveTotal');
            const subEl = document.getElementById('perfLiveBreakdown');
            if (!totEl || !subEl || !stats) return;
            const total = stats.total_mb !== undefined ? Number(stats.total_mb) : Number(stats.host_mb || 0);
            if (!total || total <= 0) {
                totEl.textContent = zh ? '轻量WebView' : 'Mobile WebView';
                subEl.textContent = (stats.live_tabs !== undefined && stats.total_tabs !== undefined)
                    ? (zh ? `当前活动缓存标签：${stats.live_tabs} / ${stats.total_tabs}` : `Live cached tabs: ${stats.live_tabs} / ${stats.total_tabs}`)
                    : (zh ? '系统统一管理移动端内存池。' : 'Managed by system WebView pool.');
                return;
            }
            totEl.textContent = `${Math.round(total)} MB`;
            const host = Math.round(Number(stats.host_mb || 0));
            const wv = Math.round(Number(stats.webview_mb || 0));
            const procs = Number(stats.process_count || 1);
            const winTxt = stats.window_count > 1
                ? (zh ? ` · ${stats.window_count} 窗口` : ` · ${stats.window_count} wins`)
                : '';
            const tabsTxt = (stats.live_tabs !== undefined && stats.total_tabs !== undefined)
                ? (zh ? ` · 活跃标签 ${stats.live_tabs}/${stats.total_tabs}` : ` · Tabs ${stats.live_tabs}/${stats.total_tabs}`)
                : '';
            subEl.textContent = zh
                ? `主程序 ${host} MB + 网页多进程 ${wv} MB（共 ${procs} 个进程${winTxt}${tabsTxt}）`
                : `Host ${host} MB + WebView2 renderers ${wv} MB (${procs} processes${winTxt}${tabsTxt})`;
        }

        window.__fynexo_on_settings_perf__ = renderLivePerfTelemetry;

        function syncPerformanceSection() {
            const zh = isZh();
            const preset = String(config.perf_preset || 'low').toLowerCase();
            const est = estimateConfigRam(config);

            const estKicker = document.getElementById('lblPerfEstKicker');
            const liveKicker = document.getElementById('lblPerfLiveKicker');
            const modeBadge = document.getElementById('perfModeBadge');
            const estRange = document.getElementById('perfEstRange');
            const estFill = document.getElementById('perfEstFill');
            const estDetail = document.getElementById('perfEstDetail');
            const lblPre = document.getElementById('lblPerfPresets');
            const hintPre = document.getElementById('hintPerfPresets');
            const grpMem = document.getElementById('grpPerfMem');
            const grpGpu = document.getElementById('grpPerfGpu');
            const grpMod = document.getElementById('grpPerfModules');
            const btnRef = document.getElementById('btnPerfRefreshRam');
            const btnFl = document.getElementById('btnPerfFlushRam');

            if (estKicker) estKicker.textContent = zh ? '当前配置预计日常 RAM 占用' : 'Estimated Typical RAM Footprint';
            if (liveKicker) liveKicker.textContent = zh ? '当前实际多进程内存监测' : 'Live Multi-Process Memory';
            if (btnRef) btnRef.textContent = zh ? '刷新读数' : 'Refresh RAM';
            if (btnFl) btnFl.textContent = zh ? '立即释放内存' : 'Flush idle RAM';
            if (lblPre) lblPre.textContent = zh ? '性能与功耗预设档位（像游戏一样一键选择）' : 'Performance Quality Preset';
            if (hintPre) hintPre.textContent = zh ? '新手推荐选「低（Edge 能效）」，日常流畅且把消耗压到极限' : '1-click tuning for beginners — like game graphics presets';
            if (grpMem) grpMem.textContent = zh ? '内存与标签页休眠调度' : 'Memory & Sleeping Tabs';
            if (grpGpu) grpGpu.textContent = zh ? '渲染特效与显卡硬件' : 'Rendering & Hardware';
            if (grpMod) grpMod.textContent = zh ? '其他影响内存的功能模块' : 'Features Affecting Memory';

            if (estRange) estRange.textContent = `~${est.minMb} – ${est.maxMb} MB`;
            const mid = (est.minMb + est.maxMb) / 2;
            const pct = Math.max(12, Math.min(98, Math.round((mid / 680) * 100)));
            if (estFill) {
                estFill.style.width = `${pct}%`;
                estFill.style.background = mid < 240
                    ? 'linear-gradient(90deg, #10B981, #34D399)'
                    : (mid < 420 ? 'linear-gradient(90deg, #38BDF8, #F59E0B)' : 'linear-gradient(90deg, #F59E0B, #FB7185)');
            }
            if (modeBadge) {
                const badgeMap = {
                    ultra_low: zh ? '极低功耗' : 'Ultra Low',
                    low: zh ? '低 · Edge 推荐' : 'Low · Edge Eco',
                    medium: zh ? '一般均衡' : 'Medium',
                    high: zh ? '高流畅度' : 'High Smooth',
                    custom: zh ? '自定义调校' : 'Custom Mix'
                };
                modeBadge.textContent = badgeMap[preset] || badgeMap.low;
            }
            if (estDetail) {
                const chromeBaseline = 560;
                const savedPct = Math.max(0, Math.round(((chromeBaseline - mid) / chromeBaseline) * 100));
                const slots = cacheSlotLimitFor(config.tab_cache_mode);
                const chromeTxt = est.chromeEnabled > 0
                    ? (zh ? ` + ${est.chromeEnabled} 个 Chrome 扩展` : ` + ${est.chromeEnabled} Chrome ext`)
                    : '';
                estDetail.textContent = zh
                    ? `包含多进程内核 + 最多 ${slots} 个后台热标签（已开 ${est.extCount} 个重模块${chromeTxt}）${savedPct > 10 ? ` · 相比常规多开约省 ${savedPct}% RAM` : ''}`
                    : `Includes WebView2 multi-process + up to ${slots} live tab slots (${est.extCount} heavy ext${chromeTxt})${savedPct > 10 ? ` · ~${savedPct}% lighter than unthrottled Chrome` : ''}`;
            }

            document.querySelectorAll('#perfPresetGrid .perf-preset-card').forEach((card) => {
                const p = card.dataset.preset;
                const def = PERF_PRESET_DEFS[p];
                card.classList.toggle('active', p === preset);
                if (def) {
                    const t = card.querySelector('.perf-preset-title');
                    const g = card.querySelector('.perf-preset-tag');
                    const r = card.querySelector('.perf-preset-ram');
                    const d = card.querySelector('.perf-preset-desc');
                    if (t) t.textContent = zh ? def.titleZh : def.titleEn;
                    if (g) g.textContent = zh ? def.tagZh : def.tagEn;
                    if (r) r.textContent = zh ? def.ramZh : def.ramEn;
                    if (d) d.textContent = zh ? def.descZh : def.descEn;
                }
            });

            const customNote = document.getElementById('perfCustomNote');
            const customNoteTxt = document.getElementById('perfCustomNoteText');
            if (customNote) customNote.classList.toggle('visible', preset === 'custom');
            if (customNoteTxt) {
                customNoteTxt.textContent = zh
                    ? '当前为「自定义调校」：你已手动微调下方单项开关；点击上方任意预设卡片可随时一键归位。'
                    : 'Custom mix active — you tweaked individual controls below. Click any preset card above to snap back.';
            }

            const effOn = config.efficiency_mode !== false;
            setSwitchState('btnEfficiencyToggle', effOn);
            setSwitchState('btnAutoTrimToggle', config.auto_trim_ram !== false);
            setSwitchState('btnReduceAnimToggle', !!config.reduce_animations);
            setSwitchState('btnHwAccelToggle', config.hw_accel !== false);
            setSwitchState('btnPerfAdblockToggle', config.adblock !== false);
            setSwitchState('btnChromeExtEngineToggle', config.chrome_extensions_enabled !== false);

            const curCm = String(config.tab_cache_mode || 'balanced').toLowerCase();
            document.querySelectorAll('#perfCacheModeGroup .seg-btn').forEach((b) => {
                b.classList.toggle('active', b.dataset.cm === curCm);
            });

            const sleepSel = document.getElementById('selTabSleepMinutes');
            if (sleepSel) {
                const curSleep = String(config.tab_sleep_minutes !== undefined ? config.tab_sleep_minutes : 15);
                if ([...sleepSel.options].some((o) => o.value === curSleep)) sleepSel.value = curSleep;
                const optMapZh = { '5': '5 分钟后休眠', '15': '15 分钟后休眠 (推荐)', '30': '30 分钟后休眠', '60': '60 分钟后休眠', '0': '永不自动休眠' };
                const optMapEn = { '5': 'After 5 minutes', '15': 'After 15 minutes (Default)', '30': 'After 30 minutes', '60': 'After 60 minutes', '0': 'Never sleep' };
                Array.from(sleepSel.options).forEach((o) => {
                    o.textContent = zh ? (optMapZh[o.value] || o.value) : (optMapEn[o.value] || o.value);
                });
            }

            const setTag = (id, zhTxt, enTxt, on) => {
                const el = document.getElementById(id);
                if (!el) return;
                el.textContent = zh ? zhTxt : enTxt;
                el.classList.toggle('on', !!on);
            };
            setTag('tagEffRam', effOn ? '可省 ~25–35% 后台 RAM' : '未压缩后台内存', effOn ? 'Saves ~25–35% BG RAM' : 'Unthrottled BG RAM', effOn);
            setTag('tagCacheRam', `${cacheSlotLimitFor(curCm)} 个标签 (每页 +~30 MB)`, `${cacheSlotLimitFor(curCm)} tabs (+~30 MB each)`, cacheSlotLimitFor(curCm) <= 3);
            const slVal = Number(config.tab_sleep_minutes !== undefined ? config.tab_sleep_minutes : 15);
            setTag('tagSleepRam', slVal > 0 ? `${slVal}m 自动休眠回收` : '休眠已关闭', slVal > 0 ? `${slVal}m auto-reclaim` : 'Sleep disabled', slVal > 0);
            setTag('tagTrimRam', config.auto_trim_ram !== false ? '约降 ~10% 工作集' : '不自动整理', config.auto_trim_ram !== false ? '~10% lighter host' : 'Manual only', config.auto_trim_ram !== false);
            setTag('tagZeroFxRam', config.enforce_perf ? '已开启 · 省 GPU' : '保留原画质特效', config.enforce_perf ? 'Active · Saves GPU' : 'Full Web FX', !!config.enforce_perf);
            setTag('tagAnimRam', config.reduce_animations ? '动画已精简' : '完整流畅动效', config.reduce_animations ? 'Reduced motion' : 'Full transitions', !!config.reduce_animations);
            setTag('tagHwRam', config.hw_accel !== false ? 'GPU 加速中 · 改后重启' : '低显存软件模式 · 改后重启', config.hw_accel !== false ? 'GPU On · Restart on change' : 'Low VRAM · Restart on change', config.hw_accel !== false);
            setTag('tagPerfAdblockRam', config.adblock !== false ? '已省 ~35–75 MB 广告' : '未拦截广告 (+~50 MB)', config.adblock !== false ? 'Saves ~35–75 MB' : '+~50 MB ad scripts', config.adblock !== false);
            setTag('tagPerfExtRam', est.extCount > 0 ? `已开 ${est.extCount} 个 (+~${est.extCount * 18} MB)` : '全部关闭 · 0 MB', est.extCount > 0 ? `${est.extCount} active (+~${est.extCount * 18} MB)` : '0 MB when off', est.extCount === 0);
            const ceOn = config.chrome_extensions_enabled !== false;
            const ceNeedRestart = ceOn !== chromeExtEngineActive ? (zh ? ' · 改后重启' : ' · Restart') : '';
            setTag(
                'tagChromeExtRam',
                ceOn ? `已启用 ${est.chromeEnabled} 个扩展 (+~${est.chromeEnabled * 25} MB)${ceNeedRestart}` : `引擎关闭 · 0 MB${ceNeedRestart}`,
                ceOn ? `${est.chromeEnabled} active (+~${est.chromeEnabled * 25} MB)${ceNeedRestart}` : `Engine off · 0 MB${ceNeedRestart}`,
                !ceOn || est.chromeEnabled === 0
            );

            const hintExt = document.getElementById('hintPerfExtSummary');
            if (hintExt) {
                hintExt.textContent = zh
                    ? (est.extCount > 0 ? `当前开启了 ${est.extCount} 个耗能内建模块（分屏/阅读/视频跳广告等），平时不用建议关掉。` : '所有高耗能内建扩展均已关闭，处于零内存占用状态。')
                    : (est.extCount > 0 ? `${est.extCount} optional module(s) active; disable unused ones to save per-tab RAM.` : 'All heavy optional modules are off (0 extra RAM).');
            }
            const btnDisExt = document.getElementById('btnPerfDisableHeavyExt');
            const btnManExt = document.getElementById('btnPerfManageExt');
            if (btnDisExt) {
                btnDisExt.textContent = zh ? '关闭重负载扩展' : 'Disable heavy';
                btnDisExt.disabled = est.extCount === 0;
            }
            if (btnManExt) btnManExt.textContent = zh ? '前往管理' : 'Manage';

            const isSingle = String(config.window_mode || 'multi').toLowerCase() === 'single';
            setTag('tagPerfWinRam', isSingle ? '单窗口更省内存' : '多窗口 (+80~150 MB/新窗)', isSingle ? 'Single window (leaner)' : '+80–150 MB / extra win', isSingle);
            const btnWin = document.getElementById('btnPerfToggleWinMode');
            if (btnWin) btnWin.textContent = isSingle ? (zh ? '单窗口' : 'Single-Window') : (zh ? '多窗口（默认）' : 'Multi-Window');
        }

        document.querySelectorAll('#perfPresetGrid .perf-preset-card').forEach((card) => {
            card.onclick = () => {
                const preset = card.dataset.preset;
                if (!preset) return;
                applyPerfPresetClient(preset);
                renderAllSections();
                const def = PERF_PRESET_DEFS[preset];
                const label = def ? (isZh() ? def.titleZh : def.titleEn) : preset;
                saveConfig(isZh() ? `已切换性能预设：${label}` : `Applied preset: ${label}`);
            };
        });
        const btnPerfRefresh = document.getElementById('btnPerfRefreshRam');
        if (btnPerfRefresh) {
            btnPerfRefresh.onclick = () => {
                sendIpc({ type: 'GET_RESOURCE_USAGE' });
                showToast(isZh() ? '正在刷新实时内存读数…' : 'Refreshing live RAM metrics...');
            };
        }
        const btnPerfFlush = document.getElementById('btnPerfFlushRam');
        if (btnPerfFlush) {
            btnPerfFlush.onclick = () => sendIpc({ type: 'FORCE_TRIM_MEMORY' });
        }
        document.getElementById('btnEfficiencyToggle').onclick = () => {
            config.efficiency_mode = !(config.efficiency_mode !== false);
            config.perf_preset = 'custom';
            syncFormControls();
            saveConfig(config.efficiency_mode
                ? (isZh() ? '已开启 Edge 能效后台压缩' : 'Efficiency Mode enabled')
                : (isZh() ? '已关闭后台内存压缩' : 'Efficiency Mode disabled'));
        };
        document.querySelectorAll('#perfCacheModeGroup .seg-btn').forEach((b) => {
            b.onclick = () => {
                config.tab_cache_mode = b.dataset.cm || 'balanced';
                config.perf_preset = 'custom';
                syncFormControls();
                saveConfig(isZh() ? `后台标签缓存上限已调整为 ${b.textContent}` : `Cached tab limit set to ${b.textContent}`);
            };
        });
        document.getElementById('selTabSleepMinutes').onchange = (e) => {
            config.tab_sleep_minutes = Math.max(0, Math.min(240, Number(e.target.value) || 0));
            config.perf_preset = 'custom';
            syncFormControls();
            saveConfig(config.tab_sleep_minutes > 0
                ? (isZh() ? `闲置 ${config.tab_sleep_minutes} 分钟后自动休眠标签` : `Auto-sleep set to ${config.tab_sleep_minutes}m`)
                : (isZh() ? '已关闭标签自动休眠' : 'Tab auto-sleep disabled'));
        };
        document.getElementById('btnAutoTrimToggle').onclick = () => {
            config.auto_trim_ram = !(config.auto_trim_ram !== false);
            config.perf_preset = 'custom';
            syncFormControls();
            saveConfig(config.auto_trim_ram
                ? (isZh() ? '已开启自动工作集整理' : 'Auto working-set trim enabled')
                : (isZh() ? '已关闭自动工作集整理' : 'Auto working-set trim disabled'));
        };
        document.getElementById('btnPerfToggle').onclick = () => {
            config.enforce_perf = !config.enforce_perf;
            config.perf_preset = 'custom';
            syncFormControls();
            saveConfig(config.enforce_perf
                ? (isZh() ? '已开启 Zero-FX 渲染过滤' : 'Zero-FX filter enabled')
                : (isZh() ? '已关闭 Zero-FX（恢复原网页特效）' : 'Zero-FX filter disabled'));
        };
        document.getElementById('btnReduceAnimToggle').onclick = () => {
            config.reduce_animations = !config.reduce_animations;
            config.perf_preset = 'custom';
            syncFormControls();
            saveConfig(config.reduce_animations
                ? (isZh() ? '已减弱界面与网页动画' : 'Reduced animations enabled')
                : (isZh() ? '已恢复完整动画效果' : 'Full animations restored'));
        };
        document.getElementById('btnHwAccelToggle').onclick = () => {
            config.hw_accel = !(config.hw_accel !== false);
            syncFormControls();
            saveConfig(isZh()
                ? (config.hw_accel ? 'GPU 硬件加速已开启（重启浏览器后生效）' : 'GPU 硬件加速已关闭（重启浏览器后生效）')
                : (config.hw_accel ? 'Hardware acceleration enabled (restart to apply)' : 'Hardware acceleration disabled (restart to apply)'));
        };
        document.getElementById('btnPerfAdblockToggle').onclick = () => {
            config.adblock = !(config.adblock !== false);
            syncFormControls();
            saveConfig(config.adblock
                ? (isZh() ? '已开启网络广告与追踪拦截' : 'Network adblock enabled')
                : (isZh() ? '已暂停网络广告拦截' : 'Network adblock paused'));
        };
        document.getElementById('btnPerfDisableHeavyExt').onclick = () => {
            config.ext_side_panel = false;
            config.ext_reader_mode = false;
            config.ext_yt_ad_skipper = false;
            config.ext_gh_harmonizer = false;
            config.ext_media_booster = false;
            renderAllSections();
            saveConfig(isZh() ? '已一键关闭所有高耗内建扩展' : 'Disabled all heavy built-in modules');
        };
        document.getElementById('btnPerfManageExt').onclick = () => {
            switchSettingsView('sec-extensions', true);
        };
        const btnCeEngine = document.getElementById('btnChromeExtEngineToggle');
        if (btnCeEngine) {
            btnCeEngine.onclick = () => {
                config.chrome_extensions_enabled = !(config.chrome_extensions_enabled !== false);
                config.perf_preset = 'custom';
                syncFormControls();
                renderChromeExtensionsPane();
                saveConfig(config.chrome_extensions_enabled
                    ? (isZh() ? '已开启 Chrome 原生扩展引擎（重启后生效）' : 'Chrome extension engine enabled (restart to apply)')
                    : (isZh() ? '已关闭 Chrome 原生扩展引擎（重启后零内存占用）' : 'Chrome extension engine disabled (restart for 0 MB)'));
            };
        }
        const btnInstStore = document.getElementById('btnInstallChromeStore');
        const inpStoreUrl = document.getElementById('inputChromeStoreUrl');
        if (btnInstStore && inpStoreUrl) {
            const doStoreInst = () => {
                const raw = inpStoreUrl.value.trim();
                if (!raw) return showToast(isZh() ? '请粘贴 Chrome/Edge 商店链接或 32 位 ID' : 'Enter store URL or 32-char extension ID');
                sendIpc({ type: 'INSTALL_CHROME_EXT_STORE', input: raw });
            };
            btnInstStore.onclick = doStoreInst;
            inpStoreUrl.onkeydown = (ev) => {
                if (ev.key === 'Enter') { ev.preventDefault(); doStoreInst(); }
            };
        }
        const btnOpenWebStore = document.getElementById('btnOpenChromeWebStore');
        if (btnOpenWebStore) {
            btnOpenWebStore.onclick = () => sendIpc({ type: 'NEW_TAB', url: 'https://chromewebstore.google.com/' });
        }
        const inpFileCrx = document.getElementById('fileImportCrx');
        const btnImpCrx = document.getElementById('btnImportCrx');
        if (btnImpCrx && inpFileCrx) {
            btnImpCrx.onclick = () => inpFileCrx.click();
            inpFileCrx.addEventListener('change', (ev) => {
                const file = ev.target.files && ev.target.files[0];
                if (!file) return;
                if (file.size > 48 * 1024 * 1024) {
                    showToast(isZh() ? '扩展文件超过 48 MB 上限' : 'Extension archive exceeds 48 MB limit');
                    ev.target.value = '';
                    return;
                }
                const reader = new FileReader();
                reader.onload = () => {
                    const buf = new Uint8Array(reader.result || new ArrayBuffer(0));
                    const b64 = bytesToBase64Chunked(buf);
                    sendIpc({ type: 'INSTALL_CHROME_EXT_FILE', filename: file.name || 'extension.crx', base64: b64 });
                };
                reader.readAsArrayBuffer(file);
                ev.target.value = '';
            });
        }
        const btnInstDir = document.getElementById('btnInstallChromeDir');
        const inpExtDir = document.getElementById('inputChromeExtDir');
        if (btnInstDir && inpExtDir) {
            btnInstDir.onclick = () => {
                const p = inpExtDir.value.trim();
                if (!p) return showToast(isZh() ? '请输入已解压扩展文件夹路径' : 'Enter unpacked extension folder path');
                sendIpc({ type: 'INSTALL_CHROME_EXT_DIR', path: p });
            };
        }
        const btnOpenCeDir = document.getElementById('btnOpenChromeExtDir');
        if (btnOpenCeDir) {
            btnOpenCeDir.onclick = () => sendIpc({ type: 'OPEN_CHROME_EXT_DIR' });
        }
        document.getElementById('btnPerfToggleWinMode').onclick = () => {
            config.window_mode = String(config.window_mode || 'multi').toLowerCase() === 'single' ? 'multi' : 'single';
            syncFormControls();
            saveConfig(config.window_mode === 'single'
                ? (isZh() ? '已切换为单窗口模式' : 'Switched to Single-Window mode')
                : (isZh() ? '已切换为多窗口模式（默认）' : 'Switched to Multi-Window mode'));
        };
        document.getElementById('btnRestoreToggle').onclick = () => {
            config.restore_session = !(config.restore_session !== false);
            syncFormControls();
            saveConfig(config.restore_session
                ? (isZh() ? '已开启重启自动恢复上次页面' : 'Session restore enabled')
                : (isZh() ? '已关闭重启恢复会话' : 'Session restore disabled'));
        };
        document.getElementById('btnAllowPageEvalToggle').onclick = () => {
            config.allow_page_eval = !config.allow_page_eval;
            syncFormControls();
            saveConfig(config.allow_page_eval
                ? (isZh() ? '已开启控制台 JS 执行与原生 DevTools（新标签生效原生开关）' : 'Console JS Eval & Native DevTools enabled')
                : (isZh() ? '已关闭控制台 JS 执行与原生 DevTools' : 'Console JS Eval & Native DevTools disabled'));
        };

        document.getElementById('btnClearHistory').onclick = () => {
            sendIpc({ type: 'CLEAR_HISTORY' });
            historyEntries = [];
            historyCount = 0;
            renderHistoryTable();
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
            saveConfig(config.adblock
                ? (isZh() ? '已开启网络广告与追踪拦截' : 'Network adblock enabled')
                : (isZh() ? '已暂停网络广告拦截' : 'Network adblock paused'));
        };
        document.getElementById('btnAddBlockDomain').onclick = () => {
            const zh = isZh();
            const inp = document.getElementById('inputBlockDomain');
            const v = inp.value.trim().toLowerCase().replace(/^https?:\/\//, '').split('/')[0];
            if (!v || !v.includes('.')) return showToast(zh ? '请输入有效域名（如 ads.example.com）' : 'Enter a valid domain (e.g. ads.example.com)');
            adblockCfg.custom_blocked_domains = adblockCfg.custom_blocked_domains || [];
            if (!adblockCfg.custom_blocked_domains.includes(v)) adblockCfg.custom_blocked_domains.push(v);
            inp.value = '';
            renderAdblockDomainLists();
            sendIpc({ type: 'SAVE_ADBLOCK', adblock: adblockCfg });
            showToast(zh ? `已屏蔽域名：${v}` : `Blocked ${v}`);
        };
        document.getElementById('btnAddWhiteDomain').onclick = () => {
            const zh = isZh();
            const inp = document.getElementById('inputWhiteDomain');
            const v = inp.value.trim().toLowerCase().replace(/^https?:\/\//, '').split('/')[0];
            if (!v || !v.includes('.')) return showToast(zh ? '请输入有效域名（如 example.org）' : 'Enter a valid domain (e.g. example.org)');
            adblockCfg.whitelisted_domains = adblockCfg.whitelisted_domains || [];
            if (!adblockCfg.whitelisted_domains.includes(v)) adblockCfg.whitelisted_domains.push(v);
            inp.value = '';
            renderAdblockDomainLists();
            sendIpc({ type: 'SAVE_ADBLOCK', adblock: adblockCfg });
            showToast(zh ? `已加入白名单：${v}` : `Whitelisted ${v}`);
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
                let disabledCount = 0;
                let lastName = '';

                files.forEach((file, idx) => {
                    const reader = new FileReader();
                    reader.onload = () => {
                        const js = String(reader.result || '');
                        if (js.trim()) {
                            const zh = isZh();
                            const meta = parseUserscriptHeader(js);
                            const baseFileName = (file.name || 'script.user.js')
                                .replace(/\.user\.js$/i, '')
                                .replace(/\.js$/i, '')
                                .trim();
                            const nm = (meta && meta.name) ? meta.name : (baseFileName || 'Userscript');
                            const dom = (meta && meta.domain) ? meta.domain : '*';
                            const audit = auditExtensionPayload('', js, dom);
                            const grants = (meta && Array.isArray(meta.grants)) ? meta.grants : [];
                            const connects = (meta && Array.isArray(meta.connects)) ? meta.connects : [];
                            const matches = (meta && Array.isArray(meta.matches)) ? meta.matches.concat(meta.includes || []) : [];
                            const riskyGrants = grants.filter(g => /^(GM_xmlhttpRequest|GM\.xmlHttpRequest|unsafeWindow|GM_openInTab)$/i.test(g));

                            let shouldEnable = true;
                            if (riskyGrants.length > 0 || connects.length > 0 || audit.hasWarnings) {
                                const msgLines = [
                                    zh
                                        ? `即将导入 ScriptForge 用户脚本：「${nm}」`
                                        : `Importing ScriptForge userscript: "${nm}"`,
                                    `${zh ? '作用域 (@match / domain)' : 'Scope (@match / domain)'}: ${matches.length > 0 ? matches.slice(0, 3).join(', ') : dom}`,
                                    `${zh ? '申请权限 (@grant)' : 'Permissions (@grant)'}: ${grants.length > 0 ? grants.join(', ') : (zh ? '无特殊声明' : 'none declared')}`,
                                    `${zh ? '外连域名 (@connect)' : 'Network (@connect)'}: ${connects.length > 0 ? connects.join(', ') : (zh ? '未限制（跟随页面同域策略）' : 'unspecified')}`,
                                ];
                                if (audit.hasWarnings) {
                                    msgLines.push(`${zh ? '安全提示' : 'Security notice'}: ${audit.warnings.slice(0, 3).join('; ')}`);
                                }
                                msgLines.push(
                                    zh
                                        ? '点击「确定」确认信任并立即启用；点击「取消」将以停用状态导入供你先审查源码。'
                                        : 'Click OK to trust and enable now, or Cancel to import disabled for code review.'
                                );
                                shouldEnable = window.confirm(msgLines.join('\n\n'));
                            }

                            const slug = nm.toLowerCase().replace(/[^a-z0-9_-]+/g, '-').replace(/^-+|-+$/g, '') || 'userscript';
                            const id = `${slug}-${Date.now() % 10000}-${idx}`;
                            const entry = {
                                id: id,
                                name: nm,
                                domain: dom,
                                enabled: shouldEnable,
                                css: '',
                                js: js
                            };
                            if (meta && meta.version) entry.version = meta.version;
                            siteRules.push(entry);
                            importedCount += 1;
                            if (!shouldEnable) disabledCount += 1;
                            lastName = nm;
                        }
                        remaining -= 1;
                        if (remaining === 0) {
                            e.target.value = '';
                            if (importedCount > 0) {
                                renderSiteRulesList();
                                sendIpc({ type: 'SAVE_RULES', rules: siteRules });
                                if (disabledCount > 0) {
                                    showToast(isZh()
                                        ? `已导入 ${importedCount} 个脚本（其中 ${disabledCount} 个默认停用，请审查源码后启用）`
                                        : `Imported ${importedCount} scripts (${disabledCount} disabled pending review)`);
                                } else {
                                    showToast(importedCount === 1
                                        ? (isZh() ? `已导入 ScriptForge 脚本：${lastName}` : `Imported ScriptForge script: ${lastName}`)
                                        : (isZh() ? `已导入 ${importedCount} 个 ScriptForge 脚本` : `Imported ${importedCount} ScriptForge scripts`));
                                }
                            } else {
                                showToast(isZh() ? '所选文件内容为空' : 'Selected file was empty');
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
            if (!css.trim() && !js.trim()) return showToast(isZh() ? '请填写至少一段 CSS 或 JS 代码' : 'Provide CSS or JS for this rule');

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
            showToast(isZh() ? `已保存扩展规则：${nm}` : `Saved rule: ${nm}`);
        };

        bindJsonFilePicker('btnImportKeybinds', 'fileImportKeybinds', 'IMPORT_KEYBINDS_JSON');
        document.getElementById('btnExportKeybinds').onclick = () => sendIpc({ type: 'EXPORT_KEYBINDS' });
        document.getElementById('btnResetKeybinds').onclick = () => sendIpc({ type: 'RESET_KEYBINDS' });

        document.getElementById('qsDeskTopToggle').onclick = () => {
            styles.topbar = styles.topbar || {};
            styles.topbar.visible_on_desktop = styles.topbar.visible_on_desktop === false ? true : false;
            pushStyles(isZh()
                ? (styles.topbar.visible_on_desktop ? '已显示桌面顶部栏' : '已隐藏桌面顶部栏')
                : (styles.topbar.visible_on_desktop ? 'Desktop topbar shown' : 'Desktop topbar hidden'));
        };
        document.getElementById('qsTopbarLayout').onchange = (e) => {
            styles.topbar = styles.topbar || {};
            styles.topbar.layout = e.target.value;
            const zh = isZh();
            const msg = e.target.value === 'vertical'
                ? (zh ? '已切换为左侧竖排边栏' : 'Switched to left vertical sidebar')
                : e.target.value === 'deck'
                    ? (zh ? '已切换为经典双层顶栏' : 'Switched to 2-row layered deck')
                    : (zh ? '已切换为单行紧凑顶栏' : 'Switched to single-row compact bar');
            pushStyles(msg);
        };
        document.getElementById('qsBookmarksBar').onchange = (e) => {
            styles.topbar = styles.topbar || {};
            styles.topbar.bookmarks_bar = e.target.value;
            pushStyles((isZh() ? '书签栏显示：' : 'Shortcuts bar: ') + e.target.value);
        };
        document.getElementById('qsTopbarHeight').onchange = (e) => {
            styles.topbar = styles.topbar || {};
            styles.topbar.height = e.target.value.trim() || '40px';
            pushStyles(isZh() ? '已更新顶栏高度' : 'Nav bar height updated');
        };
        const qsTitlebarInput = document.getElementById('qsTitlebarHeight');
        if (qsTitlebarInput) {
            qsTitlebarInput.onchange = (e) => {
                styles.topbar = styles.topbar || {};
                styles.topbar.titlebar_height = e.target.value.trim() || '42px';
                pushStyles(isZh() ? '已更新标签行高度' : 'Titlebar height updated');
            };
        }
        const qsVertWInput = document.getElementById('qsVertWidth');
        if (qsVertWInput) {
            qsVertWInput.onchange = (e) => {
                styles.topbar = styles.topbar || {};
                styles.topbar.vertical_width = e.target.value.trim() || '216px';
                pushStyles(isZh() ? '已更新竖排边栏宽度' : 'Vertical tab sidebar width updated');
            };
        }
        [
            ['qsBtnCopyToggle', 'show_copy_btn', '复制网址按钮', 'Copy URL button'],
            ['qsBtnStarToggle', 'show_star_btn', '收藏按钮', 'Bookmark button'],
            ['qsBtnShieldToggle', 'show_shield_btn', '拦截盾按钮', 'Shield button'],
            ['qsBtnDlToggle', 'show_downloads_btn', '下载按钮', 'Downloads button']
        ].forEach(([elId, field, zhLabel, enLabel]) => {
            const btn = document.getElementById(elId);
            if (!btn) return;
            btn.onclick = () => {
                styles.topbar = styles.topbar || {};
                const next = styles.topbar[field] === false;
                styles.topbar[field] = next;
                pushStyles(isZh()
                    ? `${zhLabel}${next ? '已显示' : '已隐藏'}`
                    : `${enLabel} ${next ? 'shown' : 'hidden'}`);
            };
        });
        document.getElementById('qsAddrMode').onchange = (e) => {
            styles.address_bar = styles.address_bar || {};
            styles.address_bar.mode = e.target.value;
            pushStyles((isZh() ? '地址栏样式：' : 'Address bar mode: ') + e.target.value);
        };
        document.getElementById('qsAddrPos').onchange = (e) => {
            styles.address_bar = styles.address_bar || {};
            styles.address_bar.position = e.target.value;
            pushStyles((isZh() ? '地址栏对齐：' : 'Omnibox aligned ') + e.target.value);
        };
        document.getElementById('qsAddrWidth').onchange = (e) => {
            styles.address_bar = styles.address_bar || {};
            styles.address_bar.width = e.target.value.trim() || '320px';
            pushStyles(isZh() ? '已更新地址栏宽度' : 'Address capsule width updated');
        };
        const tabHInput = document.getElementById('qsTabHeight');
        if (tabHInput) {
            tabHInput.onchange = (e) => {
                styles.tabs = styles.tabs || {};
                styles.tabs.height = e.target.value.trim() || '36px';
                pushStyles(isZh() ? '已更新标签高度' : 'Tab item height updated');
            };
        }
        document.getElementById('qsTabWidth').onchange = (e) => {
            styles.tabs = styles.tabs || {};
            styles.tabs.width = e.target.value.trim() || '152px';
            pushStyles(isZh() ? '已更新标签宽度' : 'Tab width updated');
        };
        document.getElementById('qsTabRadius').onchange = (e) => {
            styles.tabs = styles.tabs || {};
            styles.tabs.border_radius = e.target.value.trim() || '7px';
            pushStyles(isZh() ? '已更新标签圆角' : 'Tab corner radius updated');
        };
        const tabGapInput = document.getElementById('qsTabGap');
        if (tabGapInput) {
            tabGapInput.onchange = (e) => {
                styles.tabs = styles.tabs || {};
                styles.tabs.gap = e.target.value.trim() || '0px';
                pushStyles(isZh() ? '已更新标签间距' : 'Tab spacing (gap) updated');
            };
        }
        document.getElementById('qsTabDotToggle').onclick = () => {
            styles.tabs = styles.tabs || {};
            styles.tabs.show_indicator_dot = styles.tabs.show_indicator_dot === false ? true : false;
            pushStyles(isZh() ? '已更新网站图标徽标显示' : 'Updated tab badge');
        };
        document.getElementById('qsMobilePos').onchange = (e) => {
            styles.mobile_bar = styles.mobile_bar || {};
            styles.mobile_bar.position = e.target.value;
            pushStyles((isZh() ? '手机底栏位置：' : 'Mobile bar docked to ') + e.target.value);
        };
        document.getElementById('qsMobileHeight').onchange = (e) => {
            styles.mobile_bar = styles.mobile_bar || {};
            styles.mobile_bar.height = e.target.value.trim() || '54px';
            pushStyles(isZh() ? '已更新手机底栏高度' : 'Mobile bar height updated');
        };
        document.getElementById('qsFullWidthToggle').onclick = () => {
            styles.settings_page = styles.settings_page || {};
            styles.settings_page.full_width = styles.settings_page.full_width === false ? true : false;
            pushStyles(isZh() ? '已更新设置页宽屏模式' : 'Workspace width updated');
        };

        document.getElementById('qsNtTitle').onchange = (e) => {
            styles.newtab = styles.newtab || {};
            styles.newtab.title = e.target.value;
            pushStyles(isZh() ? '已更新起始页标题' : 'Start page title updated');
        };
        document.getElementById('qsNtSub').onchange = (e) => {
            styles.newtab = styles.newtab || {};
            styles.newtab.subtitle = e.target.value;
            pushStyles(isZh() ? '已更新起始页副标题' : 'Start page subtitle updated');
        };
        document.getElementById('qsNtCols').onchange = (e) => {
            styles.newtab = styles.newtab || {};
            styles.newtab.shortcuts_columns = Number(e.target.value) || 0;
            pushStyles(isZh() ? '已更新常用网站列数' : 'Shortcuts grid columns updated');
        };
        document.getElementById('qsNtClockToggle').onclick = () => {
            styles.newtab = styles.newtab || {};
            styles.newtab.show_clock = styles.newtab.show_clock === false ? true : false;
            pushStyles(isZh() ? '已切换起始页大时钟显示' : 'Start page clock updated');
        };
        document.getElementById('qsNt24hToggle').onclick = () => {
            styles.newtab = styles.newtab || {};
            styles.newtab.use_24h_clock = styles.newtab.use_24h_clock === false ? true : false;
            pushStyles(isZh() ? '已切换小时制格式' : 'Clock format updated');
        };
        document.getElementById('qsNtScToggle').onclick = () => {
            styles.newtab = styles.newtab || {};
            styles.newtab.show_shortcuts = styles.newtab.show_shortcuts === false ? true : false;
            pushStyles(isZh() ? '已更新常用网站模块显示' : 'Start page shortcuts visibility updated');
        };
        document.getElementById('qsNtHistToggle').onclick = () => {
            styles.newtab = styles.newtab || {};
            styles.newtab.show_history = styles.newtab.show_history === false ? true : false;
            pushStyles(isZh() ? '已更新最近访问记录显示' : 'Start page history visibility updated');
        };
        const qsNtWpEl = document.getElementById('qsNtWallpaper');
        if (qsNtWpEl) {
            qsNtWpEl.onchange = (e) => {
                styles.newtab = styles.newtab || {};
                styles.newtab.background_image = e.target.value.trim();
                pushStyles(isZh()
                    ? (styles.newtab.background_image ? '已更新起始页壁纸' : '已清除起始页壁纸')
                    : (styles.newtab.background_image ? 'Start page wallpaper updated' : 'Start page wallpaper cleared'));
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
                    showToast(isZh() ? '请选择小于 4 MB 的图片文件' : 'Please choose an image under 4 MB');
                    return;
                }
                const reader = new FileReader();
                reader.onload = () => {
                    styles.newtab = styles.newtab || {};
                    styles.newtab.background_image = String(reader.result || '');
                    pushStyles(isZh() ? '已上传并应用起始页壁纸' : 'Start page wallpaper uploaded');
                };
                reader.readAsDataURL(file);
            });
        }
        const btnWpClr = document.getElementById('btnNtWallpaperClear');
        if (btnWpClr) {
            btnWpClr.onclick = () => {
                styles.newtab = styles.newtab || {};
                styles.newtab.background_image = '';
                pushStyles(isZh() ? '已清除起始页壁纸' : 'Start page wallpaper cleared');
            };
        }
        const qsNtDimEl = document.getElementById('qsNtDim');
        if (qsNtDimEl) {
            qsNtDimEl.onchange = (e) => {
                styles.newtab = styles.newtab || {};
                const n = Math.max(0, Math.min(90, Number(e.target.value) || 0));
                styles.newtab.background_dim = Math.round(n);
                pushStyles(isZh() ? `壁纸暗度设为 ${styles.newtab.background_dim}%` : `Wallpaper dim set to ${styles.newtab.background_dim}%`);
            };
        }
        const qsNtBlurEl = document.getElementById('qsNtBlur');
        if (qsNtBlurEl) {
            qsNtBlurEl.onchange = (e) => {
                styles.newtab = styles.newtab || {};
                const n = Math.max(0, Math.min(24, Number(e.target.value) || 0));
                styles.newtab.background_blur = Math.round(n);
                pushStyles(isZh() ? `壁纸模糊设为 ${styles.newtab.background_blur}px` : `Wallpaper blur set to ${styles.newtab.background_blur}px`);
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
            },
            glass: {
                msgZh: '已切换为「亚克力毛玻璃」风格',
                msgEn: 'Applied Frosted Glass preset',
                topbar: {
                    background: 'color-mix(in srgb, var(--fnx-panel) 76%, transparent)',
                    custom_css: 'backdrop-filter: blur(16px) saturate(140%); -webkit-backdrop-filter: blur(16px) saturate(140%);'
                },
                tabs: { border_radius: '10px' },
                address_bar: { border_radius: '10px' }
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
                    setJsonStatus(false, isZh() ? 'JSON 语法正确 — 点击“应用并保存”或按 Ctrl+S 生效。' : 'Syntax valid — click Apply & save or press Ctrl+S.');
                } catch (err) {
                    setJsonStatus(true, (isZh() ? '语法错误：' : 'Syntax error: ') + err.message);
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
                setJsonStatus(false, isZh() ? 'JSON 校验通过 — 已实时应用并保存到磁盘。' : 'Valid JSON — applied live and saved to disk.');
                showToast(isZh() ? '已保存 styles.json' : 'Saved styles.json');
            } catch (err) {
                setJsonStatus(true, (isZh() ? 'JSON 格式错误：' : 'Invalid JSON: ') + err.message);
            }
        };

        document.getElementById('btnReloadJson').onclick = () => {
            sendIpc({ type: 'RELOAD_STYLES' });
            showToast(isZh() ? '已从磁盘重新加载 JSON 配置' : 'Reloaded JSON files from disk');
        };
        document.getElementById('btnResetJson').onclick = () => {
            sendIpc({ type: 'RESET_STYLES' });
            showToast(isZh() ? '已恢复组件默认样式' : 'Reset component styles to defaults');
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
        const settingsSearchInp = document.getElementById('inputSettingsSearch');

        function applySettingsSearchFilter(rawQ) {
            const q = String(rawQ || '').trim().toLowerCase();
            if (!q) {
                viewBlocks.forEach(sec => {
                    sec.querySelectorAll('.set-row').forEach(r => { r.style.display = ''; });
                });
                const activeNav = navLinks.find(l => l.classList.contains('active'));
                switchSettingsView((activeNav && activeNav.dataset.sec) || 'sec-appearance', false, true);
                return;
            }
            viewBlocks.forEach(sec => {
                const rows = Array.from(sec.querySelectorAll('.set-row'));
                let anyMatch = false;
                if (rows.length > 0) {
                    rows.forEach(r => {
                        const hit = (r.textContent || '').toLowerCase().includes(q);
                        r.style.display = hit ? '' : 'none';
                        if (hit) anyMatch = true;
                    });
                } else {
                    anyMatch = (sec.textContent || '').toLowerCase().includes(q);
                }
                if (!anyMatch) {
                    const secTitle = (sec.querySelector('.sec-title')?.textContent || '').toLowerCase();
                    if (secTitle.includes(q)) {
                        anyMatch = true;
                        rows.forEach(r => { r.style.display = ''; });
                    }
                }
                sec.classList.toggle('active', anyMatch);
            });
        }
        if (settingsSearchInp) {
            settingsSearchInp.addEventListener('input', () => {
                applySettingsSearchFilter(settingsSearchInp.value);
            });
        }

        function switchSettingsView(secId, updateHash, keepSearch) {
            if (!keepSearch && settingsSearchInp && settingsSearchInp.value) {
                settingsSearchInp.value = '';
                viewBlocks.forEach(sec => {
                    sec.querySelectorAll('.set-row').forEach(r => { r.style.display = ''; });
                });
            }
            const target = viewBlocks.find(v => v.id === secId) || viewBlocks[0];
            if (!target) return;
            viewBlocks.forEach(v => v.classList.toggle('active', v === target));
            navLinks.forEach(l => l.classList.toggle('active', l.dataset.sec === target.id));
            const activeLink = navLinks.find(l => l.dataset.sec === target.id);
            if (activeLink && typeof activeLink.scrollIntoView === 'function') {
                try {
                    activeLink.scrollIntoView({ behavior: 'smooth', block: 'nearest', inline: 'center' });
                } catch (_) {}
            }
            window.scrollTo(0, 0);
            if (target.id === 'sec-performance') {
                sendIpc({ type: 'GET_RESOURCE_USAGE' });
            }
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

        document.getElementById('btnAddModeProfile').onclick = () => {
            const profiles = ensureModeProfilesList();
            const zh = isZh();
            const n = profiles.length + 1;
            const slug = 'custom-' + (Date.now() % 10000);
            profiles.push({
                id: slug,
                name_zh: `自定义模式 ${n}`,
                name_en: `Custom Mode ${n}`,
                icon: 'bolt',
                auto_domains: [],
                adblock: true,
                force_dark: false,
                ext_side_panel: false,
                ext_reader_mode: false,
                ext_yt_ad_skipper: false,
                ext_gh_harmonizer: false,
                ext_media_booster: false,
                ext_userscript_engine: true,
                ext_mouse_gestures: true,
                dev_drawer_access: true,
                fps_hud: false,
                theme: '',
                layout: ''
            });
            renderModeProfilesSection();
            saveConfig(zh ? '已新建自定义场景模式' : 'Created new mode profile');
        };
        document.getElementById('btnResetModeProfiles').onclick = () => {
            const defs = typeof DEFAULT_MODE_PROFILES !== 'undefined' ? DEFAULT_MODE_PROFILES : [];
            config.mode_profiles = JSON.parse(JSON.stringify(defs));
            config.active_mode = 'auto';
            renderModeProfilesSection();
            saveConfig(isZh() ? '已恢复默认场景模式配置' : 'Reset mode profiles to defaults');
        };

        let workspaces = Array.isArray(boot.workspaces) ? boot.workspaces : [];
        function formatWorkspaceTime(tsSec) {
            const sec = Number(tsSec) || 0;
            if (!sec) return '';
            const zh = isZh();
            const diff = Math.max(0, Math.floor(Date.now() / 1000) - sec);
            if (diff < 60) return zh ? '刚刚保存' : 'just now';
            if (diff < 3600) {
                const m = Math.floor(diff / 60);
                return zh ? `${m} 分钟前` : `${m}m ago`;
            }
            if (diff < 86400) {
                const h = Math.floor(diff / 3600);
                return zh ? `${h} 小时前` : `${h}h ago`;
            }
            const d = new Date(sec * 1000);
            return `${d.getMonth() + 1}/${d.getDate()} ${String(d.getHours()).padStart(2, '0')}:${String(d.getMinutes()).padStart(2, '0')}`;
        }
        function syncSaveWorkspaceBtn() {
            const saveBtn = document.getElementById('btnSaveWorkspace');
            const inp = document.getElementById('inputWsName');
            if (!saveBtn || !inp) return;
            const zh = isZh();
            const typed = inp.value.trim();
            const willOverwrite = !!typed && workspaces.some((w) => w && w.name === typed);
            saveBtn.textContent = willOverwrite
                ? (zh ? '覆盖已有快照' : 'Overwrite snapshot')
                : (zh ? '保存当前标签页' : 'Save current tabs');
            saveBtn.style.borderColor = willOverwrite ? 'var(--fnx-accent)' : '';
            saveBtn.style.color = willOverwrite ? 'var(--fnx-accent)' : '';
        }
        function renderWorkspacesTable() {
            const zh = isZh();
            const box = document.getElementById('workspacesTable');
            if (!box) return;
            setPlainRowI18n(document.getElementById('lblWorkspaces'), '标签页工作区快照', 'Workspace snapshots');
            setPlainRowI18n(document.getElementById('hintWorkspaces'), '把当前所有标签页保存为命名快照，随时一键恢复；使用同名保存会覆盖更新旧快照。', 'Save all open tabs as a named snapshot and restore anytime (same name overwrites).');
            const inp = document.getElementById('inputWsName');
            if (inp) inp.placeholder = zh ? '快照名称…' : 'Snapshot name...';
            syncSaveWorkspaceBtn();
            box.textContent = '';
            if (!workspaces.length) {
                const empty = document.createElement('div');
                empty.className = 'set-hint';
                empty.style.padding = '8px 2px';
                empty.textContent = zh ? '暂无快照' : 'No snapshots yet';
                box.appendChild(empty);
                return;
            }
            workspaces.forEach((w) => {
                if (!w || !w.name) return;
                const row = document.createElement('div');
                row.className = 'rc-row';
                const nm = document.createElement('span');
                nm.className = 'rc-cell';
                nm.style.flex = '1';
                const count = Array.isArray(w.tabs) ? w.tabs.length : 0;
                const tsLabel = formatWorkspaceTime(w.updated_at);
                nm.textContent = w.name + '  ·  ' + count + (zh ? ' 个标签页' : ' tabs') + (tsLabel ? '  ·  ' + tsLabel : '');
                if (w.updated_at) {
                    nm.title = new Date(w.updated_at * 1000).toLocaleString();
                }
                const restore = document.createElement('button');
                restore.type = 'button';
                restore.className = 'action-btn compact';
                restore.textContent = zh ? '恢复' : 'Restore';
                restore.onclick = () => sendIpc({ type: 'LOAD_WORKSPACE', name: w.name });
                const del = document.createElement('button');
                del.type = 'button';
                del.className = 'action-btn compact';
                del.textContent = zh ? '删除' : 'Delete';
                del.onclick = () => sendIpc({ type: 'DELETE_WORKSPACE', name: w.name });
                row.appendChild(nm);
                row.appendChild(restore);
                row.appendChild(del);
                box.appendChild(row);
            });
        }
        (function () {
            const saveBtn = document.getElementById('btnSaveWorkspace');
            const inp = document.getElementById('inputWsName');
            if (!saveBtn || !inp) return;
            const doSave = () => {
                const name = inp.value.trim();
                if (!name) return;
                sendIpc({ type: 'SAVE_WORKSPACE', name: name });
                inp.value = '';
                syncSaveWorkspaceBtn();
            };
            saveBtn.onclick = doSave;
            inp.addEventListener('input', syncSaveWorkspaceBtn);
            inp.onkeydown = (ev) => { if (ev.key === 'Enter') { ev.preventDefault(); doSave(); } };
        })();
        function renderHistoryTable() {
            const zh = isZh();
            const box = document.getElementById('historyTable');
            if (!box) return;
            setPlainRowI18n(document.getElementById('lblHistoryManager'), '浏览历史管理', 'Browsing History');
            setPlainRowI18n(document.getElementById('hintHistoryManager'), '支持按标题或网址搜索历史记录、在新标签中打开，或单独删除某一条。', 'Search visited pages, reopen in a new tab, or remove individual items.');
            const inp = document.getElementById('inputHistorySearch');
            if (inp) inp.placeholder = zh ? '搜索历史记录（标题或网址）…' : 'Search history...';
            const q = String(inp ? inp.value : '').trim().toLowerCase();
            box.textContent = '';

            const filtered = historyEntries.filter((h) => {
                if (!h || !h.url) return false;
                if (!q) return true;
                return String(h.title || '').toLowerCase().includes(q) || String(h.url || '').toLowerCase().includes(q);
            });
            if (!filtered.length) {
                const empty = document.createElement('div');
                empty.className = 'set-hint';
                empty.style.padding = '12px';
                empty.textContent = q
                    ? (zh ? '没有匹配的历史记录' : 'No matching history entries')
                    : (zh ? '暂无浏览历史记录' : 'No browsing history recorded yet');
                box.appendChild(empty);
                return;
            }
            const frag = document.createDocumentFragment();
            filtered.slice(0, 120).forEach((item) => {
                const row = document.createElement('div');
                row.className = 'rc-row';
                const mid = document.createElement('div');
                mid.className = 'rc-mid';
                const titleEl = document.createElement('span');
                titleEl.className = 'rc-name';
                titleEl.textContent = item.title || item.url;
                const urlEl = document.createElement('span');
                urlEl.className = 'rc-url mono';
                urlEl.textContent = item.url;
                mid.appendChild(titleEl);
                mid.appendChild(urlEl);

                const metaEl = document.createElement('span');
                metaEl.className = 'set-hint mono';
                metaEl.style.fontSize = '10.5px';
                metaEl.style.whiteSpace = 'nowrap';
                const tStr = formatWorkspaceTime(item.visited_at);
                const vStr = (item.visits && item.visits > 1) ? `${item.visits}×` : '';
                metaEl.textContent = [vStr, tStr].filter(Boolean).join(' · ');

                const openBtn = document.createElement('button');
                openBtn.type = 'button';
                openBtn.className = 'action-btn compact';
                openBtn.textContent = zh ? '打开' : 'Open';
                openBtn.onclick = () => sendIpc({ type: 'NEW_TAB', url: item.url });

                const delBtn = document.createElement('button');
                delBtn.type = 'button';
                delBtn.className = 'rc-del';
                delBtn.title = zh ? '删除此条记录' : 'Remove history entry';
                delBtn.textContent = '×';
                delBtn.onclick = () => {
                    historyEntries = historyEntries.filter((x) => x.url !== item.url);
                    historyCount = historyEntries.length;
                    renderHistoryTable();
                    syncFormControls();
                    sendIpc({ type: 'DELETE_HISTORY_ITEM', url: item.url });
                };

                row.appendChild(mid);
                if (metaEl.textContent) row.appendChild(metaEl);
                row.appendChild(openBtn);
                row.appendChild(delBtn);
                frag.appendChild(row);
            });
            box.appendChild(frag);
        }
        const historySearchInp = document.getElementById('inputHistorySearch');
        if (historySearchInp) {
            historySearchInp.addEventListener('input', () => renderHistoryTable());
        }

        function renderAllSections() {
            renderWorkspacesTable();
            renderHistoryTable();
            renderThemeGrid();
            renderColorTokensStudio();
            renderModeProfilesSection();
            renderEngineTable();
            renderBangsTable();
            renderShortcutsTable();
            renderAdblockDomainLists();
            renderBuiltinExtensions();
            renderChromeExtensionsPane();
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
            if (Array.isArray(payload.workspaces)) workspaces = payload.workspaces;
            if (Array.isArray(payload.site_rules)) siteRules = payload.site_rules;
            if (payload.adblock) adblockCfg = payload.adblock;
            if (payload.keybinds) keybinds = Object.assign({}, DEFAULT_KEYBINDS, payload.keybinds);
            if (payload.themes_dir) themesDir = payload.themes_dir;
            if (payload.styles_path) stylesPath = payload.styles_path;
            if (payload.data_dir) dataDir = payload.data_dir;
            if (typeof payload.blocked_ads === 'number') rustBlockedAds = payload.blocked_ads;
            blockedAds = resolveTotalBlockedAds(rustBlockedAds);
            if (Array.isArray(payload.chrome_extensions)) chromeExtensions = payload.chrome_extensions;
            if (typeof payload.chrome_ext_engine_active === 'boolean') chromeExtEngineActive = payload.chrome_ext_engine_active;
            if (Array.isArray(payload.history)) {
                historyEntries = payload.history;
                historyCount = historyEntries.length;
            }
            renderAllSections();
        };

        renderAllSections();
