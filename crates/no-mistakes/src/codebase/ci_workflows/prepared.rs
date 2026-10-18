//! Request-owned CI documents and their lazy canonical topology projection.
use super::ParsedWorkflowSet;
use crate::codebase::ts_source::{SourceStore, VisiblePathSnapshot};
use crate::codebase::workflow_topology::{self, model::WorkflowTopology};
use crate::config::v2::{schema::CiConfig, NoMistakesConfig};
use std::path::{Path, PathBuf};
use std::sync::{Arc, OnceLock};

#[doc(hidden)]
pub struct PreparedWorkflowTopologyInputs {
    pub workflow_documents: Arc<ParsedWorkflowSet>,
    pub actions: ParsedWorkflowSet,
    root: PathBuf,
    ci: CiConfig,
    topology: OnceLock<WorkflowTopology>,
    secret_documents: OnceLock<super::secret_facts::SecretDocuments>,
    inventory_locations:
        OnceLock<dashmap::DashMap<PathBuf, Option<super::secret_facts::InventoryLocations>>>,
}

impl PreparedWorkflowTopologyInputs {
    /// Declare CI document demand once at the public request boundary.
    pub fn for_config(
        root: &Path,
        config: &NoMistakesConfig,
        snapshot: &VisiblePathSnapshot,
        sources: &SourceStore,
    ) -> Option<Self> {
        use crate::codebase::rules::{
            FORBIDDEN_DEPENDENCIES, TSCONFIG_GATE_COVERAGE, VITEST_CI_PATH_COVERAGE,
            WORKFLOW_TOPOLOGY_POLICY,
        };
        [
            FORBIDDEN_DEPENDENCIES,
            TSCONFIG_GATE_COVERAGE,
            VITEST_CI_PATH_COVERAGE,
            WORKFLOW_TOPOLOGY_POLICY,
        ]
        .iter()
        .any(|id| config.rule_configured(id))
        .then(|| {
            let workflows = Arc::new(ParsedWorkflowSet::load_from_snapshot_and_sources(
                root, &config.ci, snapshot, sources,
            ));
            let inventory = snapshot.paths_for(root);
            let actions_requested = config
                .rule_applications(WORKFLOW_TOPOLOGY_POLICY)
                .iter()
                .any(|rule| {
                    rule.options
                        .get("secretInventory")
                        .is_some_and(|value| !value.is_null())
                });
            let actions = if actions_requested {
                ParsedWorkflowSet::from_paths_and_sources(
                    root,
                    inventory
                        .iter()
                        .filter(|path| {
                            path.file_name()
                                .is_some_and(|name| name == "action.yml" || name == "action.yaml")
                        })
                        .cloned(),
                    sources,
                )
            } else {
                ParsedWorkflowSet {
                    documents: Vec::new(),
                }
            };
            Self::new(root, &config.ci, workflows, actions)
        })
    }

    pub fn new(
        root: &Path,
        ci: &CiConfig,
        workflow_documents: Arc<ParsedWorkflowSet>,
        actions: ParsedWorkflowSet,
    ) -> Self {
        Self {
            workflow_documents,
            actions,
            root: crate::codebase::ts_resolver::normalize_path(root),
            ci: ci.clone(),
            topology: OnceLock::new(),
            secret_documents: OnceLock::new(),
            inventory_locations: OnceLock::new(),
        }
    }

    /// Legacy document-only consumers do not construct topology.
    pub fn topology(&self) -> &WorkflowTopology {
        self.topology.get_or_init(|| {
            workflow_topology::load_workflow_topology_from_parsed(
                &self.root,
                &self.ci,
                &self.workflow_documents,
                &[],
            )
        })
    }

    pub(crate) fn secret_documents(
        &self,
        sources: &SourceStore,
    ) -> &super::secret_facts::SecretDocuments {
        self.secret_documents.get_or_init(|| {
            super::secret_facts::collect(
                &self.root,
                &self.workflow_documents,
                &self.actions,
                sources,
            )
        })
    }

    pub(crate) fn matches_projection(
        &self,
        root: &Path,
        ci: &CiConfig,
        projected: &ParsedWorkflowSet,
    ) -> bool {
        self.root == crate::codebase::ts_resolver::normalize_path(root)
            && self.ci.workflow_dirs == ci.workflow_dirs
            && self
                .workflow_documents
                .documents
                .iter()
                .map(|doc| &doc.path)
                .eq(projected.documents.iter().map(|doc| &doc.path))
    }

    pub(crate) fn inventory_locations(
        &self,
        path: &Path,
        sources: &SourceStore,
    ) -> impl std::ops::Deref<Target = Option<super::secret_facts::InventoryLocations>> + '_ {
        let path = crate::codebase::ts_resolver::normalize_path(path);
        self.inventory_locations
            .get_or_init(dashmap::DashMap::new)
            .entry(path.clone())
            .or_insert_with(|| {
                sources
                    .read_path(&path)
                    .ok()
                    .map(|source| super::secret_facts::InventoryLocations::parse(&source))
            })
            .downgrade()
    }
}

impl std::ops::Deref for PreparedWorkflowTopologyInputs {
    type Target = ParsedWorkflowSet;
    fn deref(&self) -> &Self::Target {
        &self.workflow_documents
    }
}

/// A legacy graph caller may supply documents without a prepared topology owner.
#[derive(Clone, Copy)]
#[doc(hidden)]
pub enum PreparedWorkflowDocuments<'a> {
    Documents(&'a ParsedWorkflowSet),
    Topology(&'a PreparedWorkflowTopologyInputs),
}

impl<'a> PreparedWorkflowDocuments<'a> {
    pub(crate) fn documents(self) -> &'a ParsedWorkflowSet {
        match self {
            Self::Documents(documents) => documents,
            Self::Topology(inputs) => &inputs.workflow_documents,
        }
    }
}

include!("prepared_projection.rs");

#[cfg(test)]
#[path = "prepared_tests.rs"]
mod tests;
