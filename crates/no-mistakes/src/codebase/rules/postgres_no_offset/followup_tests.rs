use super::tests::{config_with_options, fixture};
use super::*;
use crate::codebase::postgres::{sql_offset_uses, OffsetUse};

#[test]
fn sql_dml_wrappers_and_expression_containers_keep_every_occurrence() {
    let root = fixture("review-followups");
    let paths = [root.join("db/dml.sql"), root.join("db/containers.sql")];
    let findings =
        check_with_files(&root, &config_with_options("sqlInclude: ['*.sql']"), &paths).unwrap();
    assert_eq!(findings.len(), 21, "{findings:#?}");
    let text = std::fs::read_to_string(&paths[1]).unwrap();
    let uses = sql_offset_uses(&text).unwrap();
    assert_eq!(
        uses,
        [
            OffsetUse::Zero,
            OffsetUse::Other,
            OffsetUse::Other,
            OffsetUse::Other,
            OffsetUse::Other,
            OffsetUse::Zero,
            OffsetUse::Other,
            OffsetUse::Zero,
            OffsetUse::Other,
            OffsetUse::Other,
            OffsetUse::Zero,
            OffsetUse::Zero,
            OffsetUse::Other,
            OffsetUse::Other
        ]
    );
    for path in paths {
        let rel = crate::codebase::ts_source::relative_slash_path(&root, &path);
        let per_file: Vec<_> = findings.iter().filter(|f| f.file == rel).collect();
        let targets: std::collections::HashSet<_> = per_file.iter().map(|f| &f.target).collect();
        assert_eq!(targets.len(), per_file.len());
        assert_eq!(per_file[0].target.as_deref(), Some("offset"));
    }
}

#[test]
fn offset_keyword_locations_survive_comments_unicode_escape_strings_and_copy_data() {
    let root = fixture("review-followups");
    let config = config_with_options("sqlInclude: ['*.sql']");
    for (name, lines) in [("locations.sql", vec![4, 9]), ("copy.sql", vec![5])] {
        let findings = check_with_files(&root, &config, &[root.join("db").join(name)]).unwrap();
        assert_eq!(
            findings.iter().map(|f| f.line).collect::<Vec<_>>(),
            lines,
            "{name}: {findings:?}"
        );
    }
}

#[test]
fn shared_sql_declaration_rebases_lines_and_duplicate_calls_remain_visible() {
    let root = fixture("review-followups");
    let findings = check_with_files(
        &root,
        &config_with_options("{}"),
        &[root.join("src/shared.ts")],
    )
    .unwrap();
    assert_eq!(findings.len(), 2, "{findings:?}");
    assert!(findings.iter().all(|f| f.line == 4));
    let path = root.join("src/suppress.ts");
    let mut findings = check_with_files(
        &root,
        &config_with_options("{}"),
        std::slice::from_ref(&path),
    )
    .unwrap();
    let sources = super::super::source_store_for_files(std::slice::from_ref(&path));
    super::super::suppress_rule_findings_with_sources(&root, &mut findings, &sources);
    assert!(findings.is_empty());
}

#[test]
fn lenient_ddl_does_not_discard_query_spans_or_reorder_offset_kinds() {
    let root = fixture("review-followups");
    let path = root.join("db/lenient.sql");
    let text = std::fs::read_to_string(&path).unwrap();
    assert_eq!(
        crate::codebase::postgres::sql_file_offset_uses(&text),
        [(2, OffsetUse::Zero), (3, OffsetUse::Other)]
    );
    let findings = check_with_files(
        &root,
        &config_with_options("sqlInclude: ['*.sql']"),
        &[path],
    )
    .unwrap();
    assert_eq!(findings.iter().map(|f| f.line).collect::<Vec<_>>(), [2, 3]);
    assert!(findings[0].message.contains("optimizer fence"));
}

#[test]
fn copy_header_variants_and_unterminated_data_never_become_queries() {
    let root = fixture("review-followups");
    let config = config_with_options("sqlInclude: ['*.sql']");
    let findings = check_with_files(&root, &config, &[root.join("db/copy-edge.sql")]).unwrap();
    assert_eq!(
        findings.iter().map(|f| f.line).collect::<Vec<_>>(),
        [4, 10, 10],
        "{findings:#?}"
    );
    assert!(
        check_with_files(&root, &config, &[root.join("db/copy-unterminated.sql")])
            .unwrap()
            .is_empty()
    );
}

#[test]
fn stdin_inside_string_keeps_query_and_prepared_config_errors_propagate() {
    let root = fixture("review-followups");
    let path = root.join("db/stdin-string.sql");
    let config = config_with_options("sqlInclude: ['*.sql']");
    assert_eq!(
        check_with_files(&root, &config, std::slice::from_ref(&path))
            .unwrap()
            .len(),
        1
    );
    let sources = super::super::source_store_for_files(std::slice::from_ref(&path));
    let facts = crate::codebase::check_facts::CheckFactMap::default();
    for yaml in [
        "executorNames: 1",
        "include: ['[']",
        "exclude: ['[']",
        "sqlInclude: ['[']",
    ] {
        let bad = config_with_options(yaml);
        assert!(check_with_files_sources_and_facts(
            &root,
            &bad,
            std::slice::from_ref(&path),
            &sources,
            &facts
        )
        .is_err());
    }
}

#[test]
fn prepared_missing_sql_decision_reuses_the_captured_read_failure() {
    let root = fixture("review-followups");
    let files = [root.join("db/missing.sql")];
    let config = config_with_options("sqlInclude: ['*.sql']");
    let sources = super::super::source_store_for_files(&files);
    let facts = crate::codebase::postgres::prepare_rule_sql_facts(
        &root,
        &files,
        std::sync::Arc::clone(&sources),
        &config,
        &[RULE_ID],
    )
    .unwrap();
    assert_eq!(sources.physical_read_count(), 1);
    for _ in 0..2 {
        assert!(
            check_with_files_sources_and_facts(&root, &config, &files, &sources, &facts)
                .unwrap()
                .is_empty()
        );
    }
    assert_eq!(sources.physical_read_count(), 1);
    let path = root.join("db/read-error.sql");
    assert!(check_with_files(&root, &config, &[path]).is_err());
}

#[test]
fn cooked_escapes_multiline_interpolations_and_delayed_initializers_keep_physical_lines() {
    let root = fixture("review-followups");
    let path = root.join("src/source-map.ts");
    let mut findings = check_with_files(
        &root,
        &config_with_options("{}"),
        std::slice::from_ref(&path),
    )
    .unwrap();
    assert_eq!(
        findings.iter().map(|f| f.line).collect::<Vec<_>>(),
        [2, 8, 9, 11, 13, 16, 19, 22],
        "{findings:#?}"
    );
    let sources = super::super::source_store_for_files(std::slice::from_ref(&path));
    super::super::suppress_rule_findings_with_sources(&root, &mut findings, &sources);
    assert_eq!(
        findings.iter().map(|f| f.line).collect::<Vec<_>>(),
        [2, 8, 9, 11, 13, 16]
    );
}

#[test]
fn located_recovery_skips_unparsed_keywords_and_keeps_ordinary_then_recovered_order() {
    let root = fixture("review-followups");
    let path = root.join("db/recover-locations.sql");
    let sql = std::fs::read_to_string(&path).unwrap();
    assert_eq!(
        crate::codebase::postgres::sql_file_offset_uses(&sql),
        [(3, OffsetUse::Zero), (4, OffsetUse::Other)]
    );
}
