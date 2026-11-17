         function patchYoutubeDrawers() {
            try {
               const host = (window.location.hostname || '').toLowerCase();
               if (!host.includes('youtube.com')) return true;
               const drawers = document.querySelectorAll('tp-yt-app-drawer');
               let patchedGuide = false;
               for (let i = 0; i < drawers.length; i++) {
                  const d = drawers[i];
                  const sr = d && d.shadowRoot;
                  if (!sr) continue;
                  if (d.id === 'guide') patchedGuide = true;
                  if (!sr.getElementById('__fnx_yt_drawer_fix__')) {
                     const st = document.createElement('style');
                     st.id = '__fnx_yt_drawer_fix__';
                     st.textContent = `
                                #contentContainer {
                                    padding-top: 0 !important;
                                    padding-bottom: 0 !important;
                                    top: 0 !important;
                                    bottom: 0 !important;
                                    height: 100% !important;
                                    max-height: 100% !important;
                                }
                            `;
                     sr.appendChild(st);
                  }
               }
               return patchedGuide;
            } catch (_) {
               return true;
            }
         }

         let ytAdSkipperActive = false;
         let ytStopTeardown = null;
         function syncYouTubeAdSkipper() {
            if (!config.ext_yt_ad_skipper) {
               if (ytStopTeardown) {
                  ytStopTeardown();
                  ytStopTeardown = null;
               }
               ytAdSkipperActive = false;
               return;
            }
            if (ytAdSkipperActive) return;
            try {
               const host = (window.location.hostname || '').toLowerCase();
               if (!host.includes('youtube.com')) return;
               ytAdSkipperActive = true;

               if (!patchYoutubeDrawers()) {
                  let dTries = 0;
                  const drawerTimer = setInterval(() => {
                     if (patchYoutubeDrawers() || ++dTries > 30)
                        clearInterval(drawerTimer);
                  }, 250);
               }
               document.addEventListener(
                  'DOMContentLoaded',
                  patchYoutubeDrawers,
                  { once: true },
               );
               window.addEventListener('load', patchYoutubeDrawers, {
                  once: true,
               });
               window.addEventListener(
                  'yt-navigate-finish',
                  () => setTimeout(patchYoutubeDrawers, 120),
                  { passive: true },
               );

               let savedRate = 1.0;
               let playerEl = null;
               let adPollTimer = null;

               const skipAds = () => {
                  if (config.adblock === false) {
                     if (adPollTimer) {
                        clearInterval(adPollTimer);
                        adPollTimer = null;
                     }
                     return;
                  }
                  if (!playerEl || !playerEl.isConnected) {
                     playerEl = document.querySelector('.html5-video-player');
                  }
                  if (!playerEl) return;

                  const isAd =
                     playerEl.classList.contains('ad-showing') ||
                     playerEl.classList.contains('ad-interrupting');
                  const v = playerEl.querySelector('video');

                  if (v) {
                     if (isAd) {
                        // Mute ad and fast-forward at 16x without seeking currentTime to avoid MSE pipeline crash/black screen
                        v.muted = true;
                        v.playbackRate = 16.0;
                     } else {
                        // Restore playback speed and respect the tab's mute state as soon as ad finishes
                        if (v.playbackRate > 2.0) {
                           v.playbackRate = savedRate || 1.0;
                           v.muted = isTabMutedState;
                        } else {
                           savedRate = v.playbackRate;
                        }
                     }
                  }

                  if (isAd) {
                     if (!adPollTimer) {
                        adPollTimer = setInterval(skipAds, 250);
                     }
                     const skipBtn = playerEl.querySelector(
                        'button.ytp-ad-skip-button-modern, button.ytp-ad-skip-button, .ytp-skip-ad-button, .ytp-ad-skip-button-container button',
                     );
                     if (skipBtn && typeof skipBtn.click === 'function') {
                        try {
                           skipBtn.click();
                        } catch (_) {}
                     }
                     const overlayBtn = playerEl.querySelector(
                        '.ytp-ad-overlay-close-button',
                     );
                     if (overlayBtn && typeof overlayBtn.click === 'function') {
                        try {
                           overlayBtn.click();
                        } catch (_) {}
                     }
                  } else if (adPollTimer) {
                     clearInterval(adPollTimer);
                     adPollTimer = null;
                  }
               };

               // Observe player class changes so idle playback incurs zero 250ms DOM polling.
               let observedPlayer = null;
               let playerObserver = null;
               const attachPlayerObserver = () => {
                  const p = document.querySelector('.html5-video-player');
                  if (!p || p === observedPlayer) return false;
                  if (playerObserver) {
                     try {
                        playerObserver.disconnect();
                     } catch (_) {}
                  }
                  observedPlayer = p;
                  playerEl = p;
                  playerObserver = new MutationObserver(skipAds);
                  playerObserver.observe(p, {
                     attributes: true,
                     attributeFilter: ['class'],
                  });
                  skipAds();
                  return true;
               };
               if (!attachPlayerObserver()) {
                  let tries = 0;
                  const bootTimer = setInterval(() => {
                     if (attachPlayerObserver() || ++tries > 40)
                        clearInterval(bootTimer);
                  }, 500);
               }
               const onYtNav = () => setTimeout(attachPlayerObserver, 200);
               window.addEventListener('yt-navigate-finish', onYtNav, { passive: true });
               window.addEventListener('popstate', onYtNav, { passive: true });
               ytStopTeardown = () => {
                  if (adPollTimer) {
                     clearInterval(adPollTimer);
                     adPollTimer = null;
                  }
                  if (playerObserver) {
                     try {
                        playerObserver.disconnect();
                     } catch (_) {}
                     playerObserver = null;
                  }
                  observedPlayer = null;
                  window.removeEventListener('yt-navigate-finish', onYtNav);
                  window.removeEventListener('popstate', onYtNav);
               };
            } catch (_) {}
         }
         syncYouTubeAdSkipper();

         let ghHarmonizerActive = false;
         let ghStopTeardown = null;
         function syncGithubDrawerHarmonizer() {
            if (!config.ext_gh_harmonizer) {
               if (ghStopTeardown) {
                  ghStopTeardown();
                  ghStopTeardown = null;
               }
               ghHarmonizerActive = false;
               return;
            }
            if (ghHarmonizerActive) return;
            try {
               const host = (window.location.hostname || '').toLowerCase();
               if (host !== 'github.com' && !host.endsWith('.github.com')) return;
               ghHarmonizerActive = true;

               let ghRafPending = false;
               const resetTopOffsets = () => {
                  const patched = document.querySelectorAll('[data-fnx-gh-offset]');
                  for (let i = 0; i < patched.length; i++) {
                     const n = patched[i];
                     n.removeAttribute('data-fnx-gh-offset');
                     n.removeAttribute('data-fnx-gh-top');
                     n.style.removeProperty('top');
                     n.style.removeProperty('inset-block-start');
                     n.style.removeProperty('bottom');
                     n.style.removeProperty('inset-block-end');
                     n.style.removeProperty('height');
                     n.style.removeProperty('max-height');
                     n.style.removeProperty('margin-top');
                     n.style.removeProperty('margin-bottom');
                     n.style.removeProperty('margin-block-start');
                     n.style.removeProperty('margin-block-end');
                     n.style.removeProperty('transform');
                     n.style.removeProperty('align-self');
                  }
               };

               const applyTopOffset = (node, topPx) => {
                  const topVal = `${topPx}px`;
                  const heightVal = `calc(100vh - ${topPx}px)`;
                  node.setAttribute('data-fnx-gh-offset', '1');
                  node.setAttribute('data-fnx-gh-top', String(topPx));
                  node.style.setProperty('top', topVal, 'important');
                  node.style.setProperty('inset-block-start', topVal, 'important');
                  node.style.setProperty('bottom', '0px', 'important');
                  node.style.setProperty('inset-block-end', '0px', 'important');
                  node.style.setProperty('height', heightVal, 'important');
                  node.style.setProperty('max-height', heightVal, 'important');
                  node.style.setProperty('margin-top', '0px', 'important');
                  node.style.setProperty('margin-bottom', '0px', 'important');
                  node.style.setProperty('margin-block-start', '0px', 'important');
                  node.style.setProperty('margin-block-end', '0px', 'important');
               };

               const patchDrawers = () => {
                  ghRafPending = false;
                  const deskTopVisible =
                     styles.topbar.visible_on_desktop !== false &&
                     !isFullscreen &&
                     !isMobilePlatform();
                  const tb =
                     shadowRoot && shadowRoot.getElementById('__fynexo_topbar__');
                  const tbRect = tb ? tb.getBoundingClientRect() : null;
                  let topPx = 0;
                  if (deskTopVisible && tbRect && tbRect.height > 1 && tbRect.bottom > 1) {
                     topPx = Math.ceil(tbRect.bottom);
                  } else if (deskTopVisible && document.body) {
                     topPx = Math.round(document.body.getBoundingClientRect().top);
                  }
                  if (!Number.isFinite(topPx) || topPx <= 1) {
                     resetTopOffsets();
                     return;
                  }

                  if (
                     document.body &&
                     !document.documentElement.hasAttribute('data-fynexo-internal') &&
                     document.body.getBoundingClientRect().top < topPx - 2
                  ) {
                     document.body.style.setProperty('position', 'fixed', 'important');
                     document.body.style.setProperty('top', `${topPx}px`, 'important');
                     document.body.style.setProperty('left', '0px', 'important');
                     document.body.style.setProperty('right', '0px', 'important');
                     document.body.style.setProperty('bottom', '0px', 'important');
                  }

                  const roots = [document];
                  const customModalHosts = document.querySelectorAll(
                     'modal-dialog, primer-dialog, anchored-position',
                  );
                  for (let m = 0; m < customModalHosts.length; m++) {
                     if (customModalHosts[m] && customModalHosts[m].shadowRoot) {
                        roots.push(customModalHosts[m].shadowRoot);
                     }
                  }

                  const seeds = [];
                  for (let r = 0; r < roots.length; r++) {
                     const list = roots[r].querySelectorAll(
                        'dialog, modal-dialog, [role="dialog"], [aria-modal="true"], [popover], .Overlay, .Overlay-backdrop, .Overlay-backdrop--side, [data-component="SidePanel"], [data-testid="global-navigation-modal"], aside.SidePanel, [class*="prc-Dialog-Backdrop"], [class*="prc-Dialog-Dialog"]',
                     );
                     for (let i = 0; i < list.length; i++) seeds.push(list[i]);
                  }
                  if (typeof document.elementsFromPoint === 'function') {
                     const probeYs = [
                        Math.min(window.innerHeight - 10, topPx + 4),
                        Math.min(window.innerHeight - 10, topPx + 24),
                     ];
                     const probeXs = [28, 120, Math.max(40, window.innerWidth - 28)];
                     for (let py = 0; py < probeYs.length; py++) {
                        for (let px = 0; px < probeXs.length; px++) {
                           const hit = document.elementsFromPoint(
                              probeXs[px],
                              probeYs[py],
                           );
                           for (let h = 0; h < hit.length; h++) {
                              if (hit[h] && hit[h].id !== '__fynexo_host__') {
                                 seeds.push(hit[h]);
                              }
                           }
                        }
                     }
                  }

                  const minH = window.innerHeight * 0.45;
                  const seen = typeof Set === 'function' ? new Set() : null;
                  for (let i = 0; i < seeds.length; i++) {
                     let node = seeds[i];
                     for (
                        let depth = 0;
                        depth < 30 &&
                        node &&
                        node !== document.body &&
                        node !== document.documentElement;
                        depth++
                     ) {
                        if (node.id === '__fynexo_host__') break;
                        if (!seen || !seen.has(node)) {
                           if (seen) seen.add(node);
                           const cs = getComputedStyle(node);
                           if (
                              cs.position === 'fixed' ||
                              cs.position === 'sticky' ||
                              cs.position === 'absolute'
                           ) {
                              const rect = node.getBoundingClientRect();
                              const prevTop = parseInt(
                                 node.getAttribute('data-fnx-gh-top') || '',
                                 10,
                              );
                              if (
                                 rect.height >= minH &&
                                 (rect.top < topPx - 2 ||
                                    (Number.isFinite(prevTop) && prevTop !== topPx))
                              ) {
                                 applyTopOffset(node, topPx);
                                 const after = node.getBoundingClientRect();
                                 if (
                                    after.top < topPx - 2 &&
                                    cs.transform &&
                                    cs.transform !== 'none' &&
                                    typeof DOMMatrix === 'function'
                                 ) {
                                    try {
                                       const mat = new DOMMatrix(cs.transform);
                                       if (mat.f < -1) {
                                          node.style.setProperty(
                                             'transform',
                                             `translateX(${mat.e}px)`,
                                             'important',
                                          );
                                       }
                                    } catch (_) {}
                                 }
                                 const afterTx = node.getBoundingClientRect();
                                 if (afterTx.top < topPx - 2) {
                                    const delta = Math.ceil(topPx - afterTx.top);
                                    if (delta > 0 && delta <= topPx + 24) {
                                       node.style.setProperty(
                                          'margin-top',
                                          `${delta}px`,
                                          'important',
                                       );
                                    }
                                 }
                              }
                           }
                        }
                        node =
                           node.parentElement ||
                           (node.getRootNode && node.getRootNode().host) ||
                           null;
                     }
                  }

                  // Final safety pass if an open dialog/drawer is still partially above topPx (e.g. centered flex child)
                  for (let r = 0; r < roots.length; r++) {
                     const dialogs = roots[r].querySelectorAll(
                        'dialog[open], [role="dialog"], [aria-modal="true"], [data-testid="global-navigation-modal"], .Overlay--drawer',
                     );
                     for (let i = 0; i < dialogs.length; i++) {
                        const d = dialogs[i];
                        const rect = d.getBoundingClientRect();
                        if (rect.height >= minH && rect.top < topPx - 2) {
                           const cs = getComputedStyle(d);
                           if (cs.position === 'static' || cs.position === 'relative') {
                              const parent = d.parentElement;
                              const pcs = parent ? getComputedStyle(parent) : null;
                              if (pcs && (pcs.display === 'flex' || pcs.display === 'inline-flex' || pcs.display === 'grid')) {
                                 d.style.setProperty('align-self', 'stretch', 'important');
                              }
                              const afterSelf = d.getBoundingClientRect();
                              if (afterSelf.top < topPx - 2) {
                                 const delta = Math.ceil(topPx - afterSelf.top);
                                 if (delta > 0 && delta <= topPx + 24 && !d.hasAttribute('data-fnx-gh-offset')) {
                                    d.setAttribute('data-fnx-gh-offset', '1');
                                    d.style.setProperty('margin-top', `${delta}px`, 'important');
                                 }
                              }
                           } else {
                              applyTopOffset(d, topPx);
                           }
                        }
                     }
                  }
               };

               let ghSettleTimer = null;
               const schedulePatch = () => {
                  if (!ghRafPending) {
                     ghRafPending = true;
                     requestAnimationFrame(patchDrawers);
                  }
                  if (ghSettleTimer) clearTimeout(ghSettleTimer);
                  ghSettleTimer = setTimeout(() => {
                     ghSettleTimer = null;
                     patchDrawers();
                  }, 80);
               };

               document.addEventListener('click', schedulePatch, { passive: true, capture: true });
               document.addEventListener('keydown', schedulePatch, { passive: true, capture: true });
               window.addEventListener('turbo:load', schedulePatch, { passive: true });
               window.addEventListener('resize', patchDrawers, { passive: true });

               let ghObserver = null;
               const attachObserver = () => {
                  const target = document.documentElement || document.body;
                  if (target && !ghObserver) {
                     ghObserver = new MutationObserver((mutations) => {
                        for (let i = 0; i < mutations.length; i++) {
                           const t = mutations[i].target;
                           if (
                              t &&
                              (t.id === '__fynexo_host__' ||
                                 (t.closest && t.closest('#__fynexo_host__')) ||
                                 (t.nodeType === 1 && t.hasAttribute('data-fnx-gh-offset')))
                           ) {
                              continue;
                           }
                           schedulePatch();
                           break;
                        }
                     });
                     ghObserver.observe(target, {
                        childList: true,
                        subtree: true,
                        attributes: true,
                        attributeFilter: ['open', 'hidden', 'aria-hidden', 'style', 'class'],
                     });
                  }
               };
               const onGhReady = () => {
                  if (!ghHarmonizerActive) return;
                  attachObserver();
                  patchDrawers();
               };
               if (document.readyState === 'loading') {
                  document.addEventListener('DOMContentLoaded', onGhReady, { once: true });
               } else {
                  attachObserver();
                  patchDrawers();
               }
               ghStopTeardown = () => {
                  if (ghSettleTimer) {
                     clearTimeout(ghSettleTimer);
                     ghSettleTimer = null;
                  }
                  if (ghObserver) {
                     try {
                        ghObserver.disconnect();
                     } catch (_) {}
                     ghObserver = null;
                  }
                  document.removeEventListener('click', schedulePatch, { capture: true });
                  document.removeEventListener('keydown', schedulePatch, { capture: true });
                  window.removeEventListener('turbo:load', schedulePatch);
                  window.removeEventListener('resize', patchDrawers);
                  resetTopOffsets();
               };
            } catch (_) {}
         }
         syncGithubDrawerHarmonizer();

         // Built-in Mouse Gestures Extension (Right-Click Drag)
         let mouseGesturesActive = false;
         let gestureStopTeardown = null;
         function syncMouseGestures() {
            const wantActive =
               !isMobilePlatform() && config.ext_mouse_gestures !== false;
            if (!wantActive) {
               if (gestureStopTeardown) {
                  gestureStopTeardown();
                  gestureStopTeardown = null;
               }
               mouseGesturesActive = false;
               return;
            }
            if (mouseGesturesActive) return;
            mouseGesturesActive = true;

            const ARROWS = { L: '←', R: '→', U: '↑', D: '↓' };
            const GESTURE_MAP = {
               L: {
                  label: 'Back',
                  run: () => requestHistoryNav('back'),
               },
               R: {
                  label: 'Forward',
                  run: () => requestHistoryNav('forward'),
               },
               U: {
                  label: 'Scroll to top',
                  run: () => {
                     try {
                        if (document.body) {
                           document.body.scrollTo({ top: 0, behavior: 'smooth' });
                        }
                        if (document.documentElement) {
                           document.documentElement.scrollTo({ top: 0, behavior: 'smooth' });
                        }
                        window.scrollTo({ top: 0, behavior: 'smooth' });
                     } catch (_) {
                        if (document.body) document.body.scrollTop = 0;
                        window.scrollTo(0, 0);
                     }
                  },
               },
               D: {
                  label: 'Scroll to bottom',
                  run: () => {
                     const targetY = Math.max(
                        document.body ? document.body.scrollHeight : 0,
                        document.documentElement
                           ? document.documentElement.scrollHeight
                           : 0,
                     );
                     try {
                        if (document.body) {
                           document.body.scrollTo({ top: targetY, behavior: 'smooth' });
                        }
                        if (document.documentElement) {
                           document.documentElement.scrollTo({ top: targetY, behavior: 'smooth' });
                        }
                        window.scrollTo({ top: targetY, behavior: 'smooth' });
                     } catch (_) {
                        if (document.body) document.body.scrollTop = targetY;
                        window.scrollTo(0, targetY);
                     }
                  },
               },
               DR: {
                  label: 'Close tab',
                  run: () => {
                     if (tabState.active_id) {
                        sendIpc({ type: 'CLOSE_TAB', id: tabState.active_id });
                     }
                  },
               },
               DL: {
                  label: 'Reopen closed tab',
                  run: () => {
                     triggerProgress();
                     sendIpc({ type: 'REOPEN_CLOSED_TAB' });
                  },
               },
               UD: {
                  label: 'Reload page',
                  run: () => {
                     triggerProgress();
                     sendIpc({ type: 'RELOAD' });
                  },
               },
               UR: {
                  label: 'Next tab',
                  run: () => {
                     if (tabState.tabs && tabState.tabs.length > 1) {
                        const idx = tabState.tabs.findIndex(
                           (t) => t.id === tabState.active_id,
                        );
                        const nextIdx = (idx + 1) % tabState.tabs.length;
                        flushScrollSync();
                        triggerProgress();
                        sendIpc({ type: 'SWITCH_TAB', id: tabState.tabs[nextIdx].id });
                     }
                  },
               },
               UL: {
                  label: 'Previous tab',
                  run: () => {
                     if (tabState.tabs && tabState.tabs.length > 1) {
                        const idx = tabState.tabs.findIndex(
                           (t) => t.id === tabState.active_id,
                        );
                        const prevIdx =
                           (idx - 1 + tabState.tabs.length) %
                           tabState.tabs.length;
                        flushScrollSync();
                        triggerProgress();
                        sendIpc({ type: 'SWITCH_TAB', id: tabState.tabs[prevIdx].id });
                     }
                  },
               },
            };

            let tracking = false;
            let lastX = 0;
            let lastY = 0;
            let path = '';
            let suppressCtxUntil = 0;
            let hudEl = null;
            let arrowSpan = null;
            let labelSpan = null;

            function ensureHudEl() {
               if (!shadowRoot) return null;
               if (hudEl && hudEl.isConnected) return hudEl;
               hudEl = shadowRoot.getElementById('__fynexo_gesture_hud__');
               if (!hudEl) {
                  hudEl = document.createElement('div');
                  hudEl.id = '__fynexo_gesture_hud__';
                  hudEl.style.cssText =
                     'position: fixed; top: 50%; left: 50%; transform: translate(-50%, -50%); pointer-events: none; z-index: 2147483647; display: none; flex-direction: column; align-items: center; gap: 4px; padding: 12px 18px; border-radius: 12px; background: var(--fnx-panel); border: 1px solid var(--fnx-border-hover); color: var(--fnx-fg); min-width: 124px; text-align: center;';
                  arrowSpan = document.createElement('div');
                  arrowSpan.style.cssText =
                     'font-size: 20px; font-weight: 700; letter-spacing: 0.14em; color: var(--fnx-accent); line-height: 1.2;';
                  labelSpan = document.createElement('div');
                  labelSpan.style.cssText =
                     'font-size: 12px; font-weight: 500; color: var(--fnx-secondary);';
                  hudEl.appendChild(arrowSpan);
                  hudEl.appendChild(labelSpan);
                  (shadowRoot.getElementById('__fynexo_shell__') || shadowRoot).appendChild(hudEl);
               }
               return hudEl;
            }

            function hideHud() {
               if (hudEl) hudEl.style.display = 'none';
            }

            function updateHud() {
               if (!path) {
                  hideHud();
                  return;
               }
               const el = ensureHudEl();
               if (!el) return;
               let arrows = '';
               for (let i = 0; i < path.length; i++) {
                  arrows += ARROWS[path[i]] || '';
               }
               const match = GESTURE_MAP[path];
               arrowSpan.textContent = arrows;
               labelSpan.textContent = match ? match.label : 'Cancel';
               el.style.display = 'flex';
            }

            const THRESHOLD = 18;

            const onMouseDown = (e) => {
               if (e.button !== 2 || e.shiftKey || isMobilePlatform()) return;
               if (isEventInsideHud(e)) return;
               tracking = true;
               lastX = e.clientX;
               lastY = e.clientY;
               path = '';
            };

            const onMouseMove = (e) => {
               if (!tracking) return;
               if ((e.buttons & 2) === 0) {
                  tracking = false;
                  path = '';
                  hideHud();
                  return;
               }
               const dx = e.clientX - lastX;
               const dy = e.clientY - lastY;
               const absX = Math.abs(dx);
               const absY = Math.abs(dy);
               if (absX < THRESHOLD && absY < THRESHOLD) return;

               const dir =
                  absX >= absY
                     ? dx > 0
                        ? 'R'
                        : 'L'
                     : dy > 0
                       ? 'D'
                       : 'U';
               lastX = e.clientX;
               lastY = e.clientY;
               if (path.length === 0 || (path.length < 2 && path[path.length - 1] !== dir)) {
                  path += dir;
                  updateHud();
               } else if (path.length > 0 && path[path.length - 1] === dir) {
                  updateHud();
               }
            };

            const onMouseUp = (e) => {
               if (!tracking || e.button !== 2) return;
               tracking = false;
               const drawn = path;
               path = '';
               hideHud();
               if (drawn) {
                  suppressCtxUntil = Date.now() + 450;
                  const match = GESTURE_MAP[drawn];
                  if (match && typeof match.run === 'function') {
                     try {
                        match.run();
                     } catch (_) {}
                  }
               }
            };

            const onContextMenu = (e) => {
               if (suppressCtxUntil && Date.now() <= suppressCtxUntil) {
                  suppressCtxUntil = 0;
                  e.preventDefault();
                  e.stopPropagation();
               }
            };

            const onBlur = () => {
               tracking = false;
               path = '';
               hideHud();
            };

            window.addEventListener('mousedown', onMouseDown, { capture: true, passive: true });
            window.addEventListener('mousemove', onMouseMove, { capture: true, passive: true });
            window.addEventListener('mouseup', onMouseUp, { capture: true, passive: true });
            window.addEventListener('contextmenu', onContextMenu, { capture: true });
            window.addEventListener('blur', onBlur, { passive: true });

            gestureStopTeardown = () => {
               tracking = false;
               path = '';
               hideHud();
               if (hudEl && hudEl.isConnected) hudEl.remove();
               hudEl = null;
               window.removeEventListener('mousedown', onMouseDown, { capture: true });
               window.removeEventListener('mousemove', onMouseMove, { capture: true });
               window.removeEventListener('mouseup', onMouseUp, { capture: true });
               window.removeEventListener('contextmenu', onContextMenu, { capture: true });
               window.removeEventListener('blur', onBlur);
            };
         }
         syncMouseGestures();