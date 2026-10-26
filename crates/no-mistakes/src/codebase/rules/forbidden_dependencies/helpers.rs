use super::{Options, RuleFinding, RULE_ID};
use crate::codebase::dependencies::graph::NodeId;
use crate::codebase::dependencies::{EdgeKind, RelationshipArg};
use crate::codebase::rules::config_diagnostic::ConfigAnchor;
use anyhow::Result;
use globset::{Glob, GlobSet, GlobSetBuilder};

pub(super) fn config_matchers(
    anchor: &ConfigAnchor,
    opts: &Options,
) -> Result<(Option<GlobSet>, Option<GlobSet>), Vec<RuleFinding>> {
    if opts.roots.is_empty()
        || (opts.forbidden_modules.is_empty() && opts.forbidden_files.is_empty())
    {
        let key = if opts.roots.is_empty() {
            "roots"
        } else if opts.forbidden_modules.is_empty() {
            "forbiddenModules"
        } else {
            "forbiddenFiles"
        };
        return Err(vec![config_emit(
            anchor,
            anchor.option_line(key),
            format!(
                "{RULE_ID}: each rule entry requires at least one root and \
                 one forbiddenModules or forbiddenFiles entry"
            ),
        )]);
    }
    let modules = match build_globset(&opts.forbidden_modules) {
        Ok(matcher) => matcher,
        Err(error) => {
            return Err(vec![config_emit(
                anchor,
                anchor.invalid_glob_line("forbiddenModules", &opts.forbidden_modules),
                format!("{RULE_ID}: invalid glob pattern in forbiddenModules: {error}"),
            )]);
        }
    };
    let files = match build_globset(&opts.forbidden_files) {
        Ok(matcher) => matcher,
        Err(error) => {
            return Err(vec![config_emit(
                anchor,
                anchor.invalid_glob_line("forbiddenFiles", &opts.forbidden_files),
                format!("{RULE_ID}: invalid glob pattern in forbiddenFiles: {error}"),
            )]);
        }
    };
    Ok((modules, files))
}

fn config_emit(anchor: &ConfigAnchor, line: usize, message: String) -> RuleFinding {
    anchor.emit(RULE_ID, line, message, None, None)
}

pub(super) fn build_globset(patterns: &[String]) -> Result<Option<GlobSet>> {
    if patterns.is_empty() {
        return Ok(None);
    }
    let mut builder = GlobSetBuilder::new();
    for p in patterns {
        builder.add(Glob::new(p)?);
    }
    Ok(Some(builder.build()?))
}

pub(super) fn edge_kind_str(k: &EdgeKind) -> String {
    k.as_str().to_string()
}

pub(super) fn repro_command(
    root_str: &str,
    target_name: &str,
    node: &NodeId,
    relationships: &[RelationshipArg],
) -> String {
    let target_flag = match node {
        NodeId::Module(_) => format!("--target-module '{}'", target_name.replace('\'', "'\\''")),
        _ => format!("--filter '{}'", target_name.replace('\'', "'\\''")),
    };
    let rel_flags = if relationships.is_empty() {
        " --relationship all".to_string()
    } else {
        relationships
            .iter()
            .map(|r| format!(" --relationship {}", r.as_str()))
            .collect::<String>()
    };
    format!(
        "no-mistakes dependencies '{}' {target_flag}{rel_flags} --format json",
        root_str.replace('\'', "'\\''")
    )
}
