use super::{allow, config::Options, *};
use crate::codebase::check_facts::{
    collect_check_facts_with_graph_files_playwright_and_sources, CheckFactPlan,
};
use crate::codebase::rules::allow_audit::AllowAudit;
use crate::codebase::ts_source::facts::TsFactPlan;

enum Scenario {
    Narrowed,
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
