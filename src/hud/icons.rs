pub const ICONS_SCRIPT: &str = r#"
    // Cached Pure-DOM SVG Factory (100% Trusted Types Immune + C++ cloneNode Speed)
    const iconCache = Object.create(null);

    function buildSvg(w, h, viewBox, elements) {
        const svg = document.createElementNS('http://www.w3.org/2000/svg', 'svg');
        svg.setAttribute('width', String(w));
        svg.setAttribute('height', String(h));
        svg.setAttribute('viewBox', viewBox);
        svg.setAttribute('fill', 'none');
        svg.setAttribute('aria-hidden', 'true');
        for (const [tag, attrs] of elements) {
            const el = document.createElementNS('http://www.w3.org/2000/svg', tag);
            for (const [k, v] of Object.entries(attrs)) {
                el.setAttribute(k, v);
            }
            svg.appendChild(el);
        }
        return svg;
    }

    function createIcon(name) {
        if (iconCache[name]) return iconCache[name].cloneNode(true);
        let svg;
        switch (name) {
            case 'back':
                svg = buildSvg(16, 16, '0 0 16 16', [
                    ['path', { d: 'M10 12.5L5.5 8L10 3.5', stroke: 'currentColor', 'stroke-width': '1.5', 'stroke-linecap': 'round', 'stroke-linejoin': 'round' }]
                ]);
                break;
            case 'forward':
                svg = buildSvg(16, 16, '0 0 16 16', [
                    ['path', { d: 'M6 3.5L10.5 8L6 12.5', stroke: 'currentColor', 'stroke-width': '1.5', 'stroke-linecap': 'round', 'stroke-linejoin': 'round' }]
                ]);
                break;
            case 'reload':
                svg = buildSvg(16, 16, '0 0 16 16', [
                    ['path', { d: 'M13.2 8a5.2 5.2 0 1 1-1.35-3.5', stroke: 'currentColor', 'stroke-width': '1.5', 'stroke-linecap': 'round', 'stroke-linejoin': 'round' }],
                    ['path', { d: 'M13.2 2.6v2.3h-2.3', stroke: 'currentColor', 'stroke-width': '1.5', 'stroke-linecap': 'round', 'stroke-linejoin': 'round' }]
                ]);
                break;
            case 'plus':
                svg = buildSvg(14, 14, '0 0 16 16', [
                    ['path', { d: 'M8 3.2v9.6M3.2 8h9.6', stroke: 'currentColor', 'stroke-width': '1.6', 'stroke-linecap': 'round' }]
                ]);
                break;
            case 'search':
                svg = buildSvg(12.5, 12.5, '0 0 16 16', [
                    ['circle', { cx: '7.2', cy: '7.2', r: '4.5', stroke: 'currentColor', 'stroke-width': '1.5' }],
                    ['path', { d: 'M10.5 10.5L13.8 13.8', stroke: 'currentColor', 'stroke-width': '1.6', 'stroke-linecap': 'round' }]
                ]);
                break;
            case 'lock':
                svg = buildSvg(12, 12, '0 0 16 16', [
                    ['rect', { x: '3.2', y: '7.2', width: '9.6', height: '6.6', rx: '1.8', stroke: 'currentColor', 'stroke-width': '1.45' }],
                    ['path', { d: 'M5.4 7.2V5.2a2.6 2.6 0 0 1 5.2 0v2', stroke: 'currentColor', 'stroke-width': '1.45', 'stroke-linecap': 'round' }]
                ]);
                break;
            case 'settings':
                svg = buildSvg(14.5, 14.5, '0 0 16 16', [
                    ['path', { d: 'M2.5 4.5h11M2.5 8h11M2.5 11.5h11', stroke: 'currentColor', 'stroke-width': '1.4', 'stroke-linecap': 'round' }],
                    ['circle', { cx: '5.5', cy: '4.5', r: '1.6', fill: 'var(--fnx-panel)', stroke: 'currentColor', 'stroke-width': '1.3' }],
                    ['circle', { cx: '10.5', cy: '8', r: '1.6', fill: 'var(--fnx-panel)', stroke: 'currentColor', 'stroke-width': '1.3' }],
                    ['circle', { cx: '6.5', cy: '11.5', r: '1.6', fill: 'var(--fnx-panel)', stroke: 'currentColor', 'stroke-width': '1.3' }]
                ]);
                break;
            case 'clear':
                svg = buildSvg(14, 14, '0 0 16 16', [
                    ['circle', { cx: '8', cy: '8', r: '6', fill: 'var(--fnx-border-hover)' }],
                    ['path', { d: 'M5.8 5.8l4.4 4.4M10.2 5.8l-4.4 4.4', stroke: 'var(--fnx-fg)', 'stroke-width': '1.4', 'stroke-linecap': 'round' }]
                ]);
                break;
            case 'go':
                svg = buildSvg(14, 14, '0 0 16 16', [
                    ['path', { d: 'M3.2 8h9.6M8.8 4l4 4-4 4', stroke: 'currentColor', 'stroke-width': '1.6', 'stroke-linecap': 'round', 'stroke-linejoin': 'round' }]
                ]);
                break;
            case 'closeSmall':
                svg = buildSvg(9, 9, '0 0 12 12', [
                    ['path', { d: 'M2.5 2.5l7 7M9.5 2.5l-7 7', stroke: 'currentColor', 'stroke-width': '1.5', 'stroke-linecap': 'round' }]
                ]);
                break;
            case 'winMin':
                svg = buildSvg(10, 10, '0 0 10 10', [
                    ['path', { d: 'M1 5h8', stroke: 'currentColor', 'stroke-width': '1.2' }]
                ]);
                break;
            case 'winMax':
                svg = buildSvg(10, 10, '0 0 10 10', [
                    ['rect', { x: '1.5', y: '1.5', width: '7', height: '7', rx: '0.8', stroke: 'currentColor', 'stroke-width': '1.2' }]
                ]);
                break;
            case 'winClose':
                svg = buildSvg(10, 10, '0 0 10 10', [
                    ['path', { d: 'M1.5 1.5l7 7M8.5 1.5l-7 7', stroke: 'currentColor', 'stroke-width': '1.3', 'stroke-linecap': 'round' }]
                ]);
                break;
            case 'star':
                svg = buildSvg(14, 14, '0 0 16 16', [
                    ['path', { d: 'M8 2.1l1.78 3.61 3.98.58-2.88 2.81.68 3.97L8 11.2l-3.56 1.87.68-3.97L2.24 6.29l3.98-.58L8 2.1z', stroke: 'currentColor', 'stroke-width': '1.35', 'stroke-linejoin': 'round' }]
                ]);
                break;
            case 'starFilled':
                svg = buildSvg(14, 14, '0 0 16 16', [
                    ['path', { d: 'M8 2.1l1.78 3.61 3.98.58-2.88 2.81.68 3.97L8 11.2l-3.56 1.87.68-3.97L2.24 6.29l3.98-.58L8 2.1z', fill: 'currentColor', stroke: 'currentColor', 'stroke-width': '1.2', 'stroke-linejoin': 'round' }]
                ]);
                break;
            case 'history':
                svg = buildSvg(13, 13, '0 0 16 16', [
                    ['circle', { cx: '8', cy: '8', r: '5.5', stroke: 'currentColor', 'stroke-width': '1.4' }],
                    ['path', { d: 'M8 5v3.3l2.3 1.4', stroke: 'currentColor', 'stroke-width': '1.4', 'stroke-linecap': 'round', 'stroke-linejoin': 'round' }]
                ]);
                break;
            case 'home':
                svg = buildSvg(16, 16, '0 0 16 16', [
                    ['path', { d: 'M2.6 7.4L8 2.8l5.4 4.6v5.4a.8.8 0 0 1-.8.8H3.4a.8.8 0 0 1-.8-.8V7.4z', stroke: 'currentColor', 'stroke-width': '1.5', 'stroke-linejoin': 'round' }]
                ]);
                break;
            case 'pin':
                svg = buildSvg(12, 12, '0 0 16 16', [
                    ['path', { d: 'M9.8 2.5l3.7 3.7-1.4 1.4-.9-.3-2.6 2.6.4 2.2-1.3 1.3-2.6-2.6-2.9 2.9-.7-.7 2.9-2.9-2.6-2.6 1.3-1.3 2.2.4 2.6-2.6-.3-.9 1.2-1.6z', stroke: 'currentColor', 'stroke-width': '1.3', 'stroke-linejoin': 'round' }]
                ]);
                break;
            case 'reader':
                svg = buildSvg(14, 14, '0 0 16 16', [
                    ['rect', { x: '2.5', y: '2.2', width: '11', height: '11.6', rx: '1.6', stroke: 'currentColor', 'stroke-width': '1.35' }],
                    ['path', { d: 'M5.2 5.4h5.6M5.2 8h5.6M5.2 10.6h3.6', stroke: 'currentColor', 'stroke-width': '1.35', 'stroke-linecap': 'round' }]
                ]);
                break;
            case 'moon':
                svg = buildSvg(13.5, 13.5, '0 0 16 16', [
                    ['path', { d: 'M12.8 9.8A5.2 5.2 0 0 1 6.2 3.2a5.3 5.3 0 1 0 6.6 6.6z', stroke: 'currentColor', 'stroke-width': '1.4', 'stroke-linejoin': 'round' }]
                ]);
                break;
            case 'fullscreen':
                svg = buildSvg(12, 12, '0 0 16 16', [
                    ['path', { d: 'M2.8 6V2.8H6M10 2.8h3.2V6M13.2 10v3.2H10M6 13.2H2.8V10', stroke: 'currentColor', 'stroke-width': '1.45', 'stroke-linecap': 'round', 'stroke-linejoin': 'round' }]
                ]);
                break;
            case 'chevUp':
            case 'chevronUp':
                svg = buildSvg(13, 13, '0 0 16 16', [
                    ['path', { d: 'M3.8 9.8L8 5.6l4.2 4.2', stroke: 'currentColor', 'stroke-width': '1.8', 'stroke-linecap': 'round', 'stroke-linejoin': 'round' }]
                ]);
                break;
            case 'chevDown':
            case 'chevronDown':
                svg = buildSvg(13, 13, '0 0 16 16', [
                    ['path', { d: 'M3.8 6.2L8 10.4l4.2-4.2', stroke: 'currentColor', 'stroke-width': '1.8', 'stroke-linecap': 'round', 'stroke-linejoin': 'round' }]
                ]);
                break;
            case 'undo':
                svg = buildSvg(13, 13, '0 0 16 16', [
                    ['path', { d: 'M3.2 6.2h5.6a3.8 3.8 0 1 1 0 7.6H6.2', stroke: 'currentColor', 'stroke-width': '1.45', 'stroke-linecap': 'round' }],
                    ['path', { d: 'M5.6 3.6L3 6.2l2.6 2.6', stroke: 'currentColor', 'stroke-width': '1.45', 'stroke-linecap': 'round', 'stroke-linejoin': 'round' }]
                ]);
                break;
            case 'bookmarksBar':
                svg = buildSvg(14.5, 14.5, '0 0 16 16', [
                    ['rect', { x: '2.2', y: '2.6', width: '11.6', height: '10.8', rx: '2', stroke: 'currentColor', 'stroke-width': '1.35' }],
                    ['path', { d: 'M2.5 6.4h11', stroke: 'currentColor', 'stroke-width': '1.35' }]
                ]);
                break;
            case 'grid':
                svg = buildSvg(12.5, 12.5, '0 0 16 16', [
                    ['rect', { x: '2.4', y: '2.4', width: '4.4', height: '4.4', rx: '1', stroke: 'currentColor', 'stroke-width': '1.35' }],
                    ['rect', { x: '9.2', y: '2.4', width: '4.4', height: '4.4', rx: '1', stroke: 'currentColor', 'stroke-width': '1.35' }],
                    ['rect', { x: '2.4', y: '9.2', width: '4.4', height: '4.4', rx: '1', stroke: 'currentColor', 'stroke-width': '1.35' }],
                    ['rect', { x: '9.2', y: '9.2', width: '4.4', height: '4.4', rx: '1', stroke: 'currentColor', 'stroke-width': '1.35' }]
                ]);
                break;
            case 'more':
            case 'dots':
                svg = buildSvg(14, 14, '0 0 16 16', [
                    ['circle', { cx: '8', cy: '3.2', r: '1.4', fill: 'currentColor' }],
                    ['circle', { cx: '8', cy: '8', r: '1.4', fill: 'currentColor' }],
                    ['circle', { cx: '8', cy: '12.8', r: '1.4', fill: 'currentColor' }]
                ]);
                break;
            case 'shield':
                svg = buildSvg(13.5, 13.5, '0 0 16 16', [
                    ['path', { d: 'M8 1.8l5 2v4.8c0 3.6-2.2 6-5 6.8-2.8-.8-5-3.2-5-6.8V3.8l5-2z', stroke: 'currentColor', 'stroke-width': '1.35', 'stroke-linejoin': 'round' }]
                ]);
                break;
            case 'media':
            case 'music':
                svg = buildSvg(13.5, 13.5, '0 0 16 16', [
                    ['path', { d: 'M6 12V3.5l7-1.8V10', stroke: 'currentColor', 'stroke-width': '1.35', 'stroke-linecap': 'round', 'stroke-linejoin': 'round' }],
                    ['circle', { cx: '4.2', cy: '12', r: '2', fill: 'currentColor' }],
                    ['circle', { cx: '11.2', cy: '10', r: '2', fill: 'currentColor' }]
                ]);
                break;
            case 'download':
                svg = buildSvg(13.5, 13.5, '0 0 16 16', [
                    ['path', { d: 'M8 2.2v8.2M4.5 7.6L8 11.2l3.5-3.6', stroke: 'currentColor', 'stroke-width': '1.45', 'stroke-linecap': 'round', 'stroke-linejoin': 'round' }],
                    ['path', { d: 'M2.5 13.5h11', stroke: 'currentColor', 'stroke-width': '1.45', 'stroke-linecap': 'round' }]
                ]);
                break;
            case 'volumeUp':
                svg = buildSvg(14, 14, '0 0 16 16', [
                    ['path', { d: 'M2.5 6h2.5l3.5-3.2v10.4L5 10H2.5a.8.8 0 0 1-.8-.8v-2.4a.8.8 0 0 1 .8-.8z', stroke: 'currentColor', 'stroke-width': '1.35', 'stroke-linejoin': 'round' }],
                    ['path', { d: 'M11.5 5.5a3.8 3.8 0 0 1 0 5M13.5 3.5a6.5 6.5 0 0 1 0 9', stroke: 'currentColor', 'stroke-width': '1.35', 'stroke-linecap': 'round' }]
                ]);
                break;
            case 'volumeMute':
                svg = buildSvg(14, 14, '0 0 16 16', [
                    ['path', { d: 'M2.5 6h2.5l3.5-3.2v10.4L5 10H2.5a.8.8 0 0 1-.8-.8v-2.4a.8.8 0 0 1 .8-.8z', stroke: 'currentColor', 'stroke-width': '1.35', 'stroke-linejoin': 'round' }],
                    ['path', { d: 'M11.5 6.5l3 3M14.5 6.5l-3 3', stroke: 'currentColor', 'stroke-width': '1.35', 'stroke-linecap': 'round' }]
                ]);
                break;
            case 'play':
                svg = buildSvg(12, 12, '0 0 16 16', [
                    ['path', { d: 'M4.5 2.8l8.5 5.2-8.5 5.2V2.8z', fill: 'currentColor' }]
                ]);
                break;
            case 'pause':
                svg = buildSvg(12, 12, '0 0 16 16', [
                    ['rect', { x: '4', y: '3', width: '2.8', height: '10', rx: '0.8', fill: 'currentColor' }],
                    ['rect', { x: '9.2', y: '3', width: '2.8', height: '10', rx: '0.8', fill: 'currentColor' }]
                ]);
                break;
            case 'minus':
                svg = buildSvg(12, 12, '0 0 16 16', [
                    ['path', { d: 'M3.2 8h9.6', stroke: 'currentColor', 'stroke-width': '1.55', 'stroke-linecap': 'round' }]
                ]);
                break;
            case 'sidepanel':
            case 'sidebar':
                svg = buildSvg(14, 14, '0 0 16 16', [
                    ['rect', { x: '2.2', y: '2.5', width: '11.6', height: '11', rx: '1.8', stroke: 'currentColor', 'stroke-width': '1.35' }],
                    ['path', { d: 'M10 2.8v10.4', stroke: 'currentColor', 'stroke-width': '1.35' }]
                ]);
                break;
            case 'cpu':
            case 'taskmgr':
                svg = buildSvg(13.5, 13.5, '0 0 16 16', [
                    ['rect', { x: '4', y: '4', width: '8', height: '8', rx: '1.5', stroke: 'currentColor', 'stroke-width': '1.35' }],
                    ['path', { d: 'M6 1.8v2.2M10 1.8v2.2M6 12v2.2M10 12v2.2M1.8 6h2.2M1.8 10h2.2M12 6h2.2M12 10h2.2', stroke: 'currentColor', 'stroke-width': '1.35', 'stroke-linecap': 'round' }]
                ]);
                break;
            case 'workspace':
                svg = buildSvg(13.5, 13.5, '0 0 16 16', [
                    ['rect', { x: '2.5', y: '3', width: '11', height: '10', rx: '1.5', stroke: 'currentColor', 'stroke-width': '1.35' }],
                    ['path', { d: 'M2.5 7h11M7.5 7v6', stroke: 'currentColor', 'stroke-width': '1.35' }]
                ]);
                break;
            default:
                return document.createElement('span');
        }
        iconCache[name] = svg;
        return svg.cloneNode(true);
    }
"#;
