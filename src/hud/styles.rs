pub const BASE_STYLES_CSS: &str = r##"
                *, *::before, *::after {
                    box-sizing: border-box !important;
                    margin: 0;
                    padding: 0;
                    font-family: "Inter", system-ui, -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif;
                    -webkit-font-smoothing: antialiased;
                    font-variant-numeric: tabular-nums;
                    -webkit-tap-highlight-color: transparent;
                    box-shadow: none !important;
                    text-shadow: none !important;
                    backdrop-filter: none !important;
                }
                button, input {
                    font-family: inherit;
                    touch-action: manipulation;
                }
                #__fynexo_tabs_container__::-webkit-scrollbar { display: none; }
                #__fynexo_mobile_drawer__, #__fynexo_drawer_list__, #__fynexo_reader__ {
                    overscroll-behavior: contain;
                    -webkit-overflow-scrolling: touch;
                }

                #__fynexo_progress__ {
                    position: fixed;
                    top: 0;
                    left: 0;
                    height: 2px;
                    width: 0%;
                    background: var(--fnx-accent);
                    opacity: 0;
                    z-index: 2147483647;
                    pointer-events: none;
                    will-change: width, opacity;
                }

                #__fynexo_topbar__ {
                    position: fixed;
                    top: 0;
                    left: 0;
                    right: 0;
                    display: flex;
                    flex-direction: column;
                    z-index: 2147483646;
                    user-select: none;
                }

                #__fynexo_tab_row__ {
                    width: 100%;
                    display: flex;
                    align-items: flex-end;
                    justify-content: space-between;
                    position: relative;
                }

                #__fynexo_tabs_deck_cluster__ {
                    display: flex;
                    align-items: flex-end;
                    flex: 1;
                    min-width: 0;
                    height: 100%;
                }

                #__fynexo_nav_row__ {
                    width: 100%;
                    display: flex;
                    align-items: center;
                    justify-content: space-between;
                    gap: 6px;
                    position: relative;
                }

                #__fynexo_bm_row__ {
                    width: 100%;
                    display: flex;
                    align-items: center;
                    gap: 4px;
                }

                #__fynexo_bm_list__ {
                    display: flex;
                    align-items: center;
                    gap: 2px;
                    flex: 1;
                    min-width: 0;
                    overflow-x: auto;
                    scrollbar-width: none;
                }
                #__fynexo_bm_list__::-webkit-scrollbar { display: none; }

                .fnx-bm-pill {
                    height: 22px;
                    padding: 0 7px;
                    border-radius: 5px;
                    border: 1px solid transparent;
                    background: transparent;
                    color: var(--fnx-secondary);
                    font-size: 11.5px;
                    font-weight: 450;
                    display: inline-flex;
                    align-items: center;
                    gap: 5px;
                    cursor: pointer;
                    white-space: nowrap;
                    flex-shrink: 0;
                    max-width: 168px;
                    transition: background 0.1s ease, color 0.1s ease;
                }
                .fnx-bm-pill:hover {
                    background: var(--fnx-hover);
                    color: var(--fnx-fg);
                }
                .fnx-bm-icon {
                    width: 14px;
                    height: 14px;
                    border-radius: 3.5px;
                    background: rgba(128, 128, 128, 0.14);
                    font-size: 9px;
                    font-weight: 700;
                    display: inline-flex;
                    align-items: center;
                    justify-content: center;
                    flex-shrink: 0;
                }
                .fnx-bm-label {
                    overflow: hidden;
                    text-overflow: ellipsis;
                    white-space: nowrap;
                }

                .fnx-icon-btn {
                    background: transparent;
                    border: 1px solid transparent;
                    color: var(--fnx-secondary);
                    display: flex;
                    align-items: center;
                    justify-content: center;
                    cursor: pointer;
                    outline: none;
                    flex-shrink: 0;
                    transition: background 0.12s ease, color 0.12s ease, border-color 0.12s ease, transform 0.08s ease;
                }
                .fnx-icon-btn:hover {
                    background: var(--fnx-hover);
                    color: var(--fnx-fg);
                }
                .fnx-icon-btn:active {
                    transform: scale(0.94);
                }
                .fnx-icon-btn.active-route {
                    background: var(--fnx-accent-bg);
                    border-color: var(--fnx-accent);
                    color: var(--fnx-fg);
                }

                .fnx-studio-pill {
                    height: 26px;
                    padding: 0 9px;
                    border-radius: 8px;
                    background: var(--fnx-surface);
                    border: 1px solid var(--fnx-border);
                    color: var(--fnx-secondary);
                    font-size: 11.5px;
                    font-weight: 500;
                    display: inline-flex;
                    align-items: center;
                    gap: 5px;
                    cursor: pointer;
                    flex-shrink: 0;
                    transition: background 0.12s ease, border-color 0.12s ease, color 0.12s ease;
                }
                .fnx-studio-pill:hover {
                    background: var(--fnx-hover);
                    border-color: var(--fnx-border-hover);
                    color: var(--fnx-fg);
                }
                .fnx-studio-pill.active-route {
                    background: var(--fnx-accent-bg);
                    border-color: var(--fnx-accent);
                    color: var(--fnx-fg);
                }

                .fnx-incog-pill {
                    display: none;
                    align-items: center;
                    height: 24px;
                    padding: 0 8px;
                    margin-right: 2px;
                    border-radius: 6px;
                    background: var(--fnx-accent-bg);
                    border: 1px solid var(--fnx-accent);
                    color: var(--fnx-fg);
                    font-size: 11px;
                    font-weight: 600;
                    cursor: pointer;
                    flex-shrink: 0;
                }

                .fnx-tab-item {
                    position: relative;
                    display: flex;
                    align-items: center;
                    gap: 4px;
                    padding: 0 6px 6px 6px;
                    font-weight: 450;
                    cursor: pointer;
                    border: 1px solid transparent;
                    color: var(--fnx-secondary);
                    white-space: nowrap;
                    user-select: none;
                    transition: background 0.12s ease, color 0.12s ease, border-color 0.12s ease;
                }
                .fnx-tab-item.active {
                    font-weight: 500;
                    background: var(--fnx-tab-active);
                    border-color: var(--fnx-border);
                    color: var(--fnx-fg);
                }
                .fnx-tab-item:not(.active)::after {
                    content: "";
                    position: absolute;
                    right: 0;
                    top: 50%;
                    transform: translateY(-50%);
                    width: 1px;
                    height: 14px;
                    background: var(--fnx-border);
                    pointer-events: none;
                }
                .fnx-tab-item:not(.active):hover::after,
                .fnx-tab-item:has(+ .fnx-tab-item.active)::after,
                .fnx-tab-item:has(+ .fnx-tab-item:hover)::after {
                    opacity: 0;
                }
                .fnx-tab-item:not(.active):hover {
                    background: rgba(128, 128, 128, 0.12);
                    color: var(--fnx-fg);
                }
                .fnx-tab-monogram {
                    width: 20px;
                    height: 20px;
                    font-size: 20px;
                    font-weight: 700;
                    display: inline-flex;
                    align-items: center;
                    justify-content: center;
                    flex-shrink: 0;
                    line-height: 1;
                }
                .fnx-tab-title {
                    overflow: hidden;
                    text-overflow: ellipsis;
                    white-space: nowrap;
                    flex: 1;
                    pointer-events: none;
                    letter-spacing: -0.01em;
                }
                .fnx-tab-close {
                    width: 16px;
                    height: 16px;
                    border: none;
                    background: transparent;
                    border-radius: 4px;
                    cursor: pointer;
                    color: var(--fnx-secondary);
                    display: flex;
                    align-items: center;
                    justify-content: center;
                    flex-shrink: 0;
                    opacity: 0.7;
                    transition: background 0.1s ease, color 0.1s ease, opacity 0.1s ease;
                }
                .fnx-tab-close:hover {
                    opacity: 1 !important;
                    color: var(--fnx-fg);
                    background: var(--fnx-hover);
                }

                #__fynexo_addr_slot__ {
                    position: relative;
                    display: flex;
                    align-items: center;
                    margin: 0 2px;
                    min-width: 160px;
                    padding: 0 3px 0 0;
                    transition: border-color 0.12s ease, background 0.12s ease;
                }
                #__fynexo_addr_slot__:hover:not(.editing) {
                    border-color: var(--fnx-border-hover);
                }

                #__fynexo_address_btn__ {
                    background: transparent;
                    border: none;
                    color: var(--fnx-fg);
                    padding: 0 6px 0 10px;
                    cursor: text;
                    outline: none;
                    display: flex;
                    align-items: center;
                    gap: 7px;
                    flex: 1;
                    min-width: 0;
                }
                #__fynexo_cmd_label__ {
                    display: flex;
                    align-items: baseline;
                    overflow: hidden;
                    white-space: nowrap;
                    flex: 1;
                    min-width: 0;
                    text-align: left;
                    letter-spacing: -0.01em;
                }
                #__fynexo_addr_scheme__ {
                    color: var(--fnx-muted);
                    font-weight: 400;
                    flex-shrink: 0;
                }
                #__fynexo_addr_domain__ {
                    font-weight: 500;
                    color: var(--fnx-fg);
                    flex-shrink: 0;
                }
                #__fynexo_addr_path__ {
                    overflow: hidden;
                    text-overflow: ellipsis;
                    white-space: nowrap;
                    color: var(--fnx-muted);
                    font-weight: 400;
                    min-width: 0;
                }
                #__fynexo_addr_actions__ {
                    display: flex;
                    align-items: center;
                    gap: 2px;
                    flex-shrink: 0;
                    padding-right: 2px;
                }
                #__fynexo_addr_slot__.editing .fnx-addr-hint {
                    display: none;
                }
                .fnx-addr-action-btn {
                    width: 22px;
                    height: 22px;
                    border-radius: 5px;
                    border: 1px solid transparent;
                    background: transparent;
                    color: var(--fnx-muted);
                    display: flex;
                    align-items: center;
                    justify-content: center;
                    cursor: pointer;
                    flex-shrink: 0;
                    transition: background 0.1s ease, color 0.1s ease;
                }
                .fnx-addr-action-btn:hover {
                    background: var(--fnx-hover);
                    color: var(--fnx-fg);
                }
                .fnx-addr-action-btn.active-route {
                    color: var(--fnx-accent);
                    background: var(--fnx-accent-bg);
                }
                .fnx-addr-hint {
                    font-family: "JetBrains Mono", ui-monospace, Consolas, monospace;
                    font-size: 9.5px;
                    color: var(--fnx-muted);
                    background: rgba(128, 128, 128, 0.1);
                    border: 1px solid var(--fnx-border);
                    padding: 1px 5px;
                    border-radius: 4px;
                    margin-right: 2px;
                    flex-shrink: 0;
                }

                /* Active Inline <input> Mode (Mounted ONLY while focused/clicked) */
                #__fynexo_inline_box__ {
                    display: flex;
                    align-items: center;
                    gap: 6px;
                    padding: 0 4px 0 6px;
                    flex: 1;
                    min-width: 0;
                    background: transparent;
                    border: none;
                }
                #__fynexo_inline_engine__ {
                    height: 20px;
                    padding: 0 7px;
                    border-radius: 5px;
                    border: 1px solid var(--fnx-border-hover);
                    background: var(--fnx-surface);
                    color: var(--fnx-fg);
                    font-size: 11px;
                    font-weight: 500;
                    cursor: pointer;
                    white-space: nowrap;
                    flex-shrink: 0;
                }
                #__fynexo_inline_engine__:hover {
                    border-color: var(--fnx-accent);
                    background: var(--fnx-accent-bg);
                }
                #__fynexo_inline_input__ {
                    flex: 1;
                    min-width: 0;
                    height: 100%;
                    background: transparent;
                    border: none;
                    outline: none;
                    color: var(--fnx-fg);
                    font-size: inherit;
                    letter-spacing: -0.01em;
                    user-select: auto;
                }
                #__fynexo_inline_input__::placeholder {
                    color: var(--fnx-muted);
                }
                .fnx-inline-clear {
                    width: 20px;
                    height: 20px;
                    border: none;
                    background: transparent;
                    color: var(--fnx-secondary);
                    display: none;
                    align-items: center;
                    justify-content: center;
                    cursor: pointer;
                    flex-shrink: 0;
                    border-radius: 4px;
                }
                .fnx-inline-clear:hover {
                    background: var(--fnx-hover);
                    color: var(--fnx-fg);
                }
                #__fynexo_inline_dropdown__ {
                    position: fixed;
                    background: var(--fnx-panel);
                    border: 1px solid var(--fnx-border-hover);
                    border-radius: 10px;
                    overflow: hidden;
                    display: flex;
                    flex-direction: column;
                    z-index: 2147483647;
                }

                #__fynexo_tab_context_menu__ {
                    position: fixed !important;
                    z-index: 2147483647 !important;
                    min-width: 195px;
                    padding: 5px;
                    background: var(--fnx-panel);
                    border: 1px solid var(--fnx-border);
                    border-radius: 9px;
                    box-shadow: 0 14px 36px rgba(0, 0, 0, 0.72) !important;
                    font-size: 12.5px;
                    user-select: none;
                    display: none;
                    flex-direction: column;
                    gap: 2px;
                }

                #__fynexo_btn_bm_toggle__ {
                    width: 30px;
                    height: 30px;
                    margin: 5px;
                    border-radius: 8px;
                    align-self: flex-end;
                    display: flex;
                    align-items: center;
                    justify-content: center;
                    background: var(--fnx-surface);
                    color: var(--fnx-secondary);
                    flex-shrink: 0;
                    cursor: pointer;
                    transition: background 0.12s ease, color 0.12s ease, border-color 0.12s ease, transform 0.08s ease;
                }
                #__fynexo_btn_bm_toggle__:hover {
                    background: var(--fnx-hover);
                    border-color: var(--fnx-border-hover);
                    color: var(--fnx-fg);
                }
                #__fynexo_btn_bm_toggle__:active {
                    transform: scale(0.94);
                }

                .fnx-nav-popover {
                    position: fixed !important;
                    z-index: 2147483647 !important;
                    background: var(--fnx-panel);
                    border: 1px solid var(--fnx-border);
                    border-radius: 12px;
                    box-shadow: 0 16px 42px rgba(0, 0, 0, 0.72) !important;
                    font-size: 13px;
                    user-select: none;
                    display: none;
                    flex-direction: column;
                    padding: 8px;
                    color: var(--fnx-fg);
                    animation: fnxPopoverIn 0.12s cubic-bezier(0.16, 1, 0.3, 1);
                }
                @keyframes fnxPopoverIn {
                    from { opacity: 0; transform: translateY(-4px) scale(0.98); }
                    to { opacity: 1; transform: translateY(0) scale(1); }
                }
                .fnx-menu-item {
                    display: flex;
                    align-items: center;
                    justify-content: space-between;
                    padding: 7px 10px;
                    border-radius: 8px;
                    color: var(--fnx-fg);
                    cursor: pointer;
                    transition: background 0.08s ease;
                    gap: 12px;
                    font-size: 12.5px;
                }
                .fnx-menu-item:hover {
                    background: var(--fnx-hover);
                }
                .fnx-menu-sep {
                    height: 1px;
                    background: var(--fnx-border);
                    margin: 4px 6px;
                }
                .fnx-btn-badge {
                    position: absolute;
                    top: 1px;
                    right: 1px;
                    min-width: 14px;
                    height: 14px;
                    padding: 0 3px;
                    border-radius: 8px;
                    background: var(--fnx-accent);
                    color: #fff;
                    font-size: 9px;
                    font-weight: 700;
                    display: flex;
                    align-items: center;
                    justify-content: center;
                    line-height: 1;
                    box-shadow: 0 1px 3px rgba(0, 0, 0, 0.5);
                    pointer-events: none;
                }
                .fnx-pulse-dot {
                    width: 6px;
                    height: 6px;
                    border-radius: 50%;
                    background: #34d399;
                    box-shadow: 0 0 6px #34d399;
                    display: inline-block;
                    pointer-events: none;
                }
                .fnx-chip {
                    font-size: 11.5px;
                    padding: 3px 8px;
                    border-radius: 6px;
                    background: var(--fnx-surface);
                    border: 1px solid var(--fnx-border);
                    color: var(--fnx-secondary);
                    cursor: pointer;
                    display: inline-flex;
                    align-items: center;
                    gap: 4px;
                    transition: all 0.1s ease;
                }
                .fnx-chip:hover {
                    background: var(--fnx-hover);
                    color: var(--fnx-fg);
                    border-color: var(--fnx-border-hover);
                }
                .fnx-chip.active {
                    background: var(--fnx-accent-bg);
                    color: var(--fnx-fg);
                    border-color: var(--fnx-accent);
                }

                #__fynexo_win_controls__ {
                    display: flex;
                    align-items: center;
                    height: 40px;
                    flex-shrink: 0;
                }
                .fnx-win-btn {
                    background: transparent;
                    border: none;
                    color: var(--fnx-secondary);
                    width: 38px;
                    height: 100%;
                    display: flex;
                    align-items: center;
                    justify-content: center;
                    cursor: pointer;
                    outline: none;
                    transition: background 0.1s ease, color 0.1s ease;
                }
                .fnx-win-btn:hover {
                    background: var(--fnx-hover);
                    color: var(--fnx-fg);
                }
                .fnx-win-btn.close:hover {
                    background: #e11d48;
                    color: #ffffff;
                }

                /* Find in Page Floating Bar */
                #__fynexo_find_bar__ {
                    position: fixed;
                    top: calc(var(--fnx-topbar-live, 38px) + 8px);
                    right: 14px;
                    height: 36px;
                    padding: 0 6px 0 10px;
                    background: var(--fnx-panel);
                    border: 1px solid var(--fnx-border-hover);
                    border-radius: 9px;
                    display: none;
                    align-items: center;
                    gap: 6px;
                    z-index: 2147483647;
                }
                #__fynexo_find_input__ {
                    width: 175px;
                    height: 28px;
                    background: transparent;
                    border: none;
                    outline: none;
                    color: var(--fnx-fg);
                    font-size: 12.5px;
                }
                #__fynexo_find_count__ {
                    font-family: "JetBrains Mono", ui-monospace, Consolas, monospace;
                    font-size: 11px;
                    color: var(--fnx-muted);
                    padding: 0 4px;
                    white-space: nowrap;
                }
                .fnx-find-btn {
                    width: 26px;
                    height: 26px;
                    border: 1px solid transparent;
                    border-radius: 6px;
                    background: transparent;
                    color: var(--fnx-secondary);
                    display: flex;
                    align-items: center;
                    justify-content: center;
                    cursor: pointer;
                }
                .fnx-find-btn:hover {
                    background: var(--fnx-hover);
                    color: var(--fnx-fg);
                }

                /* Pure-DOM Reader Mode Overlay */
                #__fynexo_reader__ {
                    position: fixed;
                    top: var(--fnx-topbar-live, 38px);
                    left: 0;
                    right: 0;
                    bottom: 0;
                    background: var(--fnx-bg);
                    color: var(--fnx-fg);
                    display: none;
                    flex-direction: column;
                    align-items: center;
                    overflow-y: auto;
                    z-index: 2147483645;
                }
                .fnx-rd-toolbar {
                    position: sticky;
                    top: 0;
                    width: 100%;
                    background: var(--fnx-panel);
                    border-bottom: 1px solid var(--fnx-border);
                    padding: 9px 18px;
                    display: flex;
                    align-items: center;
                    justify-content: space-between;
                    gap: 12px;
                    z-index: 2;
                    flex-shrink: 0;
                }
                .fnx-rd-meta {
                    font-size: 12px;
                    font-weight: 500;
                    color: var(--fnx-secondary);
                    overflow: hidden;
                    text-overflow: ellipsis;
                    white-space: nowrap;
                }
                .fnx-rd-controls {
                    display: flex;
                    align-items: center;
                    gap: 6px;
                    flex-shrink: 0;
                }
                .fnx-rd-btn {
                    height: 28px;
                    padding: 0 10px;
                    border-radius: 6px;
                    border: 1px solid var(--fnx-border);
                    background: var(--fnx-surface);
                    color: var(--fnx-fg);
                    font-size: 12px;
                    font-weight: 500;
                    cursor: pointer;
                }
                .fnx-rd-btn:hover {
                    border-color: var(--fnx-border-hover);
                    background: var(--fnx-elevated);
                }
                #__fynexo_reader_article__ {
                    width: 100%;
                    max-width: 680px;
                    padding: 36px 24px 96px;
                    line-height: 1.78;
                    color: var(--fnx-fg);
                    user-select: text;
                }
                .fnx-rd-h1 { font-size: 1.65em; font-weight: 700; line-height: 1.28; margin: 0 0 0.75em; letter-spacing: -0.025em; }
                .fnx-rd-h2 { font-size: 1.32em; font-weight: 650; line-height: 1.35; margin: 1.5em 0 0.55em; letter-spacing: -0.015em; }
                .fnx-rd-h3, .fnx-rd-h4 { font-size: 1.12em; font-weight: 600; margin: 1.25em 0 0.45em; }
                .fnx-rd-p { margin: 0 0 1.08em; color: var(--fnx-fg); }
                .fnx-rd-blockquote {
                    margin: 1.1em 0;
                    padding: 8px 16px;
                    border-left: 3px solid var(--fnx-accent);
                    background: var(--fnx-panel);
                    color: var(--fnx-secondary);
                    border-radius: 0 6px 6px 0;
                }
                .fnx-rd-pre {
                    margin: 1.1em 0;
                    padding: 13px 15px;
                    background: var(--fnx-panel);
                    border: 1px solid var(--fnx-border);
                    border-radius: 8px;
                    overflow-x: auto;
                    font-family: "JetBrains Mono", ui-monospace, Consolas, monospace;
                    font-size: 0.84em;
                    line-height: 1.55;
                }
                .fnx-rd-list {
                    margin: 0 0 1.1em 1.35em;
                }
                .fnx-rd-list li {
                    margin-bottom: 0.4em;
                }
                .fnx-rd-img {
                    display: block;
                    max-width: 100%;
                    height: auto;
                    margin: 1.2em auto;
                    border-radius: 8px;
                    border: 1px solid var(--fnx-border);
                }

                /* Studio Mobile Bottom Dock */
                #__fynexo_bottom_bar__ {
                    position: fixed;
                    left: 0;
                    right: 0;
                    display: flex;
                    align-items: center;
                    gap: 6px;
                    z-index: 2147483646;
                    user-select: none;
                }
                .fnx-mb-btn {
                    border: 1px solid transparent;
                    background: transparent;
                    color: var(--fnx-secondary);
                    border-radius: 10px;
                    display: flex;
                    align-items: center;
                    justify-content: center;
                    cursor: pointer;
                    outline: none;
                    flex-shrink: 0;
                    transition: transform 0.08s ease, background 0.12s ease;
                }
                .fnx-mb-btn:active {
                    transform: scale(0.92);
                    background: var(--fnx-hover);
                    color: var(--fnx-fg);
                }
                .fnx-mb-btn.active-route {
                    background: var(--fnx-accent-bg);
                    border-color: var(--fnx-accent);
                    color: var(--fnx-fg);
                }
                #__fynexo_mb_pill__ {
                    flex: 1;
                    min-width: 0;
                    background: var(--fnx-surface);
                    border: 1px solid var(--fnx-border);
                    display: flex;
                    align-items: center;
                    padding: 0 4px 0 11px;
                    gap: 6px;
                }
                #__fynexo_mb_pill_trigger__ {
                    flex: 1;
                    min-width: 0;
                    height: 100%;
                    border: none;
                    background: transparent;
                    color: var(--fnx-fg);
                    display: flex;
                    align-items: center;
                    gap: 7px;
                    cursor: pointer;
                    outline: none;
                    text-align: left;
                }
                #__fynexo_mb_label__ {
                    font-size: 13px;
                    font-weight: 450;
                    letter-spacing: -0.01em;
                    overflow: hidden;
                    text-overflow: ellipsis;
                    white-space: nowrap;
                    flex: 1;
                }
                #__fynexo_mb_reload__ {
                    width: 30px;
                    height: 30px;
                    border: none;
                    background: transparent;
                    color: var(--fnx-muted);
                    border-radius: 8px;
                    display: flex;
                    align-items: center;
                    justify-content: center;
                    cursor: pointer;
                    flex-shrink: 0;
                }
                #__fynexo_mb_reload__:active {
                    transform: scale(0.9);
                    background: var(--fnx-hover);
                    color: var(--fnx-fg);
                }
                .fnx-tab-badge {
                    border: 1.5px solid var(--fnx-secondary);
                    border-radius: 6px;
                    min-width: 21px;
                    height: 21px;
                    padding: 0 4px;
                    display: flex;
                    align-items: center;
                    justify-content: center;
                    font-size: 11px;
                    font-weight: 600;
                    color: var(--fnx-fg);
                }

                /* Ergonomic Thumb-First Mobile Tab Sheet */
                #__fynexo_mobile_drawer__ {
                    position: fixed;
                    inset: 0;
                    background: rgba(0, 0, 0, 0.66);
                    z-index: 2147483647;
                    display: none;
                    flex-direction: column;
                    justify-content: flex-end;
                }
                #__fynexo_drawer_sheet__ {
                    background: var(--fnx-panel);
                    border-top: 1px solid var(--fnx-border-hover);
                    display: flex;
                    flex-direction: column;
                    padding: 10px 14px calc(12px + max(env(safe-area-inset-bottom, 0px), 12px));
                }
                .fnx-sheet-handle {
                    width: 36px;
                    height: 4px;
                    border-radius: 2px;
                    background: var(--fnx-border-hover);
                    margin: 0 auto 10px;
                    flex-shrink: 0;
                }
                .fnx-drawer-card {
                    display: flex;
                    align-items: center;
                    gap: 9px;
                    margin-bottom: 8px;
                    background: var(--fnx-surface);
                    border: 1px solid var(--fnx-border);
                    cursor: pointer;
                    user-select: none;
                }
                .fnx-drawer-card.active {
                    background: var(--fnx-accent-bg);
                    border-color: var(--fnx-accent);
                }
                .fnx-drawer-monogram {
                    width: 32px;
                    height: 32px;
                    border-radius: 8px;
                    background: var(--fnx-bg);
                    border: 1px solid var(--fnx-border);
                    color: var(--fnx-secondary);
                    font-size: 13px;
                    font-weight: 600;
                    display: flex;
                    align-items: center;
                    justify-content: center;
                    flex-shrink: 0;
                }
                .fnx-drawer-card.active .fnx-drawer-monogram {
                    border-color: var(--fnx-accent);
                    color: var(--fnx-fg);
                }
                .fnx-drawer-meta {
                    overflow: hidden;
                    flex: 1;
                    min-width: 0;
                }
                .fnx-drawer-title {
                    font-size: 13.5px;
                    font-weight: 500;
                    color: var(--fnx-fg);
                    overflow: hidden;
                    text-overflow: ellipsis;
                    white-space: nowrap;
                }
                .fnx-drawer-url {
                    font-size: 11.5px;
                    color: var(--fnx-muted);
                    overflow: hidden;
                    text-overflow: ellipsis;
                    white-space: nowrap;
                    margin-top: 1px;
                }
                .fnx-drawer-close {
                    width: 30px;
                    height: 30px;
                    border: 1px solid var(--fnx-border);
                    background: var(--fnx-panel);
                    color: var(--fnx-muted);
                    border-radius: 8px;
                    display: flex;
                    align-items: center;
                    justify-content: center;
                    cursor: pointer;
                    flex-shrink: 0;
                }
                .fnx-sheet-bottom-bar {
                    display: flex;
                    align-items: center;
                    gap: 8px;
                    padding-top: 12px;
                    margin-top: 4px;
                    border-top: 1px solid var(--fnx-border);
                }

                /* Spotlight Omnibox Modal */
                #__fynexo_hud_root__ {
                    position: fixed;
                    inset: 0;
                    display: none;
                    flex-direction: column;
                    align-items: center;
                    z-index: 2147483647;
                }
                #__fynexo_hud_card__ {
                    overflow: hidden;
                    display: flex;
                    flex-direction: column;
                }
                #__fynexo_hud_suggest__,
                #__fynexo_inline_suggest__ {
                    display: none;
                    flex-direction: column;
                    background: var(--fnx-panel);
                    max-height: 272px;
                    overflow-y: auto;
                }
                #__fynexo_hud_suggest__ {
                    border-top: 1px solid var(--fnx-border);
                }
                #__fynexo_inline_suggest__ {
                    border-bottom: 1px solid var(--fnx-border);
                }
                .fnx-suggest-row {
                    width: 100%;
                    border: none;
                    border-bottom: 1px solid var(--fnx-border);
                    background: transparent;
                    color: var(--fnx-fg);
                    padding: 8px 11px;
                    display: flex;
                    align-items: center;
                    gap: 9px;
                    cursor: pointer;
                    text-align: left;
                    outline: none;
                    transition: background 0.1s ease;
                }
                .fnx-suggest-row:last-child {
                    border-bottom: none;
                }
                .fnx-suggest-row:hover,
                .fnx-suggest-row.selected {
                    background: var(--fnx-hover);
                }
                .fnx-suggest-icon {
                    width: 24px;
                    height: 24px;
                    border-radius: 6px;
                    background: var(--fnx-surface);
                    border: 1px solid var(--fnx-border);
                    color: var(--fnx-muted);
                    display: flex;
                    align-items: center;
                    justify-content: center;
                    flex-shrink: 0;
                }
                .fnx-suggest-main {
                    display: flex;
                    flex-direction: column;
                    min-width: 0;
                    flex: 1;
                }
                .fnx-suggest-title {
                    font-size: 12.5px;
                    font-weight: 500;
                    color: var(--fnx-fg);
                    overflow: hidden;
                    text-overflow: ellipsis;
                    white-space: nowrap;
                }
                .fnx-suggest-url {
                    font-size: 11px;
                    color: var(--fnx-muted);
                    overflow: hidden;
                    text-overflow: ellipsis;
                    white-space: nowrap;
                }
                .fnx-suggest-badge {
                    font-size: 10px;
                    font-weight: 600;
                    color: var(--fnx-secondary);
                    background: var(--fnx-surface);
                    border: 1px solid var(--fnx-border);
                    border-radius: 5px;
                    padding: 2px 6px;
                    flex-shrink: 0;
                }
                #__fynexo_hud_quick__,
                #__fynexo_inline_quick__ {
                    display: flex;
                    align-items: center;
                    justify-content: space-between;
                    gap: 8px;
                    padding: 7px 11px;
                    background: var(--fnx-bg);
                }
                #__fynexo_hud_quick__ {
                    border-top: 1px solid var(--fnx-border);
                }
                .fnx-quick-tabs, .fnx-quick-actions {
                    display: flex;
                    align-items: center;
                    gap: 5px;
                    overflow-x: auto;
                    scrollbar-width: none;
                }
                .fnx-quick-tabs::-webkit-scrollbar, .fnx-quick-actions::-webkit-scrollbar { display: none; }
                .fnx-quick-chip {
                    padding: 3px 8px;
                    font-size: 11px;
                    font-weight: 500;
                    color: var(--fnx-secondary);
                    background: var(--fnx-surface);
                    border: 1px solid var(--fnx-border);
                    border-radius: 6px;
                    cursor: pointer;
                    white-space: nowrap;
                    flex-shrink: 0;
                }
                .fnx-quick-chip:hover {
                    border-color: var(--fnx-border-hover);
                    color: var(--fnx-fg);
                }
                .fnx-quick-chip.active {
                    border-color: var(--fnx-accent);
                    background: var(--fnx-accent-bg);
                    color: var(--fnx-fg);
                }
                #__fynexo_hud_toast__ {
                    position: fixed;
                    bottom: 22px;
                    right: 18px;
                    background: var(--fnx-elevated);
                    border: 1px solid var(--fnx-accent);
                    color: var(--fnx-fg);
                    font-size: 12px;
                    font-weight: 500;
                    padding: 7px 12px;
                    border-radius: 8px;
                    display: none;
                    align-items: center;
                    gap: 7px;
                    z-index: 2147483647;
                    pointer-events: none;
                }
                @media (max-width: 768px) {
                    #__fynexo_find_bar__ {
                        top: 10px;
                        left: 10px;
                        right: 10px;
                    }
                    #__fynexo_find_input__ {
                        flex: 1;
                        width: auto;
                    }
                    #__fynexo_reader__ {
                        top: 0;
                        padding-bottom: calc(56px + max(env(safe-area-inset-bottom, 0px), 12px));
                    }
                    #__fynexo_hud_toast__ {
                        left: 14px;
                        right: 14px;
                        bottom: calc(64px + max(env(safe-area-inset-bottom, 0px), 12px));
                        justify-content: center;
                    }
                }
"##;