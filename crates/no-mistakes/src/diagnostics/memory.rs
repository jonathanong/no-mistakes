//! Process-lifetime memory diagnostics, sampled only at the verbose CLI boundary.

pub(super) fn render(verbose: bool, output: &mut Option<String>) {
    if let Some(memory) = diagnostic(verbose, peak_rss_kib) {
        match output {
            Some(output) => output.push_str(&memory),
            None => eprint!("{memory}"),
        }
    }
}

fn diagnostic(verbose: bool, sample: impl FnOnce() -> Option<u64>) -> Option<String> {
    verbose
        .then(sample)
        .flatten()
        .map(|rss| format!("[memory] peak_rss_kib: {rss}\n"))
}

#[cfg(unix)]
pub(super) fn peak_rss_kib() -> Option<u64> {
    let mut usage = std::mem::MaybeUninit::<libc::rusage>::uninit();
    // SAFETY: getrusage initializes the supplied rusage on success.
    if unsafe { libc::getrusage(libc::RUSAGE_SELF, usage.as_mut_ptr()) } != 0 {
        return None;
    }
    // SAFETY: the successful syscall initialized usage.
    let rss = u64::try_from(unsafe { usage.assume_init() }.ru_maxrss).ok()?;
    // Darwin reports bytes; Linux and the other supported Unix targets use KiB.
    Some(if cfg!(target_os = "macos") {
        rss / 1024
    } else {
        rss
    })
}

#[cfg(not(unix))]
pub(super) fn peak_rss_kib() -> Option<u64> {
    None
}

#[cfg(test)]
mod tests;
