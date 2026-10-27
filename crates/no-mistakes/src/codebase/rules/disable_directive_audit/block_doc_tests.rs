use super::tests::root;
use super::*;

#[test]
fn rust_block_docs_share_fence_groups_and_preserve_physical_locations() {
    let source =
        include_str!("../../../../../../fixtures/rules/disable-directive-audit/rust-block-docs.rs");
    let directives = audit_directives_for_path(source, Path::new("example.rs"));
    let docs: Vec<_> = directives
        .iter()
        .filter(|(_, doc)| *doc)
        .map(|(item, _)| (item.rule.as_str(), project(item.directive)))
        .collect();
    assert_eq!(
        docs.iter().map(|(rule, _)| *rule).collect::<Vec<_>>(),
        vec![
            "rust-max-lines-per-file",
            "outer-single",
            "outer-decorated",
            "inner-multiline",
            "after-backtick",
            "after-tilde",
            "after-cross-block",
            "after-code-boundary",
            "inner-kind-boundary",
            "after-lifetime",
        ]
    );
    let lines: Vec<_> = directives
        .iter()
        .filter(|(_, doc)| *doc)
        .map(|(item, _)| match item.directive {
            DisableDirective::File { line }
            | DisableDirective::Line { line }
            | DisableDirective::NextLine { line } => line,
        })
        .collect();
    assert_eq!(lines, vec![1, 2, 4, 8, 15, 24, 32, 37, 39, 49]);
    // These audit records must never become effective suppressions.
    let ordinary: Vec<_> = audit_directives_for_path(source, Path::new("example.rs"))
        .into_iter()
        .filter_map(|(item, ineffective)| (!ineffective).then_some(item))
        .collect();
    assert_eq!(ordinary.len(), 1);
    assert_eq!(
        ordinary[0].directive,
        DisableDirective::NextLine { line: 50 }
    );
}

#[test]
fn rust_block_doc_audit_reuses_sources_and_explains_plain_comment_fix() {
    let root = root();
    let file = root.join("rust-block-docs.rs");
    let mut config: NoMistakesConfig = serde_yaml::from_str(include_str!(
        "../../../../../../fixtures/rules/disable-directive-audit/disabled.yml"
    ))
    .unwrap();
    config.rules[0].rule = "rust-max-lines-per-file".into();
    config.rules[0].include = vec!["rust-block-docs.rs".into()];
    let sources = super::super::source_store_for_files(std::slice::from_ref(&file));
    let prepared = PreparedDisableDirectives::prepare(&root, &config, &[file], &sources).unwrap();
    let findings = prepared.findings(&[]);
    assert!(findings.iter().any(|item| item.line == 1
        && item.rule == "invalid-disable-directive"
        && item.message.contains("rust-max-lines-per-file")
        && item.message.contains("plain `//`")));
    assert_eq!(sources.physical_read_count(), 1);
}

#[test]
fn rust_block_docs_preserve_bom_and_crlf_line_numbers() {
    let source = include_str!(
        "../../../../../../fixtures/rules/disable-directive-audit/rust-block-docs-crlf.rs"
    );
    let docs = audit_directives_for_path(source, Path::new("example.rs"));
    assert_eq!(docs.len(), 2);
    assert_eq!(docs[0].0.directive, DisableDirective::File { line: 1 });
    assert_eq!(docs[1].0.directive, DisableDirective::NextLine { line: 3 });
    assert!(docs.iter().all(|(_, ineffective)| *ineffective));
}
