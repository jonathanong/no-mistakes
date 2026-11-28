use super::RULE_ID;
use crate::codebase::rules::validate_options::{
    finish, format_invalid_option, indexed_rule_applications, push_application_errors,
};
use crate::codebase::workspace_resolution::IndexedWorkspaceMap;
use crate::config::v2::{schema::RuleDef, NoMistakesConfig};
use anyhow::{anyhow, Result};
use globset::{Glob, GlobMatcher};
use serde::Deserialize;
use std::path::Path;

#[derive(Default, Deserialize)]
#[serde(deny_unknown_fields)]
struct Options {
    #[serde(default)]
    allow: Vec<AllowEntry>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct AllowEntry {
    importer: String,
    target: String,
}

pub(super) struct PackagePair {
    importer: GlobMatcher,
    target: GlobMatcher,
}

impl PackagePair {
    pub(super) fn matches(&self, importer: &str, target: &str) -> bool {
        self.importer.is_match(importer) && self.target.is_match(target)
    }
}

pub(super) fn compile(rule: &RuleDef, workspace: &IndexedWorkspaceMap) -> Result<Vec<PackagePair>> {
    let options: Options = rule.try_rule_options()?;
    options
        .allow
        .iter()
        .enumerate()
        .map(|(index, entry)| {
            Ok(PackagePair {
                importer: package_glob(&entry.importer, index, "importer", workspace)?,
                target: package_glob(&entry.target, index, "target", workspace)?,
            })
        })
        .collect()
}

fn package_glob(
    pattern: &str,
    index: usize,
    field: &str,
    workspace: &IndexedWorkspaceMap,
) -> Result<GlobMatcher> {
    let invalid = |expected| {
        anyhow!(format_invalid_option(
            RULE_ID,
            &format!("allow[{index}].{field}"),
            pattern,
            expected
        ))
    };
    if pattern.trim().is_empty() {
        return Err(invalid("a nonempty workspace package name glob"));
    }
    let matcher = Glob::new(pattern)
        .map_err(|_| invalid("a valid workspace package name glob"))?
        .compile_matcher();
    if !workspace.packages.iter().any(|pkg| matcher.is_match(&pkg.name)) {
        return Err(invalid("a glob matching at least one declared workspace package name"));
    }
    Ok(matcher)
}

#[derive(Default, Deserialize)]
#[serde(deny_unknown_fields)]
struct DeclaredShape {
    #[serde(default)]
    allow: Vec<serde_yaml::Value>,
}

pub(in crate::codebase::rules) fn accept_declared_options(rule: &RuleDef) -> Result<()> {
    // `allow` is known here. `compile` checks each entry so those errors stay unstamped.
    let shape = rule.strict_rule_options::<DeclaredShape>()?;
    drop(shape.allow);
    Ok(())
}

pub(in crate::codebase::rules) fn validate(
    root: &Path,
    config_path: Option<&Path>,
    config: &NoMistakesConfig,
    workspace: &IndexedWorkspaceMap,
) -> Result<()> {
    let mut errors = Vec::new();
    for (index, rule) in indexed_rule_applications(config, RULE_ID) {
        if let Err(error) = compile(rule, workspace) {
            push_application_errors(
                root,
                config,
                index,
                config_path,
                [format!("{error:#}")].into(),
                &mut errors,
            );
        }
    }
    finish(errors)
}
