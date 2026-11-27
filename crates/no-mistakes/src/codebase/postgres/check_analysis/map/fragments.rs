use crate::codebase::check_facts::CheckFactMap;

impl CheckFactMap {
    pub(crate) fn postgres_fragments(
        &self,
        path: &std::path::Path,
        profile: &crate::codebase::postgres::EmbeddedSqlOptions,
    ) -> anyhow::Result<&[crate::codebase::postgres::prepared::PreparedSqlFragment]> {
        self.postgres().fragments(path, profile)
    }
}
