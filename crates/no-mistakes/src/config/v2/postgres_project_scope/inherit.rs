use super::narrow::{escapes_project_root, prove_patterns, validate_glob};
use super::scope::merge;
use super::yaml::{
    contains_key, ensure_mapping, set_bool, set_string, set_strings, set_tags, string_list,
};
use super::Scope;
use crate::config::v2::schema::{
    postgres_executors, NoMistakesConfig, Project, RuleDef, RuleScope,
};
use anyhow::{bail, Result};
use serde_yaml::Value;

const RESTATED: &[&str] = &[
    "importSpecifier",
    "executorNames",
    "executorFactoryNames",
    "executorTypeNames",
    "trustedSqlTags",
    "schemaCatalogPath",
    "unanalyzableSql",
];

pub(super) fn apply(config: &mut NoMistakesConfig) -> Result<bool> {
    validate_project_globs(config)?;
    let mut plans = Vec::new();
    for (index, rule) in config.rules.iter().enumerate() {
        if let Some(plan) = plan_rule(config, rule)? {
            plans.push((index, plan));
        }
    }
    // Catalog rules write schemaCatalogPath without sqlIncludeLocked. Any applied
    // plan must set the materialize flag, or a second call treats that path as a restate.
    let applied = !plans.is_empty();
    for (index, (scope, keep_rule_sql)) in plans {
        let rule_name = config.rules[index].rule.clone();
        let options = ensure_mapping(&mut config.rules[index].options, &rule_name)?;
        write_scope(&rule_name, options, &scope, keep_rule_sql);
    }
    Ok(applied)
}

fn plan_rule(config: &NoMistakesConfig, rule: &RuleDef) -> Result<Option<(Scope, bool)>> {
    if !rule.enabled || !rule.rule.starts_with("postgres-") {
        return Ok(None);
    }
    let scopes = referenced(config, rule)?;
    if scopes.is_empty() {
        return Ok(None);
    }
    reject_restate(rule)?;
    let scope = merge(rule, &scopes)?;
    // Empty sqlInclude inherits the project list. A non-empty list was proven narrower.
    let keep_rule_sql = narrow(rule, &scope)?;
    if postgres_executors::RULE_IDS.contains(&rule.rule.as_str())
        && scope.import_specifier.is_empty()
    {
        bail!(
            "{} references a postgres project without a non-empty executors.importSpecifier; set it on the project (the rule cannot restate executor options)",
            rule.rule
        );
    }
    Ok(Some((scope, keep_rule_sql)))
}

fn referenced<'a>(
    config: &'a NoMistakesConfig,
    rule: &RuleDef,
) -> Result<Vec<(String, &'a Project)>> {
    let mut found = Vec::new();
    let mut plain = Vec::new();
    for name in &rule.projects {
        match config.projects.get(name) {
            Some(project) if project.postgres.is_some() => found.push((name.clone(), project)),
            Some(_) => plain.push(name.clone()),
            None => plain.push(name.clone()),
        }
    }
    if found.is_empty() {
        return Ok(Vec::new());
    }
    if let Some(name) = plain.first() {
        bail!(
            "{} references project `{name}` without a postgres block while also referencing `{}`",
            rule.rule,
            found[0].0
        );
    }
    Ok(found)
}

fn reject_restate(rule: &RuleDef) -> Result<()> {
    if rule.scope == Some(RuleScope::Repository) {
        bail!(
            "{0} cannot set scope: repository when it references a postgres project; repository scope is a second, wider pass",
            rule.rule
        );
    }
    let Some(map) = rule.options.as_mapping() else {
        if rule.options.is_null() {
            return Ok(());
        }
        bail!("{} options must be a mapping", rule.rule);
    };
    for key in RESTATED {
        if contains_key(map, key) {
            bail!(
                "{0} options.{key} restates the postgres project scope; remove it and narrow with include, exclude, or sqlInclude",
                rule.rule
            );
        }
    }
    Ok(())
}

fn narrow(rule: &RuleDef, scope: &Scope) -> Result<bool> {
    for pattern in &rule.include {
        validate_glob(pattern, &format!("{} include", rule.rule))?;
        if scope
            .roots
            .iter()
            .any(|root| escapes_project_root(root.as_deref(), pattern))
        {
            bail!("{} include `{pattern}` escapes the project root", rule.rule);
        }
    }
    prove_patterns(&rule.rule, "include", &rule.include, &scope.raw_includes)?;
    let options = rule.options.as_mapping();
    let option_include = string_list(
        options.and_then(|map| map.get(Value::String("include".to_string()))),
        &rule.rule,
        "include",
    )?;
    prove_patterns(
        &rule.rule,
        "options.include",
        &option_include,
        &scope.option_includes,
    )?;
    let sql_include = string_list(
        options.and_then(|map| map.get(Value::String("sqlInclude".to_string()))),
        &rule.rule,
        "sqlInclude",
    )?;
    if sql_include.is_empty() {
        return Ok(false);
    }
    if scope.sql_include.is_empty() {
        bail!(
            "{} options.sqlInclude widens an empty project sqlInclude",
            rule.rule
        );
    }
    prove_patterns(
        &rule.rule,
        "options.sqlInclude",
        &sql_include,
        &scope.sql_include,
    )?;
    Ok(true)
}

fn write_scope(rule: &str, options: &mut serde_yaml::Mapping, scope: &Scope, keep_rule_sql: bool) {
    if super::yaml::write_catalog_only(rule, options, &scope.schema_catalog_path) {
        return;
    }
    if !scope.import_specifier.is_empty() {
        set_string(options, "importSpecifier", &scope.import_specifier);
    }
    if !scope.factory_names.is_empty() {
        set_strings(options, "executorFactoryNames", &scope.factory_names);
    }
    if !scope.type_names.is_empty() {
        set_strings(options, "executorTypeNames", &scope.type_names);
    }
    if !scope.trusted_sql_tags.is_empty() {
        set_tags(options, &scope.trusted_sql_tags);
    }
    if !scope.schema_catalog_path.is_empty() {
        set_string(options, "schemaCatalogPath", &scope.schema_catalog_path);
    }
    if !scope.unanalyzable_sql.is_empty() {
        set_string(options, "unanalyzableSql", &scope.unanalyzable_sql);
    }
    if !keep_rule_sql {
        set_strings(options, "sqlInclude", &scope.sql_include);
    }
    set_bool(options, "sqlIncludeLocked", true);
}

fn validate_project_globs(config: &NoMistakesConfig) -> Result<()> {
    for (name, project) in &config.projects {
        if let Some(postgres) = &project.postgres {
            for pattern in &postgres.sql_include {
                validate_glob(pattern, &format!("projects.{name}.postgres.sqlInclude"))?;
            }
        }
    }
    Ok(())
}
