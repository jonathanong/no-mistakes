use super::CheckFactMap;

impl CheckFactMap {
    pub(crate) fn postgres_schema_catalog(
        &self,
        path: &str,
    ) -> anyhow::Result<&crate::codebase::postgres::SchemaCatalog> {
        let catalog = self.postgres_ordering_catalog(path)?;
        if catalog.coverage() != crate::codebase::postgres::CatalogCoverage::Complete {
            anyhow::bail!("schemaCatalogPath {path} has ordering-only coverage; this rule requires a complete schema catalog");
        }
        Ok(catalog)
    }

    pub(crate) fn postgres_ordering_catalog(
        &self,
        path: &str,
    ) -> anyhow::Result<&crate::codebase::postgres::SchemaCatalog> {
        let normalized = crate::codebase::postgres::normalize_schema_catalog_path(path)?
            .to_string_lossy()
            .into_owned();
        match self.postgres_schema_catalogs.get(&normalized) {
            Some(Ok(catalog)) => Ok(catalog),
            Some(Err(error)) => Err(anyhow::anyhow!(error.to_string())),
            None => Err(anyhow::anyhow!(
                "prepared schema catalog is missing for schemaCatalogPath {path}"
            )),
        }
    }
}
