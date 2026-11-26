        __FNX_DEFAULTS__;

        const boot = __FNX_BOOT_JSON__ || {};
        let config = Object.assign({ language: 'zh' }, boot.config || {});
        let activeTheme = boot.active_theme || {};
        let bookmarks = Array.isArray(boot.bookmarks) ? boot.bookmarks : [];

        let activeFolder = '__all__'; // '__all__' | '__unfiled__' | folderName
        let searchQuery = '';
        let editingOrigUrl = null;

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

            root.style.setProperty('--bm-bg', bg);
            root.style.setProperty('--bm-fg', fg);
            root.style.setProperty('--bm-surface', surface);
            root.style.setProperty('--bm-accent', accent);
            root.style.setProperty('--bm-accent-soft', `color-mix(in srgb, ${accent} 16%, transparent)`);
            root.style.setProperty('--bm-border', border);
            root.style.setProperty('--bm-border-hover', borderHover);
            root.style.setProperty('--bm-muted', muted);
            root.style.setProperty('--bm-secondary', secondary);
            root.style.setProperty('--bm-hover', hover);
            root.style.setProperty('--bm-danger', danger);
        }

        function formatHost(u) {
            try {
                return new URL(u).hostname.replace(/^www\./i, '') || u;
            } catch (_) {
                return String(u || '');
            }
        }

        function monogramColor(url) {
            const palette = ['#7A88F7', '#38BDF8', '#34D399', '#F472B6', '#FB923C', '#A78BFA'];
            const s = String(url || '');
            let h = 0;
            for (let i = 0; i < s.length; i++) h = ((h << 5) - h + s.charCodeAt(i)) | 0;
            return palette[Math.abs(h) % palette.length];
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
            strong.className = 'bm-hl';
            strong.textContent = str.slice(idx, idx + needle.length);
            el.appendChild(strong);
            if (idx + needle.length < str.length) {
                el.appendChild(document.createTextNode(str.slice(idx + needle.length)));
            }
        }

        function createActionSvg(kind) {
            if (kind === 'edit') {
                const svg = createSvgEl('svg', { width: 13, height: 13, viewBox: '0 0 16 16', fill: 'none' });
                svg.appendChild(createSvgEl('path', { d: 'M11.2 2.4l2.4 2.4-7.6 7.6H3.6V10l7.6-7.6Z', stroke: 'currentColor', 'stroke-width': 1.35, 'stroke-linejoin': 'round' }));
                return svg;
            }
            if (kind === 'copy') {
                const svg = createSvgEl('svg', { width: 13, height: 13, viewBox: '0 0 16 16', fill: 'none' });
                svg.appendChild(createSvgEl('rect', { x: 5.5, y: 5.5, width: 7.5, height: 7.5, rx: 1.5, stroke: 'currentColor', 'stroke-width': 1.35 }));
                svg.appendChild(createSvgEl('path', { d: 'M10.5 5.5V4.2A1.2 1.2 0 0 0 9.3 3H4.2A1.2 1.2 0 0 0 3 4.2v5.1a1.2 1.2 0 0 0 1.2 1.2h1.3', stroke: 'currentColor', 'stroke-width': 1.35 }));
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

        const pageTitleEl = document.getElementById('bmPageTitle');
        const totalPillEl = document.getElementById('bmTotalPill');
        const btnAddToggle = document.getElementById('btnBmAddToggle');
        const btnImport = document.getElementById('btnBmImport');
        const btnExport = document.getElementById('btnBmExport');
        const importFileEl = document.getElementById('inputBmImportFile');
        const searchInput = document.getElementById('inputBmSearch');
        const searchKbd = document.getElementById('bmSearchKbd');
        const btnClearSearch = document.getElementById('btnBmClearSearch');
        const folderBar = document.getElementById('bmFolderBar');
        const editorCard = document.getElementById('bmEditorCard');
        const editorTitleEl = document.getElementById('bmEditorTitle');
        const btnEditorCancel = document.getElementById('btnBmEditorCancel');
        const btnEditorSave = document.getElementById('btnBmEditorSave');
        const inputTitle = document.getElementById('inputBmTitle');
        const inputUrl = document.getElementById('inputBmUrl');
        const inputFolder = document.getElementById('inputBmFolder');
        const folderSuggestionsEl = document.getElementById('bmFolderSuggestions');
        const groupsContainer = document.getElementById('bmGroupsContainer');
        const emptyState = document.getElementById('bmEmptyState');
        const emptyTitleEl = document.getElementById('bmEmptyTitle');
        const emptyDescEl = document.getElementById('bmEmptyDesc');
        const btnResetFilters = document.getElementById('btnBmResetFilters');

        function openEditor(existingBm) {
            const zh = isZh();
            editingOrigUrl = existingBm ? existingBm.url : null;
            editorTitleEl.textContent = existingBm
                ? (zh ? '编辑书签' : 'Edit bookmark')
                : (zh ? '添加新书签' : 'Add bookmark');
            inputTitle.value = existingBm ? (existingBm.title || '') : '';
            inputUrl.value = existingBm ? (existingBm.url || '') : '';
            inputFolder.value = existingBm ? (existingBm.folder || '') : (activeFolder !== '__all__' && activeFolder !== '__unfiled__' ? activeFolder : '');
            editorCard.style.display = 'flex';
            (existingBm ? inputTitle : inputUrl).focus();
        }

        function closeEditor() {
            editingOrigUrl = null;
            editorCard.style.display = 'none';
            inputTitle.value = '';
            inputUrl.value = '';
            inputFolder.value = '';
        }

        function submitEditor() {
            const urlVal = inputUrl.value.trim();
            if (!urlVal) {
                inputUrl.focus();
                return;
            }
            const titleVal = inputTitle.value.trim();
            const folderVal = inputFolder.value.trim();
            if (editingOrigUrl) {
                sendIpc({
                    type: 'EDIT_BOOKMARK',
                    orig_url: editingOrigUrl,
                    title: titleVal,
                    url: urlVal,
                    folder: folderVal,
                });
            } else {
                sendIpc({
                    type: 'ADD_BOOKMARK',
                    title: titleVal,
                    url: urlVal,
                    folder: folderVal,
                });
            }
            closeEditor();
        }

        function syncI18n() {
            const zh = isZh();
            document.title = zh ? '书签管理 — Fynexo' : 'Bookmarks — Fynexo';
            pageTitleEl.textContent = zh ? '书签管理' : 'Bookmarks';
            btnAddToggle.textContent = zh ? '+ 添加' : '+ Add';
            btnImport.textContent = zh ? '导入' : 'Import';
            btnExport.textContent = zh ? '导出' : 'Export';
            searchInput.placeholder = zh ? '搜索书签标题、网址或文件夹...' : 'Search bookmarks, URLs, or folders...';
            btnEditorCancel.textContent = zh ? '取消' : 'Cancel';
            btnEditorSave.textContent = zh ? '保存' : 'Save';
            inputTitle.placeholder = zh ? '名称（如 GitHub）' : 'Title (e.g. GitHub)';
            inputUrl.placeholder = zh ? '网址（如 https://github.com）' : 'URL (e.g. https://github.com)';
            inputFolder.placeholder = zh ? '文件夹（可选，如 开发 / 工具）' : 'Folder (optional, e.g. Dev / Tools)';
            btnResetFilters.textContent = zh ? '重置筛选' : 'Reset filters';
        }

        function renderAll() {
            syncI18n();
            const zh = isZh();
            btnClearSearch.style.display = searchQuery.length > 0 ? 'flex' : 'none';
            if (searchKbd) searchKbd.style.display = searchQuery.length > 0 ? 'none' : '';

            // Collect folders
            const folderCounts = Object.create(null);
            let unfiledCount = 0;
            bookmarks.forEach((b) => {
                const f = String((b && b.folder) || '').trim();
                if (!f) unfiledCount++;
                else folderCounts[f] = (folderCounts[f] || 0) + 1;
            });
            const folderNames = Object.keys(folderCounts).sort((a, b) => a.localeCompare(b));

            folderSuggestionsEl.textContent = '';
            folderNames.forEach((fn) => {
                const opt = document.createElement('option');
                opt.value = fn;
                folderSuggestionsEl.appendChild(opt);
            });

            if (activeFolder !== '__all__' && activeFolder !== '__unfiled__' && !folderCounts[activeFolder]) {
                activeFolder = '__all__';
            }

            folderBar.textContent = '';
            const tabs = [
                ['__all__', zh ? '全部' : 'All', bookmarks.length],
            ];
            if (unfiledCount > 0 && folderNames.length > 0) {
                tabs.push(['__unfiled__', zh ? '未分类' : 'Unfiled', unfiledCount]);
            }
            folderNames.forEach((fn) => tabs.push([fn, fn, folderCounts[fn]]));
            tabs.forEach(([id, label, count]) => {
                const chip = document.createElement('button');
                chip.type = 'button';
                chip.className = 'bm-chip' + (activeFolder === id ? ' active' : '');
                chip.appendChild(document.createTextNode(label));
                const num = document.createElement('span');
                num.className = 'bm-chip-num';
                num.textContent = String(count);
                chip.appendChild(num);
                chip.onclick = () => {
                    activeFolder = id;
                    renderAll();
                };
                folderBar.appendChild(chip);
            });

            // Filter bookmarks
            const q = searchQuery.trim().toLowerCase();
            const filtered = bookmarks.filter((b) => {
                if (!b || !b.url) return false;
                const f = String(b.folder || '').trim();
                if (activeFolder === '__unfiled__' && f !== '') return false;
                if (activeFolder !== '__all__' && activeFolder !== '__unfiled__' && f !== activeFolder) return false;
                if (q) {
                    const t = String(b.title || '').toLowerCase();
                    const u = String(b.url || '').toLowerCase();
                    if (!t.includes(q) && !u.includes(q) && !f.toLowerCase().includes(q)) return false;
                }
                return true;
            });

            totalPillEl.textContent = zh ? `${filtered.length} 个` : String(filtered.length);
            groupsContainer.textContent = '';

            if (filtered.length === 0) {
                const hasFilter = activeFolder !== '__all__' || !!q;
                emptyState.style.display = 'flex';
                emptyTitleEl.textContent = hasFilter
                    ? (zh ? '未找到匹配的书签' : 'No matching bookmarks')
                    : (zh ? '暂无保存的书签' : 'No bookmarks yet');
                emptyDescEl.textContent = hasFilter
                    ? (zh ? '试试切换文件夹或清空搜索关键词。' : 'Try switching folders or clearing your search.')
                    : (zh ? '点击右上角「+ 添加」或「导入」快速整理常用网站。' : 'Click + Add or Import to organize your favorite sites.');
                btnResetFilters.style.display = hasFilter ? 'inline-flex' : 'none';
                return;
            }
            emptyState.style.display = 'none';

            // Group by folder when viewing All without search & multiple folders exist
            const groups = [];
            if (activeFolder === '__all__' && !q && folderNames.length > 0) {
                const unfiled = filtered.filter((b) => !String(b.folder || '').trim());
                if (unfiled.length > 0) {
                    groups.push({ label: zh ? '未分类' : 'Unfiled', items: unfiled });
                }
                folderNames.forEach((fn) => {
                    const items = filtered.filter((b) => String(b.folder || '').trim() === fn);
                    if (items.length > 0) groups.push({ label: fn, items: items });
                });
            } else {
                const heading = activeFolder === '__all__'
                    ? (zh ? '全部书签' : 'All bookmarks')
                    : activeFolder === '__unfiled__'
                      ? (zh ? '未分类' : 'Unfiled')
                      : activeFolder;
                groups.push({ label: heading, items: filtered });
            }

            const frag = document.createDocumentFragment();
            groups.forEach((grp) => {
                const card = document.createElement('section');
                card.className = 'bm-card';

                const head = document.createElement('div');
                head.className = 'bm-group-head';
                const gt = document.createElement('span');
                gt.className = 'bm-group-title';
                gt.textContent = grp.label;
                const gc = document.createElement('span');
                gc.className = 'bm-group-count';
                gc.textContent = String(grp.items.length);
                head.appendChild(gt);
                head.appendChild(gc);
                card.appendChild(head);

                grp.items.forEach((b) => {
                    const row = document.createElement('div');
                    row.className = 'bm-row';

                    const emblem = document.createElement('span');
                    emblem.className = 'bm-emblem';
                    const host = formatHost(b.url);
                    emblem.textContent = (host || b.title || '?').charAt(0).toUpperCase();
                    emblem.style.color = monogramColor(host || b.url);
                    row.appendChild(emblem);

                    const main = document.createElement('div');
                    main.className = 'bm-row-main';
                    const tEl = document.createElement('div');
                    tEl.className = 'bm-row-title';
                    appendHighlighted(tEl, b.title || host, searchQuery);
                    const uEl = document.createElement('div');
                    uEl.className = 'bm-row-url';
                    uEl.title = b.url;
                    appendHighlighted(uEl, b.url.replace(/^https?:\/\//i, ''), searchQuery);
                    main.appendChild(tEl);
                    main.appendChild(uEl);
                    row.appendChild(main);

                    if (b.folder && (q || activeFolder === '__all__')) {
                        const fTag = document.createElement('span');
                        fTag.className = 'bm-folder-tag';
                        fTag.textContent = b.folder;
                        row.appendChild(fTag);
                    }

                    const acts = document.createElement('div');
                    acts.className = 'bm-row-acts';

                    const editBtn = document.createElement('button');
                    editBtn.type = 'button';
                    editBtn.className = 'bm-act-btn';
                    editBtn.title = zh ? '编辑书签' : 'Edit bookmark';
                    editBtn.appendChild(createActionSvg('edit'));
                    editBtn.onclick = (e) => {
                        e.stopPropagation();
                        openEditor(b);
                    };

                    const copyBtn = document.createElement('button');
                    copyBtn.type = 'button';
                    copyBtn.className = 'bm-act-btn';
                    copyBtn.title = zh ? '复制网址' : 'Copy URL';
                    copyBtn.appendChild(createActionSvg('copy'));
                    copyBtn.onclick = (e) => {
                        e.stopPropagation();
                        try { navigator.clipboard.writeText(b.url); } catch (_) {}
                    };

                    const ntBtn = document.createElement('button');
                    ntBtn.type = 'button';
                    ntBtn.className = 'bm-act-btn';
                    ntBtn.title = zh ? '后台新标签页打开' : 'Open in new tab';
                    ntBtn.appendChild(createActionSvg('newtab'));
                    ntBtn.onclick = (e) => {
                        e.stopPropagation();
                        sendIpc({ type: 'NEW_TAB', url: b.url, title: b.title || '', background: true });
                    };

                    const delBtn = document.createElement('button');
                    delBtn.type = 'button';
                    delBtn.className = 'bm-act-btn del';
                    delBtn.title = zh ? '删除书签' : 'Delete bookmark';
                    delBtn.appendChild(createActionSvg('del'));
                    delBtn.onclick = (e) => {
                        e.stopPropagation();
                        bookmarks = bookmarks.filter((x) => x.url !== b.url);
                        sendIpc({ type: 'REMOVE_BOOKMARK', url: b.url });
                        renderAll();
                    };

                    acts.appendChild(editBtn);
                    acts.appendChild(copyBtn);
                    acts.appendChild(ntBtn);
                    acts.appendChild(delBtn);
                    row.appendChild(acts);

                    row.onclick = () => sendIpc({ type: 'NAVIGATE', url: b.url });
                    card.appendChild(row);
                });

                frag.appendChild(card);
            });
            groupsContainer.appendChild(frag);
        }

        btnAddToggle.onclick = () => {
            if (editorCard.style.display === 'flex' && !editingOrigUrl) closeEditor();
            else openEditor(null);
        };
        btnEditorCancel.onclick = closeEditor;
        btnEditorSave.onclick = submitEditor;
        [inputTitle, inputUrl, inputFolder].forEach((el) => {
            el.addEventListener('keydown', (e) => {
                if (e.key === 'Enter') {
                    e.preventDefault();
                    submitEditor();
                } else if (e.key === 'Escape') {
                    closeEditor();
                }
            });
        });

        btnImport.onclick = () => importFileEl.click();
        importFileEl.onchange = () => {
            const file = importFileEl.files && importFileEl.files[0];
            if (!file) return;
            const reader = new FileReader();
            reader.onload = () => {
                const raw = String(reader.result || '');
                if (raw.trim()) {
                    sendIpc({ type: 'IMPORT_SHORTCUTS_JSON', json: raw, merge: true });
                }
                importFileEl.value = '';
            };
            reader.readAsText(file);
        };
        btnExport.onclick = () => sendIpc({ type: 'EXPORT_SHORTCUTS' });

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
        btnResetFilters.onclick = () => {
            activeFolder = '__all__';
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
            const inInput = active && (active.tagName === 'INPUT' || active.tagName === 'TEXTAREA');
            if (e.key === '/' && !inInput && !e.ctrlKey && !e.metaKey && !e.altKey) {
                e.preventDefault();
                searchInput.focus();
                searchInput.select();
            } else if (e.key === 'Escape') {
                if (editorCard.style.display === 'flex') closeEditor();
                else if (searchQuery) {
                    searchQuery = '';
                    searchInput.value = '';
                    renderAll();
                }
            }
        });

        window.__fynexo_on_settings_runtime__ = function (payload) {
            if (!payload) return;
            if (payload.config) config = Object.assign(config, payload.config);
            if (payload.active_theme) activeTheme = payload.active_theme;
            if (Array.isArray(payload.bookmarks)) bookmarks = payload.bookmarks;
            applyTheme();
            renderAll();
        };

        applyTheme();
        renderAll();
