use std::sync::{Arc, Condvar, Mutex};
use std::time::{Duration, Instant};
use tao::event_loop::EventLoopProxy;

use crate::util::LockExt;

use super::events::UserEvent;

type RevealEntry = (Instant, usize, usize, u64);
type RevealQueue = Arc<(Mutex<Vec<RevealEntry>>, Condvar)>;

pub struct RevealScheduler {
    proxy: EventLoopProxy<UserEvent>,
    queue: RevealQueue,
}

impl RevealScheduler {
    pub fn new(proxy: EventLoopProxy<UserEvent>) -> Self {
        let queue: RevealQueue = Arc::new((Mutex::new(Vec::new()), Condvar::new()));
        let worker_q = Arc::clone(&queue);
        let worker_proxy = proxy.clone();
        let _ = std::thread::Builder::new()
            .name("fnx-reveal-timer".to_string())
            .spawn(move || {
                let (lock, cv) = &*worker_q;
                let mut next_sweep = Instant::now() + Duration::from_secs(30);
                loop {
                    let due_events: Vec<(usize, usize, u64)> = {
                        let mut guard = lock.lock_safe();
                        loop {
                            let now = Instant::now();
                            let next_reveal = guard.iter().map(|(at, _, _, _)| *at).min();
                            let wake_at = match next_reveal {
                                Some(r) if r < next_sweep => r,
                                _ => next_sweep,
                            };
                            if wake_at <= now {
                                break;
                            }
                            let (g, _) = cv
                                .wait_timeout(guard, wake_at.saturating_duration_since(now))
                                .unwrap_or_else(|p| p.into_inner());
                            guard = g;
                        }
                        let now = Instant::now();
                        let mut fired = Vec::new();
                        let mut i = 0;
                        while i < guard.len() {
                            if guard[i].0 <= now {
                                let (_, win_id, id, gen) = guard.swap_remove(i);
                                fired.push((win_id, id, gen));
                            } else {
                                i += 1;
                            }
                        }
                        fired
                    };
                    for (win_id, id, gen) in due_events {
                        let _ = worker_proxy.send_event(UserEvent::RevealPendingTab {
                            win_id,
                            id,
                            gen,
                        });
                    }
                    let now = Instant::now();
                    if now >= next_sweep {
                        next_sweep = now + Duration::from_secs(30);
                        let _ = worker_proxy.send_event(UserEvent::SweepIdleTabs);
                    }
                }
            });
        Self { proxy, queue }
    }

    pub fn proxy(&self) -> &EventLoopProxy<UserEvent> {
        &self.proxy
    }

    pub fn schedule(&self, win_id: usize, id: usize, gen: u64, delay: Duration) {
        let (lock, cv) = &*self.queue;
        let mut guard = lock.lock_safe();
        guard.retain(|&(_, wid, tid, _)| wid != win_id || tid != id);
        guard.push((Instant::now() + delay, win_id, id, gen));
        cv.notify_one();
    }
}
