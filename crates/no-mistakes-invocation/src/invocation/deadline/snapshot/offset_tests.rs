use super::super::super::super::super::deadline_test_lock;
use super::super::super::super::Deadline;
use super::super::super::active_deadline;
use super::super::{load, publish};
use super::{
    decode_instant, decode_timeout, encode_instant, encode_timeout, epoch, shift, signed_offset,
};
use std::time::{Duration, Instant};

#[test]
fn timeout_parts_keep_subseconds_and_durations_past_u64_nanos() {
    for timeout in sample_timeouts() {
        let (secs, subsec_nanos) = encode_timeout(timeout);
        assert_eq!(
            (secs, subsec_nanos),
            (timeout.as_secs(), timeout.subsec_nanos())
        );
        assert_eq!(decode_timeout(secs, subsec_nanos), timeout);
    }
    let millennium = Duration::from_secs(31_556_952_000);
    assert!(millennium.as_nanos() > u128::from(u64::MAX));
    assert!(u64::try_from(millennium.as_nanos()).is_err());
}

#[test]
fn snapshot_words_keep_timeouts_the_nanos_encoding_drops() {
    let _serial = deadline_test_lock()
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    let previous = active_deadline()
        .read()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .as_ref()
        .copied();
    let _restore = Restore(previous);
    let epoch = epoch();
    for timeout in sample_timeouts() {
        let expires_at = epoch
            .checked_add(timeout)
            .unwrap_or_else(|| epoch + Duration::from_secs(1));
        publish(Some(Deadline {
            expires_at,
            timeout,
            owner: None,
            committed: false,
        }));
        let observed = load().expect("published deadline");
        assert_eq!(
            (observed.timeout, observed.expires_at),
            (timeout, expires_at)
        );
    }
}

#[test]
fn instant_offsets_round_trip_subseconds_and_a_millennium_when_representable() {
    let epoch = epoch();
    for duration in sample_timeouts() {
        let (secs, subsec_nanos) = signed_offset(false, duration);
        assert_eq!(
            (secs, subsec_nanos),
            (duration.as_secs() as i64, duration.subsec_nanos())
        );
        let Some(later) = epoch.checked_add(duration) else {
            continue;
        };
        assert_eq!(encode_instant(later), (secs, subsec_nanos));
        assert_eq!(decode_instant(secs, subsec_nanos), later);
    }
}

#[test]
fn negative_epoch_offset_normalizes_subsec_nanos() {
    let epoch = epoch();
    let cases = [
        (Duration::from_nanos(1), -1, 999_999_999),
        (Duration::from_millis(1), -1, 999_000_000),
        (Duration::new(0, 999_999_999), -1, 1),
        (Duration::from_secs(2), -2, 0),
        (Duration::new(1, 1), -2, 999_999_999),
        (Duration::new(4, 500_000_000), -5, 500_000_000),
    ];
    for (duration, secs, subsec_nanos) in cases {
        assert_eq!(signed_offset(true, duration), (secs, subsec_nanos));
        let Some(earlier) = epoch.checked_sub(duration) else {
            continue;
        };
        assert_eq!(encode_instant(earlier), (secs, subsec_nanos));
        assert_eq!(decode_instant(secs, subsec_nanos), earlier);
        publish_and_restore(earlier, duration);
    }
    let past_i64_nanos = Duration::from_nanos(i64::MAX as u64) + Duration::from_secs(1);
    assert!(past_i64_nanos.as_nanos() > u128::from(i64::MAX as u64));
    let (secs, subsec_nanos) = signed_offset(true, past_i64_nanos);
    assert_ne!((secs, subsec_nanos), (-1, 0));
    if let Some(earlier) = epoch.checked_sub(past_i64_nanos) {
        assert_eq!(encode_instant(earlier), (secs, subsec_nanos));
        assert_eq!(decode_instant(secs, subsec_nanos), earlier);
        publish_and_restore(earlier, past_i64_nanos);
    }
}

#[test]
fn offsets_wider_than_i64_seconds_saturate_and_still_decode() {
    let epoch = epoch();
    assert_eq!(
        signed_offset(false, Duration::new(I64_LIMIT, 0)),
        (i64::MAX, 999_999_999)
    );
    assert_eq!(
        signed_offset(false, Duration::new(i64::MAX as u64, 999_999_999)),
        (i64::MAX, 999_999_999)
    );
    assert_eq!(
        signed_offset(true, Duration::new(u64::MAX, 0)),
        (i64::MIN, 0)
    );
    assert_eq!(
        signed_offset(true, Duration::new(u64::MAX, 1)),
        (i64::MIN, 0)
    );
    assert_eq!(
        signed_offset(true, Duration::new(I64_LIMIT, 0)),
        (i64::MIN, 0)
    );
    assert_eq!(
        signed_offset(true, Duration::new(I64_LIMIT, 1)),
        (i64::MIN, 0)
    );
    let just_inside = Duration::new(I64_LIMIT - 1, 1);
    assert_eq!(signed_offset(true, just_inside), (i64::MIN, 999_999_999));
    assert_eq_instant(
        epoch.checked_sub(just_inside),
        decode_instant(i64::MIN, 999_999_999),
        epoch,
    );
    assert_eq_instant(
        epoch.checked_sub(Duration::new(I64_LIMIT, 0)),
        decode_instant(i64::MIN, 0),
        epoch,
    );
    assert_eq_instant(
        epoch.checked_add(Duration::new(i64::MAX as u64, 999_999_999)),
        decode_instant(i64::MAX, 999_999_999),
        epoch,
    );
    assert_eq!(shift(epoch, Duration::MAX, true), epoch);
    assert_eq!(shift(epoch, Duration::MAX, false), epoch);
    let one_nanos = Duration::from_nanos(1);
    assert_eq!(shift(epoch, one_nanos, true), epoch + one_nanos);
    assert_eq_instant(
        epoch.checked_sub(one_nanos),
        shift(epoch, one_nanos, false),
        epoch,
    );
}

fn publish_and_restore(expires_at: Instant, timeout: Duration) {
    let _serial = deadline_test_lock()
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    let previous = active_deadline()
        .read()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .as_ref()
        .copied();
    let _restore = Restore(previous);
    publish(Some(Deadline {
        expires_at,
        timeout,
        owner: None,
        committed: false,
    }));
    let observed = load().expect("published negative offset");
    assert_eq!(
        (observed.expires_at, observed.timeout),
        (expires_at, timeout)
    );
}

fn assert_eq_instant(expected: Option<Instant>, decoded: Instant, epoch: Instant) {
    assert_eq!(decoded, expected.unwrap_or(epoch));
}

fn sample_timeouts() -> [Duration; 6] {
    [
        Duration::from_nanos(1),
        Duration::from_millis(1),
        Duration::from_nanos(999_999_999),
        Duration::new(12, 34),
        Duration::from_secs(31_556_952_000),
        Duration::new(31_556_952_000, 999_999_999),
    ]
}

const I64_LIMIT: u64 = (i64::MAX as u64) + 1;

struct Restore(Option<Deadline>);

impl Drop for Restore {
    fn drop(&mut self) {
        publish(self.0);
    }
}
