use std::sync::{Arc, Condvar, Mutex};
use std::time::{Duration, Instant};
use tao::event_loop::EventLoopProxy;

use crate::util::LockExt;

use super::events::UserEvent;

type RevealEntry = (Instant, usize, u64);
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
                loop {
                    let due_events: Vec<(usize, u64)> = {
                        let mut guard = lock.lock_safe();
                        loop {
                            let now = Instant::now();
                            if let Some(earliest) = guard.iter().map(|(at, _, _)| *at).min() {
                                if earliest <= now {
                                    break;
                                }
                                let (g, _) = cv
                                    .wait_timeout(guard, earliest.saturating_duration_since(now))
                                    .unwrap_or_else(|p| p.into_inner());
                                guard = g;
                            } else {
                                guard = cv.wait(guard).unwrap_or_else(|p| p.into_inner());
                            }
                        }
                        let now = Instant::now();
                        let mut fired = Vec::new();
                        let mut i = 0;
                        while i < guard.len() {
                            if guard[i].0 <= now {
                                let (_, id, gen) = guard.swap_remove(i);
                                fired.push((id, gen));
                            } else {
                                i += 1;
                            }
                        }
                        fired
                    };
                    for (id, gen) in due_events {
                        let _ = worker_proxy.send_event(UserEvent::RevealPendingTab { id, gen });
                    }
                }
            });
        Self { proxy, queue }
    }

    pub fn proxy(&self) -> &EventLoopProxy<UserEvent> {
        &self.proxy
    }

    pub fn schedule(&self, id: usize, gen: u64, delay: Duration) {
        let (lock, cv) = &*self.queue;
        let mut guard = lock.lock_safe();
        guard.retain(|&(_, tid, _)| tid != id);
        guard.push((Instant::now() + delay, id, gen));
        cv.notify_one();
    }
}
