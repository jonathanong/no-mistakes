use super::{
    prepared_facts::{
        check_prepared_schema_rule, check_prepared_sql_rule, run_prepared_file_rule,
        selected_schema_catalog,
    },
    source_access::source_store_for_files,
    PreparedRuleContext,
};
use crate::codebase::check_facts::CheckFactMap;
use crate::config::v2::schema::{Project, RuleDef, RuleScope};
use crate::config::v2::NoMistakesConfig;
use anyhow::anyhow;
use std::path::{Path, PathBuf};

fn options(name: &str) -> serde_yaml::Value {
    let source = match name {
        "parent-catalog.yml" => {
            include_str!(
                "../../../../../../fixtures/postgres-facts/prepared-rules/parent-catalog.yml"
            )
        }
        "low-alias.yml" => {
            include_str!("../../../../../../fixtures/postgres-facts/prepared-rules/low-alias.yml")
        }
        "invalid-include.yml" => {
            include_str!(
                "../../../../../../fixtures/postgres-facts/prepared-rules/invalid-include.yml"
            )
        }
        "empty.yml" => {
            include_str!("../../../../../../fixtures/postgres-facts/prepared-rules/empty.yml")
        }
        _ => unreachable!("saved prepared-rule fixture"),
    };
    serde_yaml::from_str(source).unwrap_or_else(|error| panic!("{name}: {error}"))
}

fn rule(id: &str, fixture: &str) -> RuleDef {
    RuleDef {
        rule: id.to_string(),
        options: options(fixture),
        scope: Some(RuleScope::Repository),
        ..Default::default()
    }
}

fn configured(rule: RuleDef) -> NoMistakesConfig {
    NoMistakesConfig {
        rules: vec![rule],
        ..Default::default()
    }
}

fn detail(error: anyhow::Error) -> String {
    format!("{error:#}")
}

#[test]
fn coverage_gap_prepared_rule_errors() {
    let root = Path::new("/repo");
    let files: &[PathBuf] = &[];
    let sources = source_store_for_files(files);
    let facts = CheckFactMap::default();

    let schema = configured(rule("postgres-fk-index", "parent-catalog.yml"));
    let err = check_prepared_schema_rule(
        root,
        &schema,
        files,
        &sources,
        "postgres-fk-index",
        |_, _, _, _, _| panic!("schema scan runs after catalog resolution"),
    )
    .expect_err("parent catalog path");
    assert!(
        detail(err).contains("repository-relative"),
        "schema catalog path must fail before a scan"
    );

    let low = configured(rule("postgres-no-offset", "low-alias.yml"));
    let err = check_prepared_sql_rule(
        root,
        &low,
        files,
        &sources,
        "postgres-no-offset",
        |_, _, _, _, _| panic!("low scan runs after alias validation"),
    )
    .expect_err("low alias");
    assert!(
        detail(err).contains("invalid options.unanalyzableSql"),
        "low effort must reject an unknown unanalyzableSql alias"
    );

    let insert = configured(rule("postgres-idempotent-insert", "invalid-include.yml"));
    let err = check_prepared_sql_rule(
        root,
        &insert,
        files,
        &sources,
        "postgres-idempotent-insert",
        |_, _, _, _, _| Ok(Vec::new()),
    )
    .expect_err("insert glob");
    assert!(
        detail(err).contains("postgres-idempotent-insert include contains invalid glob"),
        "idempotent-insert include is compiled while reporting dropped SQL"
    );

    let columns = configured(rule("postgres-explicit-columns", "invalid-include.yml"));
    let err = check_prepared_sql_rule(
        root,
        &columns,
        files,
        &sources,
        "postgres-explicit-columns",
        |_, _, _, _, _| Ok(Vec::new()),
    )
    .expect_err("executor glob");
    assert!(
        detail(err).contains("postgres-explicit-columns include contains invalid glob"),
        "undeclared-executor scope fails on its own include glob"
    );

    let mut projects = configured(rule("postgres-fk-index", "empty.yml"));
    projects.rules[0].projects = vec!["web".into()];
    projects.projects.insert(
        "web".into(),
        Project {
            include: vec!["[".into()],
            ..Default::default()
        },
    );
    let err = selected_schema_catalog(root, &projects, &projects.rules[0], &facts, "a.ts")
        .expect_err("project glob");
    assert!(
        detail(err).contains("project `web` include contains invalid glob"),
        "schema lookup must reject a project include before reading a catalog"
    );

    let compiled = configured(rule("postgres-fk-index", "empty.yml"));
    let context = PreparedRuleContext::new(root, &compiled, files, &sources, &facts);
    let err = run_prepared_file_rule(
        context,
        "postgres-fk-index",
        |_: &RuleDef| -> anyhow::Result<()> { Err(anyhow!("compile failed")) },
        |_, _, _, _, _| panic!("scan runs after compile"),
    )
    .expect_err("compile");
    assert!(detail(err).contains("compile failed"));

    let mut included = compiled;
    included.rules[0].include = vec!["[".into()];
    let context = PreparedRuleContext::new(root, &included, files, &sources, &facts);
    let err = run_prepared_file_rule(
        context,
        "postgres-fk-index",
        |_| Ok(()),
        |_, _, _, _, _| Ok(Vec::new()),
    )
    .expect_err("rule include");
    assert!(
        detail(err).contains("rule `postgres-fk-index` include contains invalid glob"),
        "file selection must reject the application include"
    );
}
