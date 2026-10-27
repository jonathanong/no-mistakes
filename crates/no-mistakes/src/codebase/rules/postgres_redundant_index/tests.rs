use super::*;
use crate::codebase::postgres::{SqlCreateIndexMetadata, SqlIndexParam};
use std::path::Path;

// Output cases moved to cli_postgres_snapshot_redundant_index.

fn param(name: &str, ordering: Option<&str>, nulls: Option<&str>) -> SqlIndexParam {
    SqlIndexParam {
        name: Some(name.to_string()),
        expression: name.to_string(),
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
