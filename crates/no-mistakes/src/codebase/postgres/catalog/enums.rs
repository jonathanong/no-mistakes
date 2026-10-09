//! Enum identity follows the catalog's schema-aware relation matching semantics.
use super::{names, CatalogEnum, SchemaCatalog};
impl SchemaCatalog {
    /// Enums owned by the selected schema, excluding external enum types retained only for
    /// classifying columns that directly use them.
    pub(crate) fn schema_enums(&self) -> impl Iterator<Item = &CatalogEnum> {
        self.enums.values().filter(|enum_type| {
            let Some(schema) = self.schema.as_deref() else {
                return true;
            };
            let parts = names::decoded_parts(&enum_type.name);
            parts.len() == 1 || parts[..parts.len() - 1].join(".") == schema
        })
    }

    pub(crate) fn enum_type(&self, name: &str) -> Option<&CatalogEnum> {
        let (qualifier, bare) = names::split_name(name);
        if let Some(entry) = self.enums.iter().find_map(|(key, entry)| {
            (names::split_key(key) == (qualifier.clone(), bare.clone())).then_some(entry)
        }) {
            return Some(entry);
        }
        if let (Some(qualifier), Some(own)) = (&qualifier, &self.schema) {
            if qualifier != own {
                return None;
            }
        }
        let mut matches = self.enums.iter().filter(|(key, _)| {
            let (key_qualifier, key_bare) = names::split_key(key);
            let key_is_local = self.schema.as_deref().is_none_or(|schema| {
                key_qualifier
                    .as_deref()
                    .is_none_or(|key_schema| key_schema == schema)
            });
            key_bare == bare
                && (qualifier.is_some() || key_is_local)
                && (qualifier.is_none() || key_qualifier.is_none())
        });
        let (_, entry) = matches.next()?;
        matches.next().is_none().then_some(entry)
    }
}
