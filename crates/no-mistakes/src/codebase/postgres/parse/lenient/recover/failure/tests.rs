use sqlparser::dialect::PostgreSqlDialect;
use sqlparser::tokenizer::Tokenizer;
use std::path::PathBuf;

fn fixture(name: &str) -> String {
    std::fs::read_to_string(
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../test-cases/check-runner/postgres-parse-failure-locations/fixture")
            .join(name),
    )
    .unwrap()
}

#[test]
fn postgres_regress_query_shapes_are_supported_or_advised() {
    let mut unrecognized = Vec::new();
    for statement in ["regress-query-valid.sql", "regress-query-guards-valid.sql"]
        .into_iter()
        .flat_map(|name| {
            fixture(name)
                .split(';')
                .map(str::to_owned)
                .collect::<Vec<_>>()
        })
    {
        let statement = statement.trim();
        if statement.is_empty() {
            continue;
        }
        let parsed =
            crate::codebase::postgres::parse::parse_postgres_sql_with_function_sources(statement);
        let failures = parsed
            .iter()
            .filter_map(|item| item.recovery_failure.as_ref())
            .collect::<Vec<_>>();
        if statement.to_ascii_uppercase().starts_with("INSERT")
            || statement.to_ascii_uppercase().starts_with("UPDATE")
        {
            assert!(
                failures.is_empty(),
                "DML must retain typed facts: {statement}: {failures:#?}"
            );
        }
        if failures.iter().any(|failure| !failure.valid_unsupported) {
            unrecognized.push(statement.to_string());
        }
    }
    assert!(unrecognized.is_empty(), "{}", unrecognized.join("\n---\n"));
}

#[test]
fn malformed_regress_query_near_misses_still_report() {
    // These are parser near misses: a compatibility rewrite must preserve each error.
    let mut unexpectedly_accepted = Vec::new();
    for statement in [
        "regress-query-malformed.sql",
        "regress-query-guards-malformed.sql",
    ]
    .into_iter()
    .flat_map(|name| {
        fixture(name)
            .split(';')
            .map(str::to_owned)
            .collect::<Vec<_>>()
    }) {
        let statement = statement.trim();
        if statement.is_empty() {
            continue;
        }
        let parsed =
            crate::codebase::postgres::parse::parse_postgres_sql_with_function_sources(statement);
        let failures = parsed
            .iter()
            .filter_map(|item| item.recovery_failure.as_ref())
            .collect::<Vec<_>>();
        if !failures.iter().any(|failure| !failure.valid_unsupported) {
            unexpectedly_accepted.push(statement.to_owned());
        }
    }
    assert!(
        unexpectedly_accepted.is_empty(),
        "{}",
        unexpectedly_accepted.join("\n")
    );
}

#[test]
fn canonical_capture_ignores_chunks_without_code_or_source_tokens() {
    for name in ["canonical-empty.sql", "canonical-comments.sql"] {
        let source = fixture(name);
        let tokens = Tokenizer::new(&PostgreSqlDialect {}, &source)
            .tokenize_with_location()
            .unwrap();
        let code: Vec<_> = tokens
            .into_iter()
            .filter(|token| token.token != sqlparser::tokenizer::Token::SemiColon)
            .collect();
        assert!(super::unrecoverable_chunk(&code, 0).is_none());
    }
    let source = fixture("failed-insert.sql");
    let tokens = Tokenizer::new(&PostgreSqlDialect {}, &source)
        .tokenize()
        .unwrap();
    // An unlocated parser fragment cannot invent a source-backed failure occurrence.
    let parsed = super::super::parse_chunks_with_function_calls(
        vec![tokens],
        &[],
        true,
        false,
        &crate::codebase::postgres::parse::RecursiveViews::default(),
    );
    assert!(crate::codebase::postgres::migration::recovery_failures(&parsed).is_empty());
}

#[test]
fn publication_alter_and_drop_require_complete_grammar() {
    let root =
        crate::test_support::rule_fixture_root("postgres-ddl-unanalyzable").join("publication");
    for (file, valid) in [("valid.sql", true), ("malformed.sql", false)] {
        for line in std::fs::read_to_string(root.join(file)).unwrap().lines() {
            let tokens = Tokenizer::new(&PostgreSqlDialect {}, line.trim_end_matches(';'))
                .tokenize_with_location()
                .unwrap();
            assert_eq!(super::unsupported::valid(&tokens), valid, "{line}");
        }
    }
}

#[test]
fn full_regress_context_forms_remain_recognized() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/postgres-facts/source/regress-full-context");
    for name in [
        "index-opclass.sql",
        "view-row-values.sql",
        "xmltable.sql",
        "tablesample-null.sql",
        "cte-search.sql",
        "cte-search-nonrecursive.sql",
        "cte-merge-row-cast.sql",
    ] {
        let sql = std::fs::read_to_string(root.join(name)).unwrap();
        let failures = crate::codebase::postgres::migration::extract_migration_facts(&sql)
            .unanalyzable_statements;
        assert!(
            failures.iter().all(|failure| failure.valid_unsupported),
            "{name}: {failures:#?}"
        );
    }
    for sql in std::fs::read_to_string(root.join("malformed.sql"))
        .unwrap()
        .lines()
    {
        let failures = crate::codebase::postgres::migration::extract_migration_facts(sql)
            .unanalyzable_statements;
        assert!(
            failures.iter().any(|failure| !failure.valid_unsupported),
            "{sql}: {failures:#?}"
        );
    }
}
