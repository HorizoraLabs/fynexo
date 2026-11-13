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
                    align-items: center;
                    justify-content: space-between;
                    position: relative;
                }

                #__fynexo_tabs_deck_cluster__ {
                    display: flex;
                    align-items: center;
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
                    width: 15px;
                    height: 15px;
                    border-radius: 4px;
                    background: transparent;
                    font-size: 11px;
                    font-weight: 700;
                    display: inline-flex;
                    align-items: center;
                    justify-content: center;
                    flex-shrink: 0;
                    line-height: 1;
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
                .fnx-icon-btn:disabled {
                    opacity: 0.34;
                    cursor: default;
                }
                .fnx-icon-btn:disabled:hover,
                .fnx-icon-btn:disabled:active {
                    background: transparent;
                    color: var(--fnx-secondary);
                    transform: none;
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
                    gap: 7px;
                    padding: 0 9px 0 11px;
                    font-weight: 450;
                    cursor: pointer;
                    border: 1px solid transparent;
                    color: color-mix(in srgb, var(--fnx-fg) 68%, var(--fnx-bg));
                    white-space: nowrap;
                    user-select: none;
                    transition: background 0.14s cubic-bezier(0.2, 0.8, 0.2, 1), color 0.14s ease, border-color 0.14s ease;
                }
                .fnx-tab-item.active {
                    font-weight: 550;
                    background: var(--fnx-elevated);
                    border-color: var(--fnx-border);
                    border-top-color: var(--fnx-border-hover);
                    color: var(--fnx-fg);
                }
                .fnx-tab-item:not(.active):hover {
                    background: rgba(128, 128, 128, 0.14);
                    color: var(--fnx-fg);
                }
                .fnx-tab-monogram {
                    width: 16px;
                    height: 16px;
                    font-size: 12px;
                    font-weight: 700;
                    display: inline-flex;
                    align-items: center;
                    justify-content: center;
                    flex-shrink: 0;
                    line-height: 1;
                    background: transparent;
                    border: none;
                    letter-spacing: -0.02em;
                    transition: opacity 0.14s ease, filter 0.14s ease;
                }
                .fnx-tab-item:not(.active) .fnx-tab-monogram {
                    opacity: 0.65;
                    filter: grayscale(35%);
                }
                .fnx-tab-item:not(.active):hover .fnx-tab-monogram {
                    opacity: 1;
                    filter: grayscale(0%);
                }
                .fnx-tab-title {
                    overflow: hidden;
                    text-overflow: ellipsis;
                    white-space: nowrap;
                    flex: 1;
                    pointer-events: none;
                    letter-spacing: -0.012em;
                    font-size: 12.5px;
                }
                .fnx-tab-close {
                    width: 16px;
                    height: 16px;
                    border: none;
                    background: transparent;
                    border-radius: 5px;
                    cursor: pointer;
                    color: var(--fnx-secondary);
                    display: flex;
                    align-items: center;
                    justify-content: center;
                    flex-shrink: 0;
                    opacity: 0.62;
                    transition: width 0.14s ease, opacity 0.12s ease, margin-left 0.14s ease, background 0.1s ease, color 0.1s ease;
                }
                .fnx-tab-close:hover {
                    opacity: 1 !important;
                    color: var(--fnx-fg);
                    background: rgba(128, 128, 128, 0.24);
                }
                .fnx-tab-item.suspended:not(.active) .fnx-tab-title {
                    opacity: 0.76;
                }
                .fnx-tab-item.suspended:not(.active):hover .fnx-tab-title {
                    opacity: 1;
                }
                .fnx-tab-item.dragging {
                    opacity: 0.42;
                }
                .fnx-tab-item.drag-over {
                    border-left: 2px solid var(--fnx-accent) !important;
                }
                #__fynexo_tabs_container__.fnx-tabs-crowded .fnx-tab-item:not(.active) .fnx-tab-close {
                    display: none;
                }
                #__fynexo_tabs_container__.fnx-tabs-crowded .fnx-tab-item:not(.active):hover .fnx-tab-close {
                    display: flex;
                }
                #__fynexo_bg_media_pill__ {
                    display: none;
                    align-items: center;
                    gap: 5px;
                    height: 24px;
                    max-width: 170px;
                    padding: 0 7px 0 8px;
                    border-radius: 999px;
                    border: 1px solid rgba(52, 211, 153, 0.32);
                    background: rgba(16, 185, 129, 0.12);
                    color: var(--fnx-fg);
                    font-size: 11px;
                    font-weight: 500;
                    cursor: pointer;
                    flex-shrink: 0;
                    transition: background 0.12s ease, border-color 0.12s ease;
                }
                #__fynexo_bg_media_pill__:hover {
                    background: rgba(16, 185, 129, 0.22);
                    border-color: rgba(52, 211, 153, 0.55);
                }
                #__fynexo_bg_media_label__ {
                    overflow: hidden;
                    text-overflow: ellipsis;
                    white-space: nowrap;
                }

                #__fynexo_addr_slot__ {
                    position: relative;
                    display: flex;
                    align-items: center;
                    margin: 0;
                    min-width: 180px;
                    padding: 0 5px 0 0;
                    transition: max-width 0.18s cubic-bezier(0.2, 0.8, 0.2, 1), border-color 0.15s ease, background 0.15s ease;
                }
                #__fynexo_addr_slot__:hover:not(.editing) {
                    border-color: var(--fnx-border-hover) !important;
                    background: var(--fnx-hover) !important;
                }
                #__fynexo_addr_slot__.editing {
                    border-color: var(--fnx-accent) !important;
                    background: var(--fnx-bg) !important;
                }

                #__fynexo_address_btn__ {
                    background: transparent;
                    border: none;
                    color: var(--fnx-fg);
                    padding: 0 8px 0 10px;
                    cursor: text;
                    outline: none;
                    display: flex;
                    align-items: center;
                    gap: 8px;
                    flex: 1;
                    min-width: 0;
                }
                #__fynexo_desk_proto__ {
                    display: flex;
                    align-items: center;
                    flex-shrink: 0;
                    opacity: 0.55;
                    transition: opacity 0.12s ease;
                }
                #__fynexo_addr_slot__:hover #__fynexo_desk_proto__ {
                    opacity: 0.85;
                }
                #__fynexo_cmd_label__ {
                    display: flex;
                    align-items: baseline;
                    overflow: hidden;
                    white-space: nowrap;
                    flex: 1;
                    min-width: 0;
                    text-align: left;
                    letter-spacing: -0.015em;
                }
                #__fynexo_addr_scheme__ {
                    color: var(--fnx-muted);
                    font-weight: 400;
                    flex-shrink: 0;
                    opacity: 0.65;
                }
                #__fynexo_addr_domain__ {
                    font-weight: 550;
                    color: var(--fnx-fg);
                    flex-shrink: 0;
                    letter-spacing: -0.01em;
                }
                #__fynexo_addr_path__ {
                    overflow: hidden;
                    text-overflow: ellipsis;
                    white-space: nowrap;
                    color: var(--fnx-muted);
                    font-weight: 400;
                    min-width: 0;
                    opacity: 0.65;
                }
                #__fynexo_addr_actions__ {
                    display: flex;
                    align-items: center;
                    gap: 4px;
                    flex-shrink: 0;
                    padding-right: 4px;
                }
                #__fynexo_addr_slot__.editing .fnx-addr-hint {
                    display: none !important;
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
                    opacity: 0.45;
                    transition: background 0.1s ease, color 0.1s ease, opacity 0.14s ease;
                }
                #__fynexo_addr_slot__:hover .fnx-addr-action-btn {
                    opacity: 0.85;
                }
                .fnx-addr-action-btn:hover {
                    background: rgba(128, 128, 128, 0.16);
                    color: var(--fnx-fg);
                    opacity: 1 !important;
                }
                .fnx-addr-action-btn.active-route {
                    color: var(--fnx-accent);
                    background: transparent;
                    opacity: 1 !important;
                }
                .fnx-addr-action-btn.active-route:hover {
                    background: rgba(128, 128, 128, 0.16);
                }
                .fnx-addr-hint {
                    display: none !important;
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
                    min-width: 218px;
                    max-width: calc(100vw - 16px);
                    max-height: calc(100dvh - 24px);
                    overflow-y: auto;
                    padding: 4px;
                    background: var(--fnx-panel);
                    border: 1px solid var(--fnx-border-hover);
                    border-radius: 10px;
                    font-size: 12.5px;
                    user-select: none;
                    display: none;
                    flex-direction: column;
                    gap: 1px;
                    animation: fnxPopoverIn 0.1s cubic-bezier(0.16, 1, 0.3, 1);
                }

                .fnx-nav-popover {
                    position: fixed !important;
                    z-index: 2147483647 !important;
                    max-width: calc(100vw - 16px);
                    background: var(--fnx-panel);
                    border: 1px solid var(--fnx-border-hover);
                    border-radius: 10px;
                    font-size: 12.5px;
                    user-select: none;
                    display: none;
                    flex-direction: column;
                    padding: 4px;
                    color: var(--fnx-fg);
                    animation: fnxPopoverIn 0.1s cubic-bezier(0.16, 1, 0.3, 1);
                }
                .fnx-nav-popover--up {
                    animation: fnxPopoverUpIn 0.12s cubic-bezier(0.16, 1, 0.3, 1);
                }
                .fnx-nav-popover--panel {
                    padding: 0;
                    overflow-x: hidden;
                    overflow-y: auto;
                }
                @keyframes fnxPopoverIn {
                    from { opacity: 0; transform: translateY(-3px) scale(0.985); }
                    to { opacity: 1; transform: translateY(0) scale(1); }
                }
                @keyframes fnxPopoverUpIn {
                    from { opacity: 0; transform: translateY(4px) scale(0.985); }
                    to { opacity: 1; transform: translateY(0) scale(1); }
                }
                .fnx-menu-item {
                    display: flex;
                    align-items: center;
                    justify-content: space-between;
                    padding: 5px 8px;
                    min-height: 28px;
                    border-radius: 6px;
                    color: var(--fnx-fg);
                    cursor: pointer;
                    transition: background 0.08s ease;
                    gap: 14px;
                    font-size: 12.5px;
                    line-height: 1.25;
                }
                .fnx-menu-item:hover {
                    background: var(--fnx-hover);
                }
                .fnx-menu-icon {
                    width: 14px;
                    height: 14px;
                    display: inline-flex;
                    align-items: center;
                    justify-content: center;
                    flex-shrink: 0;
                    color: var(--fnx-secondary);
                    transition: color 0.08s ease;
                }
                .fnx-menu-icon svg {
                    width: 14px;
                    height: 14px;
                    display: block;
                }
                .fnx-menu-item:hover .fnx-menu-icon {
                    color: var(--fnx-fg);
                }
                .fnx-menu-item.danger {
                    color: #f87171;
                }
                .fnx-menu-item.danger .fnx-menu-icon {
                    color: #f87171;
                }
                .fnx-menu-item.danger:hover {
                    background: rgba(239, 68, 68, 0.14);
                }
                .fnx-menu-item.disabled {
                    opacity: 0.38;
                    pointer-events: none;
                }
                .fnx-menu-label {
                    overflow: hidden;
                    text-overflow: ellipsis;
                    white-space: nowrap;
                    max-width: 196px;
                }
                .fnx-menu-kbd {
                    font-size: 10.5px;
                    color: var(--fnx-muted);
                    letter-spacing: 0.01em;
                    font-weight: 500;
                    flex-shrink: 0;
                }
                .fnx-menu-sep {
                    height: 1px;
                    background: var(--fnx-border);
                    margin: 4px -4px;
                }
                .fnx-pop-head {
                    display: flex;
                    align-items: center;
                    justify-content: space-between;
                    gap: 10px;
                    padding: 10px 12px;
                    border-bottom: 1px solid var(--fnx-border);
                }
                .fnx-pop-title {
                    display: inline-flex;
                    align-items: center;
                    gap: 7px;
                    font-size: 12.5px;
                    font-weight: 600;
                    letter-spacing: -0.01em;
                    color: var(--fnx-fg);
                    min-width: 0;
                }
                .fnx-pop-title svg {
                    width: 14px;
                    height: 14px;
                    color: var(--fnx-secondary);
                    flex-shrink: 0;
                }
                .fnx-pop-status {
                    display: inline-flex;
                    align-items: center;
                    gap: 5px;
                    font-size: 11px;
                    font-weight: 500;
                    color: var(--fnx-secondary);
                    flex-shrink: 0;
                }
                .fnx-pop-dot {
                    width: 5px;
                    height: 5px;
                    border-radius: 50%;
                    background: #10B981;
                    flex-shrink: 0;
                }
                .fnx-pop-dot.off {
                    background: var(--fnx-muted);
                }
                .fnx-pop-body {
                    display: flex;
                    flex-direction: column;
                    gap: 10px;
                    padding: 11px 12px;
                }
                .fnx-pop-section {
                    padding: 10px 12px;
                    border-top: 1px solid var(--fnx-border);
                    display: flex;
                    flex-direction: column;
                    gap: 8px;
                }
                .fnx-pop-row {
                    display: flex;
                    align-items: center;
                    justify-content: space-between;
                    gap: 12px;
                    min-height: 24px;
                }
                .fnx-pop-label {
                    font-size: 12px;
                    font-weight: 500;
                    color: var(--fnx-fg);
                    min-width: 0;
                }
                .fnx-pop-sub {
                    font-size: 11px;
                    color: var(--fnx-muted);
                    white-space: nowrap;
                    overflow: hidden;
                    text-overflow: ellipsis;
                }
                .fnx-pop-footer {
                    padding: 8px 12px;
                    border-top: 1px solid var(--fnx-border);
                    display: flex;
                    align-items: center;
                    gap: 6px;
                }
                .fnx-pop-switch {
                    position: relative;
                    width: 30px;
                    height: 17px;
                    border-radius: 999px;
                    border: 1px solid var(--fnx-border-hover);
                    background: var(--fnx-surface);
                    cursor: pointer;
                    flex-shrink: 0;
                    padding: 0;
                    transition: background 0.14s ease, border-color 0.14s ease;
                }
                .fnx-pop-switch::after {
                    content: '';
                    position: absolute;
                    top: 2px;
                    left: 2px;
                    width: 11px;
                    height: 11px;
                    border-radius: 50%;
                    background: var(--fnx-secondary);
                    transition: transform 0.14s ease, background 0.14s ease;
                }
                .fnx-pop-switch.on {
                    background: var(--fnx-accent);
                    border-color: var(--fnx-accent);
                }
                .fnx-pop-switch.on::after {
                    transform: translateX(13px);
                    background: #ffffff;
                }
                .fnx-pop-switch:disabled {
                    opacity: 0.36;
                    cursor: default;
                }
                .fnx-pop-seg {
                    display: grid;
                    gap: 2px;
                    padding: 2px;
                    border-radius: 7px;
                    background: var(--fnx-surface);
                    border: 1px solid var(--fnx-border);
                }
                .fnx-pop-seg-btn {
                    height: 24px;
                    padding: 0 6px;
                    border-radius: 5px;
                    border: 1px solid transparent;
                    background: transparent;
                    color: var(--fnx-secondary);
                    font-size: 11px;
                    font-weight: 500;
                    cursor: pointer;
                    transition: background 0.1s ease, color 0.1s ease, border-color 0.1s ease;
                }
                .fnx-pop-seg-btn:hover {
                    color: var(--fnx-fg);
                    background: var(--fnx-hover);
                }
                .fnx-pop-seg-btn.active {
                    background: var(--fnx-hover);
                    border-color: var(--fnx-border-hover);
                    color: var(--fnx-fg);
                    font-weight: 600;
                }
                .fnx-pop-btn {
                    width: 100%;
                    height: 30px;
                    padding: 0 10px;
                    border-radius: 7px;
                    background: var(--fnx-surface);
                    border: 1px solid var(--fnx-border);
                    color: var(--fnx-secondary);
                    font-size: 12px;
                    font-weight: 450;
                    cursor: pointer;
                    display: inline-flex;
                    align-items: center;
                    justify-content: center;
                    gap: 6px;
                    transition: background 0.12s ease, border-color 0.12s ease, color 0.12s ease;
                }
                .fnx-pop-btn:hover {
                    background: var(--fnx-hover);
                    border-color: var(--fnx-border-hover);
                    color: var(--fnx-fg);
                }
                .fnx-pop-footer-row {
                    width: 100%;
                    padding: 9px 12px;
                    border: none;
                    border-top: 1px solid var(--fnx-border);
                    background: transparent;
                    color: var(--fnx-secondary);
                    font-size: 12px;
                    font-weight: 450;
                    cursor: pointer;
                    display: flex;
                    align-items: center;
                    justify-content: space-between;
                    gap: 8px;
                    transition: background 0.1s ease, color 0.1s ease;
                }
                .fnx-pop-footer-row:hover {
                    background: var(--fnx-hover);
                    color: var(--fnx-fg);
                }
                .fnx-pop-link {
                    height: 22px;
                    padding: 0 6px;
                    border-radius: 5px;
                    border: 1px solid transparent;
                    background: transparent;
                    color: var(--fnx-secondary);
                    font-size: 11px;
                    font-weight: 500;
                    cursor: pointer;
                    transition: background 0.1s ease, color 0.1s ease;
                }
                .fnx-pop-link:hover {
                    background: var(--fnx-hover);
                    color: var(--fnx-fg);
                }
                .fnx-pop-icon-circle {
                    width: 30px;
                    height: 30px;
                    border-radius: 7px;
                    border: 1px solid var(--fnx-border);
                    background: var(--fnx-surface);
                    color: var(--fnx-fg);
                    display: inline-flex;
                    align-items: center;
                    justify-content: center;
                    cursor: pointer;
                    flex-shrink: 0;
                    transition: background 0.12s ease, border-color 0.12s ease;
                }
                .fnx-pop-icon-circle:hover {
                    background: var(--fnx-hover);
                    border-color: var(--fnx-border-hover);
                }
                .fnx-pop-range {
                    width: 100%;
                    height: 4px;
                    accent-color: var(--fnx-accent);
                    cursor: pointer;
                    margin: 2px 0;
                }
                .fnx-pop-dl-row {
                    display: flex;
                    align-items: center;
                    justify-content: space-between;
                    gap: 8px;
                    padding: 6px 8px;
                    border-radius: 6px;
                    cursor: pointer;
                    transition: background 0.08s ease;
                }
                .fnx-pop-dl-row:hover {
                    background: var(--fnx-hover);
                }
                .fnx-pop-dl-open {
                    opacity: 0;
                    font-size: 11px;
                    color: var(--fnx-secondary);
                    font-weight: 500;
                    flex-shrink: 0;
                    transition: opacity 0.1s ease, color 0.1s ease;
                }
                .fnx-pop-dl-row:hover .fnx-pop-dl-open {
                    opacity: 1;
                    color: var(--fnx-fg);
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
                    pointer-events: none;
                }
                .fnx-pulse-dot {
                    width: 6px;
                    height: 6px;
                    border-radius: 50%;
                    background: #34d399;
                    border: 1.5px solid var(--fnx-panel);
                    box-sizing: content-box;
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
                    height: 100%;
                    flex-shrink: 0;
                }
                .fnx-win-btn {
                    background: transparent;
                    border: none;
                    color: var(--fnx-fg);
                    opacity: 0.84;
                    width: 38px;
                    height: 100%;
                    display: flex;
                    align-items: center;
                    justify-content: center;
                    cursor: pointer;
                    outline: none;
                    transition: background 0.1s ease, color 0.1s ease, opacity 0.1s ease;
                }
                .fnx-win-btn:hover {
                    background: var(--fnx-hover);
                    color: var(--fnx-fg);
                    opacity: 1;
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
                    border-top-color: rgba(255, 255, 255, 0.14);
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
                    padding: 8px 18px;
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
                    gap: 8px;
                    flex-shrink: 0;
                }
                .fnx-rd-seg {
                    display: inline-flex;
                    align-items: center;
                    gap: 2px;
                    padding: 2px;
                    border-radius: 7px;
                    background: var(--fnx-surface);
                    border: 1px solid var(--fnx-border);
                }
                .fnx-rd-seg .fnx-pop-seg-btn {
                    min-width: 28px;
                    padding: 0 8px;
                    display: inline-flex;
                    align-items: center;
                    justify-content: center;
                }
                .fnx-rd-seg .fnx-pop-seg-btn.active {
                    background: var(--fnx-accent-bg);
                    border-color: var(--fnx-accent);
                    color: var(--fnx-fg);
                }
                .fnx-rd-size {
                    font-family: "JetBrains Mono", ui-monospace, Consolas, monospace;
                    font-size: 10.5px;
                    font-variant-numeric: tabular-nums;
                    color: var(--fnx-muted);
                    padding: 0 6px;
                    user-select: none;
                }
                .fnx-rd-btn {
                    height: 28px;
                    padding: 0 10px;
                    border-radius: 7px;
                    border: 1px solid var(--fnx-border);
                    background: var(--fnx-surface);
                    color: var(--fnx-secondary);
                    font-size: 12px;
                    font-weight: 500;
                    display: inline-flex;
                    align-items: center;
                    gap: 5px;
                    cursor: pointer;
                    transition: background 0.1s ease, border-color 0.1s ease, color 0.1s ease;
                }
                .fnx-rd-btn:hover {
                    border-color: var(--fnx-border-hover);
                    background: var(--fnx-hover);
                    color: var(--fnx-fg);
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
                    position: relative;
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
                .fnx-mb-dot {
                    position: absolute;
                    top: 6px;
                    right: 6px;
                    width: 6px;
                    height: 6px;
                    border-radius: 50%;
                    background: var(--fnx-accent);
                    display: none;
                    pointer-events: none;
                }
                .fnx-mb-dot.active {
                    display: block;
                }
                .fnx-mb-btn:active {
                    transform: scale(0.92);
                    background: var(--fnx-hover);
                    color: var(--fnx-fg);
                }
                .fnx-mb-btn:disabled {
                    opacity: 0.34;
                    cursor: default;
                }
                .fnx-mb-btn:disabled:active {
                    transform: none;
                    background: transparent;
                    color: var(--fnx-secondary);
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
                    display: flex;
                    align-items: center;
                    gap: 5px;
                    min-width: 0;
                }
                .fnx-drawer-title-text {
                    overflow: hidden;
                    text-overflow: ellipsis;
                    white-space: nowrap;
                    min-width: 0;
                }
                .fnx-drawer-pin-icon {
                    width: 12px;
                    height: 12px;
                    color: var(--fnx-accent);
                    display: inline-flex;
                    align-items: center;
                    justify-content: center;
                    flex-shrink: 0;
                }
                .fnx-drawer-pin-icon svg {
                    width: 12px;
                    height: 12px;
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
                    width: 36px;
                    height: 36px;
                    border: 1px solid var(--fnx-border);
                    background: var(--fnx-panel);
                    color: var(--fnx-secondary);
                    border-radius: 9px;
                    display: flex;
                    align-items: center;
                    justify-content: center;
                    cursor: pointer;
                    flex-shrink: 0;
                    transition: background 0.1s ease, color 0.1s ease, border-color 0.1s ease;
                }
                .fnx-drawer-close:active {
                    transform: scale(0.93);
                    background: var(--fnx-hover);
                    color: var(--fnx-fg);
                }
                .fnx-drawer-head-btn {
                    height: 30px;
                    padding: 0 9px;
                    border-radius: 7px;
                    border: 1px solid var(--fnx-border);
                    background: var(--fnx-surface);
                    color: var(--fnx-secondary);
                    font-size: 11.5px;
                    font-weight: 500;
                    display: inline-flex;
                    align-items: center;
                    gap: 5px;
                    cursor: pointer;
                    flex-shrink: 0;
                }
                .fnx-drawer-head-btn svg {
                    width: 13px;
                    height: 13px;
                    display: block;
                }
                .fnx-drawer-head-btn.active-route {
                    border-color: var(--fnx-accent);
                    background: var(--fnx-accent-bg);
                    color: var(--fnx-fg);
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
                    display: inline-flex;
                    align-items: center;
                    gap: 4px;
                }
                .fnx-quick-chip svg {
                    width: 12px;
                    height: 12px;
                    display: block;
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
                #__fynexo_shell__.fnx-is-mobile .fnx-desktop-only-row {
                    display: none !important;
                }
                #__fynexo_shell__:not(.fnx-is-mobile) .fnx-mobile-only-row {
                    display: none !important;
                }
                #__fynexo_shell__.fnx-is-mobile .fnx-menu-kbd {
                    display: none !important;
                }
                #__fynexo_shell__.fnx-is-mobile .fnx-menu-item {
                    min-height: 36px;
                    padding: 7px 10px;
                }
                @media (pointer: coarse) {
                    .fnx-menu-item {
                        min-height: 36px;
                        padding: 7px 10px;
                    }
                }
                #__fynexo_shell__.fnx-is-mobile #__fynexo_mb_reload__ {
                    width: 34px;
                    height: 34px;
                }
                #__fynexo_shell__.fnx-is-mobile #__fynexo_hud_card__ {
                    max-height: calc(100dvh - 20px);
                }
                #__fynexo_shell__.fnx-is-mobile #__fynexo_hud_suggest__ {
                    max-height: min(240px, 34dvh);
                }
                #__fynexo_shell__.fnx-is-mobile .fnx-suggest-row {
                    min-height: 44px;
                    padding: 9px 12px;
                }
                #__fynexo_shell__.fnx-is-mobile .fnx-quick-chip {
                    padding: 6px 10px;
                    font-size: 12px;
                }
                @media (max-width: 768px) {
                    #__fynexo_hud_card__ {
                        max-height: calc(100dvh - 20px);
                    }
                    #__fynexo_hud_suggest__ {
                        max-height: min(240px, 34dvh);
                    }
                    .fnx-suggest-row {
                        min-height: 44px;
                        padding: 9px 12px;
                    }
                    .fnx-quick-chip {
                        padding: 6px 10px;
                        font-size: 12px;
                    }
                    #__fynexo_find_bar__ {
                        top: calc(10px + env(safe-area-inset-top, 0px));
                        left: 10px;
                        right: 10px;
                    }
                    #__fynexo_find_input__ {
                        flex: 1;
                        width: auto;
                        min-height: 32px;
                        font-size: 13px;
                    }
                    #__fynexo_reader__ {
                        top: 0;
                        padding: calc(18px + env(safe-area-inset-top, 0px)) 16px calc(68px + max(env(safe-area-inset-bottom, 0px), 12px));
                    }
                    #__fynexo_hud_toast__ {
                        left: 14px;
                        right: 14px;
                        bottom: calc(64px + max(env(safe-area-inset-bottom, 0px), 12px));
                        justify-content: center;
                    }
                    #__fynexo_side_panel__ {
                        width: 100vw;
                        max-width: 100vw;
                        border-left: none;
                    }
                    #__fynexo_history_root__ {
                        align-items: stretch;
                        justify-content: flex-end;
                        padding: 0;
                    }
                    #__fynexo_history_card__ {
                        width: 100%;
                        max-height: 82dvh;
                        border-radius: 16px 16px 0 0;
                        border-bottom: none;
                        padding-bottom: max(env(safe-area-inset-bottom, 0px), 8px);
                    }
                }

                /* Studio Side Panel (Alt+S) */
                #__fynexo_side_panel__ {
                    position: fixed;
                    right: 0;
                    top: var(--fnx-host-top-offset, 0px);
                    width: 380px;
                    max-width: 90vw;
                    height: calc(100dvh - var(--fnx-host-top-offset, 0px));
                    background: var(--fnx-bg);
                    border-left: 1px solid var(--fnx-border);
                    z-index: 2147483640;
                    display: flex;
                    flex-direction: column;
                    transform: translateX(100%);
                    transition: transform 0.2s cubic-bezier(0.16, 1, 0.3, 1);
                    font-family: inherit;
                }
                #__fynexo_shell__.fnx-is-mobile #__fynexo_side_panel__ {
                    width: 100vw;
                    max-width: 100vw;
                    height: calc(100dvh - var(--fnx-host-top-offset, 0px) - calc(var(--fnx-host-bottom-offset, 0px) + max(env(safe-area-inset-bottom, 0px), 12px)));
                    border-left: none;
                }
                .fnx-sp-header {
                    display: flex;
                    align-items: center;
                    justify-content: space-between;
                    padding: 8px 10px;
                    border-bottom: 1px solid var(--fnx-border);
                    background: var(--fnx-panel);
                    gap: 8px;
                    flex-shrink: 0;
                }
                .fnx-sp-tabs {
                    grid-template-columns: repeat(2, auto);
                }
                .fnx-sp-tab {
                    display: inline-flex;
                    align-items: center;
                    gap: 5px;
                    padding: 0 9px;
                }
                .fnx-sp-tab svg,
                .fnx-sp-close svg {
                    width: 13px;
                    height: 13px;
                    display: block;
                    flex-shrink: 0;
                }
                .fnx-sp-close {
                    width: 26px;
                    height: 26px;
                    border-radius: 6px;
                    border: 1px solid transparent;
                    background: transparent;
                    color: var(--fnx-muted);
                    display: inline-flex;
                    align-items: center;
                    justify-content: center;
                    cursor: pointer;
                    transition: background 0.1s ease, color 0.1s ease;
                }
                .fnx-sp-close:hover {
                    background: var(--fnx-hover);
                    color: var(--fnx-fg);
                }
                .fnx-sp-body {
                    display: flex;
                    flex-direction: column;
                    flex: 1;
                    min-height: 0;
                }
                .fnx-sp-body.hidden {
                    display: none;
                }
                .fnx-sp-urlbar {
                    display: flex;
                    align-items: center;
                    gap: 6px;
                    padding: 7px 10px;
                    border-bottom: 1px solid var(--fnx-border);
                    background: var(--fnx-panel);
                }
                .fnx-sp-input {
                    flex: 1;
                    min-width: 0;
                    height: 28px;
                    font-size: 12px;
                    border-radius: 6px;
                    border: 1px solid var(--fnx-border);
                    background: var(--fnx-surface);
                    color: var(--fnx-fg);
                    padding: 0 9px;
                    outline: none;
                    transition: border-color 0.1s ease;
                }
                .fnx-sp-input:focus {
                    border-color: var(--fnx-accent);
                }
                .fnx-sp-go {
                    height: 28px;
                    padding: 0 11px;
                    border-radius: 6px;
                    border: 1px solid var(--fnx-border);
                    background: var(--fnx-surface);
                    color: var(--fnx-fg);
                    font-size: 11.5px;
                    font-weight: 500;
                    cursor: pointer;
                    flex-shrink: 0;
                    transition: background 0.1s ease, border-color 0.1s ease;
                }
                .fnx-sp-go:hover {
                    background: var(--fnx-hover);
                    border-color: var(--fnx-border-hover);
                }
                .fnx-sp-icon-btn {
                    width: 28px;
                    height: 28px;
                    border-radius: 6px;
                    border: 1px solid var(--fnx-border);
                    background: var(--fnx-surface);
                    color: var(--fnx-secondary);
                    display: inline-flex;
                    align-items: center;
                    justify-content: center;
                    cursor: pointer;
                    flex-shrink: 0;
                    transition: background 0.1s ease, border-color 0.1s ease, color 0.1s ease;
                }
                .fnx-sp-icon-btn:hover {
                    background: var(--fnx-hover);
                    border-color: var(--fnx-border-hover);
                    color: var(--fnx-fg);
                }
                .fnx-sp-icon-btn svg {
                    width: 13px;
                    height: 13px;
                    display: block;
                }
                .fnx-sp-iframe {
                    flex: 1;
                    width: 100%;
                    border: none;
                    background: #000;
                }
                .fnx-sp-notes-wrap {
                    padding: 10px;
                    gap: 8px;
                }
                .fnx-sp-textarea {
                    flex: 1;
                    width: 100%;
                    border: 1px solid var(--fnx-border);
                    border-radius: 8px;
                    background: var(--fnx-surface);
                    color: var(--fnx-fg);
                    font-family: "JetBrains Mono", ui-monospace, Consolas, monospace;
                    font-size: 12px;
                    line-height: 1.55;
                    padding: 10px;
                    resize: none;
                    outline: none;
                }
                .fnx-sp-textarea:focus {
                    border-color: var(--fnx-accent);
                }

                /* History Drawer Modal (Linear / Raycast Style) */
                #__fynexo_history_root__ {
                    position: fixed;
                    inset: 0;
                    background: rgba(0, 0, 0, 0.58);
                    z-index: 2147483647;
                    display: none;
                    align-items: flex-start;
                    justify-content: center;
                    padding: 64px 16px 24px;
                }
                #__fynexo_history_card__ {
                    width: min(560px, 100%);
                    max-height: min(540px, 78dvh);
                    background: var(--fnx-panel);
                    border: 1px solid var(--fnx-border-hover);
                    border-top-color: rgba(255, 255, 255, 0.14);
                    border-radius: 12px;
                    display: flex;
                    flex-direction: column;
                    overflow: hidden;
                }
                .fnx-hist-search-wrap {
                    display: flex;
                    align-items: center;
                    gap: 8px;
                    padding: 8px 12px;
                    border-bottom: 1px solid var(--fnx-border);
                    background: var(--fnx-surface);
                    color: var(--fnx-muted);
                }
                .fnx-hist-search-input {
                    flex: 1;
                    min-width: 0;
                    height: 26px;
                    background: transparent;
                    border: none;
                    outline: none;
                    color: var(--fnx-fg);
                    font-size: 12.5px;
                    font-family: inherit;
                }
                .fnx-hist-list {
                    flex: 1;
                    min-height: 120px;
                    overflow-y: auto;
                    padding: 4px;
                    display: flex;
                    flex-direction: column;
                    gap: 1px;
                }
                .fnx-hist-row {
                    width: 100%;
                    display: flex;
                    align-items: center;
                    gap: 9px;
                    padding: 6px 8px;
                    border-radius: 7px;
                    border: 1px solid transparent;
                    background: transparent;
                    color: var(--fnx-fg);
                    cursor: pointer;
                    text-align: left;
                    transition: background 0.08s ease;
                }
                .fnx-hist-row:hover {
                    background: var(--fnx-hover);
                }
                .fnx-hist-emblem {
                    width: 24px;
                    height: 24px;
                    border-radius: 6px;
                    background: var(--fnx-surface);
                    border: 1px solid var(--fnx-border);
                    color: var(--fnx-secondary);
                    font-size: 11px;
                    font-weight: 600;
                    display: flex;
                    align-items: center;
                    justify-content: center;
                    flex-shrink: 0;
                    overflow: hidden;
                }
                .fnx-hist-meta {
                    flex: 1;
                    min-width: 0;
                    display: flex;
                    flex-direction: column;
                }
                .fnx-hist-title {
                    font-size: 12.5px;
                    font-weight: 500;
                    color: var(--fnx-fg);
                    overflow: hidden;
                    text-overflow: ellipsis;
                    white-space: nowrap;
                }
                .fnx-hist-url {
                    font-size: 11px;
                    color: var(--fnx-muted);
                    overflow: hidden;
                    text-overflow: ellipsis;
                    white-space: nowrap;
                }
                .fnx-hist-open-btn {
                    width: 26px;
                    height: 26px;
                    border-radius: 6px;
                    border: 1px solid transparent;
                    background: transparent;
                    color: var(--fnx-muted);
                    display: inline-flex;
                    align-items: center;
                    justify-content: center;
                    flex-shrink: 0;
                    cursor: pointer;
                }
                .fnx-hist-row:hover .fnx-hist-open-btn {
                    color: var(--fnx-secondary);
                }
                .fnx-hist-open-btn:hover {
                    background: var(--fnx-surface);
                    border-color: var(--fnx-border);
                    color: var(--fnx-fg);
                }
                .fnx-hist-empty {
                    padding: 28px 16px;
                    text-align: center;
                    font-size: 12px;
                    color: var(--fnx-muted);
                }

                /* Custom Prompt Dialog (Replaces native window.prompt) */
                #__fynexo_prompt_root__ {
                    position: fixed;
                    inset: 0;
                    background: rgba(0, 0, 0, 0.62);
                    z-index: 2147483647;
                    display: none;
                    align-items: center;
                    justify-content: center;
                    padding: 16px;
                }
                #__fynexo_prompt_card__ {
                    width: min(380px, 100%);
                    background: var(--fnx-panel);
                    border: 1px solid var(--fnx-border-hover);
                    border-top-color: rgba(255, 255, 255, 0.14);
                    border-radius: 12px;
                    overflow: hidden;
                    display: flex;
                    flex-direction: column;
                }
                .fnx-prompt-body {
                    padding: 12px 14px 14px;
                    display: flex;
                    flex-direction: column;
                    gap: 10px;
                }
                .fnx-prompt-desc {
                    font-size: 12px;
                    line-height: 1.45;
                    color: var(--fnx-secondary);
                }
                .fnx-prompt-input {
                    width: 100%;
                    height: 34px;
                    padding: 0 10px;
                    border-radius: 7px;
                    border: 1px solid var(--fnx-border);
                    background: var(--fnx-surface);
                    color: var(--fnx-fg);
                    font-size: 13px;
                    font-family: inherit;
                    outline: none;
                    transition: border-color 0.1s ease;
                }
                .fnx-prompt-input:focus {
                    border-color: var(--fnx-accent);
                }
                .fnx-prompt-actions {
                    display: flex;
                    align-items: center;
                    justify-content: flex-end;
                    gap: 6px;
                    margin-top: 2px;
                }
                .fnx-prompt-btn {
                    height: 30px;
                    padding: 0 12px;
                    border-radius: 7px;
                    border: 1px solid var(--fnx-border);
                    background: var(--fnx-surface);
                    color: var(--fnx-secondary);
                    font-size: 12px;
                    font-weight: 500;
                    cursor: pointer;
                    transition: background 0.1s ease, border-color 0.1s ease, color 0.1s ease;
                }
                .fnx-prompt-btn:hover {
                    background: var(--fnx-hover);
                    border-color: var(--fnx-border-hover);
                    color: var(--fnx-fg);
                }
                .fnx-prompt-btn.primary {
                    background: var(--fnx-accent);
                    border-color: var(--fnx-accent);
                    color: #ffffff;
                }
                .fnx-prompt-btn.primary:hover {
                    filter: brightness(1.08);
                }
"##;