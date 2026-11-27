use super::CheckFactMap;
use std::path::PathBuf;

impl CheckFactMap {
    pub fn files(&self) -> &[PathBuf] {
        &self.files
    }

    pub(crate) fn graph_file_universe(&self) -> &[PathBuf] {
        if self.graph_files_complete {
            self.graph_files.as_slice()
        } else {
            self.files.as_slice()
        }
    }

    pub(crate) fn graph_plan(&self) -> crate::codebase::ts_source::facts::TsFactPlan {
        self.graph_plan
    }

    pub(crate) fn graph_view_with_supplemental(&self, supplemental: &Self) -> Self {
        let mut graph_files = self.graph_files.clone();
        graph_files.extend(supplemental.ts.keys().cloned());
        graph_files.sort();
        graph_files.dedup();
        self.view_with_supplemental(supplemental, graph_files)
    }

    /// Add prepared facts without admitting their source paths to the primary
    /// file or graph universes.
    ///
    /// This is used for configured, out-of-scope fact consumers such as
    /// finite-set call sources. They need their facts to be addressable, but
    /// must not widen filesystem-scoped checks or canonical graph traversal.
    #[doc(hidden)]
    pub fn fact_view_with_supplemental(&self, supplemental: &Self) -> Self {
        self.view_with_supplemental(supplemental, self.graph_files.clone())
    }
    pub(crate) fn with_graph_file_universe(&self, graph_files: Vec<PathBuf>) -> Self {
        self.view_with_supplemental(&Self::default(), graph_files)
    }
}

impl CheckFactMap {
    pub(crate) fn lint_suppressions(
        &self,
        path: &std::path::Path,
    ) -> Option<&[crate::codebase::rules::forbidden_lint_suppressions::Suppression]> {
        self.ts.get(path)?.lint_suppressions.as_deref()
    }
}
