use super::traversal::EdgeTraversalIndex;
use super::{
    attach_jobs_pool_warning, edge_view, init_rayon_threads, init_rayon_threads_if_requested,
    jobs_pool_mismatch, rayon_thread_count, recorded_rayon_threads, resolve_optional_root,
    resolve_root, JobsArg, TraversableEdge,
};
use crate::queue::{Edge, EdgeKind};
use std::path::Path;

fn edge(from: &str, to: &str) -> Edge {
    Edge {
        from: from.into(),
        to: to.into(),
        kind: EdgeKind::QueueEnqueue,
    }
}

#[derive(Clone, Eq, Ord, PartialEq, PartialOrd)]
enum NonHashKind {
    Dependency,
}

#[derive(Clone, PartialEq, Debug)]
struct PublicCompatibilityEdge {
    from: String,
    to: String,
}

impl TraversableEdge for PublicCompatibilityEdge {
    type Kind = NonHashKind;

    fn source(&self) -> &str {
        &self.from
    }

    fn target(&self) -> &str {
        &self.to
    }

    fn kind(&self) -> Self::Kind {
        NonHashKind::Dependency
    }
}

#[test]
fn resolve_root_preserves_absolute_paths() {
    let cwd = Path::new("/repo");
    let root = Path::new("/workspace/app");

    assert_eq!(resolve_root(root, cwd), root);
}

#[test]
fn resolve_root_joins_relative_paths() {
    assert_eq!(
        resolve_root(Path::new("app"), Path::new("/repo")),
        Path::new("/repo/app")
    );
}

#[test]
fn resolve_optional_root_defaults_to_cwd() {
    let cwd = Path::new("/repo");

    assert_eq!(resolve_optional_root(None, cwd), cwd);
}

#[test]
fn resolve_optional_root_resolves_provided_root() {
    assert_eq!(
        resolve_optional_root(Some(Path::new("app")), Path::new("/repo")),
        Path::new("/repo/app")
    );
}

#[test]
fn init_rayon_threads_uses_cpu_default_without_jobs_or_env() {
    init_rayon_threads(JobsArg { jobs: 0 });
}

#[test]
fn init_rayon_threads_if_requested_defaults_unset_jobs_to_cpu_count() {
    init_rayon_threads_if_requested(None);
    init_rayon_threads_if_requested(Some(0));
}

#[test]
fn rayon_thread_count_prefers_jobs_then_env_then_cpu_default() {
    assert_eq!(rayon_thread_count(JobsArg { jobs: 4 }, Some("2")), 4);
    assert_eq!(rayon_thread_count(JobsArg { jobs: 0 }, Some("2")), 2);
    assert_eq!(
        rayon_thread_count(JobsArg { jobs: 0 }, Some("not-a-number")),
        num_cpus::get()
    );
    assert_eq!(
        rayon_thread_count(JobsArg { jobs: 0 }, None),
        num_cpus::get()
    );
    assert_eq!(
        rayon_thread_count(JobsArg { jobs: 0 }, Some("0")),
        num_cpus::get()
    );
}

#[test]
fn jobs_pool_mismatch_warns_only_when_resolved_count_differs() {
    assert!(jobs_pool_mismatch(4, None).is_none());
    assert!(jobs_pool_mismatch(4, Some(4)).is_none());
    let warning = jobs_pool_mismatch(8, Some(2)).expect("mismatch");
    assert_eq!(warning.code, "rayon-pool-size");
    assert_eq!(warning.requested_jobs, 8);
    assert_eq!(warning.effective_jobs, 2);
    assert!(warning.message.contains('8'));
    assert!(warning.message.contains('2'));
}

#[test]
fn later_init_with_different_jobs_warns_and_does_not_resize() {
    let _ = init_rayon_threads(JobsArg { jobs: 0 });
    let recorded = recorded_rayon_threads().expect("pool recorded");
    let before = rayon::current_num_threads();
    assert_eq!(before, recorded);
    let requested = if recorded == 1 { 2 } else { 1 };
    let warning = init_rayon_threads(JobsArg { jobs: requested }).expect("mismatch warns");
    assert_eq!(warning.requested_jobs, requested);
    assert_eq!(warning.effective_jobs, recorded);
    assert_eq!(rayon::current_num_threads(), before);
    assert_eq!(recorded_rayon_threads(), Some(recorded));
    assert!(init_rayon_threads(JobsArg { jobs: recorded }).is_none());
}

#[test]
fn attach_jobs_pool_warning_omits_empty_and_keeps_non_objects() {
    assert_eq!(
        attach_jobs_pool_warning(r#"{"reports":[]}"#.to_string(), None),
        r#"{"reports":[]}"#
    );
    let warning = jobs_pool_mismatch(8, Some(2));
    let with_warning = attach_jobs_pool_warning(r#"{"reports":[]}"#.to_string(), warning.clone());
    let value: serde_json::Value = serde_json::from_str(&with_warning).expect("object");
    assert_eq!(value["jobsPoolWarning"]["code"], "rayon-pool-size");
    assert_eq!(value["jobsPoolWarning"]["requestedJobs"], 8);
    assert_eq!(value["jobsPoolWarning"]["effectiveJobs"], 2);
    assert_eq!(value["reports"], serde_json::json!([]));
    assert_eq!(attach_jobs_pool_warning("1".to_string(), warning), "1");
}

#[test]
fn edge_view_preserves_no_root_duplicates_and_depth_order() {
    let edges = vec![
        edge("b", "d"),
        edge("a", "c"),
        edge("a", "b"),
        edge("a", "b"),
        edge("c", "e"),
    ];
    assert_eq!(edge_view(&edges, &[], Some(0)), edges);
    assert_eq!(
        edge_view(&edges, &["a".into()], Some(1)),
        vec![edge("a", "c"), edge("a", "b")]
    );
    assert_eq!(
        edge_view(&edges, &["a".into()], None),
        vec![
            edge("a", "c"),
            edge("a", "b"),
            edge("b", "d"),
            edge("c", "e")
        ]
    );
}

#[test]
fn public_edge_view_accepts_non_hash_kinds_without_reversal() {
    let edges = vec![PublicCompatibilityEdge {
        from: "a".into(),
        to: "b".into(),
    }];
    assert_eq!(
        edge_view(&edges, &["a".into()], Some(1)),
        vec![PublicCompatibilityEdge {
            from: "a".into(),
            to: "b".into(),
        }]
    );
}

#[test]
fn indexed_related_handles_multi_roots_self_loops_and_reciprocals() {
    let index = EdgeTraversalIndex::new(&[
        edge("a", "a"),
        edge("a", "b"),
        edge("b", "a"),
        edge("b", "c"),
        edge("d", "c"),
    ]);
    assert_eq!(
        index.related(&["a".into(), "d".into()], true, true),
        vec![
            edge("a", "a"),
            edge("a", "b"),
            edge("b", "a"),
            edge("b", "c"),
            edge("c", "b"),
            edge("c", "d"),
            edge("d", "c"),
        ]
    );
}
