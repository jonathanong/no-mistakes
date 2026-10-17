use super::super::super::expired_timeout_from;
use super::super::offset::{decode_instant, encode_instant, epoch};
use super::*;
use std::cell::Cell;

fn image(at: Instant) -> Encoded {
    let (expires_secs, expires_nanos) = encode_instant(at);
    Encoded {
        expires_secs,
        expires_nanos,
        timeout_secs: 0,
        timeout_nanos: 1234,
        committed: false,
        owner_token: 0,
    }
}

#[test]
fn signed_expiry_matches_full_decoder_before_and_after_epoch() {
    let center = epoch();
    let span = Duration::from_nanos(i64::MAX as u64);
    let mut instants = vec![center];
    for distance in [
        Duration::from_nanos(1),
        Duration::from_secs(1),
        span,
        span + Duration::from_secs(1),
    ] {
        instants.extend(center.checked_add(distance));
        instants.extend(center.checked_sub(distance));
    }
    for &expires in &instants {
        let encoded = image(expires);
        for &now in &instants {
            assert_eq!(
                encoded.expired_at(now),
                now >= decode_instant(encoded.expires_secs, encoded.expires_nanos)
            );
        }
    }
}

#[test]
fn future_skips_metadata_decode_but_expiry_decodes_second_image() {
    let now = epoch();
    let future = now + Duration::from_secs(1);
    let old_decodes = Cell::new(0);
    let old = image(future).decode_with(|()| old_decodes.set(old_decodes.get() + 1));
    assert!(old.applies_here());
    assert!(now < old.expires_at);
    let decodes = Cell::new(0);
    let result = expired_timeout_from(
        || Some(image(future)),
        || now,
        |raw| {
            decodes.set(decodes.get() + 1);
            raw.timeout()
        },
    );
    assert!(result.is_none());
    assert_eq!((old_decodes.get(), decodes.get()), (2, 0));
    let reads = Cell::new(0u32);
    let result = expired_timeout_from(
        || {
            reads.set(reads.get() + 1);
            let mut raw = image(now);
            raw.timeout_nanos = reads.get();
            Some(raw)
        },
        || now,
        |raw| {
            decodes.set(decodes.get() + 1);
            raw.timeout()
        },
    );
    assert_eq!(result, Some(Duration::from_nanos(2)));
    assert_eq!((reads.get(), decodes.get()), (2, 1));
}

#[test]
fn commit_foreign_owner_clear_and_future_restore_do_not_fire() {
    let now = epoch();
    for mode in 0..4 {
        let mut reads = 0;
        let result = expired_timeout_from(
            || {
                reads += 1;
                if reads == 1 {
                    return Some(image(now));
                }
                let mut raw = image(now);
                match mode {
                    0 => raw.committed = true,
                    1 => raw.owner_token = u64::MAX,
                    2 => return None,
                    _ => raw = image(now + Duration::from_secs(1)),
                }
                Some(raw)
            },
            || now,
            |_| panic!("non-applicable image must not decode timeout"),
        );
        assert!(result.is_none());
        assert_eq!(reads, 2);
    }
    let clocks = Cell::new(0);
    let result = expired_timeout_from(
        || {
            let mut raw = image(now);
            raw.committed = true;
            Some(raw)
        },
        || {
            clocks.set(clocks.get() + 1);
            now
        },
        |_| panic!("committed"),
    );
    assert!(result.is_none());
    assert_eq!(clocks.get(), 0);
}
