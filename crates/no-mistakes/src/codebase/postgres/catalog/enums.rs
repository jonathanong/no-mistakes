//! Enum identity follows the catalog's schema-aware relation matching semantics.
use super::{names, CatalogEnum, SchemaCatalog};
impl SchemaCatalog {
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
            key_bare == bare && (qualifier.is_none() || key_qualifier.is_none())
        });
        let (_, entry) = matches.next()?;
        matches.next().is_none().then_some(entry)
    }
}
