use std::time::Duration;

#[derive(Clone, Copy)]
pub(super) struct Usage {
    pub(super) user: Duration,
    pub(super) sys: Duration,
    pub(super) peak_rss_kib: u64,
}

#[cfg(unix)]
pub(super) fn sample() -> Option<Usage> {
    // no-mistakes-disable-next-line rust-no-inline-tests
    #[cfg(test)]
    tests::record_sample();
    // SAFETY: rusage contains only integer fields and integer-only time values.
    let mut usage: libc::rusage = unsafe { std::mem::zeroed() };
    // SAFETY: usage is initialized writable storage for RUSAGE_SELF.
    let status = unsafe { libc::getrusage(libc::RUSAGE_SELF, &mut usage) };
    from_sample(status, &usage, cfg!(target_os = "macos"))
}

#[cfg(unix)]
fn from_sample(status: libc::c_int, usage: &libc::rusage, bytes: bool) -> Option<Usage> {
    if status != 0 {
        return None;
    }
    let rss = u64::try_from(usage.ru_maxrss).ok()?;
    Some(Usage {
        user: timeval(usage.ru_utime)?,
        sys: timeval(usage.ru_stime)?,
        peak_rss_kib: if bytes { rss / 1024 } else { rss },
    })
}

#[cfg(unix)]
fn timeval(time: libc::timeval) -> Option<Duration> {
    Some(
        Duration::from_secs(u64::try_from(time.tv_sec).ok()?)
            + Duration::from_micros(u64::try_from(time.tv_usec).ok()?),
    )
}

#[cfg(not(unix))]
pub(super) fn sample() -> Option<Usage> {
    None
}

#[cfg(test)]
pub(super) mod tests;
