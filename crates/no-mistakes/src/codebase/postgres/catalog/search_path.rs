//! Ordered search-path resolution from explicitly requested catalog inventory.
use super::SchemaCatalog;
use crate::codebase::postgres::decoded_parts;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum SearchPathResolution {
    Temporary,
    Physical(String),
    Unknown,
}

impl SchemaCatalog {
    pub(crate) fn resolve_before_temp(
        &self,
        earlier: &[String],
        name: &str,
    ) -> SearchPathResolution {
        let bare = decoded_parts(name).last().cloned().unwrap_or_default();
        for schema in earlier {
            if schema == "$user" {
                return SearchPathResolution::Unknown;
            }
            let Some(Some(relations)) = self.search_path_evidence.get(schema) else {
                return SearchPathResolution::Unknown;
            };
            if relations.contains(&bare) {
                return SearchPathResolution::Physical(schema.clone());
            }
        }
        SearchPathResolution::Temporary
    }
}
