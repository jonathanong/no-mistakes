use super::paths::frameworks;
use super::{context::Context, field::Field};
use crate::codebase::rules::path_filter::GlobMatcher;
use crate::codebase::rules::RuleFinding;
use crate::config::v2::NoMistakesConfig;
use anyhow::Result;
use std::collections::BTreeSet;

pub(super) fn lint(
    config: &NoMistakesConfig,
    tracked: &BTreeSet<String>,
    context: &Context<'_>,
) -> Result<Vec<RuleFinding>> {
    let mut findings = Vec::new();
    for (framework, plan) in frameworks(config) {
        for (env_name, env) in &plan.environments {
            lint_patterns(
                &context
                    .field("testPlan")
                    .key(framework)
                    .key("environments")
                    .key(env_name)
                    .key("include"),
                &env.include,
                tracked,
                context,
                None,
                &mut findings,
            )?;
            lint_patterns(
                &context
                    .field("testPlan")
                    .key(framework)
                    .key("environments")
                    .key(env_name)
                    .key("exclude"),
                &env.exclude,
                tracked,
                context,
                None,
                &mut findings,
            )?;
        }
    }
    for (name, project) in &config.projects {
        lint_patterns(
            &context.field("projects").key(name).key("include"),
            &project.include,
            tracked,
            context,
            project.root.as_deref(),
            &mut findings,
        )?;
        lint_patterns(
            &context.field("projects").key(name).key("exclude"),
            &project.exclude,
            tracked,
            context,
            project.root.as_deref(),
            &mut findings,
        )?;
    }
    for (index, rule) in config.rules.iter().enumerate() {
        lint_patterns(
            &context.field("rules").index(index).key("include"),
            &rule.include,
            tracked,
            context,
            None,
            &mut findings,
        )?;
        lint_patterns(
            &context.field("rules").index(index).key("exclude"),
            &rule.exclude,
            tracked,
            context,
            None,
            &mut findings,
        )?;
    }
    Ok(findings)
}

fn lint_patterns(
    field: &Field,
    patterns: &[String],
    tracked: &BTreeSet<String>,
    context: &Context<'_>,
    project_root: Option<&str>,
    findings: &mut Vec<RuleFinding>,
) -> Result<()> {
    for (index, pattern) in patterns.iter().enumerate() {
        if pattern.trim().is_empty()
            || pattern.trim_start().starts_with('!')
            || !looks_like_glob(pattern)
        {
            continue;
        }
        let effective_pattern = project_root
            .filter(|root| !root.is_empty() && *root != ".")
            .map(|root| {
                format!(
                    "{}/{}",
                    root.trim_matches('/'),
                    pattern.trim_start_matches("./")
                )
            })
            .unwrap_or_else(|| pattern.clone());
        let matcher = GlobMatcher::new(std::slice::from_ref(&effective_pattern), &field.label)
            .map_err(|error| {
                let finding = context.finding(&field.index(index), format!(": {error}"));
                anyhow::anyhow!("{}:{}: {}", finding.file, finding.line, finding.message)
            })?;
        if !tracked.iter().any(|rel| matcher.is_match(rel)) {
            findings.push(context.finding(
                &field.index(index),
                format!(": glob `{pattern}` matches no tracked files"),
            ));
        }
    }
    Ok(())
}

fn looks_like_glob(pattern: &str) -> bool {
    pattern.contains('*') || pattern.contains('?') || pattern.contains('{')
}
