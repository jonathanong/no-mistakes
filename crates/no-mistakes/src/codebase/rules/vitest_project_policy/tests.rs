use super::*;

fn run(case: &str) -> Diagnostics {
    let root = crate::codebase::ts_resolver::normalize_path(
        &Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../test-cases/rules/vitest-project-policy/fixture")
            .join(case),
    );
    let config =
        crate::config::v2::load_v2_config(&root, Some(&root.join(".no-mistakes.yml"))).unwrap();
    let files = vec![root.join("vitest.config.ts"), root.join("helper.ts")];
    diagnostics_with_files_and_sources(
        &root,
        &config,
        &files,
        &SourceStore::new(std::sync::Arc::new(
            crate::codebase::ts_source::FileInventory::from_paths(&files),
        )),
        false,
    )
    .unwrap()
}

#[test]
fn literal_projects_and_declared_helper_inheritance_pass() {
    for case in [
        "pass",
        "literal-forms",
        "helper",
        "signed-literal",
        "wrapper-alias",
        "helper-chain",
        "spread-before",
        "excluded-root",
    ] {
        let output = run(case);
        assert!(output.findings.is_empty(), "{case}: {:?}", output.findings);
        assert!(
            output.incomplete.is_empty(),
            "{case}: {:?}",
            output.incomplete
        );
    }
}
#[test]
fn violations_are_reported_for_missing_and_inconsistent_properties() {
    for case in [
        "missing",
        "mismatch",
        "root-mismatch",
        "required-forbidden",
        "missing-test",
    ] {
        let output = run(case);
        assert!(!output.findings.is_empty(), "{case}");
        assert!(
            output.incomplete.is_empty(),
            "{case}: {:?}",
            output.incomplete
        );
    }
}
#[test]
fn dynamic_inputs_are_incomplete() {
    for case in [
        "nonliteral",
        "spread",
        "dynamic-project",
        "project-hole",
        "undeclared-helper",
        "mutated-object",
        "unknown-inheritance",
        "root-missing",
        "reexport-undeclared",
        "arbitrary-call",
        "mutable-binding",
        "helper-cycle",
        "spread-after",
    ] {
        let output = run(case);
        assert!(!output.incomplete.is_empty(), "{case}");
        assert!(output.findings.is_empty(), "{case}: {:?}", output.findings);
        assert!(output
            .incomplete
            .iter()
            .all(|failure| failure.consumer == RULE_ID));
    }
}
#[test]
fn suppression_filters_literal_policy_findings() {
    for case in ["suppression", "line-suppression", "next-line-suppression"] {
        assert!(run(case).findings.is_empty(), "{case}");
    }
}

mod errors;
