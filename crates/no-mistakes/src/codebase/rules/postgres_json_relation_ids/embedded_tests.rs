use super::tests::{fixture, run, IDS};
use crate::codebase::rules::RuleFinding;

fn suppress(files: &[&str], mut findings: Vec<RuleFinding>) -> Vec<RuleFinding> {
    let root = fixture();
    let files = files.iter().map(|file| root.join(file)).collect::<Vec<_>>();
    let sources = crate::codebase::rules::source_store_for_files(&files);
    crate::codebase::rules::suppress_rule_findings_with_sources(&root, &mut findings, &sources);
    findings
}

#[test]
fn executed_embedded_inserts_and_updates_use_physical_source_lines() {
    let findings = run(IDS, &["src/query.ts"]);
    assert_eq!(findings.len(), 2, "{findings:#?}");
    assert_eq!(
        findings
            .iter()
            .map(|f| (f.file.as_str(), f.line))
            .collect::<Vec<_>>(),
        [("src/query.ts", 3), ("src/query.ts", 5)]
    );
}

#[test]
fn opaque_embedded_statements_and_unexecuted_prose_are_ignored() {
    assert!(run(IDS, &["src/dynamic.ts"]).is_empty());
}

#[test]
fn key_locations_follow_multiline_literals_and_cooked_sources() {
    let findings = run(
        IDS,
        &["db/multiline.sql", "src/multiline.ts", "src/origins.ts"],
    );
    let locations = findings
        .iter()
        .map(|f| (f.file.as_str(), f.line))
        .collect::<Vec<_>>();
    assert_eq!(
        locations,
        [
            ("db/multiline.sql", 4),
            ("src/multiline.ts", 5),
            ("src/origins.ts", 3),
            ("src/origins.ts", 5)
        ]
    );
}

#[test]
fn all_three_sql_suppressions_use_the_finding_line() {
    let files = [
        "suppress/sql/line.sql",
        "suppress/sql/next-line.sql",
        "suppress/sql/file.sql",
    ];
    let findings = run(IDS, &files);
    assert_eq!(findings.len(), 3, "{findings:#?}");
    assert!(suppress(&files, findings).is_empty());
}

#[test]
fn embedded_suppressions_cover_calls_files_and_multiline_sql_keys() {
    let files = [
        "suppress/ts/line.ts",
        "suppress/ts/next-line.ts",
        "suppress/ts/file.ts",
        "suppress/ts/multiline.ts",
    ];
    let findings = run(IDS, &files);
    assert!(suppress(&files, findings).is_empty());
}

#[test]
fn imported_sql_is_attributed_to_the_helper_that_owns_the_literal() {
    let files = ["suppress/imports/helper.mts", "suppress/imports/caller.mts"];
    let findings = run(IDS, &files);
    assert_eq!(findings.len(), 1, "{findings:#?}");
    assert_eq!(findings[0].file, "suppress/imports/helper.mts");
    assert_eq!(findings[0].line, 2);
}

#[test]
fn imported_helpers_honor_each_suppression_at_the_physical_origin() {
    let files = [
        "suppress/imports/line-helper.mts",
        "suppress/imports/line-caller.mts",
        "suppress/imports/next-helper.mts",
        "suppress/imports/next-caller.mts",
        "suppress/imports/file-helper.mts",
        "suppress/imports/file-caller.mts",
    ];
    let findings = run(IDS, &files);
    assert_eq!(findings.len(), 3, "{findings:#?}");
    assert!(suppress(&files, findings).is_empty());
}

#[test]
fn direct_collector_honors_json_write_demand_for_embedded_calls() {
    use crate::codebase::check_facts::CheckFactPlan;
    use crate::codebase::postgres::{
        collect_postgres_facts, EmbeddedSqlOptions, PostgresSchemaOptions,
    };
    let root = fixture();
    let files = [root.join("src/query.ts")];
    let sources = crate::codebase::rules::source_store_for_files(&files);
    for requested in [false, true] {
        let facts = collect_postgres_facts(
            &root,
            &sources,
            &files,
            &CheckFactPlan {
                postgres_dml: true,
                postgres_json_writes: requested,
                ..Default::default()
            },
            &PostgresSchemaOptions::default(),
            &EmbeddedSqlOptions::configured("@example/db", &[]),
        )
        .unwrap();
        let lines: Vec<_> = facts
            .statements
            .iter()
            .flat_map(|statement| statement.json_writes.iter().map(|write| write.line))
            .collect();
        assert_eq!(lines, if requested { vec![3, 5, 7] } else { vec![] });
    }
}
