use std::sync::OnceLock;
use std::time::{Duration, Instant};

const NANOS_PER_SEC: u32 = 1_000_000_000;
/// `i64` seconds cannot represent a magnitude of `2^63` or more.
const I64_MAGNITUDE_LIMIT: u64 = (i64::MAX as u64) + 1;

static EPOCH: OnceLock<Instant> = OnceLock::new();

pub(super) fn epoch() -> Instant {
    *EPOCH.get_or_init(Instant::now)
}

pub(super) fn encode_timeout(timeout: Duration) -> (u64, u32) {
    (timeout.as_secs(), timeout.subsec_nanos())
}

pub(super) fn decode_timeout(secs: u64, subsec_nanos: u32) -> Duration {
    Duration::new(secs, subsec_nanos)
}

pub(super) fn encode_instant(instant: Instant) -> (i64, u32) {
    let epoch = epoch();
    if instant >= epoch {
        signed_offset(false, instant.duration_since(epoch))
    } else {
        signed_offset(true, epoch.duration_since(instant))
    }
}

pub(super) fn decode_instant(secs: i64, subsec_nanos: u32) -> Instant {
    let epoch = epoch();
    if secs >= 0 {
        shift(epoch, Duration::new(secs as u64, subsec_nanos), true)
    } else {
        shift(epoch, duration_before(secs, subsec_nanos), false)
    }
}

/// `-1.5s` is `-2s + 500_000_000ns`: the remainder stays non-negative.
fn signed_offset(before_epoch: bool, duration: Duration) -> (i64, u32) {
    if before_epoch {
        negative_offset(duration)
    } else {
        positive_offset(duration)
    }
}

fn positive_offset(duration: Duration) -> (i64, u32) {
    match i64::try_from(duration.as_secs()) {
        Ok(secs) => (secs, duration.subsec_nanos()),
        // Largest `i64` magnitude, one nanosecond short of `2^63` seconds.
        // When `shift` cannot add that clamp it returns the process epoch, so
        // the deadline is already expired. Saturation used to stay far ahead.
        Err(_) => (i64::MAX, NANOS_PER_SEC - 1),
    }
}

fn negative_offset(duration: Duration) -> (i64, u32) {
    let secs = duration.as_secs();
    let nanos = duration.subsec_nanos();
    let (magnitude, subsec_nanos) = if nanos == 0 {
        (secs, 0)
    } else {
        let Some(magnitude) = secs.checked_add(1) else {
            // Same epoch fallback as a positive clamp: already expired.
            return (i64::MIN, 0);
        };
        (magnitude, NANOS_PER_SEC - nanos)
    };
    if magnitude < I64_MAGNITUDE_LIMIT {
        (-(magnitude as i64), subsec_nanos)
    } else if magnitude == I64_MAGNITUDE_LIMIT {
        (i64::MIN, subsec_nanos)
    } else {
        // Wider than `i64`. Decode falls back to the epoch when `Instant`
        // cannot hold it, so the deadline is already expired.
        (i64::MIN, 0)
    }
}

fn duration_before(secs: i64, subsec_nanos: u32) -> Duration {
    if subsec_nanos == 0 {
        Duration::new(secs.unsigned_abs(), 0)
    } else {
        Duration::new(secs.unsigned_abs() - 1, NANOS_PER_SEC - subsec_nanos)
    }
}

/// Offsets `Instant` cannot hold clamp toward the process epoch.
///
/// A normal check then sees `now >= epoch`, so the deadline is already
/// expired. This does not restore the old far-future saturation.
fn shift(base: Instant, duration: Duration, forward: bool) -> Instant {
    let shifted = if forward {
        base.checked_add(duration)
    } else {
        base.checked_sub(duration)
    };
    shifted.unwrap_or(base)
}

#[cfg(test)]
#[path = "offset_tests.rs"]
mod tests;
