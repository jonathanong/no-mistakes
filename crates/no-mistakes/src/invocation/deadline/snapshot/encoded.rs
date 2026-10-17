use super::offset::{decode_instant, decode_timeout};
use super::{current_owner_token, Observed};
use std::time::{Duration, Instant};

/// Accepted atomic image; decoding is deferred for the ordinary expiry check.
pub(in crate::invocation::deadline) struct Encoded {
    pub(super) expires_secs: i64,
    pub(super) expires_nanos: u32,
    pub(super) timeout_secs: u64,
    pub(super) timeout_nanos: u32,
    pub(super) committed: bool,
    pub(super) owner_token: u64,
}

impl Encoded {
    pub(in crate::invocation::deadline) fn applies_here(&self) -> bool {
        !self.committed && (self.owner_token == 0 || self.owner_token == current_owner_token())
    }

    pub(in crate::invocation::deadline) fn expired_at(&self, now: Instant) -> bool {
        // `now` stays a real Instant. Saturating it to the stored range would
        // hide an offset that is still ahead of a clamped publication.
        now >= decode_instant(self.expires_secs, self.expires_nanos)
    }

    pub(in crate::invocation::deadline) fn timeout(&self) -> Duration {
        decode_timeout(self.timeout_secs, self.timeout_nanos)
    }

    pub(super) fn decode(self) -> Observed {
        self.decode_with(std::mem::drop)
    }

    fn decode_with(self, mut decoded: impl FnMut(())) -> Observed {
        decoded(());
        let expires_at = decode_instant(self.expires_secs, self.expires_nanos);
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
