    (function () {
        __FNX_DEFAULTS__;

        const boot = __FNX_BOOT_JSON__ || {};
        let config = Object.assign({ language: 'zh', downloads_dir: '' }, boot.config || {});
        let activeTheme = Object.assign({}, DEFAULT_STYLES.colors, boot.active_theme || {});
        let downloads = Array.isArray(boot.downloads) ? boot.downloads.slice() : [];
        let downloadsDir = String(boot.downloads_dir || '');
        let activeFilter = 'all';
        let searchQuery = '';

        function isZh() {
            if (typeof FynexoI18n !== 'undefined') return FynexoI18n.isZhLocale(config.language);
            return String(config.language || 'zh').toLowerCase().startsWith('zh');
        }

        function sendIpc(data) {
            if (window.ipc && typeof window.ipc.postMessage === 'function') {
                window.ipc.postMessage(JSON.stringify(data));
            }
        }

        function contrastOnAccent(hex) {
            const s = String(hex || '').trim();
            if (!/^#[0-9a-f]{6}$/i.test(s)) return '#ffffff';
            const r = parseInt(s.slice(1, 3), 16);
            const g = parseInt(s.slice(3, 5), 16);
            const b = parseInt(s.slice(5, 7), 16);
            return (0.299 * r + 0.587 * g + 0.114 * b) > 155 ? '#0b0e17' : '#ffffff';
        }

        function applyTheme() {
            const r = document.documentElement.style;
            const d = (typeof DEFAULT_STYLES !== 'undefined' && DEFAULT_STYLES.colors) || {};
            const c = activeTheme || d;
            r.setProperty('--dl-bg', c.bg || d.bg || '');
            r.setProperty('--dl-panel', c.panel || d.panel || '');
            r.setProperty('--dl-surface', c.surface || d.surface || '');
            r.setProperty('--dl-border', c.border || d.border || '');
            r.setProperty('--dl-border-hover', c.border_hover || d.border_hover || '');
            r.setProperty('--dl-fg', c.fg || d.fg || '');
            r.setProperty('--dl-secondary', c.secondary || d.secondary || '');
            r.setProperty('--dl-muted', c.muted || d.muted || '');
            r.setProperty('--dl-accent', c.accent || d.accent || '');
            r.setProperty('--dl-on-accent', contrastOnAccent(c.accent || d.accent || ''));
            r.setProperty('--dl-accent-bg', c.accent_bg || d.accent_bg || '');
            r.setProperty('--dl-hover', c.hover || d.hover || '');
        }

        function fmtBytes(n) {
            const v = Number(n) || 0;
            if (v <= 0) return '0 B';
            if (v < 1024) return v + ' B';
            if (v < 1024 * 1024) return (v / 1024).toFixed(1) + ' KB';
            if (v < 1024 * 1024 * 1024) return (v / (1024 * 1024)).toFixed(1) + ' MB';
            return (v / (1024 * 1024 * 1024)).toFixed(2) + ' GB';
        }

        function fmtTime(ts) {
            const sec = Number(ts) || 0;
            if (!sec) return '';
            try {
                const d = new Date(sec * 1000);
                const now = new Date();
                const pad = (x) => String(x).padStart(2, '0');
                if (d.toDateString() === now.toDateString()) {
                    return pad(d.getHours()) + ':' + pad(d.getMinutes());
                }
                return d.getFullYear() + '-' + pad(d.getMonth() + 1) + '-' + pad(d.getDate()) + ' ' + pad(d.getHours()) + ':' + pad(d.getMinutes());
            } catch (_) {
                return '';
            }
        }

        function extOf(name) {
            const s = String(name || '');
            const dot = s.lastIndexOf('.');
            if (dot <= 0 || dot === s.length - 1) return 'FILE';
            return s.slice(dot + 1, dot + 5).toUpperCase();
        }

        function hostOf(rawUrl) {
            if (!rawUrl) return '';
            try {
                return new URL(rawUrl).hostname.replace(/^www\./, '');
            } catch (_) {
                return '';
            }
        }

        function applyI18n() {
            const zh = isZh();
            document.title = zh ? '下载管理 — Fynexo' : 'Downloads — Fynexo';
            document.getElementById('dlTitle').textContent = zh ? '下载管理' : 'Downloads';
            document.getElementById('dlSearchInput').placeholder = zh ? '搜索文件名或来源域名...' : 'Search files or source domains...';
            document.getElementById('btnOpenFolder').textContent = zh ? '打开下载目录' : 'Open folder';
            document.getElementById('btnClearFinished').textContent = zh ? '清空已完成记录' : 'Clear finished';
            document.getElementById('dlEmptyTitle').textContent = zh ? '暂无下载记录' : 'No downloads yet';
            document.getElementById('dlEmptySub').textContent = zh ? '下载的文件或截取的网页截图都会集中显示在这里' : 'Files you download or screenshots you capture appear here';
            const chipLabels = {
                all: zh ? '全部' : 'All',
                active: zh ? '进行中 / 已暂停' : 'Active / Paused',
                completed: zh ? '已完成' : 'Completed',
                stopped: zh ? '已取消 / 失败' : 'Stopped'
            };
            document.querySelectorAll('#dlFilterChips .dl-chip').forEach((btn) => {
                const key = btn.dataset.filter;
                if (chipLabels[key]) btn.textContent = chipLabels[key];
            });
            const dirEl = document.getElementById('dlDirHint');
            if (dirEl && downloadsDir) {
                dirEl.textContent = (zh ? '保存位置：' : 'Folder: ') + downloadsDir;
                dirEl.title = downloadsDir;
            }
            const tInp = document.getElementById('dlTurboInput');
            const tBtn = document.getElementById('btnTurboDownload');
            const tBadge = document.getElementById('dlTurboThreadsBadge');
            if (tInp) tInp.placeholder = zh ? '粘贴直链 http(s):// 地址，使用多线程分片高速下载…' : 'Paste direct http(s):// file URL for multi-threaded turbo download...';
            if (tBtn) tBtn.textContent = zh ? '高速下载' : 'Turbo Download';
            if (tBadge) {
                const th = config.parallel_downloads !== false ? Math.max(1, Math.min(16, Number(config.download_threads) || 8)) : 1;
                tBadge.textContent = zh ? (th + ' 线程') : (th + 'x');
            }
        }

        function statusMeta(st, zh) {
            if (st === 'downloading') return { cls: 'downloading', label: zh ? '下载中' : 'Downloading' };
            if (st === 'paused') return { cls: 'paused', label: zh ? '已暂停' : 'Paused' };
            if (st === 'completed') return { cls: 'completed', label: zh ? '已完成' : 'Completed' };
            if (st === 'cancelled') return { cls: 'stopped', label: zh ? '已取消' : 'Cancelled' };
            if (st === 'interrupted') return { cls: 'stopped', label: zh ? '已中断' : 'Interrupted' };
            return { cls: 'stopped', label: zh ? '失败' : 'Failed' };
        }

        function filteredItems() {
            const q = searchQuery.trim().toLowerCase();
            return downloads.filter((d) => {
                if (!d) return false;
                const st = String(d.status || '');
                if (activeFilter === 'active' && st !== 'downloading' && st !== 'paused') return false;
                if (activeFilter === 'completed' && st !== 'completed') return false;
                if (activeFilter === 'stopped' && st !== 'cancelled' && st !== 'failed' && st !== 'interrupted') return false;
                if (q) {
                    const hay = (String(d.name || '') + ' ' + String(d.url || '')).toLowerCase();
                    if (!hay.includes(q)) return false;
                }
                return true;
            });
        }

        function renderList() {
            const zh = isZh();
            const listEl = document.getElementById('dlList');
            const emptyEl = document.getElementById('dlEmpty');
            const badgeEl = document.getElementById('dlCountBadge');
            const items = filteredItems();
            badgeEl.textContent = String(items.length);
            listEl.textContent = '';

            if (items.length === 0) {
                emptyEl.style.display = 'flex';
                return;
            }
            emptyEl.style.display = 'none';

            const frag = document.createDocumentFragment();
            items.forEach((dl) => {
                const st = String(dl.status || 'completed');
                const meta = statusMeta(st, zh);
                const card = document.createElement('div');
                card.className = 'dl-card';

                const badge = document.createElement('div');
                badge.className = 'dl-ext-badge' + (st === 'paused' ? ' paused' : (meta.cls === 'stopped' ? ' failed' : ''));
                badge.textContent = extOf(dl.name);
                card.appendChild(badge);

                const info = document.createElement('div');
                info.className = 'dl-info';

                const nameRow = document.createElement('div');
                nameRow.className = 'dl-name-row';
                const nameEl = document.createElement('div');
                nameEl.className = 'dl-name';
                nameEl.textContent = dl.name || 'download';
                nameEl.title = dl.name || '';
                const pill = document.createElement('span');
                pill.className = 'dl-status-pill ' + meta.cls;
                pill.textContent = meta.label;
                nameRow.appendChild(nameEl);
                nameRow.appendChild(pill);
                info.appendChild(nameRow);

                const metaRow = document.createElement('div');
                metaRow.className = 'dl-meta';
                const parts = [];
                const recv = Number(dl.bytes_received) || 0;
                const tot = Number(dl.total_bytes) || 0;
                const spd = Number(dl.speed_bps) || 0;
                if (st === 'downloading' || st === 'paused') {
                    if (tot > 0) {
                        const pct = Math.min(100, Math.round((recv / tot) * 100));
                        parts.push(fmtBytes(recv) + ' / ' + fmtBytes(tot) + ' (' + pct + '%)');
                    } else if (recv > 0) {
                        parts.push(fmtBytes(recv));
                    }
                    if (st === 'downloading' && spd > 0) {
                        parts.push(fmtBytes(spd) + '/s');
                    }
                } else if (tot > 0 || recv > 0) {
                    parts.push(fmtBytes(tot || recv));
                }
                const host = hostOf(dl.url);
                if (host) parts.push(host);
                const tm = fmtTime(dl.updated_at);
                if (tm) parts.push(tm);
                parts.forEach((p, i) => {
                    if (i > 0) {
                        const dot = document.createElement('span');
                        dot.textContent = '·';
                        metaRow.appendChild(dot);
                    }
                    const sp = document.createElement('span');
                    if (p === host) sp.className = 'dl-url';
                    sp.textContent = p;
                    metaRow.appendChild(sp);
                });
                info.appendChild(metaRow);

                if (st === 'downloading' || st === 'paused') {
                    const track = document.createElement('div');
                    track.className = 'dl-prog-track';
                    const fill = document.createElement('div');
                    fill.className = 'dl-prog-fill' + (st === 'paused' ? ' paused' : '');
                    const pct = tot > 0 ? Math.max(3, Math.min(100, Math.round((recv / tot) * 100))) : 28;
                    fill.style.width = pct + '%';
                    track.appendChild(fill);
                    info.appendChild(track);
                }

                card.appendChild(info);

                const acts = document.createElement('div');
                acts.className = 'dl-actions';
                const makeBtn = (label, cls, fn) => {
                    const b = document.createElement('button');
                    b.type = 'button';
                    b.className = 'dl-act-btn' + (cls ? ' ' + cls : '');
                    b.textContent = label;
                    b.onclick = fn;
                    return b;
                };

                if (st === 'downloading') {
                    acts.appendChild(makeBtn(zh ? '暂停' : 'Pause', '', () => {
                        sendIpc({ type: 'PAUSE_DOWNLOAD', id: Number(dl.id) || 0 });
                    }));
                    acts.appendChild(makeBtn(zh ? '取消' : 'Cancel', 'danger', () => {
                        sendIpc({ type: 'CANCEL_DOWNLOAD', id: Number(dl.id) || 0 });
                    }));
                } else if (st === 'paused') {
                    acts.appendChild(makeBtn(zh ? '继续' : 'Resume', 'accent', () => {
                        sendIpc({ type: 'RESUME_DOWNLOAD', id: Number(dl.id) || 0 });
                    }));
                    acts.appendChild(makeBtn(zh ? '取消' : 'Cancel', 'danger', () => {
                        sendIpc({ type: 'CANCEL_DOWNLOAD', id: Number(dl.id) || 0 });
                    }));
                } else if (st === 'completed') {
                    acts.appendChild(makeBtn(zh ? '打开文件' : 'Open', 'accent', () => {
                        sendIpc({ type: 'OPEN_DOWNLOAD_FILE', id: Number(dl.id) || 0, filename: dl.name });
                    }));
                    acts.appendChild(makeBtn(zh ? '定位文件夹' : 'Show in folder', '', () => {
                        sendIpc({ type: 'REVEAL_DOWNLOAD_FILE', id: Number(dl.id) || 0, filename: dl.name });
                    }));
                    acts.appendChild(makeBtn(zh ? '移除' : 'Remove', 'danger', () => {
                        sendIpc({ type: 'DELETE_DOWNLOAD_ENTRY', id: Number(dl.id) || 0, name: dl.name, updated_at: Number(dl.updated_at) || 0 });
                    }));
                } else {
                    if (dl.url && /^https?:\/\//i.test(dl.url)) {
                        acts.appendChild(makeBtn(zh ? '重新下载' : 'Retry', 'accent', () => {
                            sendIpc({ type: 'RETRY_DOWNLOAD', url: dl.url });
                        }));
                    }
                    acts.appendChild(makeBtn(zh ? '移除' : 'Remove', 'danger', () => {
                        sendIpc({ type: 'DELETE_DOWNLOAD_ENTRY', id: Number(dl.id) || 0, name: dl.name, updated_at: Number(dl.updated_at) || 0 });
                    }));
                }

                card.appendChild(acts);
                frag.appendChild(card);
            });
            listEl.appendChild(frag);
        }

        const searchEl = document.getElementById('dlSearchInput');
        const clearEl = document.getElementById('dlClearSearch');
        searchEl.oninput = () => {
            searchQuery = searchEl.value;
            clearEl.classList.toggle('visible', searchQuery.length > 0);
            renderList();
        };
        clearEl.onclick = () => {
            searchEl.value = '';
            searchQuery = '';
            clearEl.classList.remove('visible');
            renderList();
            searchEl.focus();
        };

        document.querySelectorAll('#dlFilterChips .dl-chip').forEach((chip) => {
            chip.onclick = () => {
                activeFilter = chip.dataset.filter || 'all';
                document.querySelectorAll('#dlFilterChips .dl-chip').forEach((c) => {
                    c.classList.toggle('active', c === chip);
                });
                renderList();
            };
        });

        document.getElementById('btnOpenFolder').onclick = () => sendIpc({ type: 'OPEN_DOWNLOADS_DIR' });
        document.getElementById('btnClearFinished').onclick = () => sendIpc({ type: 'CLEAR_DOWNLOADS_HISTORY' });
        const turboInp = document.getElementById('dlTurboInput');
        const turboBtn = document.getElementById('btnTurboDownload');
        const fireTurbo = () => {
            if (!turboInp) return;
            const u = turboInp.value.trim();
            if (!/^https?:\/\//i.test(u)) return;
            sendIpc({ type: 'START_TURBO_DOWNLOAD', url: u });
            turboInp.value = '';
        };
        if (turboBtn) turboBtn.onclick = fireTurbo;
        if (turboInp) {
            turboInp.onkeydown = (ev) => {
                if (ev.key === 'Enter') {
                    ev.preventDefault();
                    fireTurbo();
                }
            };
        }

        window.__fynexo_on_settings_runtime__ = (payload) => {
            if (!payload) return;
            if (payload.config) config = Object.assign(config, payload.config);
            if (payload.active_theme) activeTheme = Object.assign(activeTheme, payload.active_theme);
            if (Array.isArray(payload.downloads)) downloads = payload.downloads.slice();
            applyTheme();
            applyI18n();
            renderList();
        };

        window.__fynexo_on_downloads_update__ = (list) => {
            if (Array.isArray(list)) {
                downloads = list.slice();
                renderList();
            }
        };

        applyTheme();
        applyI18n();
        renderList();
    })();
