//! Ordered search-path resolution from explicitly requested catalog inventory.
use super::SchemaCatalog;
use crate::codebase::postgres::decoded_parts;

pub(crate) use no_mistakes_postgres_sql::SearchPathResolution;

impl no_mistakes_postgres_sql::CatalogLifecycleLookup for SchemaCatalog {
    fn current_database(&self) -> Option<&str> {
        SchemaCatalog::current_database(self)
    }

    fn resolve_before_temp(&self, earlier: &[String], name: &str) -> SearchPathResolution {
        SchemaCatalog::resolve_before_temp(self, earlier, name)
    }

    fn scalar_function_names(&self) -> std::sync::Arc<std::collections::BTreeSet<Vec<String>>> {
        SchemaCatalog::scalar_function_names(self)
    }
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
    /// Whether explicit evidence proves an earlier search-path entry or pg_temp hides the
    /// selected catalog relation. Unknown entries preserve the conservative catalog check.
    pub(crate) fn hides_selected_relation(&self, earlier: &[String], name: &str) -> bool {
        let bare = super::decoded_parts(name)
            .last()
            .cloned()
            .unwrap_or_default();
        for schema in earlier {
            if schema == "$user" {
                return false;
            }
            let Some(relations) = self.search_path_evidence.get(schema) else {
                return false;
            };
            if let Some(relations) = relations {
                if relations.contains(&bare) {
                    return self.schema.as_deref() != Some(schema) || self.relation(name).is_none();
                }
            }
        }
        true
    }
}
