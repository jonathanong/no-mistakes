use super::*;
use anyhow::{bail, ensure};
use serde::Deserialize;

#[derive(Default, Deserialize)]
#[serde(default, rename_all = "camelCase", deny_unknown_fields)]
pub(super) struct Options {
    pub producers: Vec<Function>,
    pub assertions: Vec<Import>,
    pub ownership: Vec<Ownership>,
    pub max_depth: Option<usize>,
    pub max_nodes: Option<usize>,
}
#[derive(Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Function {
    pub file: String,
    pub symbol: String,
}
#[derive(Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Import {
    pub module: String,
    pub export: String,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Ownership {
    pub producer: Function,
    pub owner: Function,
    pub via: Import,
}

impl Options {
    pub fn validate(&self) -> Result<()> {
        ensure!(
            !self.producers.is_empty(),
            "{RULE_ID}: producers must not be empty"
        );
        ensure!(
            !self.assertions.is_empty(),
            "{RULE_ID}: assertions must not be empty"
        );
        ensure!(
            self.max_depth != Some(0) && self.max_nodes != Some(0),
            "{RULE_ID}: maxDepth and maxNodes must be positive"
        );
        for function in self.producers.iter().chain(
            self.ownership
                .iter()
                .flat_map(|contract| [&contract.producer, &contract.owner]),
        ) {
            ensure!(
                !function.file.trim().is_empty() && !function.symbol.trim().is_empty(),
                "{RULE_ID}: function selectors require non-empty file and symbol"
            );
        }
        for import in self
            .assertions
            .iter()
            .chain(self.ownership.iter().map(|contract| &contract.via))
        {
            ensure!(
                !import.module.trim().is_empty() && !import.export.trim().is_empty(),
                "{RULE_ID}: import selectors require non-empty module and export"
            );
        }
        Ok(())
    }
}
impl Function {
    pub fn resolve(
        &self,
        root: &Path,
        graph: &DepGraph,
        facts: &dyn TsFactLookup,
    ) -> Result<NodeId> {
        let file = crate::codebase::ts_resolver::normalize_path(&root.join(&self.file));
        // A known collection failure cannot validate or reject a symbol selector.
        // Keep its diagnostic identity without inventing any canonical edge.
        if facts.get_ts_facts(&file).is_some_and(failed) {
            return Ok(NodeId::symbol(&file, self.symbol.as_str()));
        }
        let nodes = graph.expand_call_roots(&[CallRoot::Function {
            file,
            symbol: self.symbol.clone(),
        }]);
        if nodes.len() != 1 {
            bail!(
                "{RULE_ID}: function `{}#{}` must select exactly one prepared callable",
                self.file,
                self.symbol
            );
        }
        Ok(nodes.into_iter().next().expect("one callable"))
    }
}
impl Import {
    pub fn matches(&self, site: &ResolvedCallSite) -> bool {
        !site.has_unknown_target() && site.targets().len() > 0 && site.targets().all(|candidate| matches!(candidate.target,
            KnownCallTargetRef::ModuleExport { specifier, export_path, .. } if specifier == self.module && export_path == self.export))
    }
}
