pub const SCRIPT: &str = r#"
    (function() {
        function sendIpc(data) {
            const payload = JSON.stringify(data);
            if (window.ipc && typeof window.ipc.postMessage === 'function') {
                window.ipc.postMessage(payload);
            } else if (window.__TAURI_INTERNALS__ && window.__TAURI_INTERNALS__.ipc) {
                window.__TAURI_INTERNALS__.ipc.postMessage(payload);
            } else {
                console.warn('[Fynexo] IPC interface not found!');
            }
        }


        function setupHUD() {
            if (document.getElementById('__fynexo_hud__')) return;

            const bar = document.createElement('div');
            bar.id = '__fynexo_hud__';
            bar.style.cssText = `
                position: fixed;
                top: 15px;
                left: 50%;
                transform: translateX(-50%);
                z-index: 2147483647;
                display: none;
                background: #111418;
                border: 1px solid #00f0ff;
                box-shadow: 0 4px 20px rgba(0, 0, 0, 0.8);
                padding: 6px 10px;
                width: 550px;
                max-width: 90vw;
            `;

            const input = document.createElement('input');
            input.id = '__fynexo_input__';
            input.type = 'text';
            input.placeholder = 'Type URL or search query... (Enter to go, Esc to close)';
            input.style.cssText = `
                width: 100%;
                background: transparent;
                border: none;
                outline: none;
                color: #00f0ff;
                font-family: monospace;
                font-size: 14px;
            `;

            bar.appendChild(input);
            document.body.appendChild(bar);

            // Global shortcut key listener
            window.addEventListener('keydown', (e) => {
                if ((e.ctrlKey && e.key.toLowerCase() === 'l') || (e.altKey && e.key.toLowerCase() === 'd')) {
                    e.preventDefault();
                    bar.style.display = 'block';
                    input.value = window.location.href;
                    input.focus();
                    input.select();
                }
                if (e.key === 'Escape' && bar.style.display === 'block') {
                    bar.style.display = 'none';
                }
            });

            // Handle Enter key for navigation
            input.addEventListener('keydown', (e) => {
                if (e.key === 'Enter') {
                    const target = input.value.trim();
                    if (target && window.ipc) {
                        bar.style.display = 'none';
                        console.log("[Fynexo] Navigation Request:", target);
                        sendIpc({ type: 'NAVIGATE', url: target });
                    }
                }
            });
        }
        
        if (document.readyState === 'loading') {
            document.addEventListener('DOMContentLoaded', setupHUD);
        } else {
            setupHUD();
        }
    })();
"#;

pub const INIT: &str = r#"
    window.addEventListener('DOMContentLoaded', () => {
        const style = document.createElement('style');
        style.innerHTML = `
            * {
                backdrop-filter: none !important;
                filter: none !important;
                box-shadow: none !important;
                text-shadow: none !important;
            }
        `;
        document.head.appendChild(style);
    });
"#;
