        __FNX_DEFAULTS__;

        const boot = __FNX_BOOT_JSON__ || {};
        let config = Object.assign({ language: 'zh', search_engine: 'google', search_engines: DEFAULT_ENGINES }, boot.config || {});
        let activeTheme = boot.active_theme || {};
        let bookmarks = Array.isArray(boot.bookmarks) ? boot.bookmarks : [];
        let historyItems = Array.isArray(boot.history) ? boot.history : [];
        let recentSearches = Array.isArray(boot.recent_searches) ? boot.recent_searches : [];

        let scopeFilter = 'all'; // 'all' | 'pages' | 'searches' | 'bookmarked'
        let timeFilter = 'all';  // 'all' | 'today' | 'yesterday' | 'week' | 'older'
        let domainFilter = '';
        let sortMode = 'recent'; // 'recent' | 'visits'
        let searchQuery = '';
        let batchMode = false;
        let selectedUrls = new Set();
        let clearAllConfirmTimer = null;

        function isZh() {
            return typeof FynexoI18n !== 'undefined'
                ? FynexoI18n.isZhLocale(config.language)
                : String(config.language || 'zh').toLowerCase().startsWith('zh');
        }

        function sendIpc(data) {
            const raw = JSON.stringify(data);
            if (window.ipc && typeof window.ipc.postMessage === 'function') {
                window.ipc.postMessage(raw);
            } else if (window.__TAURI_INTERNALS__ && window.__TAURI_INTERNALS__.ipc) {
                window.__TAURI_INTERNALS__.ipc.postMessage(raw);
            }
        }

        function createSvgEl(tag, attrs) {
            const el = document.createElementNS('http://www.w3.org/2000/svg', tag);
            if (attrs) {
                Object.keys(attrs).forEach((k) => el.setAttribute(k, String(attrs[k])));
            }
            return el;
        }

        function applyTheme() {
            const t = activeTheme || {};
            const root = document.documentElement;
            const bg = t.bg || '#111218';
            const fg = t.fg || '#E6E9F2';
            const surface = t.surface || t.panel || bg;
            const accent = t.accent || '#6366F1';
            const border = t.border || 'rgba(255,255,255,0.09)';
            const borderHover = t.border_hover || 'rgba(255,255,255,0.16)';
            const muted = t.muted || 'rgba(230,233,242,0.58)';
            const secondary = t.secondary || 'rgba(230,233,242,0.78)';
            const hover = t.hover || 'rgba(255,255,255,0.06)';
            const danger = t.danger || '#F43F5E';

            root.style.setProperty('--ht-bg', bg);
            root.style.setProperty('--ht-fg', fg);
            root.style.setProperty('--ht-surface', surface);
            root.style.setProperty('--ht-accent', accent);
            root.style.setProperty('--ht-accent-soft', `color-mix(in srgb, ${accent} 16%, transparent)`);
            root.style.setProperty('--ht-border', border);
            root.style.setProperty('--ht-border-hover', borderHover);
            root.style.setProperty('--ht-muted', muted);
            root.style.setProperty('--ht-secondary', secondary);
            root.style.setProperty('--ht-hover', hover);
            root.style.setProperty('--ht-danger', danger);
        }

        function normalizeUrlKey(raw) {
            try {
                const u = new URL(raw);
                let out = u.protocol + '//' + u.hostname.toLowerCase() + (u.port ? ':' + u.port : '') + u.pathname;
                if (out.endsWith('/') && u.pathname === '/') out = out.slice(0, -1);
                return out + u.search;
            } catch (_) {
                return String(raw || '').trim().toLowerCase().replace(/\/+$/, '');
            }
        }

        function formatHost(u) {
            try {
                return new URL(u).hostname.replace(/^www\./i, '') || u;
            } catch (_) {
                return String(u || '');
            }
        }

        function isBookmarked(url) {
            const k = normalizeUrlKey(url);
            return bookmarks.some((b) => b && normalizeUrlKey(b.url) === k);
        }

        function monogramColor(url) {
            const palette = ['#7A88F7', '#38BDF8', '#34D399', '#F472B6', '#FB923C', '#A78BFA'];
            const s = String(url || '');
            let h = 0;
            for (let i = 0; i < s.length; i++) h = ((h << 5) - h + s.charCodeAt(i)) | 0;
            return palette[Math.abs(h) % palette.length];
        }

        function createSiteEmblem(url, title) {
            const wrap = document.createElement('span');
            wrap.className = 'ht-emblem';
            const host = formatHost(url);
            const ch = (host || title || '?').charAt(0).toUpperCase();
            wrap.textContent = ch;
            wrap.style.color = monogramColor(host || url);
            return wrap;
        }

        function appendHighlighted(el, text, q) {
            const str = String(text || '');
            const needle = String(q || '').trim();
            el.textContent = '';
            if (!needle) {
                el.textContent = str;
                return;
            }
            const idx = str.toLowerCase().indexOf(needle.toLowerCase());
            if (idx === -1) {
                el.textContent = str;
                return;
            }
            if (idx > 0) el.appendChild(document.createTextNode(str.slice(0, idx)));
            const strong = document.createElement('strong');
            strong.className = 'ht-hl';
            strong.textContent = str.slice(idx, idx + needle.length);
            el.appendChild(strong);
            if (idx + needle.length < str.length) {
                el.appendChild(document.createTextNode(str.slice(idx + needle.length)));
            }
        }

        function classifyTimeBucket(visitedAtSecs) {
            const tsMs = (Number(visitedAtSecs) || 0) * 1000;
            if (!tsMs) return 'older';
            const now = new Date();
            const startOfToday = new Date(now.getFullYear(), now.getMonth(), now.getDate()).getTime();
            const startOfYesterday = startOfToday - 86400000;
            const startOfWeek = startOfToday - 6 * 86400000;
            if (tsMs >= startOfToday) return 'today';
            if (tsMs >= startOfYesterday) return 'yesterday';
            if (tsMs >= startOfWeek) return 'week';
            return 'older';
        }

        function formatVisitTime(visitedAtSecs, bucket) {
            const tsMs = (Number(visitedAtSecs) || 0) * 1000;
            if (!tsMs) return '';
            const d = new Date(tsMs);
            const hh = String(d.getHours()).padStart(2, '0');
            const mm = String(d.getMinutes()).padStart(2, '0');
            if (bucket === 'today' || bucket === 'yesterday') {
                return `${hh}:${mm}`;
            }
            const mo = String(d.getMonth() + 1).padStart(2, '0');
            const da = String(d.getDate()).padStart(2, '0');
            return `${mo}/${da} ${hh}:${mm}`;
        }

        function currentEngineUrl(query) {
            const list = (Array.isArray(config.search_engines) && config.search_engines.length > 0)
                ? config.search_engines
                : DEFAULT_ENGINES;
            const eng = list.find((e) => e && e.id === config.search_engine) || list[0] || DEFAULT_ENGINES[0];
            const tpl = String((eng && eng.url) || 'https://www.google.com/search?q={query}');
            const enc = encodeURIComponent(query);
            if (tpl.includes('{query}')) return tpl.split('{query}').join(enc);
            if (tpl.includes('%s')) return tpl.split('%s').join(enc);
            return tpl + enc;
        }

        function getFilteredPages() {
            const q = searchQuery.trim().toLowerCase();
            let list = historyItems.filter((h) => {
                if (!h || !h.url) return false;
                if (scopeFilter === 'searches') return false;
                if (scopeFilter === 'bookmarked' && !isBookmarked(h.url)) return false;
                if (timeFilter !== 'all' && classifyTimeBucket(h.visited_at) !== timeFilter) return false;
                if (domainFilter && formatHost(h.url).toLowerCase() !== domainFilter) return false;
                if (q) {
                    const t = String(h.title || '').toLowerCase();
                    const u = String(h.url || '').toLowerCase();
                    if (!t.includes(q) && !u.includes(q)) return false;
                }
                return true;
            });
            if (sortMode === 'visits') {
                list = list.slice().sort((a, b) => (Number(b.visits) || 1) - (Number(a.visits) || 1) || (Number(b.visited_at) || 0) - (Number(a.visited_at) || 0));
            }
            return list;
        }

        function getFilteredSearches() {
            if (scopeFilter === 'pages' || scopeFilter === 'bookmarked' || domainFilter || timeFilter !== 'all') {
                return [];
            }
            const q = searchQuery.trim().toLowerCase();
            return recentSearches.filter((s) => {
                const str = String(s || '').trim();
                return str && (!q || str.toLowerCase().includes(q));
            });
        }

        function createActionSvg(kind, active) {
            if (kind === 'star') {
                const svg = createSvgEl('svg', { width: 13.5, height: 13.5, viewBox: '0 0 16 16', fill: active ? 'currentColor' : 'none' });
                svg.appendChild(createSvgEl('path', { d: 'M8 2.2l1.76 3.56 3.93.57-2.84 2.77.67 3.92L8 11.17l-3.52 1.85.67-3.92L2.31 6.33l3.93-.57L8 2.2z', stroke: 'currentColor', 'stroke-width': 1.35, 'stroke-linejoin': 'round' }));
                return svg;
            }
            if (kind === 'newtab') {
                const svg = createSvgEl('svg', { width: 13, height: 13, viewBox: '0 0 16 16', fill: 'none' });
                svg.appendChild(createSvgEl('path', { d: 'M6.5 3.5H3.8A1.3 1.3 0 0 0 2.5 4.8v7.4a1.3 1.3 0 0 0 1.3 1.3h7.4a1.3 1.3 0 0 0 1.3-1.3V9.5M9.5 2.5h4v4M13.5 2.5L7.5 8.5', stroke: 'currentColor', 'stroke-width': 1.35, 'stroke-linecap': 'round', 'stroke-linejoin': 'round' }));
                return svg;
            }
            const svg = createSvgEl('svg', { width: 12, height: 12, viewBox: '0 0 12 12', fill: 'none' });
            svg.appendChild(createSvgEl('path', { d: 'M2.5 2.5l7 7M9.5 2.5l-7 7', stroke: 'currentColor', 'stroke-width': 1.5, 'stroke-linecap': 'round' }));
            return svg;
        }

        const pageTitleEl = document.getElementById('htPageTitle');
        const totalPillEl = document.getElementById('htTotalPill');
        const btnReopenClosed = document.getElementById('btnHtReopenClosed');
        const btnSelectMode = document.getElementById('btnHtSelectMode');
        const btnClearAll = document.getElementById('btnHtClearAll');
        const searchInput = document.getElementById('inputHtSearch');
        const searchKbd = document.getElementById('htSearchKbd');
        const btnClearSearch = document.getElementById('btnHtClearSearch');
        const selectSort = document.getElementById('selectHtSort');
        const scopeBar = document.getElementById('htScopeBar');
        const timeBar = document.getElementById('htTimeBar');
        const domainBar = document.getElementById('htDomainBar');
        const batchBar = document.getElementById('htBatchBar');
        const batchCountEl = document.getElementById('htBatchCount');
        const btnSelectAll = document.getElementById('btnHtSelectAll');
        const btnDeleteSelected = document.getElementById('btnHtDeleteSelected');
        const btnCancelBatch = document.getElementById('btnHtCancelBatch');
        const searchesSection = document.getElementById('htSearchesSection');
        const searchesTitleEl = document.getElementById('htSearchesTitle');
        const searchesCountEl = document.getElementById('htSearchesCount');
        const searchesListEl = document.getElementById('htSearchesList');
        const btnClearSearches = document.getElementById('btnHtClearSearches');
        const groupsContainer = document.getElementById('htGroupsContainer');
        const emptyState = document.getElementById('htEmptyState');
        const emptyTitleEl = document.getElementById('htEmptyTitle');
        const emptyDescEl = document.getElementById('htEmptyDesc');
        const btnResetFilters = document.getElementById('btnHtResetFilters');

        function syncI18nChrome() {
            const zh = isZh();
            document.title = zh ? '浏览历史 — Fynexo' : 'History — Fynexo';
            pageTitleEl.textContent = zh ? '浏览历史' : 'History';
            if (btnReopenClosed) btnReopenClosed.textContent = zh ? '恢复关闭' : 'Reopen tab';
            btnSelectMode.textContent = batchMode ? (zh ? '完成管理' : 'Done') : (zh ? '批量管理' : 'Select');
            if (!btnClearAll.classList.contains('confirming')) {
                btnClearAll.textContent = zh ? '清空全部' : 'Clear all';
            }
            searchInput.placeholder = zh ? '搜索网页标题、域名或搜索记录...' : 'Search titles, domains, or past searches...';
            selectSort.options[0].textContent = zh ? '按最近访问' : 'Most recent';
            selectSort.options[1].textContent = zh ? '按访问次数' : 'Most visited';
            btnSelectAll.textContent = zh ? '全选当前' : 'Select all';
            btnDeleteSelected.textContent = zh ? '删除所选' : 'Delete selected';
            btnCancelBatch.textContent = zh ? '取消' : 'Cancel';
            searchesTitleEl.textContent = zh ? '最近搜索记录' : 'Recent searches';
            btnClearSearches.textContent = zh ? '清空搜索' : 'Clear searches';
            btnResetFilters.textContent = zh ? '重置筛选条件' : 'Reset filters';
        }

        function renderFilterBars() {
            const zh = isZh();
            const bmCount = historyItems.filter((h) => h && isBookmarked(h.url)).length;
            scopeBar.textContent = '';
            [
                ['all', zh ? '全部' : 'All', historyItems.length + recentSearches.length],
                ['pages', zh ? '浏览页面' : 'Pages', historyItems.length],
                ['searches', zh ? '搜索记录' : 'Searches', recentSearches.length],
                ['bookmarked', zh ? '已收藏' : 'Saved', bmCount],
            ].forEach(([id, label, num]) => {
                const b = document.createElement('button');
                b.type = 'button';
                b.className = 'ht-chip' + (scopeFilter === id ? ' active' : '');
                b.appendChild(document.createTextNode(label));
                const n = document.createElement('span');
                n.className = 'ht-chip-num';
                n.textContent = String(num);
                b.appendChild(n);
                b.onclick = () => {
                    scopeFilter = id;
                    renderAll();
                };
                scopeBar.appendChild(b);
            });

            timeBar.textContent = '';
            [
                ['all', zh ? '全部时间' : 'Any time'],
                ['today', zh ? '今天' : 'Today'],
                ['yesterday', zh ? '昨天' : 'Yesterday'],
                ['week', zh ? '近 7 天' : 'Last 7 days'],
                ['older', zh ? '更早' : 'Older'],
            ].forEach(([id, label]) => {
                const b = document.createElement('button');
                b.type = 'button';
                b.className = 'ht-chip' + (timeFilter === id ? ' active' : '');
                b.textContent = label;
                b.onclick = () => {
                    timeFilter = id;
                    renderAll();
                };
                timeBar.appendChild(b);
            });

            // Auto-extracted top domains filter strip
            const counts = Object.create(null);
            for (let i = 0; i < historyItems.length; i++) {
                const h = historyItems[i];
                if (!h || !h.url) continue;
                const host = formatHost(h.url).toLowerCase();
                if (!host) continue;
                counts[host] = (counts[host] || 0) + 1;
            }
            const topDomains = Object.keys(counts)
                .sort((a, b) => counts[b] - counts[a] || a.localeCompare(b))
                .slice(0, 8);

            if (topDomains.length >= 2 && scopeFilter !== 'searches') {
                domainBar.style.display = 'flex';
                domainBar.textContent = '';
                const lbl = document.createElement('span');
                lbl.className = 'ht-domain-label';
                lbl.textContent = zh ? '按站点：' : 'Site:';
                domainBar.appendChild(lbl);
                topDomains.forEach((dom) => {
                    const chip = document.createElement('button');
                    chip.type = 'button';
                    chip.className = 'ht-domain-chip' + (domainFilter === dom ? ' active' : '');
                    chip.appendChild(document.createTextNode(dom));
                    const cnt = document.createElement('span');
                    cnt.className = 'ht-chip-num';
                    cnt.textContent = String(counts[dom]);
                    chip.appendChild(cnt);
                    chip.onclick = () => {
                        domainFilter = domainFilter === dom ? '' : dom;
                        renderAll();
                    };
                    domainBar.appendChild(chip);
                });
            } else {
                domainBar.style.display = 'none';
            }
        }

        function syncBatchBar() {
            const zh = isZh();
            btnSelectMode.classList.toggle('active', batchMode);
            btnSelectMode.textContent = batchMode ? (zh ? '完成管理' : 'Done') : (zh ? '批量管理' : 'Select');
            batchBar.style.display = batchMode ? 'block' : 'none';
            batchCountEl.textContent = zh ? `已选 ${selectedUrls.size} 项` : `${selectedUrls.size} selected`;
            btnDeleteSelected.disabled = selectedUrls.size === 0;
            btnDeleteSelected.style.opacity = selectedUrls.size === 0 ? '0.45' : '1';
        }

        function renderAll() {
            syncI18nChrome();
            renderFilterBars();
            syncBatchBar();

            const zh = isZh();
            const filteredPages = getFilteredPages();
            const filteredSearches = getFilteredSearches();
            const totalMatched = filteredPages.length + filteredSearches.length;
            totalPillEl.textContent = zh ? `${totalMatched} 条` : String(totalMatched);

            btnClearSearch.style.display = searchQuery.length > 0 ? 'flex' : 'none';
            if (searchKbd) searchKbd.style.display = searchQuery.length > 0 ? 'none' : '';

            // 1. Recent Searches section
            if (filteredSearches.length > 0) {
                searchesSection.style.display = 'block';
                searchesCountEl.textContent = String(filteredSearches.length);
                searchesListEl.textContent = '';
                const frag = document.createDocumentFragment();
                filteredSearches.forEach((sq) => {
                    const chip = document.createElement('div');
                    chip.className = 'ht-qchip';
                    chip.role = 'button';
                    chip.tabIndex = 0;

                    const icon = createSvgEl('svg', { width: 12, height: 12, viewBox: '0 0 16 16', fill: 'none' });
                    icon.style.color = 'var(--ht-muted)';
                    icon.style.flexShrink = '0';
                    icon.appendChild(createSvgEl('circle', { cx: 8, cy: 8, r: 5.5, stroke: 'currentColor', 'stroke-width': 1.4 }));
                    icon.appendChild(createSvgEl('path', { d: 'M8 4.8v3.4l2.3 1.4', stroke: 'currentColor', 'stroke-width': 1.4, 'stroke-linecap': 'round', 'stroke-linejoin': 'round' }));

                    const txt = document.createElement('span');
                    txt.className = 'ht-qchip-text';
                    appendHighlighted(txt, sq, searchQuery);

                    const del = document.createElement('button');
                    del.type = 'button';
                    del.className = 'ht-qchip-del';
                    del.title = zh ? '移除此搜索记录' : 'Remove search';
                    del.textContent = '×';
                    del.onclick = (e) => {
                        e.stopPropagation();
                        const low = sq.toLowerCase();
                        recentSearches = recentSearches.filter((x) => String(x || '').toLowerCase() !== low);
                        sendIpc({ type: 'REMOVE_SEARCH', query: sq });
                        renderAll();
                    };

                    const runSearch = () => sendIpc({ type: 'NAVIGATE', url: currentEngineUrl(sq) });
                    chip.onclick = runSearch;
                    chip.onkeydown = (e) => {
                        if (e.key === 'Enter' || e.key === ' ') {
                            e.preventDefault();
                            runSearch();
                        }
                    };

                    chip.appendChild(icon);
                    chip.appendChild(txt);
                    chip.appendChild(del);
                    frag.appendChild(chip);
                });
                searchesListEl.appendChild(frag);
            } else {
                searchesSection.style.display = 'none';
            }

            // 2. Timeline-grouped (or visit-ranked) browsing pages
            groupsContainer.textContent = '';
            if (filteredPages.length > 0) {
                const groups = [];
                if (sortMode === 'visits') {
                    groups.push({ id: 'visits', label: zh ? '按访问频率排序' : 'Most visited pages', items: filteredPages });
                } else {
                    const buckets = {
                        today: { id: 'today', label: zh ? '今天' : 'Today', items: [] },
                        yesterday: { id: 'yesterday', label: zh ? '昨天' : 'Yesterday', items: [] },
                        week: { id: 'week', label: zh ? '最近 7 天' : 'Last 7 days', items: [] },
                        older: { id: 'older', label: zh ? '更早之前' : 'Older', items: [] },
                    };
                    filteredPages.forEach((h) => {
                        buckets[classifyTimeBucket(h.visited_at)].items.push(h);
                    });
                    ['today', 'yesterday', 'week', 'older'].forEach((k) => {
                        if (buckets[k].items.length > 0) groups.push(buckets[k]);
                    });
                }

                const gFrag = document.createDocumentFragment();
                groups.forEach((grp) => {
                    const card = document.createElement('section');
                    card.className = 'ht-card';

                    const head = document.createElement('div');
                    head.className = 'ht-group-head';
                    const tw = document.createElement('div');
                    tw.className = 'ht-group-title-wrap';
                    const gt = document.createElement('span');
                    gt.className = 'ht-group-title';
                    gt.textContent = grp.label;
                    const gc = document.createElement('span');
                    gc.className = 'ht-group-count';
                    gc.textContent = String(grp.items.length);
                    tw.appendChild(gt);
                    tw.appendChild(gc);

                    const clearGrpBtn = document.createElement('button');
                    clearGrpBtn.type = 'button';
                    clearGrpBtn.className = 'ht-link-btn';
                    clearGrpBtn.textContent = zh ? '清除本组' : 'Clear group';
                    clearGrpBtn.onclick = () => {
                        const urls = grp.items.map((x) => x.url);
                        const keys = new Set(urls.map(normalizeUrlKey));
                        historyItems = historyItems.filter((h) => !keys.has(normalizeUrlKey(h.url)));
                        urls.forEach((u) => selectedUrls.delete(u));
                        sendIpc({ type: 'DELETE_HISTORY_BATCH', urls: urls });
                        renderAll();
                    };

                    head.appendChild(tw);
                    head.appendChild(clearGrpBtn);
                    card.appendChild(head);

                    const listEl = document.createElement('div');
                    listEl.className = 'ht-row-list';
                    grp.items.forEach((h) => {
                        const isSel = selectedUrls.has(h.url);
                        const row = document.createElement('div');
                        row.className = 'ht-row' + (isSel ? ' selected' : '');

                        if (batchMode) {
                            const cb = document.createElement('input');
                            cb.type = 'checkbox';
                            cb.className = 'ht-check';
                            cb.checked = isSel;
                            cb.onclick = (e) => {
                                e.stopPropagation();
                                if (cb.checked) selectedUrls.add(h.url);
                                else selectedUrls.delete(h.url);
                                row.classList.toggle('selected', cb.checked);
                                syncBatchBar();
                            };
                            row.appendChild(cb);
                        }

                        row.appendChild(createSiteEmblem(h.url, h.title));

                        const main = document.createElement('div');
                        main.className = 'ht-row-main';
                        const titleEl = document.createElement('div');
                        titleEl.className = 'ht-row-title';
                        appendHighlighted(titleEl, h.title || formatHost(h.url), searchQuery);

                        const sub = document.createElement('div');
                        sub.className = 'ht-row-sub';
                        const hostEl = document.createElement('span');
                        hostEl.className = 'ht-row-host';
                        hostEl.title = h.url;
                        appendHighlighted(hostEl, h.url.replace(/^https?:\/\//i, ''), searchQuery);
                        sub.appendChild(hostEl);
                        if ((Number(h.visits) || 1) > 1) {
                            const vTag = document.createElement('span');
                            vTag.className = 'ht-visit-tag';
                            vTag.textContent = `×${h.visits}`;
                            sub.appendChild(vTag);
                        }
                        main.appendChild(titleEl);
                        main.appendChild(sub);
                        row.appendChild(main);

                        const tStr = formatVisitTime(h.visited_at, classifyTimeBucket(h.visited_at));
                        if (tStr) {
                            const timeEl = document.createElement('span');
                            timeEl.className = 'ht-row-time';
                            timeEl.textContent = tStr;
                            row.appendChild(timeEl);
                        }

                        const acts = document.createElement('div');
                        acts.className = 'ht-row-acts';

                        const saved = isBookmarked(h.url);
                        const bmBtn = document.createElement('button');
                        bmBtn.type = 'button';
                        bmBtn.className = 'ht-act-btn' + (saved ? ' saved' : '');
                        bmBtn.title = saved ? (zh ? '取消收藏' : 'Remove shortcut') : (zh ? '添加到快捷方式' : 'Save to shortcuts');
                        bmBtn.appendChild(createActionSvg('star', saved));
                        bmBtn.onclick = (e) => {
                            e.stopPropagation();
                            sendIpc({ type: 'TOGGLE_BOOKMARK', url: h.url, title: h.title || formatHost(h.url) });
                        };

                        const ntBtn = document.createElement('button');
                        ntBtn.type = 'button';
                        ntBtn.className = 'ht-act-btn';
                        ntBtn.title = zh ? '在后台新标签页打开' : 'Open in new tab';
                        ntBtn.appendChild(createActionSvg('newtab', false));
                        ntBtn.onclick = (e) => {
                            e.stopPropagation();
                            sendIpc({ type: 'NEW_TAB', url: h.url, title: h.title || '', background: true });
                        };

                        const delBtn = document.createElement('button');
                        delBtn.type = 'button';
                        delBtn.className = 'ht-act-btn del';
                        delBtn.title = zh ? '删除此条记录' : 'Delete item';
                        delBtn.appendChild(createActionSvg('del', false));
                        delBtn.onclick = (e) => {
                            e.stopPropagation();
                            const k = normalizeUrlKey(h.url);
                            historyItems = historyItems.filter((x) => normalizeUrlKey(x.url) !== k);
                            selectedUrls.delete(h.url);
                            sendIpc({ type: 'DELETE_HISTORY_ITEM', url: h.url });
                            renderAll();
                        };

                        acts.appendChild(bmBtn);
                        acts.appendChild(ntBtn);
                        acts.appendChild(delBtn);
                        row.appendChild(acts);

                        row.onclick = () => {
                            if (batchMode) {
                                if (selectedUrls.has(h.url)) selectedUrls.delete(h.url);
                                else selectedUrls.add(h.url);
                                renderAll();
                                return;
                            }
                            sendIpc({ type: 'NAVIGATE', url: h.url });
                        };

                        listEl.appendChild(row);
                    });

                    card.appendChild(listEl);
                    gFrag.appendChild(card);
                });
                groupsContainer.appendChild(gFrag);
            }

            // 3. Empty state
            const hasFilter = scopeFilter !== 'all' || timeFilter !== 'all' || !!domainFilter || !!searchQuery.trim();
            if (totalMatched === 0) {
                emptyState.style.display = 'flex';
                emptyTitleEl.textContent = hasFilter
                    ? (zh ? '未找到匹配的历史记录' : 'No matching history items')
                    : (zh ? '暂无浏览历史' : 'Your history is empty');
                emptyDescEl.textContent = hasFilter
                    ? (zh ? '试试切换筛选范围、时间段或清空搜索关键词。' : 'Try adjusting your filters or search query.')
                    : (zh ? '浏览网页或使用搜索栏后，记录会自动出现在这里。' : 'Pages you visit and queries you search will appear here.');
                btnResetFilters.style.display = hasFilter ? 'inline-flex' : 'none';
            } else {
                emptyState.style.display = 'none';
            }
        }

        searchInput.addEventListener('input', () => {
            searchQuery = searchInput.value;
            renderAll();
        });
        btnClearSearch.onclick = () => {
            searchInput.value = '';
            searchQuery = '';
            renderAll();
            searchInput.focus();
        };
        selectSort.onchange = () => {
            sortMode = selectSort.value === 'visits' ? 'visits' : 'recent';
            renderAll();
        };
        btnSelectMode.onclick = () => {
            batchMode = !batchMode;
            if (!batchMode) selectedUrls.clear();
            renderAll();
        };
        btnCancelBatch.onclick = () => {
            batchMode = false;
            selectedUrls.clear();
            renderAll();
        };
        btnSelectAll.onclick = () => {
            const pages = getFilteredPages();
            const allSelected = pages.length > 0 && pages.every((h) => selectedUrls.has(h.url));
            if (allSelected) {
                pages.forEach((h) => selectedUrls.delete(h.url));
            } else {
                pages.forEach((h) => selectedUrls.add(h.url));
            }
            renderAll();
        };
        btnDeleteSelected.onclick = () => {
            if (selectedUrls.size === 0) return;
            const urls = Array.from(selectedUrls);
            const keys = new Set(urls.map(normalizeUrlKey));
            historyItems = historyItems.filter((h) => !keys.has(normalizeUrlKey(h.url)));
            selectedUrls.clear();
            batchMode = false;
            sendIpc({ type: 'DELETE_HISTORY_BATCH', urls: urls });
            renderAll();
        };
        btnClearSearches.onclick = () => {
            recentSearches = [];
            sendIpc({ type: 'CLEAR_SEARCHES' });
            renderAll();
        };
        btnClearAll.onclick = () => {
            const zh = isZh();
            if (!btnClearAll.classList.contains('confirming')) {
                btnClearAll.classList.add('confirming');
                btnClearAll.textContent = zh ? '再次点击确认清空' : 'Click again to clear';
                if (clearAllConfirmTimer) clearTimeout(clearAllConfirmTimer);
                clearAllConfirmTimer = setTimeout(() => {
                    btnClearAll.classList.remove('confirming');
                    btnClearAll.textContent = zh ? '清空全部' : 'Clear all';
                }, 3500);
                return;
            }
            if (clearAllConfirmTimer) clearTimeout(clearAllConfirmTimer);
            btnClearAll.classList.remove('confirming');
            historyItems = [];
            recentSearches = [];
            selectedUrls.clear();
            batchMode = false;
            sendIpc({ type: 'CLEAR_HISTORY' });
            renderAll();
        };
        if (btnReopenClosed) {
            btnReopenClosed.onclick = () => sendIpc({ type: 'REOPEN_CLOSED_TAB' });
        }
        btnResetFilters.onclick = () => {
            scopeFilter = 'all';
            timeFilter = 'all';
            domainFilter = '';
            searchQuery = '';
            searchInput.value = '';
            renderAll();
        };

        document.addEventListener('keydown', (e) => {
            if ((e.ctrlKey || e.metaKey) && !e.altKey && (e.key || '').toLowerCase() === 'f') {
                e.preventDefault();
                searchInput.focus();
                searchInput.select();
                return;
            }
            const active = document.activeElement;
            const inInput = active && (active.tagName === 'INPUT' || active.tagName === 'TEXTAREA' || active.tagName === 'SELECT');
            if (e.key === '/' && !inInput && !e.ctrlKey && !e.metaKey && !e.altKey) {
                e.preventDefault();
                searchInput.focus();
                searchInput.select();
            } else if (e.key === 'Escape') {
                if (searchQuery) {
                    searchQuery = '';
                    searchInput.value = '';
                    renderAll();
                } else if (batchMode) {
                    batchMode = false;
                    selectedUrls.clear();
                    renderAll();
                }
            }
        });

        window.__fynexo_on_settings_runtime__ = function (payload) {
            if (!payload) return;
            if (payload.config) config = Object.assign(config, payload.config);
            if (payload.active_theme) activeTheme = payload.active_theme;
            if (Array.isArray(payload.bookmarks)) bookmarks = payload.bookmarks;
            if (Array.isArray(payload.history)) historyItems = payload.history;
            if (Array.isArray(payload.recent_searches)) recentSearches = payload.recent_searches;
            applyTheme();
            renderAll();
        };

        applyTheme();
        renderAll();
