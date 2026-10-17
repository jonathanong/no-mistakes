use super::{current_owner_token, decode_instant, epoch, Observed};
use std::time::{Duration, Instant};

/// Accepted atomic image; decoding is deferred for the ordinary expiry check.
pub(in crate::invocation::deadline) struct Encoded {
    pub(super) expires_nanos: u64,
    pub(super) timeout_nanos: u64,
    pub(super) committed: bool,
    pub(super) owner_token: u64,
}

impl Encoded {
    pub(in crate::invocation::deadline) fn applies_here(&self) -> bool {
        !self.committed && (self.owner_token == 0 || self.owner_token == current_owner_token())
    }

    pub(in crate::invocation::deadline) fn expired_at(&self, now: Instant) -> bool {
        let expiry = self.expires_nanos as i64;
        let epoch = epoch();
        // Published offsets clamp toward the epoch, so their decoded Instant
        // remains representable. Never clamp `now`: before-epoch distances can
        // exceed i64::MAX and must retain their ordering against that boundary.
        if now >= epoch {
            expiry < 0 || now.duration_since(epoch).as_nanos() >= expiry as u128
        } else {
            expiry < 0 && epoch.duration_since(now).as_nanos() <= u128::from(expiry.unsigned_abs())
        }
    }

    pub(in crate::invocation::deadline) fn timeout(&self) -> Duration {
        Duration::from_nanos(self.timeout_nanos)
    }

    pub(super) fn decode(self) -> Observed {
        self.decode_with(std::mem::drop)
    }

    fn decode_with(self, mut decoded: impl FnMut(())) -> Observed {
        decoded(());
        let expires_at = decode_instant(self.expires_nanos);
        decoded(());
        let timeout = self.timeout();
        Observed {
            expires_at,
            timeout,
            committed: self.committed,
            owner_token: self.owner_token,
        }
    }
}

#[cfg(test)]
#[path = "encoded_tests.rs"]
mod tests;
