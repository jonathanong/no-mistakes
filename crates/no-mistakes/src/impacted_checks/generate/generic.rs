use super::super::{CheckCommand, CheckKind};
use crate::config::v2::schema::{CheckFileArgs, NoMistakesConfig};
use crate::tests::Warning;
use anyhow::Result;
use globset::{Glob, GlobSet, GlobSetBuilder};
use std::collections::{BTreeMap, BTreeSet};

pub(in crate::impacted_checks) fn generic_checks(
    config: &NoMistakesConfig,
    changed_files: &[String],
    missing: &BTreeSet<String>,
) -> Result<Vec<CheckCommand>> {
    let mut out = Vec::new();
    for def in &config.checks.commands {
        let include = build_globset(&def.include)?;
        let exclude = build_globset(&def.exclude)?;
        let matched: Vec<String> = changed_files
            .iter()
            .filter(|file| {
                include.as_ref().is_some_and(|set| set.is_match(file))
                    && exclude.as_ref().is_none_or(|set| !set.is_match(file))
            })
            .cloned()
            .collect();
        if matched.is_empty() {
            continue;
        }
        let mut command = def.command.clone();
        let files = if def.file_args == CheckFileArgs::Append {
            // Append only surviving files — a per-file command (e.g. eslint) on a
            // deleted path would fail. Whole-project checks still trigger below.
            let appended: Vec<String> = matched
                .into_iter()
                .filter(|file| !missing.contains(file))
                .collect();
            if appended.is_empty() {
                continue;
            }
            command.extend(appended.iter().cloned());
            appended
        } else {
            matched
        };
        out.push(CheckCommand {
            name: def.name.clone(),
            kind: CheckKind::Generic,
            command,
            files,
        });
    }
    Ok(out)
}

fn build_globset(patterns: &[String]) -> Result<Option<GlobSet>> {
    if patterns.is_empty() {
        return Ok(None);
    }
    let mut builder = GlobSetBuilder::new();
    for pattern in patterns {
        let pattern = crate::codebase::glob_normalize::normalize(pattern);
        builder.add(Glob::new(&pattern)?);
    }
    Ok(Some(builder.build()?))
}

pub(in crate::impacted_checks) fn dedupe_checks(checks: Vec<CheckCommand>) -> Vec<CheckCommand> {
    let mut by_command: BTreeMap<Vec<String>, CheckCommand> = BTreeMap::new();
    for mut check in checks {
        match by_command.get_mut(&check.command) {
            Some(existing) => existing.files.append(&mut check.files),
            None => {
                by_command.insert(check.command.clone(), check);
            }
        }
    }
    let mut unique: Vec<CheckCommand> = by_command.into_values().collect();
    for check in &mut unique {
        check.files.sort();
        check.files.dedup();
    }
    unique.sort_by(|a, b| a.command.cmp(&b.command));
    unique
}

pub(in crate::impacted_checks) fn dedupe_warnings(warnings: Vec<Warning>) -> Vec<Warning> {
    let mut seen = BTreeSet::new();
    let mut unique: Vec<Warning> = warnings
        .into_iter()
        .filter(|warning| {
            seen.insert((
                warning.r#type.clone(),
                warning.file.clone(),
                warning.message.clone(),
            ))
        })
        .collect();
    unique.sort_by(|a, b| (&a.file, &a.message).cmp(&(&b.file, &b.message)));
    unique
}
