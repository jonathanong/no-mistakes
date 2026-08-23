use super::finding;
use super::paths::frameworks;
use crate::codebase::rules::path_filter::GlobMatcher;
use crate::codebase::rules::RuleFinding;
use crate::config::v2::NoMistakesConfig;
use anyhow::Result;
use std::collections::BTreeSet;

pub(super) fn lint(
    config: &NoMistakesConfig,
    tracked: &BTreeSet<String>,
    config_file: &str,
) -> Result<Vec<RuleFinding>> {
    let mut findings = Vec::new();
    for (framework, plan) in frameworks(config) {
        for (env_name, env) in &plan.environments {
            lint_patterns(
                &format!("testPlan.{framework}.environments.{env_name}.include"),
                &env.include,
                tracked,
                config_file,
                &mut findings,
            )?;
            lint_patterns(
                &format!("testPlan.{framework}.environments.{env_name}.exclude"),
                &env.exclude,
                tracked,
                config_file,
                &mut findings,
            )?;
        }
    }
    for (name, project) in &config.projects {
        lint_patterns(
            &format!("projects.{name}.include"),
            &project.include,
            tracked,
            config_file,
            &mut findings,
        )?;
        lint_patterns(
            &format!("projects.{name}.exclude"),
            &project.exclude,
            tracked,
            config_file,
            &mut findings,
        )?;
    }
    for (index, rule) in config.rules.iter().enumerate() {
        lint_patterns(
            &format!("rules[{index}].include"),
            &rule.include,
            tracked,
            config_file,
            &mut findings,
        )?;
        lint_patterns(
            &format!("rules[{index}].exclude"),
            &rule.exclude,
            tracked,
            config_file,
            &mut findings,
        )?;
    }
    Ok(findings)
}

fn lint_patterns(
    field: &str,
    patterns: &[String],
    tracked: &BTreeSet<String>,
    config_file: &str,
    findings: &mut Vec<RuleFinding>,
) -> Result<()> {
    for (index, pattern) in patterns.iter().enumerate() {
        if pattern.trim().is_empty() || !looks_like_glob(pattern) {
            continue;
        }
        let matcher = GlobMatcher::new(std::slice::from_ref(pattern), field)?;
        if !tracked.iter().any(|rel| matcher.is_match(rel)) {
            findings.push(finding(
                config_file,
                format!("{field}[{index}]: glob `{pattern}` matches no tracked files"),
            ));
        }
    }
    Ok(())
}

fn looks_like_glob(pattern: &str) -> bool {
    pattern.contains('*') || pattern.contains('?') || pattern.contains('{')
}
