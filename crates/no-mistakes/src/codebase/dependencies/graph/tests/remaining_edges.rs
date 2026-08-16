#[test]
fn collect_remaining_edges_parallelizes_independent_kinds() {
    let remaining = include_str!("../builder_remaining_edges.rs");
    let independent = include_str!("../builder_remaining_edges_independent.rs");
    assert!(
        remaining.contains("collect_independent_remaining_edges"),
        "remaining-edge orchestration must collect the independent language panel"
    );
    assert!(
        independent.contains("rayon::join"),
        "markdown vs terraform/dotnet/swift must collect via rayon::join"
    );
    assert!(
        independent.contains("collect_md_edges"),
        "independent panel must collect markdown edges"
    );
    assert!(
        independent.contains("collect_terraform_edges_for_plan")
            && independent.contains("collect_dotnet_edges_for_plan")
            && independent.contains("collect_swift_edges_for_plan"),
        "independent panel must collect terraform/dotnet/swift beside markdown"
    );
    assert!(
        independent.contains("collect_unless_timed_out")
            && !independent.contains("let _ = crate::invocation::check_timeout()"),
        "independent join leaves must return an empty batch when the deadline has elapsed"
    );
    assert!(
        independent.contains("with_observer_and_timing"),
        "independent join leaves must inherit the caller's timing kind"
    );
}

#[test]
fn with_observer_and_timing_installs_kind_on_this_thread() {
    let observer = crate::diagnostics::InvocationObserver::new(true);
    super::super::with_observer_and_timing(
        Some(observer),
        crate::diagnostics::TimingKind::Parallel,
        || {
            assert_eq!(
                crate::diagnostics::current_timing_kind(),
                crate::diagnostics::TimingKind::Parallel
            );
        },
    );
}

#[test]
fn collect_unless_timed_out_skips_work_after_deadline() {
    let _deadline = crate::invocation::install_test_deadline(std::time::Duration::ZERO)
        .expect("expired deadline");
    assert!(super::super::collect_unless_timed_out(|| vec![1]).is_empty());
    assert!(
        super::super::collect_unless_timed_out_or(Ok::<Option<()>, ()>(None), || Ok(Some(())))
            .unwrap()
            .is_none()
    );
}

#[test]
fn collect_unless_timed_out_runs_when_no_deadline() {
    assert_eq!(super::super::collect_unless_timed_out(|| vec![1]), vec![1]);
    assert_eq!(
        super::super::collect_unless_timed_out_or(Ok::<Option<i32>, ()>(None), || Ok(Some(7)))
            .unwrap(),
        Some(7)
    );
}
