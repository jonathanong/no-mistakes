use super::CheckFactMap;
use std::path::PathBuf;

impl CheckFactMap {
    pub(crate) fn query_annotation_prefix(
        &self,
        path: &std::path::Path,
        options: &crate::codebase::postgres::EmbeddedSqlOptions,
        span: (u32, u32),
    ) -> Option<&Option<String>> {
        self.ts
            .get(path)?
            .query_annotation
            .iter()
            .find(|(profile, _)| profile == options)?
            .1
            .calls
            .get(&span)
    }
    #[doc(hidden)]
    pub fn postgres_schema_file(
        &self,
        path: &std::path::Path,
    ) -> anyhow::Result<&crate::codebase::postgres::SqlSchemaFileFacts> {
        self.postgres.schema(path)
    }
    pub(crate) fn postgres_statements(
        &self,
        path: &std::path::Path,
        profile: Option<&crate::codebase::postgres::EmbeddedSqlOptions>,
    ) -> anyhow::Result<&[crate::codebase::postgres::SqlStatementFileFacts]> {
        self.postgres.statements(path, profile)
    }
    pub fn files(&self) -> &[PathBuf] {
        &self.files
    }

    pub(crate) fn embedded_sql(
        &self,
        path: &std::path::Path,
        options: &crate::codebase::postgres::EmbeddedSqlOptions,
    ) -> anyhow::Result<&crate::codebase::postgres::EmbeddedSqlFileFacts> {
        let file = self
            .ts
            .get(path)
            .ok_or_else(|| anyhow::anyhow!("prepared facts are missing for {}", path.display()))?;
        file.embedded_sql
            .iter()
            .find_map(|(key, facts)| (key == options).then_some(facts))
            .ok_or_else(|| {
                file.parse_error.as_ref().map_or_else(
                    || anyhow::anyhow!("prepared embedded SQL projection is missing"),
                    |error| anyhow::anyhow!(error.clone()),
                )
            })
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
