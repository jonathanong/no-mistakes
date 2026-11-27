use super::lock::{lock_busy_error, lock_timeout_error};
use anyhow::Result;
use std::sync::{Condvar, Mutex, PoisonError};
use std::time::{Duration, Instant};

static HELD: Mutex<bool> = Mutex::new(false);
static RELEASED: Condvar = Condvar::new();

pub(super) struct ProcessLock;

impl ProcessLock {
    pub(super) fn acquire(
        timeout: Option<Duration>,
        started: Instant,
        fail_on_lock: bool,
        check_first_attempt: bool,
    ) -> Result<Self> {
        let mut held = HELD.lock().unwrap_or_else(PoisonError::into_inner);
        if *held && fail_on_lock {
            return Err(lock_busy_error(None));
        }
        let wait_timeout = timeout.filter(|_| *held || check_first_attempt);
        held = match wait_timeout {
            Some(timeout) => {
                let remaining = timeout.saturating_sub(started.elapsed());
                let (held, _) = RELEASED
                    .wait_timeout_while(held, remaining, |held| *held)
                    .unwrap_or_else(PoisonError::into_inner);
                if *held || started.elapsed() >= timeout {
                    return Err(lock_timeout_error(timeout, None));
                }
                held
            }
            None => RELEASED
                .wait_while(held, |held| *held)
                .unwrap_or_else(PoisonError::into_inner),
        };
        *held = true;
        Ok(Self)
    }
}

impl Drop for ProcessLock {
    fn drop(&mut self) {
        *HELD.lock().unwrap_or_else(PoisonError::into_inner) = false;
        RELEASED.notify_all();
    }
}
