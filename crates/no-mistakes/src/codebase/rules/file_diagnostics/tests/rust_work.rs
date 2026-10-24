use super::*;
use std::sync::atomic::{AtomicUsize, Ordering};

#[test]
fn overlapping_rust_applications_share_one_real_parse_and_one_physical_read() {
    let root = saved();
    let selected = "rust-allows/both-markers.rs";
    let mut config = config(&root, "rust-no-inline-allows");
    config.rules[0].include = vec![selected.into()];
    let mut advisory = config.rules[0].clone();
    advisory.options["unanalyzable"] = "advise".into();
    config.rules.push(advisory);
    let mut tests = config.rules[0].clone();
    tests.rule = "rust-no-inline-tests".into();
    config.rules.push(tests);
    let mut lines: NoMistakesConfig = serde_yaml::from_str(
        &std::fs::read_to_string(root.join("rust-max-lines-per-file.yml")).unwrap(),
    )
    .unwrap();
    lines.rules[0].include = vec![selected.into()];
    config.rules.push(lines.rules.remove(0));
    let files = files(&root, &config);
    let sources = super::super::super::source_store_for_files(&files);
    let calls = AtomicUsize::new(0);
    let output = super::super::super::rust_rules_combined::diagnostics_with_parser(
        &root,
        &config,
        &files,
        &sources,
        false,
        |source| {
            calls.fetch_add(1, Ordering::Relaxed);
            syn::parse_file(source)
        },
    )
    .unwrap();
    assert_eq!(calls.load(Ordering::Relaxed), 1);
    assert_eq!(sources.physical_read_count(), 1);
    assert_eq!(output.findings.len(), 3);
    assert_eq!(output.advisories.len(), 1);

    // A request for the textual line limit must not inherit AST demand from source markers.
    config
        .rules
        .retain(|rule| rule.rule == "rust-max-lines-per-file");
    let sources = super::super::super::source_store_for_files(&files);
    calls.store(0, Ordering::Relaxed);
    let output = super::super::super::rust_rules_combined::diagnostics_with_parser(
        &root,
        &config,
        &files,
        &sources,
        false,
        |source| {
            calls.fetch_add(1, Ordering::Relaxed);
            syn::parse_file(source)
        },
    )
    .unwrap();
    assert_eq!(calls.load(Ordering::Relaxed), 0);
    assert_eq!(sources.physical_read_count(), 1);
    assert_eq!(output.findings.len(), 1);
}
