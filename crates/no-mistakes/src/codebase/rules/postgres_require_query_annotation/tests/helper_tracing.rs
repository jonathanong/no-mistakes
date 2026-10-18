use super::*;

#[test]
fn helper_tracing_reports_every_executor_and_unknown_policy_is_explicit() {
    let root = fixture("helper-tracing");
    let files = crate::codebase::ts_source::discover_visible_paths(&root);
    let source = std::fs::read_to_string(root.join("src/orders.mts")).unwrap();
    for (policy, markers) in [
        ("report", vec!["// finding:", "// unanalyzable:"]),
        ("ignore", vec!["// finding:"]),
    ] {
        let config = config_with_options(&format!(
            "importSpecifier: '@app/db'\ninclude: ['src/orders.mts']\nunanalyzableSql: {policy}"
        ));
        let findings = check_with_files(&root, &config, &files).unwrap();
        let expected = source
            .lines()
            .enumerate()
            .filter_map(|(index, line)| {
                markers
                    .iter()
                    .any(|marker| line.contains(marker))
                    .then_some(index + 1)
            })
            .collect::<Vec<_>>();
        assert_eq!(
            findings
                .iter()
                .map(|finding| finding.line)
                .collect::<Vec<_>>(),
            expected,
            "{policy}: {findings:#?}"
        );
    }
}

#[test]
fn helper_tracing_fails_closed_for_unsupported_and_shadowed_bindings() {
    let root = fixture("helper-tracing-safety");
    let files = crate::codebase::ts_source::discover_visible_paths(&root);
    let source = std::fs::read_to_string(root.join("src/query.mts")).unwrap();
    let config = config_with_options(
        "importSpecifier: '@app/db'\ninclude: ['src/query.mts']\ntrustedSqlTags: [{module: './tags.mjs', name: customQuery}]",
    );
    let findings = check_with_files(&root, &config, &files).unwrap();
    let expected = source
        .lines()
        .enumerate()
        .filter_map(|(index, line)| {
            (line.contains("// unanalyzable:") || line.contains("// finding:")).then_some(index + 1)
        })
        .collect::<Vec<_>>();
    assert_eq!(
        findings
            .iter()
            .map(|finding| finding.line)
            .collect::<Vec<_>>(),
        expected,
        "{findings:#?}"
    );
}

#[test]
fn helper_projection_and_rule_reuse_the_request_sources_and_single_parse() {
    use crate::codebase::ts_source::{FileInventory, SourceStore};
    use std::sync::Arc;
    let root = crate::codebase::ts_resolver::normalize_path(&fixture("helper-tracing-ownership"));
    let mut files = crate::codebase::ts_source::discover_visible_paths(&root);
    let missing = root.join("src/missing.mts");
    files.push(missing.clone());
    let sources = Arc::new(SourceStore::new(Arc::new(FileInventory::from_paths(
        &files,
    ))));
    let config = config_with_options("importSpecifier: './db.mjs'\ninclude: ['src/query.mts']");
    crate::ast::begin_parse_count(&root);
    let facts = crate::codebase::postgres::prepare_rule_sql_facts(
        &root,
        &files,
        Arc::clone(&sources),
        &config,
        &[RULE_ID],
    )
    .unwrap();
    let reads = sources.physical_read_count();
    for _ in 0..2 {
        let findings =
            check_with_files_sources_and_facts(&root, &config, &files, &sources, &facts).unwrap();
        assert_eq!(findings.len(), 1, "{findings:?}");
        assert!(facts
            .embedded_sql(&missing, &EmbeddedSqlOptions::configured("./db.mjs", &[]))
            .is_err());
        assert_eq!(sources.physical_read_count(), reads);
    }
    let counts = crate::ast::finish_parse_count(&root);
    assert_eq!(counts.len(), 3, "{counts:?}");
    assert!(counts.values().all(|count| *count == 1), "{counts:?}");
    assert!(!counts.contains_key(&missing));

    // A Playwright demand selects the staged collector, but annotation facts
    // must still come from its single shared parse/read pass.
    let sources = Arc::new(SourceStore::new(Arc::new(FileInventory::from_paths(
        &files,
    ))));
    crate::ast::begin_parse_count(&root);
    let facts =
        crate::codebase::check_facts::collect_check_facts_with_graph_files_playwright_and_sources(
            &root,
            files.clone(),
            Vec::new(),
            crate::codebase::check_facts::CheckFactPlan {
                query_annotation: true,
                embedded_sql: true,
                embedded_sql_options: vec![EmbeddedSqlOptions::configured("./db.mjs", &[])],
                ..Default::default()
            },
            Some(Default::default()),
            Arc::clone(&sources),
        );
    let findings =
        check_with_files_sources_and_facts(&root, &config, &files, &sources, &facts).unwrap();
    assert_eq!(findings.len(), 1, "{findings:?}");
    let counts = crate::ast::finish_parse_count(&root);
    assert_eq!(counts.len(), 3, "{counts:?}");
    assert!(counts.values().all(|count| *count == 1), "{counts:?}");
    assert_eq!(sources.physical_read_count(), reads);
}

#[test]
fn helper_tracing_covers_straight_line_mutations_and_fail_closed_forms() {
    let root = fixture("helper-tracing-coverage");
    let files = crate::codebase::ts_source::discover_visible_paths(&root);
    let config = config_with_options(
        "importSpecifier: ''\nexecutorNames: [write, query]\ninclude: ['src/query.mts']\ntrustedSqlTags: [{module: '@custom/sql', name: fragment}]",
    );
    let source = std::fs::read_to_string(root.join("src/query.mts")).unwrap();
    let findings = check_with_files(&root, &config, &files).unwrap();
    let expected = source
        .lines()
        .enumerate()
        .filter_map(|(index, line)| {
            (line.contains("// finding:") || line.contains("// unanalyzable:")).then_some(index + 1)
        })
        .collect::<Vec<_>>();
    assert_eq!(
        findings
            .iter()
            .map(|finding| finding.line)
            .collect::<Vec<_>>(),
        expected,
        "{findings:#?}"
    );
}

#[test]
fn helpers_use_importer_owned_aliases_even_when_unknown_sql_is_ignored() {
    let root = fixture("helper-tracing-monorepo");
    let files = crate::codebase::ts_source::discover_visible_paths(&root);
    for policy in ["ignore", "report"] {
        let config = config_with_options(&format!(
            "importSpecifier: '@app/db'\ninclude: ['packages/*/src/query.mts']\nunanalyzableSql: {policy}"
        ));
        let findings = check_with_files(&root, &config, &files).unwrap();
        assert_eq!(findings.len(), 1, "{findings:#?}");
        assert_eq!(findings[0].file, "packages/bad/src/query.mts");
        assert_eq!(findings[0].line, 3);
        assert_eq!(findings[0].target.as_deref(), Some("annotation"));
    }
}
