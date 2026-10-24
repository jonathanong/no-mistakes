use super::*;
use crate::codebase::postgres::extract_migration_facts;
use crate::codebase::rules::{source_store_for_files, suppress_rule_findings_with_sources};
use crate::config::v2::load_v2_config;

mod copy_tests;
mod procedural_tests;
mod recursive_heads;

fn fixture() -> PathBuf {
    crate::test_support::rule_fixture_root("postgres-ddl-unanalyzable")
}

#[test]
fn valid_postgres_parser_gaps_name_suppression_and_policy() {
    let root = fixture().join("unsupported-valid");
    let (config, files, facts) = prepared(&root, "report.yml");
    let output = diagnostics(&root, &config, &files, &facts).unwrap();
    assert_eq!(output.findings.len(), 3, "{:?}", output.findings);
    for (index, finding) in output.findings.iter().enumerate() {
        assert_eq!(finding.line, index + 1);
        assert!(finding
            .message
            .contains("could not be analyzed for postgres-fk-index"));
        assert!(finding
            .message
            .contains("no-mistakes-disable-next-line postgres-fk-index"));
        assert!(finding.message.contains("options.unanalyzable"));
        assert!(finding.message.contains("advise or ignore"));
        assert!(!finding.message.contains("rewrite it as supported static"));
    }
}

#[test]
fn query_only_rules_do_not_request_schema_diagnostics() {
    let root = fixture();
    let config = load_v2_config(&root, Some(&root.join("report.yml"))).unwrap();
    let output = diagnostics_for_rule(
        &root,
        &config,
        &[root.join("migrations/001.sql")],
        &CheckFactMap::default(),
        "postgres-no-offset",
    )
    .unwrap();
    assert!(output.findings.is_empty());
    assert!(output.advisories.is_empty());
}

fn prepared(root: &Path, config_name: &str) -> (NoMistakesConfig, Vec<PathBuf>, CheckFactMap) {
    let config = load_v2_config(root, Some(&root.join(config_name))).unwrap();
    let files = vec![root.join("migrations/001.sql")];
    let sources = source_store_for_files(&files);
    let facts =
        crate::codebase::postgres::prepare_rule_sql_facts(root, &files, sources, &config, RULE_IDS)
            .unwrap();
    (config, files, facts)
}

#[test]
fn migration_facts_keep_each_failed_chunk_but_not_real_discard() {
    let sql = std::fs::read_to_string(fixture().join("migrations/001.sql")).unwrap();
    let markers = extract_migration_facts(&sql).unanalyzable_statements;
    assert_eq!(markers.len(), 2, "{markers:?}");
    assert_eq!(markers[0].line, 3);
    assert!(markers[0].head.starts_with("ALTER TABLE accounts"));
    assert_eq!(markers[1].line, 6);
    assert!(markers[1].head.starts_with("FROBULATE accounts"));
}

#[test]
fn every_schema_rule_reports_each_failed_statement_at_max_and_low() {
    let root = fixture();
    for config_name in ["report.yml", "low.yml"] {
        let (config, files, facts) = prepared(&root, config_name);
        let output = diagnostics(&root, &config, &files, &facts).unwrap();
        assert!(output.advisories.is_empty());
        for rule_id in RULE_IDS {
            let found = output
                .findings
                .iter()
                .filter(|finding| finding.rule == *rule_id)
                .collect::<Vec<_>>();
            assert_eq!(found.len(), 2, "{config_name} {rule_id}: {found:?}");
            assert_eq!(
                found.iter().map(|item| item.line).collect::<Vec<_>>(),
                [3, 6]
            );
            assert!(found.iter().all(|item| item.file == "migrations/001.sql"));
        }
    }
}

#[test]
fn policy_routes_the_same_markers_to_advisories_or_ignores_them() {
    let root = fixture();
    let (config, files, facts) = prepared(&root, "advise.yml");
    let advised = diagnostics(&root, &config, &files, &facts).unwrap();
    assert!(advised.findings.is_empty());
    assert_eq!(advised.advisories.len(), RULE_IDS.len() * 2);

    let (config, files, facts) = prepared(&root, "ignore.yml");
    let ignored = diagnostics(&root, &config, &files, &facts).unwrap();
    assert!(ignored.findings.is_empty());
    assert!(ignored.advisories.is_empty());
}

#[test]
fn nested_do_markers_keep_individual_heads_and_lines() {
    let root = fixture().join("nested");
    let sql = std::fs::read_to_string(root.join("migrations/001.sql")).unwrap();
    let markers = extract_migration_facts(&sql).unanalyzable_statements;
    assert_eq!(markers.len(), 2, "{markers:?}");
    assert_eq!(
        markers.iter().map(|item| item.line).collect::<Vec<_>>(),
        [4, 5]
    );
    assert!(markers[0].head.contains("alpha"));
    assert!(markers[1].head.contains("beta"));
    let (config, files, facts) = prepared(&root, "report.yml");
    assert_eq!(
        diagnostics(&root, &config, &files, &facts)
            .unwrap()
            .findings
            .len(),
        2
    );
}

#[test]
fn static_execute_markers_match_direct_and_prepared_schema_facts() {
    let root = fixture().join("dynamic");
    let sql = std::fs::read_to_string(root.join("migrations/001.sql")).unwrap();
    let direct = extract_migration_facts(&sql).unanalyzable_statements;
    assert_eq!(direct.len(), 1, "{direct:?}");
    assert_eq!(direct[0].line, 2);
    assert!(direct[0].head.starts_with("ALTER TABLE"));
    let (_, files, facts) = prepared(&root, "report.yml");
    let projected = &facts
        .postgres_schema_file(&files[0])
        .unwrap()
        .unanalyzable_statements;
    assert_eq!(projected, &direct);
}

#[test]
fn valid_static_execute_has_no_marker_but_unknown_execute_remains_opaque() {
    for (name, count) in [("dynamic-valid", 0), ("dynamic-unknown", 1)] {
        let root = fixture().join(name);
        let sql = std::fs::read_to_string(root.join("migrations/001.sql")).unwrap();
        let direct = extract_migration_facts(&sql).unanalyzable_statements;
        assert_eq!(direct.len(), count, "{name}: {direct:?}");
        if name == "dynamic-unknown" {
            assert!(direct[0].head.contains("EXECUTE"));
        }
        let (_, files, facts) = prepared(&root, "report.yml");
        assert_eq!(
            &facts
                .postgres_schema_file(&files[0])
                .unwrap()
                .unanalyzable_statements,
            &direct,
            "{name}"
        );
    }
}

#[test]
fn line_suppressions_apply_to_marker_findings() {
    let root = fixture().join("suppress");
    let (config, files, facts) = prepared(&root, "report.yml");
    let mut findings = diagnostics(&root, &config, &files, &facts)
        .unwrap()
        .findings;
    assert_eq!(findings.len(), 3);
    let sources = source_store_for_files(&files);
    suppress_rule_findings_with_sources(&root, &mut findings, &sources);
    assert_eq!(findings.len(), 1);
    assert_eq!(findings[0].line, 4);
}

#[test]
fn readable_schema_rules_keep_their_source_failure_tolerance() {
    let root = fixture().join("unreadable");
    let config = load_v2_config(&root, Some(&root.join("report.yml"))).unwrap();
    let files = vec![root.join("migrations/bad.sql")];
    let facts = crate::codebase::postgres::prepare_rule_sql_facts(
        &root,
        &files,
        source_store_for_files(&files),
        &config,
        &[
            "postgres-idempotent-insert",
            "postgres-no-generated-column-writes",
        ],
    )
    .unwrap();
    let output = diagnostics(&root, &config, &files, &facts).unwrap();
    assert!(output.findings.is_empty());
    assert!(output.advisories.is_empty());
}
