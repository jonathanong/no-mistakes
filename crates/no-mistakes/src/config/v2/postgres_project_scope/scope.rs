use super::narrow::prefix_pattern;
use super::Scope;
use crate::config::v2::schema::{Project, ProjectPostgresConfig, RuleDef};
use anyhow::{bail, Result};

pub(super) fn merge(rule: &RuleDef, projects: &[(String, &Project)]) -> Result<Scope> {
    let mut scopes = projects
        .iter()
        .map(|(name, project)| scope_of(name, project));
    let mut first = scopes.next().expect("at least one postgres project");
    for other in scopes {
        disagree(rule, &first, &other)?;
        first.raw_includes.extend(other.raw_includes);
        first.option_includes.extend(other.option_includes);
        first.roots.extend(other.roots);
    }
    Ok(first)
}

fn disagree(rule: &RuleDef, left: &Scope, right: &Scope) -> Result<()> {
    let checks = [
        (
            "importSpecifier",
            left.import_specifier != right.import_specifier,
        ),
        (
            "factoryNames",
            sorted(&left.factory_names) != sorted(&right.factory_names),
        ),
        (
            "typeNames",
            sorted(&left.type_names) != sorted(&right.type_names),
        ),
        (
            "trustedSqlTags",
            left.trusted_sql_tags != right.trusted_sql_tags,
        ),
        (
            "schemaCatalogPath",
            left.schema_catalog_path != right.schema_catalog_path,
        ),
        (
            "unanalyzableSql",
            left.unanalyzable_sql != right.unanalyzable_sql,
        ),
        (
            "sqlInclude",
            sorted(&left.sql_include) != sorted(&right.sql_include),
        ),
    ];
    if let Some((field, _)) = checks.iter().find(|(_, differs)| *differs) {
        bail!(
            "{} references projects `{}` and `{}` whose postgres blocks disagree on {field}",
            rule.rule,
            left.name,
            right.name
        );
    }
    Ok(())
}

fn scope_of(name: &str, project: &Project) -> Scope {
    let postgres = project.postgres.as_ref().expect("postgres project");
    let executors = postgres.executors.clone().unwrap_or_default();
    let mut tags = executors.trusted_sql_tags;
    tags.sort();
    Scope {
        name: name.to_string(),
        import_specifier: executors.import_specifier.trim().to_string(),
        factory_names: unique_preserve(executors.factory_names),
        type_names: unique_preserve(executors.type_names),
        trusted_sql_tags: tags,
        schema_catalog_path: prefixed_path(project, postgres),
        unanalyzable_sql: postgres.unanalyzable_sql.clone().unwrap_or_default(),
        sql_include: postgres
            .sql_include
            .iter()
            .map(|pattern| prefix_pattern(project.root.as_deref(), pattern))
            .collect(),
        raw_includes: project.include.clone(),
        option_includes: project
            .include
            .iter()
            .map(|pattern| prefix_pattern(project.root.as_deref(), pattern))
            .collect(),
        roots: vec![project.root.clone()],
    }
}

fn prefixed_path(project: &Project, postgres: &ProjectPostgresConfig) -> String {
    postgres
        .schema_catalog_path
        .as_deref()
        .filter(|path| !path.trim().is_empty())
        .map(|path| prefix_pattern(project.root.as_deref(), path))
        .unwrap_or_default()
}

fn unique_preserve(values: Vec<String>) -> Vec<String> {
    let mut out = Vec::new();
    for value in values {
        if !out.contains(&value) {
            out.push(value);
        }
    }
    out
}

fn sorted(values: &[String]) -> Vec<String> {
    let mut values = values.to_vec();
    values.sort();
    values
}
