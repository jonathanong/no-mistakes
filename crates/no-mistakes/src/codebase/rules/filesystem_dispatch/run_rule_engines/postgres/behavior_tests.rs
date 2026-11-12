use super::*;
use crate::config::v2::schema::{RuleDef, RuleScope};

fn fixture(path: &str) -> PathBuf {
    crate::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .join(path),
    )
}

fn configuration(rule: &str, options: &str) -> NoMistakesConfig {
    NoMistakesConfig {
        rules: vec![RuleDef {
            rule: rule.into(),
            scope: Some(RuleScope::Repository),
            options: crate::codebase::postgres::tests::fixture_rule_options(options),
            ..Default::default()
        }],
        ..Default::default()
    }
}

#[test]
fn standalone_and_prepared_dispatch_preserve_real_postgres_findings() {
    for (rule, location, options, paths, expected_file) in [
        (POSTGRES_LOCK_WRITTEN_ROWS_ONLY,
         "test-cases/rules/postgres-lock-written-rows-only/fixture/row-unwritten",
         "importSpecifier: '@example/db'\nexecutorFactoryNames: [beginTransaction]\nexecutorTypeNames: [TransactionQuery]",
         vec!["src/query.ts"], "src/query.ts"),
        (POSTGRES_NO_GENERATED_COLUMN_WRITES,
         "test-cases/rules/postgres-no-generated-column-writes/unit-fixture/do-write-lines",
         "sqlInclude: [schema.sql]", vec!["schema.sql", "writes.sql"], "writes.sql"),
        (POSTGRES_NO_TRANSFORMED_UUID_PREDICATES,
         "test-cases/rules/postgres-no-transformed-uuid-predicates/fixture",
         "schemaCatalogPath: schema.json\nimportSpecifier: db\nexecutorNames: [query]",
         vec!["embedded.ts", "schema.json"], "embedded.ts"),
        (POSTGRES_REQUIRE_QUERY_ANNOTATION,
         "test-cases/rules/postgres-require-query-annotation/fixture/fail",
         "importSpecifier: '@example/db'", vec!["src/query.ts"], "src/query.ts"),
        (POSTGRES_GENERATED_COLUMN_PREDICATES,
         "test-cases/rules/postgres-generated-column-predicates/fixture/fail",
         "sqlInclude: ['sql/**/*.sql']",
         vec!["sql/schema.sql", "sql/where.sql"], "sql/where.sql"),
        (POSTGRES_GROUPING_NOT_AGGREGATION,
         "test-cases/rules/postgres-grouping-not-aggregation/fixture",
         "schemaCatalogPath: schema.json\nsqlInclude: ['review/raw/**/*.sql']",
         vec!["schema.json", "review/raw/review-controls.sql"], "review/raw/review-controls.sql"),
        (POSTGRES_KEY_COLUMN_TYPES,
         "test-cases/rules/postgres-key-column-types/fixture/fail",
         "schemaCatalogPath: schema.json\nallowedTypes: [uuid]",
         vec!["schema.json"], "schema.json"),
        (POSTGRES_LOCK_ORDERING,
         "test-cases/rules/postgres-lock-ordering/fixture/fail-multi-target",
         "{}", vec!["src/lock.ts"], "src/lock.ts"),
    ] {
        let root = fixture(location);
        let files = paths.iter().map(|path| root.join(path)).collect::<Vec<_>>();
        assert!(files.iter().all(|path| path.is_file()), "{files:?}");
        let config = configuration(rule, options);
        let sources = crate::codebase::rules::source_store_for_files(&files);
        let standalone = run(rule, &root, &config, &files, &sources, None)
            .unwrap().unwrap();
        assert!(standalone.iter().any(|finding| finding.rule == rule
            && finding.file == expected_file && finding.line > 0), "{rule}: {standalone:#?}");
        let facts = crate::codebase::postgres::prepare_rule_sql_facts(
            &root, &files, sources.clone(), &config, &[rule],
        ).unwrap();
        let prepared = run(rule, &root, &config, &files, &sources, Some(&facts))
            .unwrap().unwrap();
        assert_eq!(prepared, standalone, "{rule}");
    }
}
