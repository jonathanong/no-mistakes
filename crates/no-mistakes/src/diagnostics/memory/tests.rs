#[test]
#[cfg(unix)]
fn process_peak_rss_is_available_and_positive() {
    assert!(super::peak_rss_kib().is_some_and(|rss| rss > 0));
}

#[test]
#[cfg(not(unix))]
fn unsupported_platform_omits_peak_rss() {
    assert_eq!(super::peak_rss_kib(), None);
}

#[test]
fn disabled_memory_diagnostics_never_sample_or_format() {
    assert_eq!(
        super::diagnostic(false, || panic!("disabled diagnostics sampled memory")),
        None
    );
    assert_eq!(
        super::diagnostic(true, || Some(42)).as_deref(),
        Some("[memory] peak_rss_kib: 42\n")
    );
    assert_eq!(super::diagnostic(true, || None), None);
}
