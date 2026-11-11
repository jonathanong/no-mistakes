use super::with_injected_panic;
use crate::codebase::analysis_session::AnalysisSession;
use crate::codebase::check_facts::{
    collect_check_facts_with_graph_files_playwright_and_session, CheckFactPlan,
};
use crate::codebase::ts_source::facts::{
    collect_ts_facts_with_session_and_context, TsFactContext, TsFactPlan,
};
use std::path::PathBuf;

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/codebase-analysis/fact-panic-isolation/fixture")
        .canonicalize()
        .unwrap()
}

fn injected<T>(run: impl FnOnce() -> T) -> T {
    static SERIAL: std::sync::Mutex<()> = std::sync::Mutex::new(());
    let _serial = SERIAL.lock().unwrap_or_else(|error| error.into_inner());
    with_injected_panic(&root().join("first.ts"), || {
        with_injected_panic(&root().join("second.ts"), run)
    })
}

#[test]
fn fact_panic_keeps_healthy_check_findings_and_deterministic_incomplete_report() {
    injected(|| {
        let reports = [1, 2].map(|jobs| {
            rayon::ThreadPoolBuilder::new()
                .num_threads(jobs)
                .build()
                .unwrap()
                .install(|| {
                    let report = crate::check_runner::run_all_with_options(
                        root(),
                        None,
                        None,
                        false,
                        false,
                        Default::default(),
                    )
                    .unwrap();
                    assert_eq!(
                        report
                            .incomplete
                            .iter()
                            .filter(|failure| failure.consumer == "fact-extraction")
                            .count(),
                        2
                    );
                    assert_eq!(report.rules.len(), 1);
                    assert_eq!(report.rules[0].file, PathBuf::from("healthy.ts"));
                    for failure in report
                        .incomplete
                        .iter()
                        .filter(|failure| failure.consumer == "fact-extraction")
                    {
                        assert!([root().join("first.ts"), root().join("second.ts")]
                            .contains(failure.file.as_ref().unwrap()));
                        assert!(failure.message.contains("injected fact extraction failure"));
                    }
                    normalize(report)
                })
        });
        assert_eq!(reports[0], reports[1]);
        insta::assert_snapshot!("fact_panic_check_report", reports[0]);
    });
}

#[test]
fn fact_panic_discards_all_facts_in_check_and_graph_collection() {
    injected(|| {
        let root = root();
        let files = ["first.ts", "second.ts", "healthy.ts"].map(|name| root.join(name));
        let session = AnalysisSession::disabled();
        let check = collect_check_facts_with_graph_files_playwright_and_session(
            &session,
            &root,
            files.to_vec(),
            vec![],
            CheckFactPlan {
                imports: true,
                ..Default::default()
            },
            None,
        );
        assert_eq!(check.stats.parse_errors, 2);
        for path in &files[..2] {
            let facts = &check.ts[path];
            assert!(!facts.parsed);
            assert!(facts.source.is_none());
            assert!(facts.ts.imports.is_empty());
            assert!(facts
                .ts
                .operational_error
                .as_ref()
                .unwrap()
                .contains("injected fact extraction failure"));
        }
        assert!(!check.ts[&files[2]].ts.imports.is_empty());
        let graph = collect_ts_facts_with_session_and_context(
            &session,
            &files,
            TsFactPlan {
                imports: true,
                ..Default::default()
            },
            &TsFactContext::default(),
        );
        for path in &files[..2] {
            let facts = graph.get(path).unwrap();
            assert!(facts.imports.is_empty());
            assert!(facts.source.is_none());
            assert!(facts.operational_error.is_some());
        }
        assert!(!graph.get(&files[2]).unwrap().imports.is_empty());
        assert_eq!(session.check_failures().len(), 2);
    });
}

#[path = "../../../../tests/common/check_json_normalize.rs"]
mod normalize;
#[path = "../../../../tests/common/check_json_source_lines.rs"]
mod source_lines;

fn normalize(report: crate::check_runner::CheckResults) -> String {
    let root = root();
    let mut json = crate::check_runner::json_value(&report);
    source_lines::add_source_lines(&mut json, &root, root.ancestors().nth(4).unwrap());
    normalize::normalize_report(&json.to_string(), &normalize::root_variants(&root))
}

#[test]
fn fact_panic_marks_a_check_with_no_findings_incomplete() {
    injected(|| {
        let report = crate::check_runner::run_all_with_options(
            root(),
            Some("clean.yml".into()),
            None,
            false,
            false,
            Default::default(),
        )
        .unwrap();
        assert!(report.rules.is_empty());
        assert_eq!(report.incomplete.len(), 2);
    });
}
