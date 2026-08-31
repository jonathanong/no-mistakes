use super::*;

fn fixture(name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/rules/no-sparse-checkout/fixture")
        .join(name)
}

#[test]
fn default_paths_report_sparse_checkout() {
    let root = fixture("fail");
    let config =
        crate::config::v2::load_v2_config(&root, Some(&root.join(".no-mistakes.yml"))).unwrap();
    let files = vec![root.join(".github/workflows/ci.yml")];
    assert_eq!(check_with_files(&root, &config, &files).unwrap().len(), 1);
}

#[test]
fn reports_both_checkout_inputs_and_ignores_non_checkout_step_content() {
    let root = fixture("fail");
    let config =
        crate::config::v2::load_v2_config(&root, Some(&root.join(".no-mistakes.yml"))).unwrap();
    let both =
        check_with_files(&root, &config, &[root.join(".github/workflows/both.yml")]).unwrap();
    assert_eq!(both.len(), 2, "{both:?}");
    let targets: std::collections::BTreeSet<_> = both
        .iter()
        .filter_map(|finding| finding.target.as_deref())
        .collect();
    assert_eq!(
        targets,
        ["sparse-checkout", "sparse-checkout-cone-mode"]
            .into_iter()
            .collect()
    );
    assert!(check_with_files(
        &root,
        &config,
        &[root.join(".github/workflows/ignored.yml")]
    )
    .unwrap()
    .is_empty());
}

#[test]
fn malformed_selected_yaml_is_reported() {
    let root = fixture("fail");
    let config =
        crate::config::v2::load_v2_config(&root, Some(&root.join(".no-mistakes.yml"))).unwrap();
    let findings =
        check_with_files(&root, &config, &[root.join(".github/workflows/bad.yml")]).unwrap();
    assert_eq!(findings.len(), 1, "{findings:?}");
    assert!(findings[0].message.contains("invalid YAML"), "{findings:?}");
}

#[test]
fn scalar_yaml_is_ignored_without_a_finding() {
    let root = fixture("fail");
    let config =
        crate::config::v2::load_v2_config(&root, Some(&root.join(".no-mistakes.yml"))).unwrap();
    assert!(
        check_with_files(&root, &config, &[root.join(".github/workflows/scalar.yml")])
            .unwrap()
            .is_empty()
    );
}

#[test]
fn line_suppression_uses_the_matching_checkout_step_not_an_earlier_ignored_key() {
    let root = fixture("fail");
    let config =
        crate::config::v2::load_v2_config(&root, Some(&root.join(".no-mistakes.yml"))).unwrap();
    let file = root.join(".github/workflows/mixed.yml");
    let sources = crate::codebase::rules::source_store_for_files(std::slice::from_ref(&file));
    let direct = check_with_files_sources_and_deferred_suppression(
        &root,
        &config,
        std::slice::from_ref(&file),
        &sources,
        false,
    )
    .unwrap();
    assert!(direct.is_empty(), "{direct:?}");
    let deferred =
        check_with_files_sources_and_deferred_suppression(&root, &config, &[file], &sources, true)
            .unwrap();
    assert_eq!(deferred.len(), 1, "{deferred:?}");
    assert_eq!(deferred[0].line, 12, "{deferred:?}");
}

#[test]
fn each_checkout_step_maps_its_own_inputs_and_line_suppression() {
    let root = fixture("fail");
    let config =
        crate::config::v2::load_v2_config(&root, Some(&root.join(".no-mistakes.yml"))).unwrap();
    let file = root.join(".github/workflows/two-checkouts.yml");
    let sources = crate::codebase::rules::source_store_for_files(std::slice::from_ref(&file));
    let deferred = check_with_files_sources_and_deferred_suppression(
        &root,
        &config,
        std::slice::from_ref(&file),
        &sources,
        true,
    )
    .unwrap();
    let lines: std::collections::BTreeSet<_> =
        deferred.iter().map(|finding| finding.line).collect();
    assert_eq!(lines, [6, 7, 11].into_iter().collect(), "{deferred:?}");
    let direct =
        check_with_files_sources_and_deferred_suppression(&root, &config, &[file], &sources, false)
            .unwrap();
    let direct_lines: std::collections::BTreeSet<_> =
        direct.iter().map(|finding| finding.line).collect();
    assert_eq!(direct_lines, [6, 7].into_iter().collect(), "{direct:?}");
}

#[test]
fn option_include_adds_another_workflow_root() {
    let root = fixture("custom");
    let config =
        crate::config::v2::load_v2_config(&root, Some(&root.join(".no-mistakes.yml"))).unwrap();
    let files = vec![root.join("ci/no-mistakes-workflows/check.yml")];
    assert_eq!(check_with_files(&root, &config, &files).unwrap().len(), 1);
}

#[test]
fn non_yaml_file_is_not_checked_even_when_selected() {
    let root = fixture("custom");
    let config =
        crate::config::v2::load_v2_config(&root, Some(&root.join(".no-mistakes.yml"))).unwrap();
    let file = root.join("ci/no-mistakes-workflows/check.txt");
    assert!(check_with_files(&root, &config, &[file])
        .unwrap()
        .is_empty());
}

#[test]
fn default_selection_includes_composite_action_yaml() {
    let root = fixture("action");
    let config =
        crate::config::v2::load_v2_config(&root, Some(&root.join(".no-mistakes.yml"))).unwrap();
    let file = root.join(".github/actions/check/action.yaml");
    assert_eq!(check_with_files(&root, &config, &[file]).unwrap().len(), 1);
}
