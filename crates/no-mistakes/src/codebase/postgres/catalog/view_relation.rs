use super::{fallback, names, CatalogTable, CatalogView, SchemaCatalog};

impl SchemaCatalog {
    pub(crate) fn bounded_view_relation(&self, name: &str) -> Option<&CatalogView> {
        self.view_relation(name)
    }

    /// A table identity safe for cardinality proof when a bare name could also be a view.
    pub(crate) fn bounded_relation(&self, name: &str) -> Option<&CatalogTable> {
        let (qualifier, bare) = names::split_name(name);
        let normalized = names::normalize_table_name(name);
        if qualifier.is_some() {
            if self.views.contains_key(&normalized)
                || (qualifier.as_ref() == self.schema.as_ref()
                    && self.views.keys().any(|key| {
                        let (schema, view) = names::split_key(key);
                        schema.is_none() && view == bare
                    }))
            {
                return None;
            }
            if let Some(table) = self.model_tables.get(&normalized) {
                return Some(table);
            }
            // A bare stored key has no known qualified identity without a declared schema.
            if qualifier.as_ref() != self.schema.as_ref() {
                return None;
            }
            return self.relation(name);
        }
        if self.views.keys().any(|key| names::split_key(key).1 == bare) {
            return None;
        }
        self.relation(name)
    }

    /// Resolve only one known view and never treat a colliding table as a view.
    pub fn view_relation(&self, name: &str) -> Option<&CatalogView> {
        let normalized = names::normalize_table_name(name);
        let (qualifier, bare) = names::split_name(name);
        if qualifier.is_none()
            && self
                .model_tables
                .keys()
                .any(|key| names::split_key(key).1 == bare)
        {
            return None;
        }
        if let Some(view) = self.views.get(&normalized) {
            let bare_table_in_schema = qualifier.as_ref() == self.schema.as_ref()
                && self.model_tables.keys().any(|key| {
                    let (schema, table) = names::split_key(key);
                    schema.is_none() && table == bare
                });
            return (!self.model_tables.contains_key(&normalized) && !bare_table_in_schema)
                .then_some(view);
        }
        if qualifier.is_some() && qualifier.as_ref() != self.schema.as_ref() {
            return None;
        }
        if self.relation(name).is_some() {
            return None;
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
