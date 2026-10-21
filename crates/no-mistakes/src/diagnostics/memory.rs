//! Process-lifetime memory diagnostics, sampled only at the verbose CLI boundary.

pub(super) fn memory_line(verbose: bool, rss: Option<u64>) -> Option<String> {
    diagnostic(verbose, || rss)
}

fn diagnostic(verbose: bool, sample: impl FnOnce() -> Option<u64>) -> Option<String> {
    verbose.then(sample).flatten().map(format_peak_rss)
}

fn format_peak_rss(rss: u64) -> String {
    format!("[memory] peak_rss_kib: {rss}\n")
}

#[cfg(unix)]
pub(super) fn peak_rss_kib() -> Option<u64> {
    sampled_peak_rss(
        |usage| {
            // SAFETY: usage points to an initialized writable rusage value.
            unsafe { libc::getrusage(libc::RUSAGE_SELF, usage) }
        },
        cfg!(target_os = "macos"),
    )
}

#[cfg(unix)]
fn sampled_peak_rss(
    sample: impl FnOnce(&mut libc::rusage) -> libc::c_int,
    bytes: bool,
) -> Option<u64> {
    // SAFETY: rusage contains only integer fields and integer-only time values.
    let mut usage: libc::rusage = unsafe { std::mem::zeroed() };
    if sample(&mut usage) != 0 {
        return None;
    }
    let rss = u64::try_from(usage.ru_maxrss).ok()?;
    // Darwin reports bytes; Linux and the other supported Unix targets use KiB.
    Some(if bytes { rss / 1024 } else { rss })
}

#[cfg(not(unix))]
pub(super) fn peak_rss_kib() -> Option<u64> {
    None
}

#[cfg(test)]
mod tests;
