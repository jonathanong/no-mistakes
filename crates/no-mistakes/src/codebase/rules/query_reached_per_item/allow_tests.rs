use super::{allow, config::Options, *};
use crate::codebase::check_facts::{
    collect_check_facts_with_graph_files_playwright_and_sources, CheckFactPlan,
};
use crate::codebase::rules::allow_audit::AllowAudit;
use crate::codebase::ts_source::facts::TsFactPlan;

enum Scenario {
    Narrowed,
    NarrowedGraph,
    Failed,
    Missing,
}

fn assert_audit_silent(scenario: Scenario) {
    let failed = matches!(scenario, Scenario::Failed);
    let root =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/check/unused-allow-entries");
    let root = crate::codebase::ts_resolver::normalize_path(&root);
    let config = crate::config::v2::load_v2_config(&root, None).unwrap();
    let application = config.rule_applications(RULE_ID)[0];
    let options: Options = application.try_rule_options().unwrap();
    let files = [
        root.join("src/entry.mts"),
        if failed {
            root.join("../unused-allow-entries-failed/query.mts")
        } else {
            root.join("src/query.mts")
        },
    ];
    let sources = super::super::source_store_for_files(&files);
    let mut facts = collect_check_facts_with_graph_files_playwright_and_sources(
        &root,
        files.to_vec(),
        files.to_vec(),
        CheckFactPlan {
            graph: TsFactPlan::imports(),
            ..Default::default()
        },
        None,
        sources.clone(),
    );
    match scenario {
        Scenario::Narrowed => facts.files = vec![files[1].clone()],
        Scenario::NarrowedGraph => facts.graph_files = vec![files[1].clone()],
        Scenario::Missing => {
            assert!(facts.ts.remove(&files[1]).is_some());
        }
        Scenario::Failed => {}
    }
    let filter =
        super::super::path_filter::RulePathFilter::new(&root, &config, application).unwrap();
    let audit = AllowAudit::new(
        &root,
        &config,
        application,
        Some(&sources),
        "allow",
        options.allow.len(),
    );
    assert!(allow::finish(&root, &facts, Some(&sources), &filter, &options, &audit).is_empty());
}

#[test]
fn supporting_graph_files_outside_selected_scope_are_not_audited() {
    assert_audit_silent(Scenario::Narrowed);
}

#[test]
fn a_failed_supporting_fact_cannot_prove_an_allowance_stale() {
    assert_audit_silent(Scenario::Failed);
}

#[test]
fn missing_supporting_facts_cannot_prove_an_allowance_stale() {
    assert_audit_silent(Scenario::Missing);
}

#[test]
fn primary_facts_outside_the_query_graph_are_not_audited() {
    assert_audit_silent(Scenario::NarrowedGraph);
}

fn selector_root() -> std::path::PathBuf {
    crate::test_support::rule_fixture_root("query-reached-per-item")
        .with_file_name("allow-function")
}

#[test]
fn function_allowance_leaves_other_callers_visible() {
    let root = selector_root();
    let unallowed =
        crate::codebase::rules::run_check(&root, Some(&root.join("no-allow.yml")), None).unwrap();
    assert_eq!(
        unallowed
            .iter()
            .map(|finding| finding.line)
            .collect::<Vec<_>>(),
        [4, 8]
    );
    let findings = crate::codebase::rules::run_check(&root, None, None).unwrap();
    assert_eq!(findings.len(), 1, "{findings:#?}");
    assert_eq!(findings[0].file, "entry.mts");
    assert_eq!(findings[0].line, 8);
    assert!(findings[0].message.contains("processItems"));
    let file_only =
        crate::codebase::rules::run_check(&root, Some(&root.join("file-only.yml")), None).unwrap();
    assert!(file_only.is_empty(), "{file_only:#?}");
}

#[test]
fn unused_function_and_legacy_file_allowances_are_reported() {
    let root = selector_root();
    let findings =
        crate::codebase::rules::run_check(&root, Some(&root.join("hygiene.yml")), None).unwrap();
    let unused = findings
        .iter()
        .filter(|finding| finding.rule == "unused-allow-entry")
        .collect::<Vec<_>>();
    assert_eq!(unused.len(), 4, "{findings:#?}");
    for selector in ["removedFunction", "otherQuery", "entry.mts:4", "clean.mts"] {
        assert!(
            unused
                .iter()
                .any(|finding| finding.message.contains(selector)),
            "{unused:#?}"
        );
    }
    assert!(findings
        .iter()
        .any(|finding| finding.rule == RULE_ID && finding.line == 8));
}

#[test]
fn function_selectors_require_a_nonempty_name() {
    let root = selector_root();
    let error =
        crate::codebase::rules::run_check(&root, Some(&root.join("empty-function.yml")), None)
            .unwrap_err();
    assert!(error.to_string().contains("non-empty function"), "{error}");
}
