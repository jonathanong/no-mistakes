use crate::codebase::postgres::migration::extract_migration_facts;
use std::path::PathBuf;

fn fixture(name: &str) -> String {
    std::fs::read_to_string(
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../fixtures/postgres-facts/source/ddl-compat/body-recovery/query-loop-matrix")
            .join(name),
    )
    .unwrap()
}

#[test]
fn line_breaks_any_and_row_locks_preserve_loop_query_and_first_action_facts() {
    for layout in ["singleline", "multiline"] {
        for predicate in ["comparison", "any"] {
            for lock in ["locked", "unlocked"] {
                for action in ["", "-update"] {
                    let name = format!("{layout}-{predicate}-{lock}{action}.sql");
                    assert_valid(&name);
                }
            }
        }
    }
    for action in ["insert", "delete", "merge"] {
        assert_valid(&format!("multiline-any-locked-{action}.sql"));
    }
    assert_valid("returning-header-update.sql");
    for prefix in [
        "labelled",
        "nested",
        "guarded-nested",
        "labelled-outer-begin",
    ] {
        assert_valid(&format!("{prefix}-locked-update.sql"));
    }
}

fn assert_valid(name: &str) {
    let sql = fixture(name);
    let facts = extract_migration_facts(&sql);
    assert!(
        facts.unanalyzable_statements.is_empty(),
        "{name}: {facts:#?}"
    );
    let expected = sql
        .lines()
        .enumerate()
        .filter_map(|(line, text)| text.find("abs(").map(|column| (line + 1, column + 1)))
        .collect::<Vec<_>>();
    let mut actual = facts
        .function_calls
        .iter()
        .filter(|call| call.name_parts == ["abs"])
        .map(|call| (call.line, call.column))
        .collect::<Vec<_>>();
    actual.sort_unstable();
    assert_eq!(
        actual, expected,
        "{name}: header and action are distinct facts"
    );
    assert!(facts
        .tables
        .iter()
        .any(|table| table.table_name == "following_query_loop"));
}

#[test]
fn malformed_loop_headers_and_actions_keep_shared_failure_markers() {
    for (name, expected_line) in [
        ("malformed-lock.sql", 6),
        ("malformed-any.sql", 6),
        ("malformed-action.sql", 6),
        ("malformed-action-tail.sql", 6),
        ("malformed-header-update.sql", 6),
        ("malformed-labelled-action.sql", 5),
        ("malformed-nested-action.sql", 5),
        ("malformed-leading-action-junk.sql", 6),
        ("malformed-outer-if.sql", 6),
        ("malformed-outer-if-select.sql", 6),
        ("malformed-inner-if.sql", 5),
        ("malformed-inner-if-no-then.sql", 5),
        ("malformed-labelled-if.sql", 5),
    ] {
        let sql = fixture(name);
        let facts = extract_migration_facts(&sql);
        assert_eq!(facts.unanalyzable_statements.len(), 1, "{name}: {facts:#?}");
        let failure = &facts.unanalyzable_statements[0];
        assert!(
            failure.schema_unanalyzable && failure.procedural_body,
            "{name}: {failure:#?}"
        );
        assert_eq!(failure.line, expected_line, "{name}: {failure:#?}");
        assert!(facts
            .tables
            .iter()
            .any(|table| table.table_name == "following_query_loop"));
    }
}

#[test]
fn loop_compatibility_does_not_accept_top_level_procedural_sql() {
    let sql = fixture("top-level-locked-update.sql");
    assert!(crate::codebase::postgres::parse_postgres_sql(&sql).is_err());
    let facts = extract_migration_facts(&sql);
    assert!(facts
        .unanalyzable_statements
        .iter()
        .any(|failure| failure.schema_unanalyzable && !failure.procedural_body));
}

#[test]
fn uncertain_headers_preserve_only_independently_valid_actions() {
    for (name, actions) in [
        ("unsupported-header-truncate.sql", vec![5]),
        ("unsupported-header-truncate-loop-table.sql", vec![5]),
        ("unsupported-header-nested-truncate.sql", vec![6]),
        ("invalid-header-truncate.sql", vec![5]),
        ("invalid-header-action.sql", vec![]),
    ] {
        let facts = extract_migration_facts(&fixture(name));
        assert_eq!(facts.unanalyzable_statements.len(), 1, "{name}: {facts:#?}");
        let failure = &facts.unanalyzable_statements[0];
        assert!(
            failure.schema_unanalyzable && failure.procedural_body,
            "{name}: {failure:#?}"
        );
        assert_eq!(failure.line, 4, "{name}: {failure:#?}");
        assert_eq!(
            facts
                .statement_kinds
                .iter()
                .filter(|item| item.kind == "TRUNCATE")
                .map(|item| item.line)
                .collect::<Vec<_>>(),
            actions,
            "{name}: {facts:#?}"
        );
    }
}

#[test]
fn loop_boundary_recognition_keeps_incomplete_prefixes_and_empty_actions_distinct() {
    use sqlparser::{dialect::PostgreSqlDialect, tokenizer::Tokenizer};
    for name in ["boundary-invalid-label.sql", "boundary-if-without-then.sql"] {
        let source = fixture(name);
        let tokens = Tokenizer::new(&PostgreSqlDialect {}, &source)
            .tokenize()
            .unwrap();
        assert_eq!(super::loop_action_start(&tokens), Ok(None), "{name}");
    }
    let source = fixture("boundary-header-only.sql");
    let tokens = Tokenizer::new(&PostgreSqlDialect {}, &source)
        .tokenize()
        .unwrap();
    // The boundary is EOF, so callers recover no SQL action from a bare header.
    assert_eq!(super::loop_action_start(&tokens), Ok(Some(tokens.len())));
}
