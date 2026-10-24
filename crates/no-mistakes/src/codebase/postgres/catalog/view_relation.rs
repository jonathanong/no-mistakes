use super::{fallback, names, CatalogView, SchemaCatalog};

impl SchemaCatalog {
    /// Resolve only one known view and never treat a colliding table as a view.
    pub fn view_relation(&self, name: &str) -> Option<&CatalogView> {
        if self.relation(name).is_some() {
            return None;
        }
        let normalized = names::normalize_table_name(name);
        if let Some(view) = self.views.get(&normalized) {
            return Some(view);
        }
        let (qualifier, bare) = names::split_name(name);
        if let (Some(qualifier), Some(own)) = (&qualifier, &self.schema) {
            if qualifier != own {
                return None;
            }
        }
        let fallback = self
            .view_fallback
            .get_or_init(|| fallback::build(&self.views));
        let key = if qualifier.is_some() {
            fallback.bare_keys.get(&bare)?
        } else {
            fallback.unique.get(&bare)?.as_ref()?
        };
        self.views.get(key)
    }
}
