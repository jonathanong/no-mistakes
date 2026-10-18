use super::CompiledInventory;
use crate::codebase::ci_workflows::PreparedWorkflowTopologyInputs;
use crate::codebase::rules::RuleFinding;
use crate::codebase::ts_source::SourceStore;
use std::collections::BTreeSet;
use std::path::Path;

pub(super) fn finding(path: &str, line: usize, message: String) -> RuleFinding {
    RuleFinding {
        file: path.to_owned(),
        line,
        message,
        ..super::super::finding(String::new())
    }
}

pub(super) fn incomplete(findings: &mut Vec<RuleFinding>, path: &str, line: usize, reason: &str) {
    findings.push(finding(
        path,
        line,
        format!(
            "secret inventory analysis incomplete: {reason}; declarations cannot be proven unused"
        ),
    ));
}

pub(super) fn unused(
    root: &Path,
    inputs: &PreparedWorkflowTopologyInputs,
    config_path: Option<&Path>,
    sources: &SourceStore,
    inventory: &CompiledInventory,
    used: &BTreeSet<(String, String)>,
    findings: &mut Vec<RuleFinding>,
) {
    let Some(config_path) = config_path else {
        incomplete(
            findings,
            ".",
            1,
            "configuration path unavailable for unused declarations",
        );
        return;
    };
    let absolute = if config_path.is_absolute() {
        config_path.to_owned()
    } else {
        root.join(config_path)
    };
    let path = crate::codebase::ci_graph::relative_slash(root, &absolute);
    let cached_locations = inputs.inventory_locations(&absolute, sources);
    let Some(locations) = cached_locations.as_ref() else {
        incomplete(
            findings,
            &path,
            1,
            "configuration source unavailable for unused declarations",
        );
        return;
    };
    let repository = inventory
        .repository
        .iter()
        .map(|(key, name)| ("", "repository", key, name));
    let environments = inventory
        .environments
        .iter()
        .flat_map(|(key, (name, values))| {
            values
                .iter()
                .map(move |(secret, original)| (key.as_str(), name.as_str(), secret, original))
        });
    let unused: Vec<_> = repository
        .chain(environments)
        .filter(|(environment, _, key, _)| {
            !used.contains(&(environment.to_string(), (*key).clone()))
        })
        .collect();
    let lines: Option<Vec<_>> = unused
        .iter()
        .map(|(environment, _, key, _)| locations.line(environment, key))
        .collect();
    let Some(lines) = lines else {
        incomplete(
            findings,
            &path,
            1,
            "unused declaration location is unavailable or ambiguous",
        );
        return;
    };
    for ((_, scope, _, name), line) in unused.into_iter().zip(lines) {
        findings.push(finding(&path, line, format!("unused declared secret {name} in {scope}; remove its name or add a statically resolved reference")));
    }
}
