use super::super::diagnostics_with_files;
use std::path::PathBuf;

#[test]
fn handles_nonce_templates_and_unresolvable_dynamic_csp_expressions() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/rules/csp-allowed-origins/fixture/dynamic-policies");
    let page = root.join("src/page.tsx");

    let nonce_cfg = crate::config::v2::load_v2_config_with_path(
        &root,
        Some(&root.join("nonce-join.yml")),
    )
    .unwrap()
    .0;
    let nonce_diag = diagnostics_with_files(
        &root,
        &nonce_cfg,
        &[root.join("src/nonce-join.ts"), page.clone()],
    )
    .unwrap();
    assert_eq!(nonce_diag.findings.len(), 2);
    assert!(nonce_diag.advisories.is_empty());
    assert!(nonce_diag
        .findings
        .iter()
        .all(|f| f.message.contains("unlisted-")));

    let dyn_cfg = crate::config::v2::load_v2_config_with_path(
        &root,
        Some(&root.join("dynamic-join-advise.yml")),
    )
    .unwrap()
    .0;
    let dyn_diag = diagnostics_with_files(
        &root,
        &dyn_cfg,
        &[root.join("src/dynamic-join.ts"), page.clone()],
    )
    .unwrap();
    assert!(dyn_diag.findings.is_empty());
    assert_eq!(dyn_diag.advisories.len(), 1);
    assert_eq!(dyn_diag.advisories[0].file, "src/dynamic-join.ts");
    assert_eq!(
        dyn_diag.advisories[0].target.as_deref(),
        Some("unanalyzable")
    );

    let forms_cfg = crate::config::v2::load_v2_config_with_path(
        &root,
        Some(&root.join("dynamic-forms.yml")),
    )
    .unwrap()
    .0;
    let forms_diag = diagnostics_with_files(
        &root,
        &forms_cfg,
        &[root.join("src/dynamic-forms.ts"), page],
    )
    .unwrap();
    assert!(forms_diag.findings.is_empty());
    assert_eq!(forms_diag.advisories.len(), 8);
    assert!(forms_diag.advisories.iter().all(|item| {
        item.file == "src/dynamic-forms.ts" && item.target.as_deref() == Some("unanalyzable")
    }));
}

#[test]
fn abstains_on_origins_when_dynamic_csp_builder_coexists_with_static_policy() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/rules/csp-allowed-origins/fixture/mixed-static-dynamic");
    let page = root.join("src/page.tsx");

    let report_cfg = crate::config::v2::load_v2_config_with_path(
        &root,
        Some(&root.join("report.yml")),
    )
    .unwrap()
    .0;
    let report_diag = diagnostics_with_files(
        &root,
        &report_cfg,
        &[root.join("src/csp.ts"), page.clone()],
    )
    .unwrap();
    assert_eq!(report_diag.findings.len(), 1, "{:?}", report_diag.findings);
    assert_eq!(report_diag.findings[0].file, "src/csp.ts");
    assert_eq!(report_diag.findings[0].line, 10);
    assert_eq!(
        report_diag.findings[0].target.as_deref(),
        Some("unanalyzable")
    );
    assert!(report_diag.advisories.is_empty());

    let extra_cfg = crate::config::v2::load_v2_config_with_path(
        &root,
        Some(&root.join("extra-builders.yml")),
    )
    .unwrap()
    .0;
    let extra_diag = diagnostics_with_files(
        &root,
        &extra_cfg,
        &[root.join("src/extra-builders.ts"), page],
    )
    .unwrap();
    assert!(extra_diag.findings.is_empty(), "{:?}", extra_diag.findings);
    assert_eq!(extra_diag.advisories.len(), 7, "{:?}", extra_diag.advisories);
    assert!(extra_diag.advisories.iter().all(|item| {
        item.file == "src/extra-builders.ts" && item.target.as_deref() == Some("unanalyzable")
    }));
}
