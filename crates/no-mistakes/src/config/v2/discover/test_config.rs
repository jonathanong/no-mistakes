use super::path_globs::{field_error, validate_globs};
use super::NoMistakesConfig;
use crate::config::source_locations::ConfigPath;
use crate::config::v2::schema::{RouteCoverageSource, TestPlanEnvironment, TestPlanLimit};
use anyhow::Result;

pub(super) fn validate(config: &NoMistakesConfig) -> Result<()> {
    let impact = &config.tests.impact;
    let path = ConfigPath::default().key("tests").key("impact");
    for (name, patterns) in [
        ("registries", &impact.registries),
        ("alwaysIncludeTests", &impact.always_include_tests),
    ] {
        validate_paths(patterns, &path.clone().key(name))?;
    }
    let playwright = &config.tests.playwright;
    let path = ConfigPath::default().key("tests").key("playwright");
    validate_sources(&playwright.route_coverage_sources, &path)?;
    for (name, app) in &playwright.apps {
        validate_sources(
            &app.route_coverage_sources,
            &path.clone().key("apps").key(name),
        )?;
    }
    Ok(())
}

pub(super) fn validate_paths(patterns: &[String], field: &ConfigPath) -> Result<()> {
    validate_globs(patterns, &field.to_string(), |index| {
        field.clone().index(index)
    })
}

fn validate_sources(sources: &[RouteCoverageSource], base: &ConfigPath) -> Result<()> {
    for (index, source) in sources.iter().enumerate() {
        let path = base.clone().key("routeCoverageSources").index(index);
        if let Some(field) = source.empty_required_field() {
            let field = path.key(field);
            return Err(field_error(
                field.clone(),
                format!("{field} must not be empty"),
            ));
        }
        validate_paths(&source.include, &path.key("include"))?;
    }
    Ok(())
}

pub(super) fn validate_environment(
    environment: &TestPlanEnvironment,
    base: &ConfigPath,
) -> Result<()> {
    validate_paths(&environment.include, &base.clone().key("include"))?;
    validate_paths(&environment.exclude, &base.clone().key("exclude"))?;
    validate_limit(environment.limit.as_ref(), base)?;
    for (index, group) in environment.groups.iter().enumerate() {
        validate_limit(
            group.limit.as_ref(),
            &base.clone().key("groups").index(index),
        )?;
    }
    Ok(())
}

fn validate_limit(limit: Option<&TestPlanLimit>, base: &ConfigPath) -> Result<()> {
    let Some(percent) = limit.and_then(|limit| limit.percent.as_ref()) else {
        return Ok(());
    };
    if percent
        .value()
        .is_some_and(|value| (0.0..=100.0).contains(&value))
    {
        return Ok(());
    }
    let field = base.clone().key("limit").key("percent");
    Err(field_error(
        field.clone(),
        format!("{field}: invalid percentage {percent:?}; use a number from 0 to 100 or a percentage such as `25%`"),
    ))
}
