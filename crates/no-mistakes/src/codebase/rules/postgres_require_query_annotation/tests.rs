use super::*;
use crate::config::v2::{
    schema::{RuleDef, RuleScope},
    NoMistakesConfig,
};
use std::path::{Path, PathBuf};

fn fixture_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/rules/postgres-require-query-annotation")
}

fn fixture(scenario: &str) -> PathBuf {
    fixture_root().join("fixture").join(scenario)
}

fn config_with_options(yaml: &str) -> NoMistakesConfig {
    let mut config = NoMistakesConfig::default();
    config.rules.push(RuleDef {
        rule: RULE_ID.to_string(),
        scope: Some(RuleScope::Repository),
        options: crate::codebase::postgres::tests::fixture_rule_options(yaml),
        ..Default::default()
    });
    config
}

fn default_config() -> NoMistakesConfig {
    config_with_options("{}")
}

fn ts_file(root: &Path) -> PathBuf {
    root.join("src/query.ts")
}

fn findings_for(scenario: &str) -> Vec<RuleFinding> {
    let root = fixture(scenario);
    let file = ts_file(&root);
    check_with_files(&root, &default_config(), &[file]).unwrap()
}

#[test]
fn fail_fixture_reports_missing_annotation() {
    let findings = findings_for("fail");
    assert_eq!(findings.len(), 1, "{findings:#?}");
    assert_eq!(findings[0].rule, RULE_ID);
    assert_eq!(findings[0].file, "src/query.ts");
    assert!(findings[0].line > 0);
    assert_eq!(findings[0].target.as_deref(), Some("annotation"));
    assert!(findings[0].message.contains("/* name */"), "{findings:#?}");
}

#[test]
fn annotated_query_is_clean() {
    assert!(findings_for("pass-annotated").is_empty());
}

#[test]
fn var_sql_retains_function_scope_and_lexical_bindings_stay_local() {
    let root = fixture("var-scopes");
    let file = ts_file(&root);
    let source = std::fs::read_to_string(&file).unwrap();
    let findings = check_with_files(
        &root,
        // This fixture separately pins legacy SQL recovery, including opaque
        // bindings; default unknown reporting is covered by helper-tracing.
        &config_with_options("importSpecifier: '@example/db'\nunanalyzableSql: ignore"),
        std::slice::from_ref(&file),
    )
    .unwrap();
    let expected = source
        .lines()
        .enumerate()
        .filter_map(|(index, line)| line.contains("// finding:").then_some(index + 1))
        .collect::<Vec<_>>();
    assert_eq!(
        findings
            .iter()
            .map(|finding| finding.line)
            .collect::<Vec<_>>(),
        expected
    );

    let facts = crate::codebase::postgres::extract_embedded_sql_from_source(
        &file,
        &source,
        &EmbeddedSqlOptions::configured("@example/db", &[]),
    );
    assert!(facts
        .calls
        .iter()
        .all(|call| call.declaration_line != Some(0)));
    for (index, line) in source.lines().enumerate() {
        if line.contains("// unknown:") || line.contains("// known:") {
            let call = facts
                .calls
                .iter()
                .find(|call| call.line as usize == index + 1)
                .unwrap();
            assert_eq!(
                call.sql_text.is_some(),
                line.contains("// known:"),
                "{line}"
            );
        }
    }
}

#[test]
fn transaction_commands_are_clean() {
    assert!(findings_for("pass-transaction").is_empty());
}

#[test]
fn line_comment_is_not_an_annotation() {
    assert_eq!(findings_for("fail-line-comment").len(), 1);
}

#[test]
fn honors_disable_comments() {
    let root = fixture("fail");
    let file = root.join("src/disabled.ts");
    let mut findings =
        check_with_files(&root, &default_config(), std::slice::from_ref(&file)).unwrap();
    assert_eq!(findings.len(), 1, "{findings:#?}");
    let sources = super::super::source_store_for_files(std::slice::from_ref(&file));
    super::super::suppress_rule_findings_with_sources(&root, &mut findings, &sources);
    assert!(findings.is_empty(), "{findings:#?}");
}

#[test]
fn include_and_exclude_globs_filter_files() {
    let root = fixture("fail");
    let files = vec![ts_file(&root)];
    let excluded = check_with_files(
        &root,
        &config_with_options("exclude: ['src/query.ts']"),
        &files,
    )
    .unwrap();
    assert!(excluded.is_empty(), "{excluded:#?}");

    let included = check_with_files(
        &root,
        &config_with_options("include: ['src/query.ts']"),
        &files,
    )
    .unwrap();
    assert_eq!(included.len(), 1);

    let missed = check_with_files(
        &root,
        &config_with_options("include: ['src/missing.ts']"),
        &files,
    )
    .unwrap();
    assert!(missed.is_empty(), "{missed:#?}");
}

#[test]
fn custom_executor_and_specifier_are_required_to_match() {
    let root = fixture("fail");
    let files = vec![ts_file(&root)];
    let other = check_with_files(
        &root,
        &config_with_options("importSpecifier: '@other/db'\nexecutorNames: [run]"),
        &files,
    )
    .unwrap();
    assert!(other.is_empty(), "{other:#?}");
}

#[test]
fn invalid_include_glob_errors() {
    let root = fixture("fail");
    let error = check_with_files(
        &root,
        &config_with_options("include: ['[']"),
        &[ts_file(&root)],
    )
    .expect_err("invalid glob");
    assert!(error.to_string().contains("invalid glob"), "{error}");
}

#[test]
fn invalid_exclude_glob_errors() {
    let root = fixture("fail");
    let error = check_with_files(
        &root,
        &config_with_options("exclude: ['[']"),
        &[ts_file(&root)],
    )
    .expect_err("invalid glob");
    assert!(error.to_string().contains("invalid glob"), "{error}");
}

#[test]
fn missing_source_file_errors() {
    let root = fixture("fail");
    let missing = root.join("src/does-not-exist.ts");
    let error = check_with_files(&root, &default_config(), &[missing]).expect_err("read");
    assert!(
        error.to_string().contains("failed to collect embedded SQL"),
        "{error}"
    );
}

#[test]
fn missing_sql_text_is_ignored() {
    let call = crate::codebase::postgres::EmbeddedSqlCall {
        line: 1,
        callee: "query".to_string(),
        sql_text: None,
        ..Default::default()
    };
    assert!(super::scan::findings_for_call("src/query.ts", &call).is_empty());
}

#[test]
fn compile_options_honor_overrides() {
    let compiled = compile_options(&Options {
        import_specifier: "@other/db".to_string(),
        executor_names: vec!["run".to_string()],
        ..Default::default()
    })
    .unwrap();
    assert_eq!(compiled.embedded.import_specifier, "@other/db");
    assert_eq!(compiled.embedded.executor_names, ["run"]);
}

#[test]
fn compile_options_fill_defaults() {
    let compiled = compile_options(&Options::default()).unwrap();
    assert_eq!(
        compiled.embedded.import_specifier,
        EmbeddedSqlOptions::default().import_specifier
    );
    assert_eq!(
        compiled.embedded.executor_names,
        EmbeddedSqlOptions::default().executor_names
    );
}

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
    let config = config_with_options("importSpecifier: '@app/db'\ninclude: ['src/query.mts']\ntrustedSqlTags: [{module: './tags.mjs', name: customQuery}]");
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
fn helper_review_regressions_preserve_call_identity_exports_and_live_captures() {
    let root = fixture("helper-tracing-review");
    let files = crate::codebase::ts_source::discover_visible_paths(&root);
    let paths = [
        "src/destructured-tag.mts",
        "src/import-equals.mts",
        "src/local-tag.mts",
        "src/named-tags.mts",
        "src/query.mts",
        "src/raw-assertion.ts",
        "src/raw-deleted.mts",
        "src/raw-reassigned.mts",
        "src/raw-shadowed.mts",
    ];
    for ignore in [false, true] {
        let policy = if ignore { "ignore" } else { "report" };
        let config = config_with_options(&format!("importSpecifier: '@app/db'\ninclude: ['src/query.mts', 'src/import-equals.mts', 'src/destructured-tag.mts', 'src/local-tag.mts', 'src/named-tags.mts', 'src/raw-assertion.ts', 'src/raw-deleted.mts', 'src/raw-reassigned.mts', 'src/raw-shadowed.mts']\ntrustedSqlTags: [{{module: './tags.mjs', name: customQuery}}]\nunanalyzableSql: {policy}"));
        let findings = check_with_files(&root, &config, &files).unwrap();
        for threads in [1, 3] {
            let pool = rayon::ThreadPoolBuilder::new()
                .num_threads(threads)
                .build()
                .unwrap();
            let parallel = pool.install(|| check_with_files(&root, &config, &files).unwrap());
            assert_eq!(
                parallel, findings,
                "helper projection differs with {threads} workers"
            );
        }
        let expected = paths
            .iter()
            .flat_map(|path| {
                std::fs::read_to_string(root.join(path))
                    .unwrap()
                    .lines()
                    .enumerate()
                    .filter_map(|(index, line)| {
                        (line.contains("// finding:")
                            || (!ignore && line.contains("// unanalyzable:")))
                        .then_some((path.to_string(), index + 1))
                    })
                    .collect::<Vec<_>>()
            })
            .collect::<Vec<_>>();
        assert_eq!(
            findings
                .iter()
                .map(|finding| (finding.file.clone(), finding.line))
                .collect::<Vec<_>>(),
            expected,
            "{findings:#?}"
        );
    }
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
    let config = config_with_options("importSpecifier: './db.mjs'\ninclude: ['src/query.mts']\ntrustedSqlTags: [{module: '@custom/sql', name: fragment}]");
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
