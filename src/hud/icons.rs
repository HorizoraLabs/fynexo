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
                    ['path', { d: 'M8 3.2v9.6M3.2 8h9.6', stroke: 'currentColor', 'stroke-width': '1.5', 'stroke-linecap': 'round' }]
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
            case 'close':
                svg = buildSvg(14, 14, '0 0 16 16', [
                    ['path', { d: 'M3.8 3.8l8.4 8.4M12.2 3.8l-8.4 8.4', stroke: 'currentColor', 'stroke-width': '1.5', 'stroke-linecap': 'round' }]
                ]);
                break;
            case 'winMin':
                svg = buildSvg(11.5, 11.5, '0 0 12 12', [
                    ['path', { d: 'M1.5 6h9', stroke: 'currentColor', 'stroke-width': '1.35', 'stroke-linecap': 'round' }]
                ]);
                break;
            case 'winMax':
                svg = buildSvg(11.5, 11.5, '0 0 12 12', [
                    ['rect', { x: '2', y: '2', width: '8', height: '8', rx: '1.1', stroke: 'currentColor', 'stroke-width': '1.3' }]
                ]);
                break;
            case 'winClose':
                svg = buildSvg(11.5, 11.5, '0 0 12 12', [
                    ['path', { d: 'M2.2 2.2l7.6 7.6M9.8 2.2l-7.6 7.6', stroke: 'currentColor', 'stroke-width': '1.4', 'stroke-linecap': 'round' }]
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
                svg = buildSvg(14, 14, '0 0 16 16', [
                    ['path', { d: 'M9.8 2.4l3.8 3.8-1.3 1.3-1-.3-2.5 2.5.4 2.2-1.3 1.3-2.6-2.6-2.9 2.9-.7-.7 2.9-2.9-2.6-2.6 1.3-1.3 2.2.4 2.5-2.5-.3-1 1.1-1.5z', stroke: 'currentColor', 'stroke-width': '1.5', 'stroke-linecap': 'round', 'stroke-linejoin': 'round' }]
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
                svg = buildSvg(14, 14, '0 0 16 16', [
                    ['path', { d: 'M3.2 6.2h5.6a3.8 3.8 0 1 1 0 7.6H6.2', stroke: 'currentColor', 'stroke-width': '1.5', 'stroke-linecap': 'round' }],
                    ['path', { d: 'M5.6 3.6L3 6.2l2.6 2.6', stroke: 'currentColor', 'stroke-width': '1.5', 'stroke-linecap': 'round', 'stroke-linejoin': 'round' }]
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
                svg = buildSvg(14, 14, '0 0 16 16', [
                    ['path', { d: 'M6 11.4V3.4l7-1.6v8', stroke: 'currentColor', 'stroke-width': '1.5', 'stroke-linecap': 'round', 'stroke-linejoin': 'round' }],
                    ['circle', { cx: '4.2', cy: '11.4', r: '1.8', stroke: 'currentColor', 'stroke-width': '1.5' }],
                    ['circle', { cx: '11.2', cy: '9.8', r: '1.8', stroke: 'currentColor', 'stroke-width': '1.5' }]
                ]);
                break;
            case 'download':
                svg = buildSvg(14, 14, '0 0 16 16', [
                    ['path', { d: 'M8 2.2v8.2M4.5 7.6L8 11.2l3.5-3.6', stroke: 'currentColor', 'stroke-width': '1.5', 'stroke-linecap': 'round', 'stroke-linejoin': 'round' }],
                    ['path', { d: 'M2.5 13.5h11', stroke: 'currentColor', 'stroke-width': '1.5', 'stroke-linecap': 'round' }]
                ]);
                break;
            case 'volumeUp':
                svg = buildSvg(14, 14, '0 0 16 16', [
                    ['path', { d: 'M1.8 6h2.2l3.4-3.1v10.2L4 10H1.8a.8.8 0 0 1-.8-.8V6.8a.8.8 0 0 1 .8-.8z', stroke: 'currentColor', 'stroke-width': '1.5', 'stroke-linejoin': 'round' }],
                    ['path', { d: 'M9.8 5.7a3.2 3.2 0 0 1 0 4.6M11.8 3.8a5.6 5.6 0 0 1 0 8.4', stroke: 'currentColor', 'stroke-width': '1.5', 'stroke-linecap': 'round' }]
                ]);
                break;
            case 'volumeMute':
                svg = buildSvg(14, 14, '0 0 16 16', [
                    ['path', { d: 'M1.8 6h2.2l3.4-3.1v10.2L4 10H1.8a.8.8 0 0 1-.8-.8V6.8a.8.8 0 0 1 .8-.8z', stroke: 'currentColor', 'stroke-width': '1.5', 'stroke-linejoin': 'round' }],
                    ['path', { d: 'M10.2 6.4l3.2 3.2M13.4 6.4l-3.2 3.2', stroke: 'currentColor', 'stroke-width': '1.5', 'stroke-linecap': 'round' }]
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
                svg = buildSvg(14, 14, '0 0 16 16', [
                    ['rect', { x: '2.5', y: '3', width: '11', height: '10', rx: '1.5', stroke: 'currentColor', 'stroke-width': '1.5' }],
                    ['path', { d: 'M2.5 7h11M7.5 7v6', stroke: 'currentColor', 'stroke-width': '1.5' }]
                ]);
                break;
            case 'copy':
                svg = buildSvg(14, 14, '0 0 16 16', [
                    ['rect', { x: '5.5', y: '5.5', width: '8', height: '8', rx: '1.6', stroke: 'currentColor', 'stroke-width': '1.5' }],
                    ['path', { d: 'M10.5 5.5V4a1.5 1.5 0 0 0-1.5-1.5H4A1.5 1.5 0 0 0 2.5 4v5A1.5 1.5 0 0 0 4 10.5h1.5', stroke: 'currentColor', 'stroke-width': '1.5', 'stroke-linecap': 'round' }]
                ]);
                break;
            case 'externalLink':
                svg = buildSvg(14, 14, '0 0 16 16', [
                    ['path', { d: 'M9.5 2.5h4v4M13.5 2.5L7.5 8.5M12 9v3.2A1.3 1.3 0 0 1 10.7 13.5H3.8A1.3 1.3 0 0 1 2.5 12.2V5.3A1.3 1.3 0 0 1 3.8 4H7', stroke: 'currentColor', 'stroke-width': '1.5', 'stroke-linecap': 'round', 'stroke-linejoin': 'round' }]
                ]);
                break;
            case 'link':
                svg = buildSvg(14, 14, '0 0 16 16', [
                    ['path', { d: 'M6.8 9.2a3 3 0 0 0 4.2.2l1.8-1.8a3 3 0 0 0-4.2-4.2l-1 1M9.2 6.8a3 3 0 0 0-4.2-.2L3.2 8.4a3 3 0 0 0 4.2 4.2l1-1', stroke: 'currentColor', 'stroke-width': '1.5', 'stroke-linecap': 'round' }]
                ]);
                break;
            case 'scissors':
                svg = buildSvg(14, 14, '0 0 16 16', [
                    ['circle', { cx: '4.5', cy: '4.5', r: '1.8', stroke: 'currentColor', 'stroke-width': '1.5' }],
                    ['circle', { cx: '4.5', cy: '11.5', r: '1.8', stroke: 'currentColor', 'stroke-width': '1.5' }],
                    ['path', { d: 'M13 3.5L5.8 10.2M5.8 5.8L13 12.5', stroke: 'currentColor', 'stroke-width': '1.5', 'stroke-linecap': 'round' }]
                ]);
                break;
            case 'clipboard':
                svg = buildSvg(14, 14, '0 0 16 16', [
                    ['rect', { x: '3.2', y: '3.5', width: '9.6', height: '10.5', rx: '1.6', stroke: 'currentColor', 'stroke-width': '1.5' }],
                    ['rect', { x: '5.7', y: '2', width: '4.6', height: '2.5', rx: '0.8', stroke: 'currentColor', 'stroke-width': '1.5' }]
                ]);
                break;
            case 'printer':
                svg = buildSvg(14, 14, '0 0 16 16', [
                    ['path', { d: 'M4.5 6V2.8a.8.8 0 0 1 .8-.8h5.4a.8.8 0 0 1 .8.8V6M4.5 11.5H3.3A1.3 1.3 0 0 1 2 10.2V7.3A1.3 1.3 0 0 1 3.3 6h9.4A1.3 1.3 0 0 1 14 7.3v2.9a1.3 1.3 0 0 1-1.3 1.3h-1.2', stroke: 'currentColor', 'stroke-width': '1.5', 'stroke-linecap': 'round' }],
                    ['rect', { x: '4.5', y: '9.5', width: '7', height: '4.3', rx: '0.8', stroke: 'currentColor', 'stroke-width': '1.5' }]
                ]);
                break;
            case 'code':
                svg = buildSvg(14, 14, '0 0 16 16', [
                    ['path', { d: 'M5.5 4.8L2.2 8l3.3 3.2M10.5 4.8L13.8 8l-3.3 3.2M9 3.2l-2 9.6', stroke: 'currentColor', 'stroke-width': '1.5', 'stroke-linecap': 'round', 'stroke-linejoin': 'round' }]
                ]);
                break;
            case 'globe':
                svg = buildSvg(14, 14, '0 0 16 16', [
                    ['circle', { cx: '8', cy: '8', r: '5.5', stroke: 'currentColor', 'stroke-width': '1.5' }],
                    ['path', { d: 'M2.5 8h11', stroke: 'currentColor', 'stroke-width': '1.5', 'stroke-linecap': 'round' }],
                    ['ellipse', { cx: '8', cy: '8', rx: '2.4', ry: '5.5', stroke: 'currentColor', 'stroke-width': '1.5' }]
                ]);
                break;
            case 'fileText':
                svg = buildSvg(14, 14, '0 0 16 16', [
                    ['path', { d: 'M4.5 2.2h4.8l3.2 3.2v8.4a1 1 0 0 1-1 1h-7a1 1 0 0 1-1-1V3.2a1 1 0 0 1 1-1z', stroke: 'currentColor', 'stroke-width': '1.5', 'stroke-linejoin': 'round' }],
                    ['path', { d: 'M9.3 2.2v3.2h3.2M5.5 8.2h5M5.5 10.8h3.4', stroke: 'currentColor', 'stroke-width': '1.5', 'stroke-linecap': 'round', 'stroke-linejoin': 'round' }]
                ]);
                break;
            case 'siteYouTube':
                svg = buildSvg(15, 15, '0 0 16 16', [
                    ['rect', { x: '1', y: '2.8', width: '14', height: '10.4', rx: '3.1', fill: '#ff0033' }],
                    ['path', { d: 'M6.5 5.4l4.5 2.6-4.5 2.6V5.4z', fill: '#ffffff' }]
                ]);
                break;
            case 'siteGitHub':
                svg = buildSvg(15, 15, '0 0 16 16', [
                    ['path', { d: 'M8 1.2a6.8 6.8 0 0 0-2.15 13.25c.34.06.46-.15.46-.33v-1.15c-1.88.41-2.28-.9-2.28-.9-.3-.78-.75-.99-.75-.99-.61-.42.05-.41.05-.41.68.05 1.04.7 1.04.7.6 1.04 1.58.74 1.97.56.06-.43.24-.74.43-.91-1.5-.17-3.08-.75-3.08-3.34 0-.74.26-1.34.7-1.81-.07-.17-.3-.86.07-1.8 0 0 .57-.18 1.87.69a6.5 6.5 0 0 1 3.4 0c1.3-.87 1.87-.69 1.87-.69.37.94.14 1.63.07 1.8.44.47.7 1.07.7 1.81 0 2.6-1.58 3.17-3.09 3.34.24.21.46.62.46 1.25v1.85c0 .18.12.4.47.33A6.8 6.8 0 0 0 8 1.2z', fill: 'currentColor' }]
                ]);
                break;
            case 'siteWikipedia':
                svg = buildSvg(15, 15, '0 0 16 16', [
                    ['circle', { cx: '8', cy: '8', r: '6.5', fill: 'rgba(128,128,128,0.16)', stroke: 'rgba(128,128,128,0.32)', 'stroke-width': '0.9' }],
                    ['path', { d: 'M3.8 5.2l2 5.8 1.7-4.1M7.3 5.2l2.1 5.8 2.8-5.8', stroke: 'currentColor', 'stroke-width': '1.25', 'stroke-linecap': 'round', 'stroke-linejoin': 'round' }]
                ]);
                break;
            case 'siteGoogle':
                svg = buildSvg(15, 15, '0 0 16 16', [
                    ['path', { d: 'M14 8.2c0-.48-.04-.94-.12-1.38H8v2.62h3.36a2.87 2.87 0 0 1-1.25 1.88v1.56h2.02C13.31 11.79 14 10.16 14 8.2z', fill: '#4285F4' }],
                    ['path', { d: 'M8 14.3c1.69 0 3.1-.56 4.13-1.52l-2.02-1.56c-.56.37-1.27.6-2.11.6-1.62 0-3-1.1-3.49-2.57H2.43v1.61A6.24 6.24 0 0 0 8 14.3z', fill: '#34A853' }],
                    ['path', { d: 'M4.51 9.25a3.76 3.76 0 0 1 0-2.4V5.24H2.43a6.24 6.24 0 0 0 0 5.62l2.08-1.61z', fill: '#FBBC05' }],
                    ['path', { d: 'M8 4.28c.92 0 1.74.32 2.39.94l1.79-1.79A6 6 0 0 0 8 1.8a6.24 6.24 0 0 0-5.57 3.44l2.08 1.61C5 5.38 6.38 4.28 8 4.28z', fill: '#EA4335' }]
                ]);
                break;
            case 'siteBilibili':
                svg = buildSvg(15, 15, '0 0 16 16', [
                    ['path', { d: 'M4.8 2.2l2 2M11.2 2.2l-2 2', stroke: '#fb7299', 'stroke-width': '1.35', 'stroke-linecap': 'round' }],
                    ['rect', { x: '2', y: '4.2', width: '12', height: '9', rx: '2.4', stroke: '#fb7299', 'stroke-width': '1.4' }],
                    ['path', { d: 'M5.5 7.5v2M10.5 7.5v2', stroke: '#fb7299', 'stroke-width': '1.5', 'stroke-linecap': 'round' }]
                ]);
                break;
            case 'siteReddit':
                svg = buildSvg(15, 15, '0 0 16 16', [
                    ['circle', { cx: '8', cy: '8', r: '6.5', fill: '#ff4500' }],
                    ['path', { d: 'M8 5.2l.8-2 1.8.4', stroke: '#ffffff', 'stroke-width': '1.1', 'stroke-linecap': 'round', 'stroke-linejoin': 'round' }],
                    ['ellipse', { cx: '8', cy: '9.2', rx: '4.1', ry: '2.9', fill: '#ffffff' }],
                    ['circle', { cx: '6.4', cy: '8.9', r: '0.75', fill: '#ff4500' }],
                    ['circle', { cx: '9.6', cy: '8.9', r: '0.75', fill: '#ff4500' }]
                ]);
                break;
            case 'siteDiscord':
                svg = buildSvg(15, 15, '0 0 16 16', [
                    ['rect', { x: '1.5', y: '2.8', width: '13', height: '10.4', rx: '3.2', fill: '#5865f2' }],
                    ['circle', { cx: '5.8', cy: '8.2', r: '1.15', fill: '#ffffff' }],
                    ['circle', { cx: '10.2', cy: '8.2', r: '1.15', fill: '#ffffff' }]
                ]);
                break;
            case 'siteX':
                svg = buildSvg(14, 14, '0 0 16 16', [
                    ['path', { d: 'M2.8 2.6h2.4l3.3 4.5 3.9-4.5h1.1l-4.5 5.2 4.8 6.6h-2.4l-3.6-4.9-4.3 4.9H2.4l4.8-5.6-4.4-6.2z', fill: 'currentColor' }]
                ]);
                break;
            case 'siteNewtab':
                svg = buildSvg(14, 14, '0 0 16 16', [
                    ['path', { d: 'M8 1.6l1.55 4.85L14.4 8l-4.85 1.55L8 14.4l-1.55-4.85L1.6 8l4.85-1.55L8 1.6z', fill: 'currentColor' }]
                ]);
                break;
            case 'siteSettings':
                svg = buildSvg(14, 14, '0 0 16 16', [
                    ['circle', { cx: '8', cy: '8', r: '2.3', stroke: 'currentColor', 'stroke-width': '1.4' }],
                    ['path', { d: 'M8 1.8v1.5M8 12.7v1.5M1.8 8h1.5M12.7 8h1.5M3.6 3.6l1.1 1.1M11.3 11.3l1.1 1.1M12.4 3.6l-1.1 1.1M4.7 11.3l-1.1 1.1', stroke: 'currentColor', 'stroke-width': '1.4', 'stroke-linecap': 'round' }]
                ]);
                break;
            default:
                return document.createElement('span');
        }
        iconCache[name] = svg;
        return svg.cloneNode(true);
    }
"#;
