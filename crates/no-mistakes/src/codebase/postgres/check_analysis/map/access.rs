use crate::codebase::check_facts::CheckFactMap;

impl CheckFactMap {
    pub(crate) fn query_annotation_prefix(
        &self,
        path: &std::path::Path,
        options: &crate::codebase::postgres::EmbeddedSqlOptions,
        span: (u32, u32),
    ) -> Option<&Option<String>> {
        self.ts
            .get(path)?
            .postgres()
            .query_annotation
            .iter()
            .find(|(profile, _)| profile == options)?
            .1
            .calls
            .get(&span)
    }
    pub(crate) fn proven_query_annotation_prefix(
        &self,
        path: &std::path::Path,
        options: &crate::codebase::postgres::EmbeddedSqlOptions,
        span: (u32, u32),
    ) -> Option<&Option<String>> {
        let facts = &self
            .ts
            .get(path)?
            .postgres()
            .query_annotation
            .iter()
            .find(|(profile, _)| profile == options)?
            .1;
        facts
            .proven_call_prefixes
            .contains(&span)
            .then(|| facts.calls.get(&span))
            .flatten()
    }
    #[doc(hidden)]
    pub fn postgres_schema_file(
        &self,
        path: &std::path::Path,
    ) -> anyhow::Result<&crate::codebase::postgres::SqlSchemaFileFacts> {
        self.postgres().schema(path)
    }
    pub(crate) fn postgres_statements(
        &self,
        path: &std::path::Path,
        profile: Option<&crate::codebase::postgres::EmbeddedSqlOptions>,
    ) -> anyhow::Result<&[crate::codebase::postgres::SqlStatementFileFacts]> {
        self.postgres().statements(path, profile)
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
        file.postgres()
            .embedded_sql
            .iter()
            .find_map(|(key, facts)| (key == options).then_some(facts))
            .ok_or_else(|| {
                file.parse_error.as_ref().map_or_else(
                    || anyhow::anyhow!("prepared embedded SQL projection is missing"),
                    |error| anyhow::anyhow!(error.clone()),
                )
            })
    }
}
