use crate::config::RuntimePayload;

pub fn settings_html(payload: &RuntimePayload) -> String {
    let payload_json = serde_json::to_string(payload).unwrap_or_else(|_| "{}".to_string());

    format!(
        r##"<!DOCTYPE html>
<html lang="zh-CN" data-fynexo-internal="settings">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0, maximum-scale=5.0">
    <title>Preferences — Fynexo</title>
    <style>
        :root {{
            --fnx-bg: #19191c;
            --fnx-panel: #29292e;
            --fnx-surface: #212126;
            --fnx-elevated: #35353c;
            --fnx-border: #37373f;
            --fnx-border-hover: #4b4b56;
            --fnx-fg: #f0f0f3;
            --fnx-secondary: #c2c2cb;
            --fnx-muted: #8a8a95;
            --fnx-accent: #7a88f7;
            --fnx-accent-bg: #262942;
            --fnx-hover: #33333a;
            --cfg-sidebar-w: 244px;
            --cfg-content-pad: 28px 36px;
            --cfg-card-radius: 10px;
        }}

        *, *::before, *::after {{
            box-sizing: border-box;
            margin: 0;
            padding: 0;
            -webkit-font-smoothing: antialiased;
            -webkit-tap-highlight-color: transparent;
            box-shadow: none !important;
            text-shadow: none !important;
            backdrop-filter: none !important;
        }}

        html {{ scroll-behavior: smooth; }}

        html, body {{
            width: 100%;
            min-height: 100vh;
            background: var(--fnx-bg);
            color: var(--fnx-fg);
            font-family: "Inter", system-ui, -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif;
            line-height: 1.5;
            overflow-x: hidden;
        }}

        button, input, select, textarea, a {{
            font-family: inherit;
            touch-action: manipulation;
        }}

        #__fynexo_settings_app__ {{
            width: 100%;
            min-height: 100vh;
            padding-top: var(--fnx-host-top-offset, 38px);
            display: grid;
            grid-template-columns: var(--cfg-sidebar-w) minmax(0, 1fr);
            align-items: stretch;
        }}

        #__fynexo_settings_app__.centered-mode .workspace-inner {{
            max-width: 1180px;
            margin: 0 auto;
        }}

        /* Left Rail */
        .sidebar {{
            background: var(--fnx-panel);
            border-right: 1px solid var(--fnx-border);
            padding: 20px 14px 16px;
            display: flex;
            flex-direction: column;
            justify-content: space-between;
            position: sticky;
            top: var(--fnx-host-top-offset, 38px);
            height: calc(100vh - var(--fnx-host-top-offset, 38px));
            user-select: none;
        }}

        .sidebar-top {{
            display: flex;
            flex-direction: column;
            gap: 18px;
        }}

        .brand-header {{
            display: flex;
            align-items: center;
            justify-content: space-between;
            gap: 8px;
            padding: 0 6px 16px;
            border-bottom: 1px solid var(--fnx-border);
        }}

        .brand-identity {{
            display: flex;
            align-items: center;
            gap: 9px;
            min-width: 0;
        }}

        .brand-mark {{
            width: 26px;
            height: 26px;
            border-radius: 7px;
            background: var(--fnx-accent-bg);
            border: 1px solid var(--fnx-accent);
            color: var(--fnx-accent);
            display: flex;
            align-items: center;
            justify-content: center;
            flex-shrink: 0;
        }}

        .brand-copy {{ display: flex; flex-direction: column; min-width: 0; }}
        .brand-title {{ font-size: 13.5px; font-weight: 600; letter-spacing: -0.02em; color: var(--fnx-fg); line-height: 1.2; }}
        .brand-sub {{ font-size: 11px; color: var(--fnx-muted); line-height: 1.25; }}

        .back-btn {{
            display: inline-flex;
            align-items: center;
            gap: 4px;
            padding: 5px 9px;
            font-size: 12px;
            font-weight: 500;
            color: var(--fnx-secondary);
            background: var(--fnx-surface);
            border: 1px solid var(--fnx-border);
            border-radius: 6px;
            cursor: pointer;
            flex-shrink: 0;
            transition: border-color 0.12s ease, color 0.12s ease, background 0.12s ease;
        }}
        .back-btn:hover {{
            border-color: var(--fnx-border-hover);
            color: var(--fnx-fg);
            background: var(--fnx-elevated);
        }}

        .nav-list {{ display: flex; flex-direction: column; gap: 3px; }}
        .nav-link {{
            display: flex;
            align-items: center;
            gap: 9px;
            padding: 8px 10px;
            font-size: 13px;
            font-weight: 500;
            color: var(--fnx-secondary);
            text-decoration: none;
            border-radius: 7px;
            border: 1px solid transparent;
            transition: background 0.12s ease, color 0.12s ease, border-color 0.12s ease;
        }}
        .nav-link svg {{ color: var(--fnx-muted); flex-shrink: 0; }}
        .nav-link:hover {{ background: var(--fnx-hover); color: var(--fnx-fg); }}
        .nav-link.active {{ background: var(--fnx-surface); border-color: var(--fnx-border); color: var(--fnx-fg); }}
        .nav-link.active svg {{ color: var(--fnx-accent); }}

        .sidebar-footer {{
            background: var(--fnx-surface);
            border: 1px solid var(--fnx-border);
            border-radius: var(--cfg-card-radius);
            padding: 11px 12px;
            font-size: 11.5px;
            display: flex;
            flex-direction: column;
            gap: 6px;
        }}
        .meta-row {{ display: flex; justify-content: space-between; align-items: center; gap: 8px; }}
        .meta-key {{ color: var(--fnx-muted); }}
        .meta-val {{
            color: var(--fnx-secondary);
            font-family: "JetBrains Mono", ui-monospace, Consolas, monospace;
            font-size: 11px;
            font-variant-numeric: tabular-nums;
            display: inline-flex;
            align-items: center;
            gap: 5px;
        }}
        .status-dot {{ width: 6px; height: 6px; border-radius: 50%; background: #10b981; flex-shrink: 0; }}
        .status-dot.off {{ background: var(--fnx-muted); }}

        /* Workspace */
        .workspace {{ width: 100%; min-width: 0; padding: var(--cfg-content-pad); }}
        .workspace-inner {{ width: 100%; display: flex; flex-direction: column; gap: 34px; }}

        .bundle-banner {{
            display: flex;
            align-items: center;
            justify-content: space-between;
            gap: 14px;
            padding: 13px 16px;
            background: var(--fnx-panel);
            border: 1px solid var(--fnx-border-hover);
            border-radius: var(--cfg-card-radius);
            flex-wrap: wrap;
        }}
        .bundle-title {{ font-size: 13.5px; font-weight: 600; color: var(--fnx-fg); }}
        .bundle-sub {{ font-size: 12px; color: var(--fnx-muted); margin-top: 2px; }}

        .sec-block {{ display: flex; flex-direction: column; gap: 14px; scroll-margin-top: 62px; }}
        .sec-header {{
            display: flex;
            align-items: flex-end;
            justify-content: space-between;
            flex-wrap: wrap;
            gap: 12px;
            padding-bottom: 10px;
            border-bottom: 1px solid var(--fnx-border);
        }}
        .sec-title {{ font-size: 16px; font-weight: 600; letter-spacing: -0.02em; color: var(--fnx-fg); }}
        .sec-desc {{ font-size: 12.5px; color: var(--fnx-muted); margin-top: 2px; }}
        .theme-header-actions {{ display: flex; align-items: center; gap: 7px; flex-wrap: wrap; }}

        .themes-dir-bar {{
            display: flex;
            align-items: center;
            justify-content: space-between;
            gap: 10px;
            padding: 8px 12px;
            background: var(--fnx-panel);
            border: 1px solid var(--fnx-border);
            border-radius: 8px;
            font-size: 11.5px;
            color: var(--fnx-muted);
            flex-wrap: wrap;
        }}
        .themes-dir-code {{
            font-family: "JetBrains Mono", ui-monospace, Consolas, monospace;
            font-size: 11px;
            color: var(--fnx-secondary);
            overflow: hidden;
            text-overflow: ellipsis;
            white-space: nowrap;
            max-width: 100%;
        }}

        /* Custom Color Pickers Studio */
        .palette-studio {{
            display: none;
            flex-direction: column;
            gap: 12px;
            padding: 14px 16px;
            background: var(--fnx-panel);
            border: 1px solid var(--fnx-accent);
            border-radius: var(--cfg-card-radius);
        }}
        .palette-studio.open {{ display: flex; }}
        .color-tokens-grid {{
            display: grid;
            grid-template-columns: repeat(auto-fill, minmax(175px, 1fr));
            gap: 8px;
        }}
        .color-swatch-box {{
            display: flex;
            align-items: center;
            gap: 8px;
            padding: 6px 8px;
            background: var(--fnx-surface);
            border: 1px solid var(--fnx-border);
            border-radius: 7px;
        }}
        .color-picker-native {{
            width: 24px;
            height: 24px;
            border: 1px solid var(--fnx-border);
            border-radius: 5px;
            padding: 0;
            background: transparent;
            cursor: pointer;
            flex-shrink: 0;
        }}
        .color-token-meta {{ display: flex; flex-direction: column; min-width: 0; flex: 1; }}
        .color-token-lbl {{ font-size: 11px; font-weight: 500; color: var(--fnx-secondary); }}
        .color-token-hex {{
            background: transparent;
            border: none;
            outline: none;
            color: var(--fnx-fg);
            font-family: "JetBrains Mono", ui-monospace, Consolas, monospace;
            font-size: 11px;
            width: 100%;
        }}

        /* Theme Grid */
        .theme-grid {{
            display: grid;
            grid-template-columns: repeat(auto-fill, minmax(210px, 1fr));
            gap: 12px;
            width: 100%;
        }}
        .theme-card {{
            position: relative;
            display: flex;
            flex-direction: column;
            padding: 11px;
            background: var(--fnx-panel);
            border: 1px solid var(--fnx-border);
            border-radius: var(--cfg-card-radius);
            cursor: pointer;
            outline: none;
            text-align: left;
            min-width: 0;
            user-select: none;
            transition: border-color 0.12s ease, background 0.12s ease;
        }}
        .theme-card:hover {{ border-color: var(--fnx-border-hover); background: var(--fnx-hover); }}
        .theme-card.active {{ border-color: var(--fnx-accent); background: var(--fnx-accent-bg); }}

        .theme-meta-right {{ display: inline-flex; align-items: center; gap: 5px; flex-shrink: 0; }}
        .theme-del-btn {{
            width: 20px;
            height: 20px;
            border-radius: 5px;
            border: 1px solid var(--fnx-border);
            background: var(--fnx-surface);
            color: var(--fnx-muted);
            font-size: 12px;
            line-height: 1;
            display: inline-flex;
            align-items: center;
            justify-content: center;
            cursor: pointer;
        }}
        .theme-del-btn:hover {{ border-color: #f43f5e; color: #f43f5e; }}

        .mini-preview {{
            height: 58px;
            border-radius: 6px;
            overflow: hidden;
            display: flex;
            flex-direction: column;
            margin-bottom: 10px;
            border: 1px solid rgba(128, 128, 128, 0.22);
        }}
        .mini-bar {{
            height: 18px;
            display: flex;
            align-items: center;
            justify-content: space-between;
            padding: 0 5px;
            gap: 5px;
            border-bottom: 1px solid rgba(128, 128, 128, 0.18);
        }}
        .mini-tabs {{ display: flex; align-items: center; gap: 3px; min-width: 0; }}
        .mini-tab {{ display: flex; align-items: center; gap: 4px; height: 11px; padding: 0 5px; border-radius: 3px; }}
        .mini-dot {{ width: 4px; height: 4px; border-radius: 50%; flex-shrink: 0; }}
        .mini-line {{ width: 22px; height: 2px; border-radius: 1px; opacity: 0.82; }}
        .mini-omnibox {{ width: 34px; height: 10px; border-radius: 3px; flex-shrink: 0; }}
        .mini-body {{ flex: 1; display: flex; align-items: center; justify-content: space-between; padding: 0 8px; gap: 8px; }}
        .mini-canvas-col {{ display: flex; flex-direction: column; gap: 4px; flex: 1; }}
        .theme-info {{ display: flex; align-items: center; justify-content: space-between; gap: 6px; }}
        .theme-name-wrap {{ display: inline-flex; align-items: center; gap: 6px; min-width: 0; }}
        .theme-name {{ font-size: 13px; font-weight: 600; letter-spacing: -0.01em; color: var(--fnx-fg); }}
        .theme-active-tag {{ display: none; font-size: 10px; font-weight: 600; padding: 1px 5px; border-radius: 4px; background: var(--fnx-accent); color: #ffffff; }}
        .theme-card.active .theme-active-tag {{ display: inline-block; }}
        .theme-hex {{ font-family: "JetBrains Mono", ui-monospace, Consolas, monospace; font-size: 10.5px; color: var(--fnx-muted); }}
        .theme-sub {{ font-size: 11.5px; color: var(--fnx-muted); margin-top: 2px; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }}

        /* General Settings Cards */
        .cards-grid {{
            display: grid;
            grid-template-columns: repeat(auto-fit, minmax(320px, 1fr));
            gap: 12px;
            width: 100%;
        }}
        .setting-card {{
            background: var(--fnx-panel);
            border: 1px solid var(--fnx-border);
            border-radius: var(--cfg-card-radius);
            padding: 16px 18px;
            display: flex;
            flex-direction: column;
            justify-content: space-between;
            gap: 12px;
            min-width: 0;
        }}
        .setting-card.full-span {{
            grid-column: 1 / -1;
        }}
        .card-label {{ font-size: 13.5px; font-weight: 600; letter-spacing: -0.01em; color: var(--fnx-fg); }}
        .card-hint {{ font-size: 12px; color: var(--fnx-muted); margin-top: 2px; }}

        .seg-group {{
            display: flex;
            flex-wrap: wrap;
            background: var(--fnx-bg);
            padding: 3px;
            border-radius: 7px;
            border: 1px solid var(--fnx-border);
            gap: 3px;
            width: 100%;
        }}
        .seg-btn {{
            flex: 1 1 auto;
            display: inline-flex;
            align-items: center;
            justify-content: center;
            gap: 6px;
            padding: 6px 10px;
            font-size: 12px;
            font-weight: 500;
            background: transparent;
            border: 1px solid transparent;
            border-radius: 5px;
            color: var(--fnx-muted);
            cursor: pointer;
            outline: none;
            white-space: nowrap;
        }}
        .seg-btn:hover {{ color: var(--fnx-fg); }}
        .seg-btn.active {{
            background: var(--fnx-elevated);
            border-color: var(--fnx-border);
            color: var(--fnx-fg);
        }}

        .inline-form {{
            display: flex;
            align-items: center;
            gap: 7px;
            flex-wrap: wrap;
            width: 100%;
        }}
        .inline-form .text-input {{
            flex: 1;
            min-width: 120px;
        }}

        .chips-wrap {{
            display: flex;
            align-items: center;
            gap: 6px;
            flex-wrap: wrap;
        }}
        .tag-chip {{
            display: inline-flex;
            align-items: center;
            gap: 6px;
            padding: 4px 8px;
            font-size: 11.5px;
            background: var(--fnx-surface);
            border: 1px solid var(--fnx-border);
            border-radius: 6px;
            color: var(--fnx-fg);
            max-width: 100%;
        }}
        .tag-chip code {{
            font-family: "JetBrains Mono", ui-monospace, Consolas, monospace;
            font-size: 11px;
            color: var(--fnx-accent);
        }}
        .chip-x {{
            border: none;
            background: transparent;
            color: var(--fnx-muted);
            cursor: pointer;
            font-size: 13px;
            line-height: 1;
            padding: 0 2px;
        }}
        .chip-x:hover {{ color: #f43f5e; }}

        .text-input, .select-input, .code-area {{
            width: 100%;
            min-width: 0;
            background: var(--fnx-surface);
            border: 1px solid var(--fnx-border);
            border-radius: 7px;
            color: var(--fnx-fg);
            padding: 7px 10px;
            font-size: 12.5px;
            outline: none;
            transition: border-color 0.12s ease, background 0.12s ease;
        }}
        .text-input.mono, .code-area {{
            font-family: "JetBrains Mono", ui-monospace, Consolas, monospace;
            font-size: 12px;
        }}
        .code-area {{
            min-height: 82px;
            resize: vertical;
            line-height: 1.5;
        }}
        .text-input:focus, .select-input:focus, .code-area:focus {{
            border-color: var(--fnx-accent);
            background: var(--fnx-bg);
        }}

        .toggle-row {{ display: flex; align-items: center; justify-content: space-between; gap: 12px; }}
        .switch-btn {{
            display: inline-flex;
            align-items: center;
            gap: 7px;
            padding: 6px 12px;
            font-size: 12px;
            font-weight: 500;
            border: 1px solid var(--fnx-border);
            border-radius: 7px;
            cursor: pointer;
            background: var(--fnx-surface);
            color: var(--fnx-secondary);
            flex-shrink: 0;
            transition: border-color 0.12s ease, background 0.12s ease, color 0.12s ease;
        }}
        .switch-btn:hover {{ border-color: var(--fnx-border-hover); color: var(--fnx-fg); }}
        .switch-btn.on {{ border-color: var(--fnx-accent); background: var(--fnx-accent-bg); color: var(--fnx-fg); }}
        .switch-dot {{ width: 7px; height: 7px; border-radius: 50%; background: var(--fnx-muted); }}
        .switch-btn.on .switch-dot {{ background: #10b981; }}

        /* Site Rules List & Visual Editor */
        .rules-list {{
            display: flex;
            flex-direction: column;
            gap: 8px;
        }}
        .rule-row {{
            display: flex;
            align-items: center;
            justify-content: space-between;
            gap: 10px;
            padding: 10px 12px;
            background: var(--fnx-surface);
            border: 1px solid var(--fnx-border);
            border-radius: 8px;
            flex-wrap: wrap;
        }}
        .rule-info {{ display: flex; align-items: center; gap: 8px; flex-wrap: wrap; min-width: 0; flex: 1; }}
        .rule-name {{ font-size: 13px; font-weight: 500; color: var(--fnx-fg); }}
        .rule-domain {{
            font-family: "JetBrains Mono", ui-monospace, Consolas, monospace;
            font-size: 11px;
            padding: 2px 6px;
            border-radius: 5px;
            background: var(--fnx-bg);
            border: 1px solid var(--fnx-border);
            color: var(--fnx-secondary);
        }}
        .rule-badge {{
            font-size: 10px;
            font-weight: 600;
            padding: 1px 5px;
            border-radius: 4px;
            background: var(--fnx-accent-bg);
            border: 1px solid var(--fnx-accent);
            color: var(--fnx-fg);
        }}
        .rule-editor-panel {{
            display: none;
            flex-direction: column;
            gap: 10px;
            padding: 13px;
            background: var(--fnx-bg);
            border: 1px solid var(--fnx-border-hover);
            border-radius: 8px;
        }}
        .rule-editor-panel.open {{ display: flex; }}

        /* Component Studio */
        .studio-split {{
            display: grid;
            grid-template-columns: minmax(0, 1fr);
            gap: 14px;
            align-items: stretch;
            width: 100%;
        }}
        @media (min-width: 1180px) {{
            .studio-split {{ grid-template-columns: 380px minmax(0, 1fr); }}
        }}

        .studio-inspector {{
            background: var(--fnx-panel);
            border: 1px solid var(--fnx-border);
            border-radius: var(--cfg-card-radius);
            padding: 14px;
            display: flex;
            flex-direction: column;
            gap: 14px;
            min-width: 0;
        }}
        .inspector-group {{ display: flex; flex-direction: column; gap: 9px; }}
        .inspector-group + .inspector-group {{ padding-top: 12px; border-top: 1px solid var(--fnx-border); }}
        .inspector-heading {{ font-size: 12px; font-weight: 600; color: var(--fnx-fg); }}
        .inspector-grid {{ display: grid; grid-template-columns: repeat(2, minmax(0, 1fr)); gap: 8px; }}
        .field-box {{ display: flex; flex-direction: column; gap: 5px; min-width: 0; }}
        .field-box.span-2 {{ grid-column: span 2; }}
        .field-label {{ font-size: 11.5px; font-weight: 500; color: var(--fnx-secondary); display: flex; justify-content: space-between; align-items: center; gap: 6px; }}
        .field-tag {{ font-family: "JetBrains Mono", ui-monospace, Consolas, monospace; font-size: 10px; color: var(--fnx-muted); }}

        .inspector-toggle {{
            display: flex;
            align-items: center;
            justify-content: space-between;
            gap: 10px;
            padding: 7px 10px;
            background: var(--fnx-surface);
            border: 1px solid var(--fnx-border);
            border-radius: 7px;
        }}
        .inspector-toggle-title {{ font-size: 12px; font-weight: 500; color: var(--fnx-fg); }}
        .inspector-toggle-key {{ font-family: "JetBrains Mono", ui-monospace, Consolas, monospace; font-size: 10px; color: var(--fnx-muted); }}

        .json-studio {{
            background: var(--fnx-panel);
            border: 1px solid var(--fnx-border);
            border-radius: var(--cfg-card-radius);
            overflow: hidden;
            display: flex;
            flex-direction: column;
            min-width: 0;
            width: 100%;
        }}
        .json-toolbar {{
            display: flex;
            align-items: center;
            justify-content: space-between;
            padding: 10px 14px;
            border-bottom: 1px solid var(--fnx-border);
            background: var(--fnx-panel);
            gap: 10px;
            flex-wrap: wrap;
        }}
        .json-file-meta {{ display: flex; align-items: center; gap: 8px; min-width: 0; }}
        .json-badge {{
            padding: 2px 6px;
            border-radius: 4px;
            background: var(--fnx-surface);
            border: 1px solid var(--fnx-border);
            font-family: "JetBrains Mono", ui-monospace, Consolas, monospace;
            font-size: 10.5px;
            color: var(--fnx-accent);
        }}
        .json-path {{
            font-family: "JetBrains Mono", ui-monospace, Consolas, monospace;
            font-size: 11.5px;
            color: var(--fnx-secondary);
            overflow: hidden;
            text-overflow: ellipsis;
            white-space: nowrap;
        }}
        .json-actions {{ display: flex; align-items: center; gap: 6px; flex-wrap: wrap; }}

        .action-btn {{
            padding: 6px 11px;
            font-size: 12px;
            font-weight: 500;
            border-radius: 6px;
            border: 1px solid var(--fnx-border);
            background: var(--fnx-surface);
            color: var(--fnx-secondary);
            cursor: pointer;
            transition: border-color 0.12s ease, color 0.12s ease, background 0.12s ease;
        }}
        .action-btn:hover {{ border-color: var(--fnx-border-hover); color: var(--fnx-fg); background: var(--fnx-elevated); }}
        .action-btn.primary {{ background: var(--fnx-accent); border-color: var(--fnx-accent); color: #ffffff; }}
        .action-btn.primary:hover {{ opacity: 0.92; background: var(--fnx-accent); color: #ffffff; }}
        .action-btn.danger:hover {{ border-color: #f43f5e; color: #f43f5e; }}

        .json-editor {{
            width: 100%;
            flex: 1;
            min-height: 460px;
            padding: 14px 16px;
            background: var(--fnx-bg);
            color: var(--fnx-fg);
            border: none;
            outline: none;
            resize: vertical;
            font-family: "JetBrains Mono", ui-monospace, SFMono-Regular, Consolas, monospace;
            font-size: 12.5px;
            line-height: 1.58;
            tab-size: 2;
        }}
        .json-status {{
            padding: 8px 14px;
            border-top: 1px solid var(--fnx-border);
            background: var(--fnx-panel);
            font-size: 11.5px;
            color: var(--fnx-muted);
            display: flex;
            align-items: center;
            justify-content: space-between;
            gap: 8px;
            flex-wrap: wrap;
        }}
        .json-status.error {{ color: #f43f5e; background: rgba(244, 63, 94, 0.08); }}

        /* Interactive Click-to-Record Keybinds */
        .shortcuts-grid {{
            display: grid;
            grid-template-columns: repeat(auto-fit, minmax(270px, 1fr));
            gap: 10px;
            width: 100%;
        }}
        .shortcut-card {{
            background: var(--fnx-panel);
            border: 1px solid var(--fnx-border);
            border-radius: var(--cfg-card-radius);
            padding: 11px 14px;
            display: flex;
            align-items: center;
            justify-content: space-between;
            gap: 12px;
            font-size: 12.5px;
        }}
        .shortcut-label {{ color: var(--fnx-secondary); }}
        .kb-record-btn {{
            padding: 4px 10px;
            border-radius: 6px;
            border: 1px solid var(--fnx-border);
            background: var(--fnx-surface);
            color: var(--fnx-fg);
            font-family: "JetBrains Mono", ui-monospace, Consolas, monospace;
            font-size: 11.5px;
            cursor: pointer;
            white-space: nowrap;
        }}
        .kb-record-btn:hover {{ border-color: var(--fnx-border-hover); background: var(--fnx-elevated); }}
        .kb-record-btn.recording {{
            border-color: var(--fnx-accent);
            background: var(--fnx-accent-bg);
            color: var(--fnx-fg);
        }}

        .preset-row {{ display: flex; align-items: center; gap: 6px; flex-wrap: wrap; margin-top: 8px; }}
        .preset-chip {{
            padding: 3px 8px;
            font-size: 11px;
            font-weight: 500;
            border-radius: 5px;
            border: 1px solid var(--fnx-border);
            background: var(--fnx-surface);
            color: var(--fnx-secondary);
            cursor: pointer;
        }}
        .preset-chip:hover {{ border-color: var(--fnx-border-hover); color: var(--fnx-fg); background: var(--fnx-elevated); }}

        .storage-path {{
            padding: 7px 10px;
            border-radius: 6px;
            background: var(--fnx-bg);
            border: 1px solid var(--fnx-border);
            font-family: "JetBrains Mono", ui-monospace, Consolas, monospace;
            font-size: 11.5px;
            color: var(--fnx-secondary);
            overflow-x: auto;
            white-space: nowrap;
        }}
        .storage-actions {{ display: flex; align-items: center; gap: 8px; flex-wrap: wrap; }}

        .toast {{
            position: fixed;
            bottom: calc(20px + var(--fnx-host-bottom-offset, 0px) + max(env(safe-area-inset-bottom, 0px), 12px));
            right: 20px;
            background: var(--fnx-elevated);
            border: 1px solid var(--fnx-accent);
            color: var(--fnx-fg);
            font-size: 12.5px;
            font-weight: 500;
            padding: 8px 14px;
            border-radius: 8px;
            display: none;
            align-items: center;
            gap: 8px;
            z-index: 99999;
        }}

        @media (max-width: 768px) {{
            #__fynexo_settings_app__ {{
                display: flex;
                flex-direction: column;
                padding-top: var(--fnx-host-top-offset, 0px);
                padding-bottom: calc(var(--fnx-host-bottom-offset, 54px) + max(env(safe-area-inset-bottom, 0px), 12px) + 24px);
            }}
            .sidebar {{
                position: sticky;
                top: var(--fnx-host-top-offset, 0px);
                height: auto;
                z-index: 50;
                border-right: none;
                border-bottom: 1px solid var(--fnx-border);
                padding: 11px 14px 10px;
                gap: 10px;
            }}
            .sidebar-top {{ gap: 10px; }}
            .brand-header {{ padding: 0; border-bottom: none; }}
            .nav-list {{ flex-direction: row; overflow-x: auto; scrollbar-width: none; gap: 6px; padding-bottom: 2px; }}
            .nav-list::-webkit-scrollbar {{ display: none; }}
            .nav-link {{ white-space: nowrap; padding: 6px 11px; font-size: 12px; background: var(--fnx-surface); border-color: var(--fnx-border); }}
            .nav-link.active {{ border-color: var(--fnx-accent); background: var(--fnx-accent-bg); }}
            .sidebar-footer, .desktop-only-sec, .desktop-only-inline {{ display: none !important; }}
            .workspace {{ padding: 18px 14px !important; }}
            .workspace-inner {{ gap: 26px; }}
            .theme-grid {{ grid-template-columns: repeat(2, minmax(0, 1fr)); gap: 8px; }}
            .cards-grid {{ grid-template-columns: minmax(0, 1fr); gap: 10px; }}
            .inspector-grid {{ grid-template-columns: minmax(0, 1fr); }}
            .field-box.span-2 {{ grid-column: span 1; }}
            .json-editor {{ min-height: 260px; font-size: 11.5px; }}
            .toast {{ left: 14px; right: 14px; justify-content: center; }}
        }}
    </style>
    <style id="__fynexo_settings_custom_css__"></style>
</head>
<body>
    <div id="__fynexo_settings_app__">
        <aside class="sidebar">
            <div class="sidebar-top">
                <div class="brand-header">
                    <div class="brand-identity">
                        <div class="brand-mark" aria-hidden="true">
                            <svg width="14" height="14" viewBox="0 0 16 16" fill="none">
                                <path d="M3.5 3.5H12.5M3.5 8H10M3.5 3.5V12.5" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round"/>
                                <circle cx="12" cy="11.5" r="1.6" fill="currentColor"/>
                            </svg>
                        </div>
                        <div class="brand-copy">
                            <span class="brand-title">Preferences</span>
                            <span class="brand-sub">Fynexo studio</span>
                        </div>
                    </div>
                    <button type="button" class="back-btn" id="btnBack">← Back</button>
                </div>

                <nav class="nav-list" id="sidebarNav">
                    <a href="#sec-appearance" class="nav-link active" data-sec="sec-appearance">
                        <svg width="14" height="14" viewBox="0 0 16 16" fill="none"><circle cx="8" cy="8" r="5.5" stroke="currentColor" stroke-width="1.4"/><path d="M8 2.5v11" stroke="currentColor" stroke-width="1.4"/></svg>
                        <span>Appearance</span>
                    </a>
                    <a href="#sec-general" class="nav-link" data-sec="sec-general">
                        <svg width="14" height="14" viewBox="0 0 16 16" fill="none"><circle cx="7.2" cy="7.2" r="4.4" stroke="currentColor" stroke-width="1.4"/><path d="M10.5 10.5L13.6 13.6" stroke="currentColor" stroke-width="1.4" stroke-linecap="round"/></svg>
                        <span>Search &amp; general</span>
                    </a>
                    <a href="#sec-rules" class="nav-link" data-sec="sec-rules">
                        <svg width="14" height="14" viewBox="0 0 16 16" fill="none"><path d="M8 2.2l5 2v3.6c0 3.2-2.1 5.4-5 6.2-2.9-.8-5-3-5-6.2V4.2l5-2z" stroke="currentColor" stroke-width="1.4" stroke-linejoin="round"/></svg>
                        <span>Adblock &amp; site rules</span>
                    </a>
                    <a href="#sec-styles" class="nav-link" data-sec="sec-styles">
                        <svg width="14" height="14" viewBox="0 0 16 16" fill="none"><path d="M5.2 5L2.4 8l2.8 3M10.8 5l2.8 3-2.8 3M8.8 3.5l-1.6 9" stroke="currentColor" stroke-width="1.4" stroke-linecap="round" stroke-linejoin="round"/></svg>
                        <span>Component studio</span>
                    </a>
                    <a href="#sec-shortcuts" class="nav-link desktop-only-sec" data-sec="sec-shortcuts">
                        <svg width="14" height="14" viewBox="0 0 16 16" fill="none"><rect x="2" y="3.5" width="12" height="9" rx="1.8" stroke="currentColor" stroke-width="1.4"/><path d="M5 6.8h1.2M8.2 6.8h2.8M5 9.6h6" stroke="currentColor" stroke-width="1.4" stroke-linecap="round"/></svg>
                        <span>Keybinds</span>
                    </a>
                </nav>
            </div>

            <div class="sidebar-footer">
                <div class="meta-row">
                    <span class="meta-key">Architecture</span>
                    <span class="meta-val">Single-WebView</span>
                </div>
                <div class="meta-row">
                    <span class="meta-key">Adblock shield</span>
                    <span class="meta-val"><span class="status-dot" id="metaAdblockDot"></span><span id="metaAdblock">Active</span></span>
                </div>
                <div class="meta-row">
                    <span class="meta-key">Zero-FX filter</span>
                    <span class="meta-val"><span class="status-dot" id="metaPerfDot"></span><span id="metaPerf">Enabled</span></span>
                </div>
                <div class="meta-row">
                    <span class="meta-key">Private mode</span>
                    <span class="meta-val" id="metaIncognito">Off</span>
                </div>
            </div>
        </aside>

        <main class="workspace">
            <div class="workspace-inner">
                <!-- Full Setup Bundle Sync Bar -->
                <div class="bundle-banner">
                    <div>
                        <div class="bundle-title">Portable Full Setup Bundle (PC ↔ Android Sync)</div>
                        <div class="bundle-sub">Export or import all themes, styles, shortcuts, !bangs, site rules, adblock lists, and keybinds in one JSON file.</div>
                    </div>
                    <div class="storage-actions">
                        <input type="file" id="fileImportBundle" accept="application/json,text/plain" style="display: none;">
                        <button type="button" class="action-btn" id="btnImportBundle">Import full setup (.json)</button>
                        <button type="button" class="action-btn primary" id="btnExportBundle">Export full setup (.json)</button>
                    </div>
                </div>

                <!-- 1. Appearance & Visual Palette -->
                <section class="sec-block" id="sec-appearance">
                    <div class="sec-header">
                        <div>
                            <h2 class="sec-title">Appearance &amp; JSON themes</h2>
                            <p class="sec-desc">Pick a preset, import community <code style="font-family: monospace;">*.json</code> palettes, or design colors visually below.</p>
                        </div>
                        <div class="theme-header-actions">
                            <input type="file" id="fileImportTheme" accept="application/json,text/plain" multiple style="display: none;">
                            <button type="button" class="action-btn" id="btnImportTheme">Import .json</button>
                            <button type="button" class="action-btn" id="btnExportTheme">Export .json</button>
                            <button type="button" class="action-btn desktop-only-inline" id="btnOpenThemesDir">Open themes/</button>
                            <button type="button" id="btnCustomColorsToggle" class="switch-btn">
                                <span class="switch-dot"></span>
                                <span id="customColorsLabel">Custom palette: Off</span>
                            </button>
                        </div>
                    </div>

                    <div class="palette-studio" id="paletteStudio">
                        <div class="toggle-row" style="flex-wrap: wrap;">
                            <div>
                                <div class="card-label">Visual Color Palette Designer</div>
                                <div class="card-hint">Click any swatch to pick colors visually — updates live and can be saved as a standalone <code style="font-family: monospace;">themes/*.json</code> file.</div>
                            </div>
                            <div class="inline-form" style="width: auto;">
                                <input type="text" id="inputNewThemeName" class="text-input" placeholder="Theme name (e.g. Solarized)" style="width: 190px;">
                                <button type="button" class="action-btn primary" id="btnSaveAsTheme">Save as theme .json</button>
                            </div>
                        </div>
                        <div class="color-tokens-grid" id="colorTokensGrid"></div>
                    </div>

                    <div class="themes-dir-bar">
                        <span>Themes directory (auto-reloads on focus):</span>
                        <span class="themes-dir-code" id="themesDirLabel">themes/</span>
                    </div>
                    <div class="theme-grid" id="themeGrid"></div>
                </section>

                <!-- 2. Search, Bangs, Shortcuts, UA & Privacy -->
                <section class="sec-block" id="sec-general">
                    <div class="sec-header">
                        <div>
                            <h2 class="sec-title">Search, !bangs, shortcuts &amp; browser modes</h2>
                            <p class="sec-desc">Every engine, bang trigger, shortcut website, User-Agent, and privacy option can be edited visually here or via JSON.</p>
                        </div>
                    </div>

                    <div class="cards-grid">
                        <!-- Search Engines -->
                        <div class="setting-card">
                            <div>
                                <div class="card-label">Search engines</div>
                                <div class="card-hint">Click to set default engine, or add any custom <code style="font-family: monospace;">{{query}}</code> search URL.</div>
                            </div>
                            <div class="seg-group" id="enginePills"></div>
                            <div class="inline-form">
                                <input type="text" id="inputEngName" class="text-input" placeholder="Name (e.g. Kagi)" style="max-width: 120px;">
                                <input type="text" id="inputEngUrl" class="text-input mono" placeholder="https://kagi.com/search?q={{query}}">
                                <button type="button" class="action-btn primary" id="btnAddEngine">+ Add</button>
                            </div>
                        </div>

                        <!-- Bang Shortcuts -->
                        <div class="setting-card">
                            <div>
                                <div class="card-label">!Bang quick search triggers</div>
                                <div class="card-hint">Type e.g. <code style="font-family: monospace;">!gh wry</code> or <code style="font-family: monospace;">!yt music</code> in the address bar or start page.</div>
                            </div>
                            <div class="chips-wrap" id="bangsChips"></div>
                            <div class="inline-form">
                                <input type="text" id="inputBangTrig" class="text-input mono" placeholder="!npm" style="max-width: 74px;">
                                <input type="text" id="inputBangName" class="text-input" placeholder="Name" style="max-width: 96px;">
                                <input type="text" id="inputBangUrl" class="text-input mono" placeholder="https://npmjs.com/search?q={{query}}">
                                <button type="button" class="action-btn primary" id="btnAddBang">+ Add</button>
                            </div>
                        </div>

                        <!-- Homepage -->
                        <div class="setting-card">
                            <div>
                                <div class="card-label">New tab homepage</div>
                                <div class="card-hint">Destination loaded when opening a fresh tab.</div>
                            </div>
                            <div>
                                <input type="text" id="inputHomepage" class="text-input mono" spellcheck="false" placeholder="fynexo://newtab">
                                <div class="preset-row">
                                    <button type="button" class="preset-chip" data-hp="fynexo://newtab">fynexo://newtab</button>
                                    <button type="button" class="preset-chip" data-hp="https://www.google.com">Google</button>
                                    <button type="button" class="preset-chip" data-hp="https://duckduckgo.com">DuckDuckGo</button>
                                    <button type="button" class="preset-chip" data-hp="https://github.com">GitHub</button>
                                </div>
                            </div>
                        </div>

                        <!-- User-Agent & Default Zoom -->
                        <div class="setting-card">
                            <div>
                                <div class="card-label">User-Agent &amp; default page zoom</div>
                                <div class="card-hint">Switch between Desktop/Mobile UA strings live and set default zoom scale.</div>
                            </div>
                            <div class="seg-group" id="uaModeGroup">
                                <button type="button" class="seg-btn" data-ua="default">System</button>
                                <button type="button" class="seg-btn" data-ua="desktop">Desktop PC</button>
                                <button type="button" class="seg-btn" data-ua="mobile">Mobile</button>
                                <button type="button" class="seg-btn" data-ua="custom">Custom</button>
                            </div>
                            <input type="text" id="inputCustomUa" class="text-input mono" placeholder="Mozilla/5.0 ..." style="display: none;">
                            <div class="toggle-row">
                                <span style="font-size: 12px; color: var(--fnx-secondary);">Default page zoom</span>
                                <div class="storage-actions">
                                    <button type="button" class="action-btn" id="btnZoomOut">−</button>
                                    <button type="button" class="action-btn" id="btnZoomReset">100%</button>
                                    <button type="button" class="action-btn" id="btnZoomIn">+</button>
                                </div>
                            </div>
                        </div>

                        <!-- Shortcut Websites Visual Manager -->
                        <div class="setting-card full-span">
                            <div class="toggle-row" style="flex-wrap: wrap;">
                                <div>
                                    <div class="card-label">Shortcut websites (<code style="font-family: monospace; font-size: 12px;">fynexo_shortcuts.json</code>)</div>
                                    <div class="card-hint" id="shortcutsHint">Manage start page &amp; omnibox shortcuts visually or share as JSON.</div>
                                </div>
                                <div class="storage-actions">
                                    <input type="file" id="fileImportShortcuts" accept="application/json,text/plain" style="display: none;">
                                    <button type="button" class="action-btn" id="btnImportShortcuts">Import .json</button>
                                    <button type="button" class="action-btn" id="btnExportShortcuts">Export .json</button>
                                    <button type="button" class="action-btn" id="btnResetShortcuts">Reset defaults</button>
                                </div>
                            </div>
                            <div class="chips-wrap" id="shortcutsChips"></div>
                            <div class="inline-form">
                                <input type="text" id="inputScTitle" class="text-input" placeholder="Title (e.g. Hacker News)" style="max-width: 200px;">
                                <input type="text" id="inputScUrl" class="text-input mono" placeholder="https://news.ycombinator.com">
                                <button type="button" class="action-btn primary" id="btnAddShortcut">+ Add shortcut</button>
                            </div>
                        </div>

                        <!-- Smart Force Dark -->
                        <div class="setting-card">
                            <div class="toggle-row">
                                <div>
                                    <div class="card-label">Smart Force Dark mode</div>
                                    <div class="card-hint">Converts bright webpages to dark while skipping already-dark sites and preserving images/videos.</div>
                                </div>
                                <button type="button" id="btnForceDarkToggle" class="switch-btn">
                                    <span class="switch-dot"></span>
                                    <span id="forceDarkLabel">Off</span>
                                </button>
                            </div>
                        </div>

                        <!-- Incognito Private Mode -->
                        <div class="setting-card">
                            <div class="toggle-row">
                                <div>
                                    <div class="card-label">Private / Incognito mode</div>
                                    <div class="card-hint">Pauses browsing history recording and tab session disk persistence while enabled.</div>
                                </div>
                                <button type="button" id="btnIncognitoToggle" class="switch-btn">
                                    <span class="switch-dot"></span>
                                    <span id="incognitoLabel">Off</span>
                                </button>
                            </div>
                        </div>

                        <!-- Zero-FX -->
                        <div class="setting-card">
                            <div class="toggle-row">
                                <div>
                                    <div class="card-label">Zero-FX rendering filter</div>
                                    <div class="card-hint">Strips heavy CSS shadows and backdrop blurs on external pages for peak frame pacing.</div>
                                </div>
                                <button type="button" id="btnPerfToggle" class="switch-btn on">
                                    <span class="switch-dot"></span>
                                    <span id="perfLabel">Enabled</span>
                                </button>
                            </div>
                        </div>

                        <!-- Session Restore -->
                        <div class="setting-card">
                            <div class="toggle-row">
                                <div>
                                    <div class="card-label">Restore tabs &amp; scroll position</div>
                                    <div class="card-hint">Automatically reopens previous tabs and restores reading scroll depth on startup.</div>
                                </div>
                                <button type="button" id="btnRestoreToggle" class="switch-btn on">
                                    <span class="switch-dot"></span>
                                    <span id="restoreLabel">Enabled</span>
                                </button>
                            </div>
                        </div>

                        <!-- Storage & Cache -->
                        <div class="setting-card full-span">
                            <div>
                                <div class="card-label">Persistent profile storage &amp; WebView cache</div>
                                <div class="card-hint">Profile path storing modular JSON files, cookies, and browsing data.</div>
                            </div>
                            <div class="storage-path" id="dataDirLabel">—</div>
                            <div class="storage-actions">
                                <button type="button" class="action-btn" id="btnClearHistory">Clear history</button>
                                <button type="button" class="action-btn danger" id="btnClearSiteData">Clear cache &amp; cookies</button>
                                <button type="button" class="action-btn desktop-only-inline" id="btnOpenDataDir">Open profile folder</button>
                                <button type="button" class="action-btn desktop-only-inline" id="btnOpenDownloadsDir">Open downloads</button>
                            </div>
                        </div>
                    </div>
                </section>

                <!-- 3. Adblock Lists & Per-Site CSS/JS Rules -->
                <section class="sec-block" id="sec-rules">
                    <div class="sec-header">
                        <div>
                            <h2 class="sec-title">Adblock lists &amp; per-site CSS / JS rules</h2>
                            <p class="sec-desc">Customize network domain blocking (<code style="font-family: monospace;">fynexo_adblock.json</code>) and lightweight Stylus/Userscript rules (<code style="font-family: monospace;">fynexo_rules.json</code>) with zero background memory.</p>
                        </div>
                    </div>

                    <div class="cards-grid">
                        <!-- Adblock & Domain Lists -->
                        <div class="setting-card full-span">
                            <div class="toggle-row" style="flex-wrap: wrap;">
                                <div>
                                    <div class="card-label">Network ad &amp; tracker shield (<code style="font-family: monospace; font-size: 12px;">fynexo_adblock.json</code>)</div>
                                    <div class="card-hint" id="adblockHint">Blocks 65+ built-in ad/telemetry networks plus your custom domains at native network layer.</div>
                                </div>
                                <div class="storage-actions">
                                    <input type="file" id="fileImportAdblock" accept="application/json,text/plain" style="display: none;">
                                    <button type="button" class="action-btn" id="btnImportAdblock">Import .json</button>
                                    <button type="button" class="action-btn" id="btnExportAdblock">Export .json</button>
                                    <button type="button" id="btnAdblockToggle" class="switch-btn on">
                                        <span class="switch-dot"></span>
                                        <span id="adblockLabel">Enabled</span>
                                    </button>
                                </div>
                            </div>

                            <div class="cards-grid" style="margin-top: 4px;">
                                <div style="display: flex; flex-direction: column; gap: 8px;">
                                    <div class="field-label"><span>Custom blocked domains</span><span class="field-tag">custom_blocked_domains</span></div>
                                    <div class="chips-wrap" id="blockedDomainsChips"></div>
                                    <div class="inline-form">
                                        <input type="text" id="inputBlockDomain" class="text-input mono" placeholder="ads.example.com">
                                        <button type="button" class="action-btn primary" id="btnAddBlockDomain">Block</button>
                                    </div>
                                </div>

                                <div style="display: flex; flex-direction: column; gap: 8px;">
                                    <div class="field-label"><span>Whitelisted domains (bypass adblock)</span><span class="field-tag">whitelisted_domains</span></div>
                                    <div class="chips-wrap" id="whitelistDomainsChips"></div>
                                    <div class="inline-form">
                                        <input type="text" id="inputWhiteDomain" class="text-input mono" placeholder="example.org">
                                        <button type="button" class="action-btn primary" id="btnAddWhiteDomain">Whitelist</button>
                                    </div>
                                </div>
                            </div>
                        </div>

                        <!-- Per-Site CSS & JS Rules Engine -->
                        <div class="setting-card full-span">
                            <div class="toggle-row" style="flex-wrap: wrap;">
                                <div>
                                    <div class="card-label">Per-site CSS &amp; JS rules (<code style="font-family: monospace; font-size: 12px;">fynexo_rules.json</code>)</div>
                                    <div class="card-hint">Inject custom CSS styles or lightweight JS snippets per domain (<code style="font-family: monospace;">*</code>, <code style="font-family: monospace;">github.com</code>, <code style="font-family: monospace;">*.zhihu.com</code>). Disabled rules use 0 memory.</div>
                                </div>
                                <div class="storage-actions">
                                    <input type="file" id="fileImportRules" accept="application/json,text/plain" style="display: none;">
                                    <button type="button" class="action-btn" id="btnImportRules">Import .json</button>
                                    <button type="button" class="action-btn" id="btnExportRules">Export .json</button>
                                    <button type="button" class="action-btn" id="btnResetRules">Reset</button>
                                    <button type="button" class="action-btn primary" id="btnNewRule">+ New rule</button>
                                </div>
                            </div>

                            <div class="rule-editor-panel" id="ruleEditorPanel">
                                <div class="inspector-grid">
                                    <div class="field-box">
                                        <label class="field-label" for="ruleNameInput"><span>Rule name</span></label>
                                        <input type="text" id="ruleNameInput" class="text-input" placeholder="Clean Distractions">
                                    </div>
                                    <div class="field-box">
                                        <label class="field-label" for="ruleDomainInput"><span>Domain pattern</span><span class="field-tag">* or example.com</span></label>
                                        <input type="text" id="ruleDomainInput" class="text-input mono" placeholder="*">
                                    </div>
                                </div>
                                <div class="field-box">
                                    <label class="field-label" for="ruleCssInput"><span>Custom CSS</span><span class="field-tag">Injected into &lt;style&gt;</span></label>
                                    <textarea id="ruleCssInput" class="code-area" spellcheck="false" placeholder=".sidebar-ad, .popup-banner {{ display: none !important; }}"></textarea>
                                </div>
                                <div class="field-box">
                                    <label class="field-label" for="ruleJsInput"><span>Custom JS (optional)</span><span class="field-tag">Runs on DOM ready</span></label>
                                    <textarea id="ruleJsInput" class="code-area" spellcheck="false" placeholder="// Optional lightweight userscript..."></textarea>
                                </div>
                                <div class="storage-actions">
                                    <button type="button" class="action-btn primary" id="btnSaveRule">Save rule</button>
                                    <button type="button" class="action-btn" id="btnCancelRule">Cancel</button>
                                </div>
                            </div>

                            <div class="rules-list" id="rulesList"></div>
                        </div>
                    </div>
                </section>

                <!-- 4. Component & Start Page Studio -->
                <section class="sec-block" id="sec-styles">
                    <div class="sec-header">
                        <div>
                            <h2 class="sec-title">Component &amp; start page studio</h2>
                            <p class="sec-desc">Tune chrome geometry and start page layout visually on the left, or edit <code style="font-family: monospace;">fynexo_styles.json</code> directly on the right.</p>
                        </div>
                    </div>

                    <div class="studio-split">
                        <div class="studio-inspector">
                            <div class="inspector-group">
                                <div class="inspector-heading">Desktop topbar &amp; omnibox</div>
                                <div class="inspector-toggle">
                                    <div>
                                        <div class="inspector-toggle-title">Show desktop topbar</div>
                                        <div class="inspector-toggle-key">topbar.visible_on_desktop</div>
                                    </div>
                                    <button type="button" id="qsDeskTopToggle" class="switch-btn on">
                                        <span class="switch-dot"></span>
                                        <span id="qsDeskTopLabel">Visible</span>
                                    </button>
                                </div>
                                <div class="inspector-grid">
                                    <div class="field-box">
                                        <label class="field-label" for="qsTopbarLayout"><span>Header layout</span><span class="field-tag">layout</span></label>
                                        <select id="qsTopbarLayout" class="select-input">
                                            <option value="deck">Layered deck (2-row)</option>
                                            <option value="compact">Compact single-row</option>
                                        </select>
                                    </div>
                                    <div class="field-box">
                                        <label class="field-label" for="qsBookmarksBar"><span>Shortcuts bar</span><span class="field-tag">bookmarks_bar</span></label>
                                        <select id="qsBookmarksBar" class="select-input">
                                            <option value="always">Always show</option>
                                            <option value="newtab">Start page only</option>
                                            <option value="hidden">Hidden</option>
                                        </select>
                                    </div>
                                    <div class="field-box">
                                        <label class="field-label" for="qsTopbarHeight"><span>Nav bar height</span><span class="field-tag">height</span></label>
                                        <input type="text" id="qsTopbarHeight" class="text-input mono" placeholder="38px">
                                    </div>
                                    <div class="field-box">
                                        <label class="field-label" for="qsTitlebarHeight"><span>Titlebar / tab deck height</span><span class="field-tag">titlebar_height</span></label>
                                        <input type="text" id="qsTitlebarHeight" class="text-input mono" placeholder="40px">
                                    </div>
                                    <div class="field-box">
                                        <label class="field-label" for="qsAddrMode"><span>Address bar style</span><span class="field-tag">mode</span></label>
                                        <select id="qsAddrMode" class="select-input">
                                            <option value="inline">Inline input</option>
                                            <option value="modal">Command modal</option>
                                        </select>
                                    </div>
                                    <div class="field-box">
                                        <label class="field-label" for="qsAddrPos"><span>Omnibox align (1-row)</span><span class="field-tag">position</span></label>
                                        <select id="qsAddrPos" class="select-input">
                                            <option value="center">Center</option>
                                            <option value="right">Right</option>
                                            <option value="left">Left</option>
                                        </select>
                                    </div>
                                    <div class="field-box">
                                        <label class="field-label" for="qsAddrWidth"><span>Address width (1-row)</span><span class="field-tag">width</span></label>
                                        <input type="text" id="qsAddrWidth" class="text-input mono" placeholder="340px">
                                    </div>
                                </div>
                            </div>

                            <div class="inspector-group">
                                <div class="inspector-heading">Tabs &amp; mobile dock</div>
                                <div class="inspector-grid">
                                    <div class="field-box">
                                        <label class="field-label" for="qsTabHeight"><span>Tab item height</span><span class="field-tag">tabs.height</span></label>
                                        <input type="text" id="qsTabHeight" class="text-input mono" placeholder="36px">
                                    </div>
                                    <div class="field-box">
                                        <label class="field-label" for="qsTabWidth"><span>Tab width</span><span class="field-tag">tabs.width</span></label>
                                        <input type="text" id="qsTabWidth" class="text-input mono" placeholder="152px">
                                    </div>
                                    <div class="field-box">
                                        <label class="field-label" for="qsTabRadius"><span>Tab radius</span><span class="field-tag">border_radius</span></label>
                                        <input type="text" id="qsTabRadius" class="text-input mono" placeholder="7px">
                                    </div>
                                    <div class="field-box">
                                        <label class="field-label" for="qsTabGap"><span>Tab spacing (gap)</span><span class="field-tag">tabs.gap</span></label>
                                        <input type="text" id="qsTabGap" class="text-input mono" placeholder="0px">
                                    </div>
                                    <div class="field-box">
                                        <label class="field-label" for="qsMobilePos"><span>Mobile dock</span><span class="field-tag">position</span></label>
                                        <select id="qsMobilePos" class="select-input">
                                            <option value="bottom">Bottom</option>
                                            <option value="top">Top</option>
                                        </select>
                                    </div>
                                    <div class="field-box">
                                        <label class="field-label" for="qsMobileHeight"><span>Dock height</span><span class="field-tag">height</span></label>
                                        <input type="text" id="qsMobileHeight" class="text-input mono" placeholder="54px">
                                    </div>
                                </div>
                                <div class="inspector-toggle">
                                    <div>
                                        <div class="inspector-toggle-title">Domain monogram badge</div>
                                        <div class="inspector-toggle-key">tabs.show_indicator_dot</div>
                                    </div>
                                    <button type="button" id="qsTabDotToggle" class="switch-btn on">
                                        <span class="switch-dot"></span>
                                        <span id="qsTabDotLabel">Shown</span>
                                    </button>
                                </div>
                                <div class="inspector-toggle">
                                    <div>
                                        <div class="inspector-toggle-title">Full-bleed preferences</div>
                                        <div class="inspector-toggle-key">settings_page.full_width</div>
                                    </div>
                                    <button type="button" id="qsFullWidthToggle" class="switch-btn on">
                                        <span class="switch-dot"></span>
                                        <span id="qsFullWidthLabel">Full</span>
                                    </button>
                                </div>
                            </div>

                            <div class="inspector-group">
                                <div class="inspector-heading">Start page (<code style="font-family: monospace; font-size: 11px;">fynexo://newtab</code>)</div>
                                <div class="inspector-grid">
                                    <div class="field-box">
                                        <label class="field-label" for="qsNtTitle"><span>Hero title</span><span class="field-tag">newtab.title</span></label>
                                        <input type="text" id="qsNtTitle" class="text-input" placeholder="Fynexo">
                                    </div>
                                    <div class="field-box">
                                        <label class="field-label" for="qsNtCols"><span>Grid columns</span><span class="field-tag">columns</span></label>
                                        <select id="qsNtCols" class="select-input">
                                            <option value="0">Auto responsive</option>
                                            <option value="2">2 columns</option>
                                            <option value="3">3 columns</option>
                                            <option value="4">4 columns</option>
                                            <option value="5">5 columns</option>
                                            <option value="6">6 columns</option>
                                        </select>
                                    </div>
                                    <div class="field-box span-2">
                                        <label class="field-label" for="qsNtSub"><span>Hero subtitle</span><span class="field-tag">newtab.subtitle</span></label>
                                        <input type="text" id="qsNtSub" class="text-input" placeholder="Single-WebView studio browser · Zero-FX compositing">
                                    </div>
                                </div>
                                <div class="inspector-toggle">
                                    <div>
                                        <div class="inspector-toggle-title">Show clock &amp; 24h format</div>
                                        <div class="inspector-toggle-key">newtab.show_clock / use_24h_clock</div>
                                    </div>
                                    <div class="storage-actions">
                                        <button type="button" id="qsNtClockToggle" class="switch-btn on"><span class="switch-dot"></span><span id="qsNtClockLabel">Clock</span></button>
                                        <button type="button" id="qsNt24hToggle" class="switch-btn on"><span class="switch-dot"></span><span id="qsNt24hLabel">24h</span></button>
                                    </div>
                                </div>
                                <div class="inspector-toggle">
                                    <div>
                                        <div class="inspector-toggle-title">Start page sections</div>
                                        <div class="inspector-toggle-key">show_shortcuts / show_history</div>
                                    </div>
                                    <div class="storage-actions">
                                        <button type="button" id="qsNtScToggle" class="switch-btn on"><span class="switch-dot"></span><span id="qsNtScLabel">Shortcuts</span></button>
                                        <button type="button" id="qsNtHistToggle" class="switch-btn on"><span class="switch-dot"></span><span id="qsNtHistLabel">History</span></button>
                                    </div>
                                </div>
                            </div>
                        </div>

                        <div class="json-studio">
                            <div class="json-toolbar">
                                <div class="json-file-meta">
                                    <span class="json-badge">JSON</span>
                                    <span class="json-path" id="stylesPathLabel">fynexo_styles.json</span>
                                </div>
                                <div class="json-actions">
                                    <button type="button" class="action-btn" id="btnReloadJson">Reload</button>
                                    <button type="button" class="action-btn" id="btnResetJson">Reset</button>
                                    <button type="button" class="action-btn primary" id="btnSaveJson">Apply &amp; save</button>
                                </div>
                            </div>
                            <textarea id="jsonEditor" class="json-editor" spellcheck="false"></textarea>
                            <div class="json-status" id="jsonStatus">
                                <span id="jsonStatusText">Ready — edit visual controls or modify raw JSON (Ctrl+S to apply).</span>
                                <span>Auto-reloads on window focus</span>
                            </div>
                        </div>
                    </div>
                </section>

                <!-- 5. Interactive Click-to-Record Keyboard Shortcuts -->
                <section class="sec-block desktop-only-sec" id="sec-shortcuts">
                    <div class="sec-header">
                        <div>
                            <h2 class="sec-title">Custom keyboard shortcuts (<code style="font-family: monospace; font-size: 13px;">fynexo_keybinds.json</code>)</h2>
                            <p class="sec-desc">Click any shortcut button below and press your desired key combination to rebind it instantly.</p>
                        </div>
                        <div class="theme-header-actions">
                            <input type="file" id="fileImportKeybinds" accept="application/json,text/plain" style="display: none;">
                            <button type="button" class="action-btn" id="btnImportKeybinds">Import .json</button>
                            <button type="button" class="action-btn" id="btnExportKeybinds">Export .json</button>
                            <button type="button" class="action-btn" id="btnResetKeybinds">Reset defaults</button>
                        </div>
                    </div>
                    <div class="shortcuts-grid" id="keybindsGrid"></div>
                </section>
            </div>
        </main>
    </div>

    <div class="toast" id="toast">
        <span style="width: 6px; height: 6px; border-radius: 50%; background: var(--fnx-accent);"></span>
        <span id="toastMsg">Saved</span>
    </div>

    <script>
        const DEFAULT_ENGINES = [
            {{ id: 'google', name: 'Google', url: 'https://www.google.com/search?q={{query}}' }},
            {{ id: 'duckduckgo', name: 'DuckDuckGo', url: 'https://duckduckgo.com/?q={{query}}' }},
            {{ id: 'bing', name: 'Bing', url: 'https://www.bing.com/search?q={{query}}' }},
            {{ id: 'baidu', name: 'Baidu', url: 'https://www.baidu.com/s?wd={{query}}' }}
        ];
        const DEFAULT_BANGS = [
            {{ trigger: '!gh', name: 'GitHub', url: 'https://github.com/search?q={{query}}' }},
            {{ trigger: '!yt', name: 'YouTube', url: 'https://www.youtube.com/results?search_query={{query}}' }},
            {{ trigger: '!b', name: 'Bilibili', url: 'https://search.bilibili.com/all?keyword={{query}}' }},
            {{ trigger: '!w', name: 'Wikipedia', url: 'https://en.wikipedia.org/w/index.php?search={{query}}' }},
            {{ trigger: '!rd', name: 'Reddit', url: 'https://www.reddit.com/search/?q={{query}}' }},
            {{ trigger: '!tr', name: 'Translate', url: 'https://translate.google.com/?sl=auto&tl=zh-CN&text={{query}}' }}
        ];
        const DEFAULT_KEYBINDS = {{
            omnibox: 'Ctrl+L',
            new_tab: 'Ctrl+T',
            close_tab: 'Ctrl+W',
            reopen_closed_tab: 'Ctrl+Shift+T',
            next_tab: 'Ctrl+Tab',
            bookmark_page: 'Ctrl+D',
            find_in_page: 'Ctrl+F',
            reader_mode: 'Alt+R',
            force_dark: 'Alt+Shift+D',
            fullscreen: 'F11',
            preferences: 'Ctrl+,',
            zoom_in: 'Ctrl+=',
            zoom_out: 'Ctrl+-',
            zoom_reset: 'Ctrl+0'
        }};
        const KEYBIND_DEFS = [
            ['omnibox', 'Open omnibox & quick bar'],
            ['new_tab', 'Open new tab'],
            ['close_tab', 'Close active tab'],
            ['reopen_closed_tab', 'Reopen closed tab'],
            ['next_tab', 'Cycle open tabs'],
            ['bookmark_page', 'Save / remove shortcut'],
            ['find_in_page', 'Find in page'],
            ['reader_mode', 'Toggle Reader mode'],
            ['force_dark', 'Toggle Smart Force Dark'],
            ['fullscreen', 'Immersive fullscreen'],
            ['preferences', 'Toggle Preferences'],
            ['zoom_in', 'Zoom in'],
            ['zoom_out', 'Zoom out'],
            ['zoom_reset', 'Reset zoom (100%)']
        ];
        const COLOR_TOKEN_DEFS = [
            ['bg', 'Canvas bg'],
            ['panel', 'Topbar / panel'],
            ['surface', 'Input surface'],
            ['elevated', 'Elevated card'],
            ['border', 'Border'],
            ['border_hover', 'Border hover'],
            ['fg', 'Primary text'],
            ['secondary', 'Secondary text'],
            ['muted', 'Muted text'],
            ['accent', 'Accent color'],
            ['accent_bg', 'Accent tint'],
            ['hover', 'Hover state']
        ];

        const boot = {payload_json} || {{}};
        let config = Object.assign({{
            theme: 'dark',
            enforce_perf: true,
            adblock: true,
            restore_session: true,
            force_dark: false,
            incognito: false,
            default_zoom: 1.0,
            user_agent_mode: 'default',
            custom_user_agent: '',
            search_engine: 'google',
            homepage: 'fynexo://newtab',
            search_engines: DEFAULT_ENGINES,
            bangs: DEFAULT_BANGS
        }}, boot.config || {{}});
        let styles = boot.styles || {{}};
        let activeTheme = boot.active_theme || {{}};
        let themes = Array.isArray(boot.themes) ? boot.themes : [];
        let bookmarks = Array.isArray(boot.bookmarks) ? boot.bookmarks : [];
        let siteRules = Array.isArray(boot.site_rules) ? boot.site_rules : [];
        let adblockCfg = Object.assign({{ custom_blocked_domains: [], whitelisted_domains: [] }}, boot.adblock || {{}});
        let keybinds = Object.assign({{}}, DEFAULT_KEYBINDS, boot.keybinds || {{}});

        let themesDir = boot.themes_dir || 'themes/';
        let stylesPath = boot.styles_path || 'fynexo_styles.json';
        let dataDir = boot.data_dir || '';
        let blockedAds = Number(boot.blocked_ads) || 0;
        let historyCount = Array.isArray(boot.history) ? boot.history.length : 0;

        let editingRuleId = null;
        let recordingActionKey = null;

        function getEngines() {{
            return (Array.isArray(config.search_engines) && config.search_engines.length > 0)
                ? config.search_engines
                : DEFAULT_ENGINES;
        }}
        function getBangs() {{
            return Array.isArray(config.bangs) ? config.bangs : DEFAULT_BANGS;
        }}

        function sendIpc(data) {{
            const raw = JSON.stringify(data);
            if (window.ipc && typeof window.ipc.postMessage === 'function') {{
                window.ipc.postMessage(raw);
            }} else if (window.__TAURI_INTERNALS__ && window.__TAURI_INTERNALS__.ipc) {{
                window.__TAURI_INTERNALS__.ipc.postMessage(raw);
            }}
        }}

        let toastTimer = null;
        function showToast(text) {{
            const t = document.getElementById('toast');
            const m = document.getElementById('toastMsg');
            m.textContent = text;
            t.style.display = 'inline-flex';
            if (toastTimer) clearTimeout(toastTimer);
            toastTimer = setTimeout(() => {{ t.style.display = 'none'; }}, 1800);
        }}

        function toSixDigitHex(raw) {{
            const s = String(raw || '').trim();
            if (/^#[0-9a-fA-F]{{6}}$/.test(s)) return s;
            if (/^#[0-9a-fA-F]{{3}}$/.test(s)) {{
                return '#' + s[1] + s[1] + s[2] + s[2] + s[3] + s[3];
            }}
            return '#09090b';
        }}

        function resolveThemePalette() {{
            const useCustom = !!(styles.colors && styles.colors.use_custom_colors);
            if (useCustom) {{
                const c = styles.colors;
                return {{
                    bg: c.bg || '#09090b',
                    panel: c.panel || '#0e0e11',
                    surface: c.surface || c.panel || '#141418',
                    elevated: c.elevated || '#1c1c22',
                    border: c.border || '#222228',
                    borderHover: c.border_hover || '#34343e',
                    fg: c.fg || '#f4f4f6',
                    secondary: c.secondary || '#a1a1aa',
                    muted: c.muted || '#71717a',
                    accent: c.accent || '#6366f1',
                    accentBg: c.accent_bg || '#1b1d36',
                    hover: c.hover || '#18181e'
                }};
            }}
            const matched = themes.find(item => item && item.id === config.theme);
            const src = (matched && matched.colors) ? matched.colors : (activeTheme || {{}});
            return {{
                bg: src.bg || '#09090b',
                panel: src.panel || '#0e0e11',
                surface: src.surface || src.panel || '#141418',
                elevated: src.elevated || '#1c1c22',
                border: src.border || '#222228',
                borderHover: src.border_hover || src.borderHover || '#34343e',
                fg: src.fg || '#f4f4f6',
                secondary: src.secondary || '#a1a1aa',
                muted: src.muted || '#71717a',
                accent: src.accent || '#6366f1',
                accentBg: src.accent_bg || src.accentBg || '#1b1d36',
                hover: src.hover || '#18181e'
            }};
        }}

        function renderColorTokensStudio() {{
            const studio = document.getElementById('paletteStudio');
            const grid = document.getElementById('colorTokensGrid');
            const useCustom = !!(styles.colors && styles.colors.use_custom_colors);
            studio.classList.toggle('open', useCustom);
            if (!useCustom) return;

            grid.textContent = '';
            styles.colors = styles.colors || {{}};
            const frag = document.createDocumentFragment();

            COLOR_TOKEN_DEFS.forEach(([key, label]) => {{
                const curVal = styles.colors[key] || '#09090b';
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

                picker.oninput = () => {{
                    styles.colors[key] = picker.value;
                    hexInp.value = picker.value;
                    applySettingsSurface();
                }};
                picker.onchange = () => {{
                    pushStyles(`Updated ${{key}}`);
                }};
                hexInp.onchange = () => {{
                    const v = hexInp.value.trim();
                    if (v) {{
                        styles.colors[key] = v;
                        picker.value = toSixDigitHex(v);
                        pushStyles(`Updated ${{key}}`);
                    }}
                }};

                meta.appendChild(lbl);
                meta.appendChild(hexInp);
                box.appendChild(picker);
                box.appendChild(meta);
                frag.appendChild(box);
            }});

            grid.appendChild(frag);
        }}

        function applySettingsSurface() {{
            const useCustom = !!(styles.colors && styles.colors.use_custom_colors);
            const t = resolveThemePalette();

            const root = document.documentElement;
            root.style.setProperty('--fnx-bg', t.bg);
            root.style.setProperty('--fnx-panel', t.panel);
            root.style.setProperty('--fnx-surface', t.surface || t.panel);
            root.style.setProperty('--fnx-elevated', t.elevated);
            root.style.setProperty('--fnx-border', t.border);
            root.style.setProperty('--fnx-border-hover', t.borderHover);
            root.style.setProperty('--fnx-fg', t.fg);
            root.style.setProperty('--fnx-secondary', t.secondary);
            root.style.setProperty('--fnx-muted', t.muted);
            root.style.setProperty('--fnx-accent', t.accent);
            root.style.setProperty('--fnx-accent-bg', t.accentBg);
            root.style.setProperty('--fnx-hover', t.hover);

            const sp = styles.settings_page || {{}};
            root.style.setProperty('--cfg-sidebar-w', sp.sidebar_width || '244px');
            root.style.setProperty('--cfg-content-pad', sp.content_padding || '28px 36px');
            root.style.setProperty('--cfg-card-radius', sp.card_radius || '10px');

            const appEl = document.getElementById('__fynexo_settings_app__');
            if (sp.full_width === false) appEl.classList.add('centered-mode');
            else appEl.classList.remove('centered-mode');

            const customCssEl = document.getElementById('__fynexo_settings_custom_css__');
            if (customCssEl) customCssEl.textContent = sp.custom_css || '';

            const customBtn = document.getElementById('btnCustomColorsToggle');
            const customLbl = document.getElementById('customColorsLabel');
            if (customBtn && customLbl) {{
                customBtn.classList.toggle('on', useCustom);
                customLbl.textContent = useCustom ? 'Custom palette: On' : 'Custom palette: Off';
            }}

            document.querySelectorAll('.theme-card').forEach(card => {{
                const active = !useCustom && card.dataset.theme === config.theme;
                card.classList.toggle('active', active);
            }});
        }}

        function renderThemeGrid() {{
            const themeGrid = document.getElementById('themeGrid');
            themeGrid.textContent = '';
            const useCustom = !!(styles.colors && styles.colors.use_custom_colors);
            const frag = document.createDocumentFragment();

            themes.forEach(item => {{
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
                tab1.style.border = `1px solid ${{c.border}}`;
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
                omni.style.border = `1px solid ${{c.border}}`;

                bar.appendChild(tabsWrap);
                bar.appendChild(omni);

                const body = document.createElement('div');
                body.className = 'mini-body';
                const col = document.createElement('div');
                col.className = 'mini-canvas-col';
                const b1 = document.createElement('div');
                b1.style.cssText = `width: 52%; height: 4px; border-radius: 2px; background: ${{borderHover}};`;
                const b2 = document.createElement('div');
                b2.style.cssText = `width: 34%; height: 3px; border-radius: 2px; background: ${{c.border}};`;
                col.appendChild(b1);
                col.appendChild(b2);

                const pill = document.createElement('div');
                pill.style.cssText = `width: 22px; height: 12px; border-radius: 3px; background: ${{accentBg}}; border: 1px solid ${{c.accent}};`;

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
                hx.textContent = `${{item.id}}.json`;
                rightWrap.appendChild(hx);

                if (!item.builtin) {{
                    const delBtn = document.createElement('button');
                    delBtn.type = 'button';
                    delBtn.className = 'theme-del-btn';
                    delBtn.title = `Delete custom theme ${{item.id}}.json`;
                    delBtn.textContent = '×';
                    delBtn.onclick = (e) => {{
                        e.stopPropagation();
                        sendIpc({{ type: 'DELETE_THEME', id: item.id }});
                    }};
                    rightWrap.appendChild(delBtn);
                }}

                info.appendChild(nameWrap);
                info.appendChild(rightWrap);

                const sub = document.createElement('div');
                sub.className = 'theme-sub';
                sub.textContent = item.description || 'Custom JSON palette';

                card.appendChild(preview);
                card.appendChild(info);
                card.appendChild(sub);

                const selectThisTheme = () => {{
                    config.theme = item.id;
                    activeTheme = c;
                    if (styles.colors && styles.colors.use_custom_colors) {{
                        styles.colors.use_custom_colors = false;
                        sendIpc({{ type: 'SAVE_STYLES', styles: styles }});
                        document.getElementById('jsonEditor').value = JSON.stringify(styles, null, 2);
                    }}
                    renderColorTokensStudio();
                    applySettingsSurface();
                    saveConfig(`Theme set to ${{item.name || item.id}}`);
                }};

                card.onclick = selectThisTheme;
                card.onkeydown = (e) => {{
                    if (e.key === 'Enter' || e.key === ' ') {{
                        e.preventDefault();
                        selectThisTheme();
                    }}
                }};

                frag.appendChild(card);
            }});

            themeGrid.appendChild(frag);
        }}

        function renderEnginePills() {{
            const enginePills = document.getElementById('enginePills');
            enginePills.textContent = '';
            const list = getEngines();
            const builtinIds = ['google', 'duckduckgo', 'bing', 'baidu'];
            const frag = document.createDocumentFragment();

            list.forEach(eng => {{
                const b = document.createElement('button');
                b.type = 'button';
                b.className = eng.id === config.search_engine ? 'seg-btn active' : 'seg-btn';
                b.dataset.engine = eng.id;
                b.title = eng.url || '';

                const lbl = document.createElement('span');
                lbl.textContent = eng.name || eng.id;
                b.appendChild(lbl);

                if (!builtinIds.includes(eng.id) && list.length > 1) {{
                    const del = document.createElement('span');
                    del.className = 'chip-x';
                    del.textContent = '×';
                    del.title = 'Remove custom search engine';
                    del.onclick = (e) => {{
                        e.stopPropagation();
                        config.search_engines = list.filter(x => x.id !== eng.id);
                        if (config.search_engine === eng.id) {{
                            config.search_engine = config.search_engines[0].id;
                        }}
                        renderEnginePills();
                        saveConfig('Removed search engine');
                    }};
                    b.appendChild(del);
                }}

                b.onclick = () => {{
                    config.search_engine = eng.id;
                    renderEnginePills();
                    saveConfig(`Search engine: ${{eng.name || eng.id}}`);
                }};
                frag.appendChild(b);
            }});
            enginePills.appendChild(frag);
        }}

        function renderBangsChips() {{
            const wrap = document.getElementById('bangsChips');
            wrap.textContent = '';
            const bangs = getBangs();
            const frag = document.createDocumentFragment();

            bangs.forEach((b, idx) => {{
                if (!b || !b.trigger) return;
                const chip = document.createElement('span');
                chip.className = 'tag-chip';
                chip.title = b.url || '';
                const code = document.createElement('code');
                code.textContent = b.trigger.startsWith('!') ? b.trigger : ('!' + b.trigger);
                const nm = document.createElement('span');
                nm.textContent = b.name || '';
                const del = document.createElement('button');
                del.type = 'button';
                del.className = 'chip-x';
                del.textContent = '×';
                del.title = 'Remove !bang';
                del.onclick = () => {{
                    config.bangs = bangs.filter((_, i) => i !== idx);
                    renderBangsChips();
                    saveConfig(`Removed ${{code.textContent}}`);
                }};
                chip.appendChild(code);
                if (b.name) chip.appendChild(nm);
                chip.appendChild(del);
                frag.appendChild(chip);
            }});
            wrap.appendChild(frag);
        }}

        function renderShortcutsChips() {{
            const wrap = document.getElementById('shortcutsChips');
            wrap.textContent = '';
            const frag = document.createDocumentFragment();

            bookmarks.forEach(bm => {{
                const chip = document.createElement('span');
                chip.className = 'tag-chip';
                chip.title = bm.url || '';
                const nm = document.createElement('span');
                nm.textContent = bm.title || bm.url;
                const del = document.createElement('button');
                del.type = 'button';
                del.className = 'chip-x';
                del.textContent = '×';
                del.title = 'Remove shortcut';
                del.onclick = () => {{
                    sendIpc({{ type: 'REMOVE_BOOKMARK', url: bm.url }});
                }};
                chip.appendChild(nm);
                chip.appendChild(del);
                frag.appendChild(chip);
            }});
            wrap.appendChild(frag);
        }}

        function renderAdblockDomainChips() {{
            const blockedWrap = document.getElementById('blockedDomainsChips');
            const whiteWrap = document.getElementById('whitelistDomainsChips');
            blockedWrap.textContent = '';
            whiteWrap.textContent = '';

            const bList = Array.isArray(adblockCfg.custom_blocked_domains) ? adblockCfg.custom_blocked_domains : [];
            const wList = Array.isArray(adblockCfg.whitelisted_domains) ? adblockCfg.whitelisted_domains : [];

            if (bList.length === 0) {{
                const empty = document.createElement('span');
                empty.style.cssText = 'font-size:11.5px;color:var(--fnx-muted);';
                empty.textContent = 'No custom blocked domains yet (65+ built-in ad domains active).';
                blockedWrap.appendChild(empty);
            }} else {{
                bList.forEach(dom => {{
                    const chip = document.createElement('span');
                    chip.className = 'tag-chip';
                    const code = document.createElement('code');
                    code.textContent = dom;
                    const del = document.createElement('button');
                    del.type = 'button';
                    del.className = 'chip-x';
                    del.textContent = '×';
                    del.onclick = () => {{
                        adblockCfg.custom_blocked_domains = bList.filter(x => x !== dom);
                        renderAdblockDomainChips();
                        sendIpc({{ type: 'SAVE_ADBLOCK', adblock: adblockCfg }});
                        showToast(`Unblocked ${{dom}}`);
                    }};
                    chip.appendChild(code);
                    chip.appendChild(del);
                    blockedWrap.appendChild(chip);
                }});
            }}

            if (wList.length === 0) {{
                const empty = document.createElement('span');
                empty.style.cssText = 'font-size:11.5px;color:var(--fnx-muted);';
                empty.textContent = 'No whitelisted domains.';
                whiteWrap.appendChild(empty);
            }} else {{
                wList.forEach(dom => {{
                    const chip = document.createElement('span');
                    chip.className = 'tag-chip';
                    const code = document.createElement('code');
                    code.textContent = dom;
                    const del = document.createElement('button');
                    del.type = 'button';
                    del.className = 'chip-x';
                    del.textContent = '×';
                    del.onclick = () => {{
                        adblockCfg.whitelisted_domains = wList.filter(x => x !== dom);
                        renderAdblockDomainChips();
                        sendIpc({{ type: 'SAVE_ADBLOCK', adblock: adblockCfg }});
                        showToast(`Removed ${{dom}} from whitelist`);
                    }};
                    chip.appendChild(code);
                    chip.appendChild(del);
                    whiteWrap.appendChild(chip);
                }});
            }}
        }}

        function renderSiteRulesList() {{
            const listEl = document.getElementById('rulesList');
            listEl.textContent = '';
            const frag = document.createDocumentFragment();

            if (siteRules.length === 0) {{
                const empty = document.createElement('div');
                empty.style.cssText = 'font-size:12px;color:var(--fnx-muted);padding:8px 2px;';
                empty.textContent = 'No per-site CSS/JS rules configured. Click "+ New rule" to create one.';
                listEl.appendChild(empty);
                return;
            }}

            siteRules.forEach((rule) => {{
                const row = document.createElement('div');
                row.className = 'rule-row';

                const info = document.createElement('div');
                info.className = 'rule-info';
                const nm = document.createElement('span');
                nm.className = 'rule-name';
                nm.textContent = rule.name || rule.id;
                const dom = document.createElement('span');
                dom.className = 'rule-domain';
                dom.textContent = rule.domain || '*';
                info.appendChild(nm);
                info.appendChild(dom);

                if (rule.css && rule.css.trim()) {{
                    const b = document.createElement('span');
                    b.className = 'rule-badge';
                    b.textContent = 'CSS';
                    info.appendChild(b);
                }}
                if (rule.js && rule.js.trim()) {{
                    const b = document.createElement('span');
                    b.className = 'rule-badge';
                    b.textContent = 'JS';
                    info.appendChild(b);
                }}

                const actions = document.createElement('div');
                actions.className = 'storage-actions';

                const sw = document.createElement('button');
                sw.type = 'button';
                sw.className = rule.enabled ? 'switch-btn on' : 'switch-btn';
                const dot = document.createElement('span');
                dot.className = 'switch-dot';
                const swLbl = document.createElement('span');
                swLbl.textContent = rule.enabled ? 'On' : 'Off';
                sw.appendChild(dot);
                sw.appendChild(swLbl);
                sw.onclick = () => {{
                    rule.enabled = !rule.enabled;
                    renderSiteRulesList();
                    sendIpc({{ type: 'SAVE_RULES', rules: siteRules }});
                    showToast(`${{rule.name || 'Rule'}}: ${{rule.enabled ? 'Enabled' : 'Disabled'}}`);
                }};

                const editBtn = document.createElement('button');
                editBtn.type = 'button';
                editBtn.className = 'action-btn';
                editBtn.textContent = 'Edit';
                editBtn.onclick = () => openRuleEditor(rule);

                const delBtn = document.createElement('button');
                delBtn.type = 'button';
                delBtn.className = 'action-btn danger';
                delBtn.textContent = 'Delete';
                delBtn.onclick = () => {{
                    siteRules = siteRules.filter(x => x.id !== rule.id);
                    renderSiteRulesList();
                    sendIpc({{ type: 'SAVE_RULES', rules: siteRules }});
                    showToast('Deleted rule');
                }};

                actions.appendChild(sw);
                actions.appendChild(editBtn);
                actions.appendChild(delBtn);

                row.appendChild(info);
                row.appendChild(actions);
                frag.appendChild(row);
            }});

            listEl.appendChild(frag);
        }}

        function openRuleEditor(existingRule) {{
            const panel = document.getElementById('ruleEditorPanel');
            panel.classList.add('open');
            if (existingRule) {{
                editingRuleId = existingRule.id;
                document.getElementById('ruleNameInput').value = existingRule.name || '';
                document.getElementById('ruleDomainInput').value = existingRule.domain || '*';
                document.getElementById('ruleCssInput').value = existingRule.css || '';
                document.getElementById('ruleJsInput').value = existingRule.js || '';
            }} else {{
                editingRuleId = null;
                document.getElementById('ruleNameInput').value = '';
                document.getElementById('ruleDomainInput').value = '*';
                document.getElementById('ruleCssInput').value = '';
                document.getElementById('ruleJsInput').value = '';
            }}
            document.getElementById('ruleNameInput').focus();
        }}

        function renderKeybindsGrid() {{
            const grid = document.getElementById('keybindsGrid');
            grid.textContent = '';
            const frag = document.createDocumentFragment();

            KEYBIND_DEFS.forEach(([actionKey, label]) => {{
                const card = document.createElement('div');
                card.className = 'shortcut-card';
                const lbl = document.createElement('span');
                lbl.className = 'shortcut-label';
                lbl.textContent = label;

                const btn = document.createElement('button');
                btn.type = 'button';
                const isRec = recordingActionKey === actionKey;
                btn.className = isRec ? 'kb-record-btn recording' : 'kb-record-btn';
                btn.textContent = isRec ? 'Press keys… (Esc)' : (keybinds[actionKey] || DEFAULT_KEYBINDS[actionKey] || 'Unbound');
                btn.title = 'Click to record a new shortcut';
                btn.onclick = () => {{
                    recordingActionKey = actionKey;
                    window.__fynexo_recording_keybind__ = true;
                    renderKeybindsGrid();
                }};

                card.appendChild(lbl);
                card.appendChild(btn);
                frag.appendChild(card);
            }});

            grid.appendChild(frag);
        }}

        window.addEventListener('keydown', (e) => {{
            if (!recordingActionKey) return;
            e.preventDefault();
            e.stopPropagation();

            if (e.key === 'Escape') {{
                recordingActionKey = null;
                window.__fynexo_recording_keybind__ = false;
                renderKeybindsGrid();
                return;
            }}

            if (['Control', 'Shift', 'Alt', 'Meta'].includes(e.key)) {{
                return;
            }}

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
            sendIpc({{ type: 'SAVE_KEYBINDS', keybinds: keybinds }});
            showToast(`Bound shortcut: ${{combo}}`);
        }}, true);

        function syncFormControls() {{
            document.getElementById('themesDirLabel').textContent = themesDir;
            document.getElementById('stylesPathLabel').textContent = stylesPath;
            document.getElementById('dataDirLabel').textContent = dataDir || stylesPath;
            document.getElementById('shortcutsHint').textContent =
                `${{bookmarks.length}} saved shortcut${{bookmarks.length === 1 ? '' : 's'}} — synced to fynexo_shortcuts.json.`;

            document.getElementById('inputHomepage').value = config.homepage || 'fynexo://newtab';

            // User-Agent & Zoom
            const uaMode = (config.user_agent_mode || 'default').toLowerCase();
            document.querySelectorAll('#uaModeGroup .seg-btn').forEach(b => {{
                b.classList.toggle('active', b.dataset.ua === uaMode);
            }});
            const customUaInp = document.getElementById('inputCustomUa');
            customUaInp.style.display = uaMode === 'custom' ? 'block' : 'none';
            customUaInp.value = config.custom_user_agent || '';

            const zoomPct = Math.round((Number(config.default_zoom) || 1) * 100);
            document.getElementById('btnZoomReset').textContent = `${{zoomPct}}%`;

            // Force Dark & Incognito
            const fdOn = !!config.force_dark;
            document.getElementById('btnForceDarkToggle').classList.toggle('on', fdOn);
            document.getElementById('forceDarkLabel').textContent = fdOn ? 'Enabled' : 'Off';

            const incOn = !!config.incognito;
            document.getElementById('btnIncognitoToggle').classList.toggle('on', incOn);
            document.getElementById('incognitoLabel').textContent = incOn ? 'Active' : 'Off';
            document.getElementById('metaIncognito').textContent = incOn ? 'Active' : 'Off';

            // Zero-FX
            const perfOn = !!config.enforce_perf;
            document.getElementById('btnPerfToggle').classList.toggle('on', perfOn);
            document.getElementById('perfLabel').textContent = perfOn ? 'Enabled' : 'Disabled';
            document.getElementById('metaPerf').textContent = perfOn ? 'Enabled' : 'Disabled';
            const metaDot = document.getElementById('metaPerfDot');
            if (metaDot) metaDot.classList.toggle('off', !perfOn);

            // Adblock
            const adOn = config.adblock !== false;
            document.getElementById('btnAdblockToggle').classList.toggle('on', adOn);
            document.getElementById('adblockLabel').textContent = adOn ? 'Enabled' : 'Paused';
            document.getElementById('adblockHint').textContent = adOn
                ? `Blocks 65+ built-in ad/telemetry domains plus custom rules (${{blockedAds}} blocked this session).`
                : 'Network ad & tracker interception is currently paused.';
            document.getElementById('metaAdblock').textContent = adOn
                ? (blockedAds > 0 ? `${{blockedAds}} blocked` : 'Active')
                : 'Paused';
            const metaAdDot = document.getElementById('metaAdblockDot');
            if (metaAdDot) metaAdDot.classList.toggle('off', !adOn);

            // Restore Session
            const restOn = config.restore_session !== false;
            document.getElementById('btnRestoreToggle').classList.toggle('on', restOn);
            document.getElementById('restoreLabel').textContent = restOn ? 'Enabled' : 'Disabled';

            document.getElementById('btnClearHistory').textContent = historyCount > 0
                ? `Clear history (${{historyCount}})`
                : 'Clear history';

            // Studio Inspector controls
            const tb = styles.topbar || {{}};
            const deskTopVis = tb.visible_on_desktop !== false;
            document.getElementById('qsDeskTopToggle').classList.toggle('on', deskTopVis);
            document.getElementById('qsDeskTopLabel').textContent = deskTopVis ? 'Visible' : 'Hidden';
            document.getElementById('qsTopbarLayout').value = (tb.layout || 'deck').toLowerCase() === 'compact' ? 'compact' : 'deck';
            document.getElementById('qsBookmarksBar').value = ['always', 'newtab', 'hidden'].includes((tb.bookmarks_bar || '').toLowerCase()) ? tb.bookmarks_bar.toLowerCase() : 'always';
            document.getElementById('qsTopbarHeight').value = tb.height || '38px';
            const qsTitlebarInput = document.getElementById('qsTitlebarHeight');
            if (qsTitlebarInput) {{
                qsTitlebarInput.value = tb.titlebar_height || '40px';
            }}

            if (styles.address_bar) {{
                document.getElementById('qsAddrMode').value = (styles.address_bar.mode || 'inline').toLowerCase() === 'modal' ? 'modal' : 'inline';
                document.getElementById('qsAddrPos').value = styles.address_bar.position || 'center';
                document.getElementById('qsAddrWidth').value = styles.address_bar.width || '340px';
            }}
            if (styles.tabs) {{
                const tabH = document.getElementById('qsTabHeight');
                if (tabH) tabH.value = styles.tabs.height || '36px';
                document.getElementById('qsTabWidth').value = styles.tabs.width || '152px';
                document.getElementById('qsTabRadius').value = styles.tabs.border_radius || '7px';
                const tabGap = document.getElementById('qsTabGap');
                if (tabGap) tabGap.value = styles.tabs.gap || '0px';
                const showDot = styles.tabs.show_indicator_dot !== false;
                document.getElementById('qsTabDotToggle').classList.toggle('on', showDot);
                document.getElementById('qsTabDotLabel').textContent = showDot ? 'Shown' : 'Hidden';
            }}
            if (styles.mobile_bar) {{
                document.getElementById('qsMobilePos').value = styles.mobile_bar.position || 'bottom';
                document.getElementById('qsMobileHeight').value = styles.mobile_bar.height || '54px';
            }}
            const fullW = !styles.settings_page || styles.settings_page.full_width !== false;
            document.getElementById('qsFullWidthToggle').classList.toggle('on', fullW);
            document.getElementById('qsFullWidthLabel').textContent = fullW ? 'Full' : 'Centered';

            // Newtab Start Page Studio controls
            const nt = styles.newtab || {{}};
            document.getElementById('qsNtTitle').value = nt.title !== undefined ? nt.title : 'Fynexo';
            document.getElementById('qsNtSub').value = nt.subtitle !== undefined ? nt.subtitle : 'Single-WebView studio browser · Zero-FX compositing';
            document.getElementById('qsNtCols').value = String(Number(nt.shortcuts_columns) || 0);
            const ntClock = nt.show_clock !== false;
            const nt24 = nt.use_24h_clock !== false;
            const ntSc = nt.show_shortcuts !== false;
            const ntHi = nt.show_history !== false;
            document.getElementById('qsNtClockToggle').classList.toggle('on', ntClock);
            document.getElementById('qsNt24hToggle').classList.toggle('on', nt24);
            document.getElementById('qsNtScToggle').classList.toggle('on', ntSc);
            document.getElementById('qsNtHistToggle').classList.toggle('on', ntHi);

            const editor = document.getElementById('jsonEditor');
            if (document.activeElement !== editor) {{
                editor.value = JSON.stringify(styles, null, 2);
            }}
            setJsonStatus(false, 'Ready — edit visual controls or modify raw JSON (Ctrl+S to apply).');
        }}

        function setJsonStatus(isError, msg) {{
            const box = document.getElementById('jsonStatus');
            const txt = document.getElementById('jsonStatusText');
            box.classList.toggle('error', isError);
            txt.textContent = msg;
        }}

        function saveConfig(msg) {{
            sendIpc({{ type: 'SAVE_CONFIG', config: config }});
            showToast(msg || 'Preferences saved');
        }}

        function pushStyles(msg) {{
            document.getElementById('jsonEditor').value = JSON.stringify(styles, null, 2);
            applySettingsSurface();
            syncFormControls();
            sendIpc({{ type: 'SAVE_STYLES', styles: styles }});
            setJsonStatus(false, 'Applied live and persisted to fynexo_styles.json.');
            showToast(msg || 'Styles updated');
        }}

        function bindJsonFilePicker(btnId, inputId, ipcType, extraProps) {{
            const inp = document.getElementById(inputId);
            document.getElementById(btnId).onclick = () => inp.click();
            inp.addEventListener('change', (e) => {{
                const files = Array.from(e.target.files || []);
                files.forEach(file => {{
                    const reader = new FileReader();
                    reader.onload = () => {{
                        sendIpc(Object.assign({{
                            type: ipcType,
                            json: String(reader.result || ''),
                            filename: file.name || 'custom.json'
                        }}, extraProps || {{}}));
                    }};
                    reader.readAsText(file, 'utf-8');
                }});
                e.target.value = '';
            }});
        }}

        // Navigation Back
        document.getElementById('btnBack').onclick = () => {{
            if (window.history.length > 1) sendIpc({{ type: 'GO_BACK' }});
            else sendIpc({{ type: 'NAVIGATE', url: config.homepage || 'fynexo://newtab' }});
        }};

        // Full Setup Bundle Export / Import
        document.getElementById('btnExportBundle').onclick = () => sendIpc({{ type: 'EXPORT_FULL_SETUP' }});
        bindJsonFilePicker('btnImportBundle', 'fileImportBundle', 'IMPORT_FULL_SETUP_JSON');

        // Themes Import / Export / Folder / Custom Palette
        bindJsonFilePicker('btnImportTheme', 'fileImportTheme', 'IMPORT_THEME_JSON');
        document.getElementById('btnExportTheme').onclick = () => sendIpc({{ type: 'EXPORT_THEME', id: config.theme }});
        document.getElementById('btnOpenThemesDir').onclick = () => sendIpc({{ type: 'OPEN_THEMES_DIR' }});

        document.getElementById('btnCustomColorsToggle').onclick = () => {{
            styles.colors = styles.colors || {{}};
            if (!styles.colors.use_custom_colors) {{
                const cur = resolveThemePalette();
                styles.colors = Object.assign({{
                    bg: cur.bg, panel: cur.panel, surface: cur.surface, elevated: cur.elevated,
                    border: cur.border, border_hover: cur.borderHover, fg: cur.fg,
                    secondary: cur.secondary, muted: cur.muted, accent: cur.accent,
                    accent_bg: cur.accentBg, hover: cur.hover
                }}, styles.colors, {{ use_custom_colors: true }});
            }} else {{
                styles.colors.use_custom_colors = false;
            }}
            renderColorTokensStudio();
            pushStyles(styles.colors.use_custom_colors ? 'Visual color designer enabled' : 'Using JSON theme preset');
        }};

        document.getElementById('btnSaveAsTheme').onclick = () => {{
            const rawName = document.getElementById('inputNewThemeName').value.trim() || 'My Studio Theme';
            const slug = rawName.toLowerCase().replace(/[^a-z0-9_-]+/g, '-').replace(/^-|-$/g, '') || 'custom-theme';
            const presetObj = {{
                id: slug,
                name: rawName,
                description: 'Created in Fynexo Visual Palette Designer',
                colors: styles.colors || resolveThemePalette()
            }};
            sendIpc({{
                type: 'IMPORT_THEME_JSON',
                json: JSON.stringify(presetObj, null, 2),
                filename: slug + '.json'
            }});
            document.getElementById('inputNewThemeName').value = '';
        }};

        // Add Custom Search Engine
        document.getElementById('btnAddEngine').onclick = () => {{
            const nm = document.getElementById('inputEngName').value.trim();
            const u = document.getElementById('inputEngUrl').value.trim();
            if (!nm || !u) return showToast('Enter engine name and URL');
            const id = nm.toLowerCase().replace(/[^a-z0-9_-]+/g, '-') || ('eng-' + Date.now());
            const list = getEngines().slice();
            const fullUrl = /^https?:\/\//i.test(u) ? u : ('https://' + u);
            const existingIdx = list.findIndex(x => x.id === id);
            if (existingIdx >= 0) list[existingIdx] = {{ id, name: nm, url: fullUrl }};
            else list.push({{ id, name: nm, url: fullUrl }});
            config.search_engines = list;
            config.search_engine = id;
            document.getElementById('inputEngName').value = '';
            document.getElementById('inputEngUrl').value = '';
            renderEnginePills();
            saveConfig(`Added search engine: ${{nm}}`);
        }};

        // Add Bang Shortcut
        document.getElementById('btnAddBang').onclick = () => {{
            let trig = document.getElementById('inputBangTrig').value.trim().toLowerCase();
            const nm = document.getElementById('inputBangName').value.trim();
            const u = document.getElementById('inputBangUrl').value.trim();
            if (!trig || !u) return showToast('Enter !trigger and search URL');
            if (!trig.startsWith('!')) trig = '!' + trig;
            const fullUrl = /^https?:\/\//i.test(u) ? u : ('https://' + u);
            const list = getBangs().filter(x => (x.trigger || '').toLowerCase() !== trig);
            list.push({{ trigger: trig, name: nm || trig.slice(1), url: fullUrl }});
            config.bangs = list;
            document.getElementById('inputBangTrig').value = '';
            document.getElementById('inputBangName').value = '';
            document.getElementById('inputBangUrl').value = '';
            renderBangsChips();
            saveConfig(`Saved bang ${{trig}}`);
        }};

        // Homepage
        const inputHp = document.getElementById('inputHomepage');
        inputHp.onchange = () => {{
            const val = inputHp.value.trim();
            if (val) {{ config.homepage = val; saveConfig('Homepage updated'); }}
        }};
        document.querySelectorAll('.preset-chip').forEach(chip => {{
            chip.onclick = () => {{
                const hp = chip.dataset.hp;
                if (hp) {{ config.homepage = hp; inputHp.value = hp; saveConfig(`Homepage: ${{hp}}`); }}
            }};
        }});

        // User-Agent & Default Zoom
        document.querySelectorAll('#uaModeGroup .seg-btn').forEach(btn => {{
            btn.onclick = () => {{
                config.user_agent_mode = btn.dataset.ua;
                syncFormControls();
                saveConfig(`User-Agent mode: ${{btn.textContent}}`);
            }};
        }});
        document.getElementById('inputCustomUa').onchange = (e) => {{
            config.custom_user_agent = e.target.value.trim();
            saveConfig('Custom User-Agent updated');
        }};
        document.getElementById('btnZoomOut').onclick = () => sendIpc({{ type: 'SET_ZOOM', delta: -0.1 }});
        document.getElementById('btnZoomReset').onclick = () => sendIpc({{ type: 'SET_ZOOM', zoom: 1.0 }});
        document.getElementById('btnZoomIn').onclick = () => sendIpc({{ type: 'SET_ZOOM', delta: 0.1 }});

        // Shortcut Websites
        bindJsonFilePicker('btnImportShortcuts', 'fileImportShortcuts', 'IMPORT_SHORTCUTS_JSON', {{ merge: true }});
        document.getElementById('btnExportShortcuts').onclick = () => sendIpc({{ type: 'EXPORT_SHORTCUTS' }});
        document.getElementById('btnResetShortcuts').onclick = () => sendIpc({{ type: 'RESET_SHORTCUTS' }});
        document.getElementById('btnAddShortcut').onclick = () => {{
            const u = document.getElementById('inputScUrl').value.trim();
            const t = document.getElementById('inputScTitle').value.trim();
            if (!u) return showToast('Enter shortcut URL');
            sendIpc({{ type: 'ADD_BOOKMARK', title: t, url: u }});
            document.getElementById('inputScTitle').value = '';
            document.getElementById('inputScUrl').value = '';
        }};

        // Mode Toggles
        document.getElementById('btnForceDarkToggle').onclick = () => {{
            config.force_dark = !config.force_dark;
            syncFormControls();
            saveConfig(config.force_dark ? 'Smart Force Dark enabled' : 'Smart Force Dark off');
        }};
        document.getElementById('btnIncognitoToggle').onclick = () => {{
            config.incognito = !config.incognito;
            syncFormControls();
            saveConfig(config.incognito ? 'Private mode active' : 'Private mode off');
        }};
        document.getElementById('btnPerfToggle').onclick = () => {{
            config.enforce_perf = !config.enforce_perf;
            syncFormControls();
            saveConfig(config.enforce_perf ? 'Zero-FX filter enabled' : 'Zero-FX filter disabled');
        }};
        document.getElementById('btnRestoreToggle').onclick = () => {{
            config.restore_session = !(config.restore_session !== false);
            syncFormControls();
            saveConfig(config.restore_session ? 'Session restore enabled' : 'Session restore disabled');
        }};

        // Storage & Cache
        document.getElementById('btnClearHistory').onclick = () => {{
            sendIpc({{ type: 'CLEAR_HISTORY' }});
            historyCount = 0;
            syncFormControls();
        }};
        document.getElementById('btnClearSiteData').onclick = () => {{
            sendIpc({{ type: 'CLEAR_SITE_DATA' }});
        }};
        document.getElementById('btnOpenDataDir').onclick = () => sendIpc({{ type: 'OPEN_DATA_DIR' }});
        document.getElementById('btnOpenDownloadsDir').onclick = () => sendIpc({{ type: 'OPEN_DOWNLOADS_DIR' }});

        // Adblock Lists
        bindJsonFilePicker('btnImportAdblock', 'fileImportAdblock', 'IMPORT_ADBLOCK_JSON');
        document.getElementById('btnExportAdblock').onclick = () => sendIpc({{ type: 'EXPORT_ADBLOCK' }});
        document.getElementById('btnAdblockToggle').onclick = () => {{
            config.adblock = !(config.adblock !== false);
            syncFormControls();
            saveConfig(config.adblock ? 'Network adblock enabled' : 'Network adblock paused');
        }};
        document.getElementById('btnAddBlockDomain').onclick = () => {{
            const inp = document.getElementById('inputBlockDomain');
            const v = inp.value.trim().toLowerCase().replace(/^https?:\/\//, '').split('/')[0];
            if (!v || !v.includes('.')) return showToast('Enter a valid domain (e.g. ads.example.com)');
            adblockCfg.custom_blocked_domains = adblockCfg.custom_blocked_domains || [];
            if (!adblockCfg.custom_blocked_domains.includes(v)) adblockCfg.custom_blocked_domains.push(v);
            inp.value = '';
            renderAdblockDomainChips();
            sendIpc({{ type: 'SAVE_ADBLOCK', adblock: adblockCfg }});
            showToast(`Blocked ${{v}}`);
        }};
        document.getElementById('btnAddWhiteDomain').onclick = () => {{
            const inp = document.getElementById('inputWhiteDomain');
            const v = inp.value.trim().toLowerCase().replace(/^https?:\/\//, '').split('/')[0];
            if (!v || !v.includes('.')) return showToast('Enter a valid domain (e.g. example.org)');
            adblockCfg.whitelisted_domains = adblockCfg.whitelisted_domains || [];
            if (!adblockCfg.whitelisted_domains.includes(v)) adblockCfg.whitelisted_domains.push(v);
            inp.value = '';
            renderAdblockDomainChips();
            sendIpc({{ type: 'SAVE_ADBLOCK', adblock: adblockCfg }});
            showToast(`Whitelisted ${{v}}`);
        }};

        // Per-Site Rules
        bindJsonFilePicker('btnImportRules', 'fileImportRules', 'IMPORT_RULES_JSON');
        document.getElementById('btnExportRules').onclick = () => sendIpc({{ type: 'EXPORT_RULES' }});
        document.getElementById('btnResetRules').onclick = () => sendIpc({{ type: 'RESET_RULES' }});
        document.getElementById('btnNewRule').onclick = () => openRuleEditor(null);
        document.getElementById('btnCancelRule').onclick = () => {{
            document.getElementById('ruleEditorPanel').classList.remove('open');
            editingRuleId = null;
        }};
        document.getElementById('btnSaveRule').onclick = () => {{
            const nm = document.getElementById('ruleNameInput').value.trim() || 'Custom rule';
            const dom = document.getElementById('ruleDomainInput').value.trim() || '*';
            const css = document.getElementById('ruleCssInput').value;
            const js = document.getElementById('ruleJsInput').value;
            if (!css.trim() && !js.trim()) return showToast('Provide CSS or JS for this rule');

            if (editingRuleId) {{
                const idx = siteRules.findIndex(r => r.id === editingRuleId);
                if (idx >= 0) {{
                    siteRules[idx] = Object.assign({{}}, siteRules[idx], {{ name: nm, domain: dom, css: css, js: js }});
                }}
            }} else {{
                const id = nm.toLowerCase().replace(/[^a-z0-9_-]+/g, '-') + '-' + (Date.now() % 10000);
                siteRules.push({{ id: id, name: nm, domain: dom, enabled: true, css: css, js: js }});
            }}
            editingRuleId = null;
            document.getElementById('ruleEditorPanel').classList.remove('open');
            renderSiteRulesList();
            sendIpc({{ type: 'SAVE_RULES', rules: siteRules }});
            showToast(`Saved rule: ${{nm}}`);
        }};

        // Keybinds Import / Export / Reset
        bindJsonFilePicker('btnImportKeybinds', 'fileImportKeybinds', 'IMPORT_KEYBINDS_JSON');
        document.getElementById('btnExportKeybinds').onclick = () => sendIpc({{ type: 'EXPORT_KEYBINDS' }});
        document.getElementById('btnResetKeybinds').onclick = () => sendIpc({{ type: 'RESET_KEYBINDS' }});

        // Component & Start Page Studio Inspector Bindings
        document.getElementById('qsDeskTopToggle').onclick = () => {{
            styles.topbar = styles.topbar || {{}};
            styles.topbar.visible_on_desktop = styles.topbar.visible_on_desktop === false ? true : false;
            pushStyles(styles.topbar.visible_on_desktop ? 'Desktop topbar shown' : 'Desktop topbar hidden');
        }};
        document.getElementById('qsTopbarLayout').onchange = (e) => {{
            styles.topbar = styles.topbar || {{}};
            styles.topbar.layout = e.target.value;
            pushStyles(e.target.value === 'deck' ? 'Switched to 2-row layered deck' : 'Switched to single-row compact bar');
        }};
        document.getElementById('qsBookmarksBar').onchange = (e) => {{
            styles.topbar = styles.topbar || {{}};
            styles.topbar.bookmarks_bar = e.target.value;
            pushStyles('Shortcuts bar: ' + e.target.value);
        }};
        document.getElementById('qsTopbarHeight').onchange = (e) => {{
            styles.topbar = styles.topbar || {{}};
            styles.topbar.height = e.target.value.trim() || '38px';
            pushStyles('Nav bar height updated');
        }};
        const qsTitlebarInput = document.getElementById('qsTitlebarHeight');
        if (qsTitlebarInput) {{
            qsTitlebarInput.onchange = (e) => {{
                styles.topbar = styles.topbar || {{}};
                styles.topbar.titlebar_height = e.target.value.trim() || '40px';
                pushStyles('Titlebar height updated');
            }};
        }}
        document.getElementById('qsAddrMode').onchange = (e) => {{
            styles.address_bar = styles.address_bar || {{}};
            styles.address_bar.mode = e.target.value;
            pushStyles('Address bar mode: ' + e.target.value);
        }};
        document.getElementById('qsAddrPos').onchange = (e) => {{
            styles.address_bar = styles.address_bar || {{}};
            styles.address_bar.position = e.target.value;
            pushStyles('Omnibox aligned ' + e.target.value);
        }};
        document.getElementById('qsAddrWidth').onchange = (e) => {{
            styles.address_bar = styles.address_bar || {{}};
            styles.address_bar.width = e.target.value.trim() || '320px';
            pushStyles('Address capsule width updated');
        }};
        const tabHInput = document.getElementById('qsTabHeight');
        if (tabHInput) {{
            tabHInput.onchange = (e) => {{
                styles.tabs = styles.tabs || {{}};
                styles.tabs.height = e.target.value.trim() || '36px';
                pushStyles('Tab item height updated');
            }};
        }}
        document.getElementById('qsTabWidth').onchange = (e) => {{
            styles.tabs = styles.tabs || {{}};
            styles.tabs.width = e.target.value.trim() || '152px';
            pushStyles('Tab width updated');
        }};
        document.getElementById('qsTabRadius').onchange = (e) => {{
            styles.tabs = styles.tabs || {{}};
            styles.tabs.border_radius = e.target.value.trim() || '7px';
            pushStyles('Tab corner radius updated');
        }};
        const tabGapInput = document.getElementById('qsTabGap');
        if (tabGapInput) {{
            tabGapInput.onchange = (e) => {{
                styles.tabs = styles.tabs || {{}};
                styles.tabs.gap = e.target.value.trim() || '0px';
                pushStyles('Tab spacing (gap) updated');
            }};
        }}
        document.getElementById('qsTabDotToggle').onclick = () => {{
            styles.tabs = styles.tabs || {{}};
            styles.tabs.show_indicator_dot = styles.tabs.show_indicator_dot === false ? true : false;
            pushStyles('Updated tab badge');
        }};
        document.getElementById('qsMobilePos').onchange = (e) => {{
            styles.mobile_bar = styles.mobile_bar || {{}};
            styles.mobile_bar.position = e.target.value;
            pushStyles('Mobile bar docked to ' + e.target.value);
        }};
        document.getElementById('qsMobileHeight').onchange = (e) => {{
            styles.mobile_bar = styles.mobile_bar || {{}};
            styles.mobile_bar.height = e.target.value.trim() || '54px';
            pushStyles('Mobile bar height updated');
        }};
        document.getElementById('qsFullWidthToggle').onclick = () => {{
            styles.settings_page = styles.settings_page || {{}};
            styles.settings_page.full_width = styles.settings_page.full_width === false ? true : false;
            pushStyles('Workspace width updated');
        }};

        // Start Page (newtab) Inspector Bindings
        document.getElementById('qsNtTitle').onchange = (e) => {{
            styles.newtab = styles.newtab || {{}};
            styles.newtab.title = e.target.value;
            pushStyles('Start page title updated');
        }};
        document.getElementById('qsNtSub').onchange = (e) => {{
            styles.newtab = styles.newtab || {{}};
            styles.newtab.subtitle = e.target.value;
            pushStyles('Start page subtitle updated');
        }};
        document.getElementById('qsNtCols').onchange = (e) => {{
            styles.newtab = styles.newtab || {{}};
            styles.newtab.shortcuts_columns = Number(e.target.value) || 0;
            pushStyles('Shortcuts grid columns updated');
        }};
        document.getElementById('qsNtClockToggle').onclick = () => {{
            styles.newtab = styles.newtab || {{}};
            styles.newtab.show_clock = styles.newtab.show_clock === false ? true : false;
            pushStyles('Start page clock updated');
        }};
        document.getElementById('qsNt24hToggle').onclick = () => {{
            styles.newtab = styles.newtab || {{}};
            styles.newtab.use_24h_clock = styles.newtab.use_24h_clock === false ? true : false;
            pushStyles('Clock format updated');
        }};
        document.getElementById('qsNtScToggle').onclick = () => {{
            styles.newtab = styles.newtab || {{}};
            styles.newtab.show_shortcuts = styles.newtab.show_shortcuts === false ? true : false;
            pushStyles('Start page shortcuts visibility updated');
        }};
        document.getElementById('qsNtHistToggle').onclick = () => {{
            styles.newtab = styles.newtab || {{}};
            styles.newtab.show_history = styles.newtab.show_history === false ? true : false;
            pushStyles('Start page history visibility updated');
        }};

        // Live JSON Editor
        const jsonEditorEl = document.getElementById('jsonEditor');
        let validateTimer = null;
        jsonEditorEl.addEventListener('input', () => {{
            if (validateTimer) clearTimeout(validateTimer);
            validateTimer = setTimeout(() => {{
                try {{
                    JSON.parse(jsonEditorEl.value);
                    setJsonStatus(false, 'Syntax valid — click Apply & save or press Ctrl+S.');
                }} catch (err) {{
                    setJsonStatus(true, 'Syntax error: ' + err.message);
                }}
            }}, 180);
        }});

        document.getElementById('btnSaveJson').onclick = () => {{
            try {{
                const parsed = JSON.parse(jsonEditorEl.value);
                styles = parsed;
                renderColorTokensStudio();
                applySettingsSurface();
                syncFormControls();
                jsonEditorEl.value = JSON.stringify(styles, null, 2);
                sendIpc({{ type: 'SAVE_STYLES', styles: styles }});
                setJsonStatus(false, 'Valid JSON — applied live and saved to disk.');
                showToast('Saved fynexo_styles.json');
            }} catch (err) {{
                setJsonStatus(true, 'Invalid JSON: ' + err.message);
            }}
        }};

        document.getElementById('btnReloadJson').onclick = () => {{
            sendIpc({{ type: 'RELOAD_STYLES' }});
            showToast('Reloaded JSON files from disk');
        }};
        document.getElementById('btnResetJson').onclick = () => {{
            sendIpc({{ type: 'RESET_STYLES' }});
            showToast('Reset component styles to defaults');
        }};

        jsonEditorEl.addEventListener('keydown', (e) => {{
            if (e.key === 'Tab') {{
                e.preventDefault();
                const start = jsonEditorEl.selectionStart;
                const end = jsonEditorEl.selectionEnd;
                jsonEditorEl.value = jsonEditorEl.value.substring(0, start) + '  ' + jsonEditorEl.value.substring(end);
                jsonEditorEl.selectionStart = jsonEditorEl.selectionEnd = start + 2;
            }} else if ((e.ctrlKey || e.metaKey) && e.key.toLowerCase() === 's') {{
                e.preventDefault();
                document.getElementById('btnSaveJson').click();
            }}
        }});

        // Scroll-Spy Nav
        const navLinks = Array.from(document.querySelectorAll('.nav-link'));
        navLinks.forEach(link => {{
            link.addEventListener('click', () => {{
                navLinks.forEach(l => l.classList.remove('active'));
                link.classList.add('active');
            }});
        }});
        if ('IntersectionObserver' in window) {{
            const secObserver = new IntersectionObserver((entries) => {{
                for (const entry of entries) {{
                    if (entry.isIntersecting) {{
                        const id = entry.target.id;
                        navLinks.forEach(l => l.classList.toggle('active', l.dataset.sec === id));
                        break;
                    }}
                }}
            }}, {{ rootMargin: '-18% 0px -68% 0px', threshold: 0.01 }});
            document.querySelectorAll('.sec-block').forEach(sec => secObserver.observe(sec));
        }}

        function renderAllSections() {{
            renderThemeGrid();
            renderColorTokensStudio();
            renderEnginePills();
            renderBangsChips();
            renderShortcutsChips();
            renderAdblockDomainChips();
            renderSiteRulesList();
            renderKeybindsGrid();
            applySettingsSurface();
            syncFormControls();
        }}

        window.__fynexo_on_settings_runtime__ = function(payload) {{
            if (!payload) return;
            if (payload.config) config = Object.assign(config, payload.config);
            if (payload.styles) {{
                styles = payload.styles;
                if (document.activeElement !== jsonEditorEl) {{
                    jsonEditorEl.value = JSON.stringify(styles, null, 2);
                }}
            }}
            if (payload.active_theme) activeTheme = payload.active_theme;
            if (Array.isArray(payload.themes)) themes = payload.themes;
            if (Array.isArray(payload.bookmarks)) bookmarks = payload.bookmarks;
            if (Array.isArray(payload.site_rules)) siteRules = payload.site_rules;
            if (payload.adblock) adblockCfg = payload.adblock;
            if (payload.keybinds) keybinds = Object.assign({{}}, DEFAULT_KEYBINDS, payload.keybinds);
            if (payload.themes_dir) themesDir = payload.themes_dir;
            if (payload.styles_path) stylesPath = payload.styles_path;
            if (payload.data_dir) dataDir = payload.data_dir;
            if (typeof payload.blocked_ads === 'number') blockedAds = payload.blocked_ads;
            if (Array.isArray(payload.history)) historyCount = payload.history.length;
            renderAllSections();
        }};

        renderAllSections();
    </script>
</body>
</html>
"##
    )
}
