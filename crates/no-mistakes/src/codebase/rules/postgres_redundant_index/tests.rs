use super::*;
use crate::codebase::postgres::{SqlCreateIndexMetadata, SqlIndexParam};
use crate::config::v2::{
    schema::{RuleDef, RuleScope},
    NoMistakesConfig,
};
use std::path::{Path, PathBuf};

fn fixture(name: &str) -> PathBuf {
    crate::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../test-cases/rules/postgres-redundant-index/fixture")
            .join(name),
    )
}

fn config(extra: &str) -> NoMistakesConfig {
    NoMistakesConfig {
        rules: vec![RuleDef {
            rule: RULE_ID.to_string(),
            scope: Some(RuleScope::Repository),
            options: serde_yaml::from_str(&format!(
                "sqlInclude: [\"migrations/**/*.sql\"]\n{extra}"
            ))
            .unwrap(),
            ..Default::default()
        }],
        ..Default::default()
    }
}

fn sql_files(root: &Path) -> Vec<PathBuf> {
    [
        "001.sql",
        "0001.sql",
        "0002.sql",
        "2.sql",
        "10.sql",
        "V2__create.sql",
        "V10__drop.sql",
    ]
    .into_iter()
    .map(|name| root.join("migrations").join(name))
    .filter(|path| path.is_file())
    .collect()
}

fn run(root: &Path, extra: &str) -> Vec<RuleFinding> {
    check_with_files(root, &config(extra), &sql_files(root)).unwrap()
}

fn param(name: &str, ordering: Option<&str>, nulls: Option<&str>) -> SqlIndexParam {
    SqlIndexParam {
        name: Some(name.to_string()),
        ordering: ordering.map(str::to_string),
        nulls_ordering: nulls.map(str::to_string),
        ..Default::default()
    }
}

fn idx(name: &str, columns: Vec<SqlIndexParam>) -> SqlCreateIndexMetadata {
    SqlCreateIndexMetadata {
        table_name: "events".to_string(),
        name: Some(name.to_string()),
        leading_column: columns.first().and_then(|column| column.name.clone()),
        columns,
        ..Default::default()
    }
}

fn live<'a>(index: &'a SqlCreateIndexMetadata) -> redundancy::LiveIndex<'a> {
    redundancy::LiveIndex {
        rel: "migrations/001.sql".to_string(),
        path: Path::new("migrations/001.sql"),
        index,
    }
}

#[test]
fn flags_strict_prefix_btree_index() {
    let findings = run(&fixture("fail"), "");
    assert_eq!(findings.len(), 1, "{findings:?}");
    assert_eq!(
        findings[0].target.as_deref(),
        Some("events.idx_events__topic_id")
    );
    assert!(findings[0]
        .message
        .contains("idx_events__topic_id__created_at"));
}

#[test]
fn unique_shorter_index_is_not_redundant() {
    assert!(run(&fixture("pass"), "").is_empty());
}

#[test]
fn same_line_directive_exempts_the_index() {
    assert!(run(&fixture("allow"), "").is_empty());
}

#[test]
fn later_drop_removes_the_prefix_index() {
    assert!(run(&fixture("dropped"), "").is_empty());
}

#[test]
fn allowed_index_exempts_the_prefix() {
    assert!(run(
        &fixture("fail"),
        "allowedIndexes: [events.idx_events__topic_id]\n"
    )
    .is_empty());
}

#[test]
fn stale_allowed_index_is_a_finding() {
    let findings = run(
        &fixture("pass"),
        "allowedIndexes: [events.idx_never_matched]\n",
    );
    assert!(findings
        .iter()
        .any(|finding| finding.message.contains("stale")));
}

#[test]
fn custom_allow_directive_must_match_the_comment() {
    let findings = run(&fixture("allow"), "allowDirective: skip-index\n");
    assert_eq!(findings.len(), 1, "{findings:?}");
}

#[test]
fn option_defaults_use_schema_sql_include() {
    let compiled = compile_options(&Options::default());
    assert_eq!(
        compiled.schema.sql_include,
        crate::codebase::postgres::PostgresSchemaOptions::default().sql_include
    );
    assert!(compiled.allowed_indexes.is_empty());
    assert_eq!(compiled.allow_directive, "redundant-index-allow");
}

#[test]
fn prefix_helpers_cover_predicate_include_and_sort() {
    let short = idx("short", vec![param("topic_id", None, None)]);
    let long = idx(
        "long",
        vec![
            param("topic_id", None, None),
            param("created_at", None, None),
        ],
    );
    assert!(redundancy::is_redundant_prefix(&live(&short), &live(&long)));
    let unique = SqlCreateIndexMetadata {
        unique: true,
        ..short.clone()
    };
    assert!(!redundancy::is_redundant_prefix(
        &live(&unique),
        &live(&long)
    ));
    let other_pred = SqlCreateIndexMetadata {
        predicate_key: Some("deleted_at is null".to_string()),
        ..short.clone()
    };
    assert!(!redundancy::is_redundant_prefix(
        &live(&other_pred),
        &live(&long)
    ));
    let include_short = SqlCreateIndexMetadata {
        include_columns: vec!["status".to_string()],
        ..short.clone()
    };
    assert!(!redundancy::is_redundant_prefix(
        &live(&include_short),
        &live(&long)
    ));
    let include_long = SqlCreateIndexMetadata {
        include_columns: vec!["status".to_string()],
        ..long.clone()
    };
    assert!(redundancy::is_redundant_prefix(
        &live(&include_short),
        &live(&include_long)
    ));
    let desc = idx(
        "desc",
        vec![param("created_at", Some("desc"), Some("last"))],
    );
    let desc_long = idx(
        "desc_long",
        vec![
            param("created_at", Some("desc"), None),
            param("topic_id", None, None),
        ],
    );
    assert!(!redundancy::is_redundant_prefix(
        &live(&desc),
        &live(&desc_long)
    ));
    let desc_match = idx(
        "desc_match",
        vec![
            param("created_at", Some("desc"), Some("last")),
            param("topic_id", None, None),
        ],
    );
    assert!(redundancy::is_redundant_prefix(
        &live(&desc),
        &live(&desc_match)
    ));
    let gin = SqlCreateIndexMetadata {
        access_method: "gin".to_string(),
        ..short.clone()
    };
    assert!(!redundancy::is_redundant_prefix(&live(&gin), &live(&long)));
    let unnamed = SqlCreateIndexMetadata {
        name: None,
        ..short.clone()
    };
    assert!(redundancy::is_redundant_prefix(
        &live(&unnamed),
        &live(&long)
    ));
    let empty = idx("empty", Vec::new());
    assert!(!redundancy::is_redundant_prefix(
        &live(&empty),
        &live(&long)
    ));
}

#[test]
fn directive_on_line_ignores_empty_or_missing_lines() {
    assert!(!scan::directive_on_line("", 0, "redundant-index-allow"));
    assert!(!scan::directive_on_line("CREATE INDEX idx ON t (a)", 1, ""));
    assert!(scan::directive_on_line(
        "CREATE INDEX idx ON t (a); -- skip-index",
        1,
        "skip-index"
    ));
}

#[test]
fn different_tables_are_not_redundant() {
    let short = idx("short", vec![param("topic_id", None, None)]);
    let other = SqlCreateIndexMetadata {
        table_name: "other".to_string(),
        columns: vec![
            param("topic_id", None, None),
            param("created_at", None, None),
        ],
        ..idx("long", vec![param("topic_id", None, None)])
    };
    assert!(!redundancy::is_redundant_prefix(
        &live(&short),
        &live(&other)
    ));
}

#[test]
fn omitted_btree_sort_options_match_asc_nulls_last() {
    let short = idx("short", vec![param("topic_id", None, None)]);
    let long = idx(
        "long",
        vec![
            param("topic_id", Some("asc"), Some("last")),
            param("created_at", None, None),
        ],
    );
    assert!(redundancy::is_redundant_prefix(&live(&short), &live(&long)));
    let first = idx("first", vec![param("topic_id", Some("asc"), Some("first"))]);
    assert!(!redundancy::is_redundant_prefix(
        &live(&first),
        &live(&long)
    ));
}

#[test]
fn omitted_desc_nulls_default_to_first_not_last() {
    let omitted = idx("omitted", vec![param("created_at", Some("desc"), None)]);
    let last = idx(
        "last",
        vec![
            param("created_at", Some("desc"), Some("last")),
            param("topic_id", None, None),
        ],
    );
    assert!(!redundancy::is_redundant_prefix(
        &live(&omitted),
        &live(&last)
    ));
    let first = idx(
        "first",
        vec![
            param("created_at", Some("desc"), Some("first")),
            param("topic_id", None, None),
        ],
    );
    assert!(redundancy::is_redundant_prefix(
        &live(&omitted),
        &live(&first)
    ));
}

#[test]
fn numeric_filename_prefixes_sort_before_lexicographic_digits() {
    use super::order::cmp_sql_rel;
    use std::cmp::Ordering;
    assert_eq!(cmp_sql_rel("2.sql", "10.sql"), Ordering::Less);
    assert_eq!(
        cmp_sql_rel("migrations/V2__create.sql", "migrations/V10__drop.sql"),
        Ordering::Less
    );
    assert_eq!(
        cmp_sql_rel("migrations/2.sql", "other/10.sql"),
        Ordering::Less
    );
    assert_eq!(
        cmp_sql_rel("migrations/notes.sql", "migrations/schema.sql"),
        "migrations/notes.sql".cmp("migrations/schema.sql")
    );
    let overflow = format!("migrations/{}.sql", "9".repeat(40));
    assert_eq!(
        cmp_sql_rel(&overflow, "migrations/1.sql"),
        overflow.as_str().cmp("migrations/1.sql")
    );
}

#[test]
fn schema_qualified_tables_are_not_compared_together() {
    assert!(run(&fixture("schema"), "").is_empty());
}

#[test]
fn drop_index_in_another_schema_does_not_remove_the_prefix() {
    let findings = run(&fixture("schema-drop"), "");
    assert_eq!(findings.len(), 1, "{findings:?}");
    assert_eq!(
        findings[0].target.as_deref(),
        Some("public.events.idx_events__topic_id")
    );
}

#[test]
fn drop_table_removes_that_table_indexes() {
    assert!(run(&fixture("dropped-table"), "").is_empty());
}

#[test]
fn numeric_later_drop_removes_the_prefix_index() {
    assert!(run(&fixture("numeric-order"), "").is_empty());
}

#[test]
fn flyway_version_prefix_drop_removes_the_prefix_index() {
    assert!(run(&fixture("flyway-order"), "").is_empty());
}

#[test]
fn multiline_create_index_allow_directive_uses_the_create_line() {
    assert!(run(&fixture("multiline"), "").is_empty());
}

#[test]
fn describe_index_falls_back_to_columns_for_unnamed_indexes() {
    let unnamed = SqlCreateIndexMetadata {
        name: None,
        columns: vec![
            param("topic_id", None, None),
            SqlIndexParam {
                name: None,
                ..Default::default()
            },
        ],
        ..idx("unused", vec![param("topic_id", None, None)])
    };
    let described = redundancy::describe_index(&unnamed);
    assert!(described.contains("implicit index"));
    assert!(described.contains("<expr>"));
}

#[test]
fn earlier_drop_does_not_remove_a_later_prefix_create() {
    let findings = run(&fixture("earlier-drop"), "");
    assert_eq!(findings.len(), 1, "{findings:?}");
}

#[test]
fn predicate_string_literals_are_case_sensitive() {
    assert!(run(&fixture("predicate-case"), "").is_empty());
}
