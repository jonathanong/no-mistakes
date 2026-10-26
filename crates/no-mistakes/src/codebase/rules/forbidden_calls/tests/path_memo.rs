use super::*;
use crate::codebase::dependencies::graph::{DepGraph, GraphBuildPlan};

#[test]
fn path_policy_checks_are_bounded_by_candidate_files_per_application() {
    let root = crate::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../test-cases/rules/forbidden-calls/path-policy-memo/fixture"),
    );
    let tsconfig = crate::codebase::ts_resolver::TsConfig {
        dir: root.clone(),
        paths_dir: root.clone(),
        ..Default::default()
    };
    let graph = DepGraph::build_with_plan(
        &root,
        &tsconfig,
        GraphBuildPlan {
            calls: true,
            ..Default::default()
        },
    )
    .unwrap();
    assert_eq!(
        graph
            .resolved_call_sites()
            .iter()
            .filter(|site| site.invocation
                == crate::codebase::dependencies::extract::InvocationKind::Call)
            .count(),
        11
    );
    for traversal in ["direct", "transitive", "file"] {
        let mut config = crate::config::v2::load_v2_config(&root, None).unwrap();
        for application in &mut config.rules {
            application.options["traversal"] = serde_yaml::Value::String(traversal.to_string());
        }
        let observer = crate::diagnostics::InvocationObserver::new(true);
        let findings = crate::diagnostics::with_observer(Some(observer.clone()), || {
            check_with_graph(&root, &config, &graph, None, None, &[], None).unwrap()
        });
        let actual = findings
            .iter()
            .map(|finding| {
                (
                    finding.file.as_str(),
                    finding.line,
                    finding.target.as_deref(),
                )
            })
            .collect::<Vec<_>>();
        let timer = Some("global `setTimeout`");
        let expected = if traversal == "transitive" {
            vec![
                ("src/child.mts", 2, timer),
                ("src/child.mts", 3, timer),
                ("src/entry.mts", 5, timer),
                ("src/entry.mts", 6, timer),
                ("src/excluded.mts", 2, timer),
                ("src/excluded.mts", 3, timer),
            ]
        } else {
            vec![("src/entry.mts", 5, timer), ("src/entry.mts", 6, timer)]
        };
        assert_eq!(actual, expected, "{traversal}: {findings:#?}");
        assert!(findings[..2]
            .iter()
            .all(|finding| finding.message.contains("entry-and-child, application #1")));
        if traversal == "transitive" {
            assert!(findings[4..]
                .iter()
                .all(|finding| finding.message.contains("excluded-only, application #2")));
        }
        // Excluded and unreachable files also repeat calls. Cache negative decisions,
        // but never reuse one application's answer for the other's different policy.
        assert_eq!(
            observer
                .snapshot()
                .work
                .get("forbidden_calls.path_filter_checks"),
            Some(&(if traversal == "file" { 2 } else { 8 })),
            "{traversal}: check four candidate files per application, or one selected file"
        );
    }
}
