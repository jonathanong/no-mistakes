use super::path_globs::{field_error, validate_globs};
use super::NoMistakesConfig;
use crate::changes::config::{compile_selection_glob, from_path_parts, IdRule};
use crate::config::source_locations::ConfigPath;
use anyhow::Result;

pub(super) fn validate(config: &NoMistakesConfig) -> Result<()> {
    validate_terraform(config)?;
    validate_change_categories(config)
}

fn validate_terraform(config: &NoMistakesConfig) -> Result<()> {
    let test = &config.infra.terraform.test;
    let base = ConfigPath::default()
        .key("infra")
        .key("terraform")
        .key("test");
    let globs = base.clone().key("testGlobs");
    validate_globs(&test.test_globs, &globs.to_string(), |index| {
        globs.clone().index(index)
    })?;
    match test.match_mode.as_deref() {
        Some("module") if test.test_root.is_some() => {
            let path = base.key("testRoot");
            Err(field_error(
                path.clone(),
                format!("{path}: a shared testRoot cannot be combined with match: module"),
            ))
        }
        Some(mode) if !matches!(mode, "resource" | "module") => {
            let path = base.key("match");
            Err(field_error(
                path.clone(),
                format!("{path}: invalid mode `{mode}`; use `resource` or `module`"),
            ))
        }
        _ => Ok(()),
    }
}

fn validate_change_categories(config: &NoMistakesConfig) -> Result<()> {
    let categories = ConfigPath::default().key("changes").key("categories");
    for (name, category) in &config.changes.categories {
        let base = categories.clone().key(name);
        for (field, patterns) in [("paths", &category.paths), ("exclude", &category.exclude)] {
            validate_selection_patterns(patterns, &base.clone().key(field))?;
        }
        validate_test_patterns(&category.tests, &base.clone().key("tests"))?;
        validate_id(category.id.as_ref(), &base)?;
        for (index, rule) in category.rules.iter().enumerate() {
            let rule_path = base.clone().key("rules").index(index);
            let paths = rule_path.clone().key("paths");
            if rule.paths.is_empty() {
                return Err(field_error(
                    paths.clone(),
                    format!("{paths} must not be empty"),
                ));
            }
            validate_selection_patterns(&rule.paths, &paths)?;
            validate_id(rule.id.as_ref(), &rule_path)?;
        }
        if let Some(pattern) = &category.match_pattern {
            if !pattern.contains("{id}") && !pattern.contains("<id>") {
                let path = base.key("match");
                return Err(field_error(
                    path.clone(),
                    format!("{path}: `{pattern}` must contain an {{id}} or <id> placeholder"),
                ));
            }
        }
    }
    Ok(())
}

fn validate_selection_patterns(patterns: &[String], field: &ConfigPath) -> Result<()> {
    for (index, pattern) in patterns.iter().enumerate() {
        compile_selection_glob(pattern).map_err(|error| {
            let path = field.clone().index(index);
            field_error(path.clone(), format!("{path}: {error}"))
        })?;
    }
    Ok(())
}

fn validate_test_patterns(patterns: &[String], field: &ConfigPath) -> Result<()> {
    if let Some(index) = patterns.iter().position(String::is_empty) {
        let path = field.clone().index(index);
        return Err(field_error(path.clone(), format!("{path}: empty glob")));
    }
    validate_globs(patterns, &field.to_string(), |index| {
        field.clone().index(index)
    })
}

fn validate_id(id: Option<&IdRule>, path: &ConfigPath) -> Result<()> {
    if let Some(template) = id.and_then(|id| id.from_path.as_deref()) {
        if from_path_parts(template).is_none() {
            let field = path.clone().key("id").key("fromPath");
            return Err(field_error(
                field.clone(),
                format!("{field}: `{template}` requires exactly one {{id}} placeholder"),
            ));
        }
    }
    Ok(())
}
