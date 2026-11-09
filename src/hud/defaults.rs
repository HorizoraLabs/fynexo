pub const DEFAULTS_SCRIPT: &str = r#"
    // Default Fallback Tokens, Engines, Bangs & Keybinds
    const DEFAULT_ENGINES = [
        { id: 'google', name: 'Google', url: 'https://www.google.com/search?q={query}' },
        { id: 'duckduckgo', name: 'DuckDuckGo', url: 'https://duckduckgo.com/?q={query}' },
        { id: 'bing', name: 'Bing', url: 'https://www.bing.com/search?q={query}' },
        { id: 'baidu', name: 'Baidu', url: 'https://www.baidu.com/s?wd={query}' }
    ];

    const DEFAULT_BANGS = [
        { trigger: '!gh', name: 'GitHub', url: 'https://github.com/search?q={query}' },
        { trigger: '!yt', name: 'YouTube', url: 'https://www.youtube.com/results?search_query={query}' },
        { trigger: '!b', name: 'Bilibili', url: 'https://search.bilibili.com/all?keyword={query}' },
        { trigger: '!w', name: 'Wikipedia', url: 'https://en.wikipedia.org/w/index.php?search={query}' },
        { trigger: '!rd', name: 'Reddit', url: 'https://www.reddit.com/search/?q={query}' },
        { trigger: '!tr', name: 'Translate', url: 'https://translate.google.com/?sl=auto&tl=zh-CN&text={query}' }
    ];

    const DEFAULT_KEYBINDS = {
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
    };

    const DEFAULT_STYLES = {
        colors: {
            use_custom_colors: false,
            bg: '#19191c',
            panel: '#29292e',
            surface: '#212126',
            elevated: '#35353c',
            border: '#37373f',
            border_hover: '#4b4b56',
            fg: '#f0f0f3',
            secondary: '#c2c2cb',
            muted: '#8a8a95',
            accent: '#7a88f7',
            accent_bg: '#262942',
            hover: '#33333a',
            tab_active: '#29292e',
            tab_inactive: 'transparent'
        },
        topbar: {
            visible_on_desktop: true,
            visible_on_mobile: false,
            layout: 'deck',
            bookmarks_bar: 'always',
            height: '38px',
            titlebar_height: '40px',
            padding_x: '6px',
            background: 'var(--fnx-panel)',
            border_bottom: '1px solid var(--fnx-border)',
            button_size: '28px',
            button_radius: '6px',
            custom_css: ''
        },
        tabs: {
            height: '36px',
            width: '184px',
            min_width: '76px',
            gap: '0px',
            border_radius: '6px',
            font_size: '12px',
            show_indicator_dot: true,
            close_on_hover_only: false,
            custom_css: ''
        },
        address_bar: {
            mode: 'inline',
            position: 'center',
            height: '30px',
            width: '340px',
            border_radius: '9px',
            font_size: '12.5px',
            background: 'var(--fnx-bg)',
            border: '1px solid var(--fnx-border)',
            custom_css: ''
        },
        search_modal: {
            width: '580px',
            top_offset: '66px',
            border_radius: '12px',
            input_font_size: '14.5px',
            backdrop_color: 'rgba(0, 0, 0, 0.56)',
            background: 'var(--fnx-panel)',
            border: '1px solid var(--fnx-border-hover)',
            custom_css: ''
        },
        mobile_bar: {
            visible: true,
            position: 'bottom',
            height: '54px',
            padding_x: '8px',
            background: 'var(--fnx-panel)',
            border: '1px solid var(--fnx-border)',
            pill_height: '38px',
            pill_radius: '11px',
            button_size: '38px',
            auto_hide_on_keyboard: true,
            custom_css: ''
        },
        tab_drawer: {
            max_height: '80vh',
            border_radius: '16px',
            card_radius: '10px',
            card_padding: '11px 13px',
            custom_css: ''
        },
        settings_page: {
            full_width: true,
            sidebar_width: '244px',
            content_padding: '32px 40px',
            card_radius: '10px',
            custom_css: ''
        },
        newtab: {
            title: 'Fynexo',
            subtitle: 'Single-WebView studio browser · Zero-FX compositing',
            show_clock: true,
            use_24h_clock: true,
            show_shortcuts: true,
            shortcuts_columns: 0,
            show_history: true,
            custom_css: ''
        }
    };
"#;
