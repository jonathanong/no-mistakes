use super::Selectors;
use crate::codebase::dependencies::graph::{DepGraph, NodeId};
use crate::codebase::rules::allow_audit::{stale_finding, CheckedScope};
use crate::codebase::rules::config_diagnostic::ConfigAnchor;
use crate::codebase::rules::forbidden_dependencies::Options;
use crate::codebase::rules::RuleFinding;
use crate::codebase::ts_source::SourceStore;
use std::path::PathBuf;

impl Selectors<'_> {
    pub(crate) fn audit(
        &self,
        anchor: &ConfigAnchor,
        opts: &Options,
        graph: &DepGraph,
        sources: Option<&SourceStore>,
        universe: Option<&crate::fx::PathSet>,
        scope: (
            &[PathBuf],
            Option<&crate::codebase::rules::allow_audit::MissingFiles>,
        ),
    ) -> Vec<RuleFinding> {
        let (checked, missing) = scope;
        let Some(sources) = sources else {
            return Vec::new();
        };
        if graph.parse_errors().next().is_some() || !graph.resource_diagnostics().is_empty() {
            return Vec::new();
        }
        let target_files = graph
            .all_files()
            .filter_map(NodeId::as_file)
            .filter(|path| universe.is_none_or(|files| files.contains(*path)))
            .map(PathBuf::from)
            .collect::<Vec<_>>();
        let targets = CheckedScope::new(self.root, Some(sources), &target_files);
        let source_scope = CheckedScope::new(self.root, Some(sources), checked);
        let mut findings = Vec::new();
        let inventory = sources.inventory().paths();
        for (index, exception) in self.exceptions.iter().enumerate() {
            let source_files = inventory
                .iter()
                .filter(|path| {
                    exception
                        .roots
                        .iter()
                        .any(|selector| selector.matches(self.root, path))
                })
                .collect::<Vec<_>>();
            let complete = !source_files.is_empty()
                && source_files.iter().all(|path| {
                    source_scope.exact(&crate::codebase::ts_source::relative_slash_path(
                        self.root, path,
                    ))
                });
            for item in exception.usage.unused() {
                let pattern = &opts.exceptions[index].allowed_files[item];
                if !(complete && targets.glob(pattern))
                    && !missing.is_some_and(|scope| scope.contains(pattern))
                {
                    continue;
                }
                findings.push(stale_finding(
                    anchor,
                    super::super::RULE_ID,
                    &format!("exceptions[{index}].allowedFiles"),
                    pattern.clone(),
                    anchor.nested_option_item_line("exceptions", index, "allowedFiles", item),
                ));
            }
        }
        findings
    }
}
