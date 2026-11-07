use super::*;

#[test]
#[cfg(unix)]
fn resource_samples_convert_platform_units_and_reject_failures() {
    let mut usage: libc::rusage = unsafe { std::mem::zeroed() };
    usage.ru_utime.tv_sec = 2;
    usage.ru_utime.tv_usec = 300;
    usage.ru_stime.tv_usec = 700;
    usage.ru_maxrss = 4096;
    assert!(from_sample(-1, &usage, false).is_none());
    let linux = from_sample(0, &usage, false).unwrap();
    assert_eq!(linux.user, Duration::from_micros(2_000_300));
    assert_eq!(linux.sys, Duration::from_micros(700));
    assert_eq!(linux.peak_rss_kib, 4096);
    assert_eq!(from_sample(0, &usage, true).unwrap().peak_rss_kib, 4);
    usage.ru_maxrss = -1;
    assert!(from_sample(0, &usage, false).is_none());
    usage.ru_maxrss = 1;
    usage.ru_utime.tv_sec = -1;
    assert!(from_sample(0, &usage, false).is_none());
    usage.ru_utime.tv_sec = 0;
    usage.ru_stime.tv_usec = -1;
    assert!(from_sample(0, &usage, false).is_none());
}

#[test]
#[cfg(not(unix))]
fn unsupported_platform_has_no_resources() {
    assert!(sample().is_none());
}

thread_local! {
    static SAMPLES: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
}
#[cfg(unix)]
pub(crate) fn record_sample() {
    SAMPLES.with(|count| count.set(count.get() + 1));
}
pub(crate) fn sample_count() -> usize {
    SAMPLES.with(std::cell::Cell::get)
}
