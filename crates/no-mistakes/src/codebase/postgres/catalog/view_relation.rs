use super::{fallback, names, CatalogTable, CatalogView, SchemaCatalog};

impl SchemaCatalog {
    pub(crate) fn bounded_view_relation(&self, name: &str) -> Option<&CatalogView> {
        self.view_relation(name)
    }

    /// A table identity safe for cardinality proof when a bare name could also be a view.
    pub(crate) fn bounded_relation(&self, name: &str) -> Option<&CatalogTable> {
        let (qualifier, bare) = names::split_name(name);
        let normalized = names::normalize_table_name(name);
        let views = self
            .view_fallback
            .get_or_init(|| fallback::build(&self.views));
        if qualifier.is_some() {
            if self.views.contains_key(&normalized)
                || (qualifier.as_ref() == self.schema.as_ref()
                    && views.bare_keys.contains_key(&bare))
            {
                return None;
            }
            if let Some(table) = self.model_tables.get(&normalized) {
                return Some(table);
            }
            // Without a declared schema, a bare table key retains relation()'s qualified
            // fallback. A competing view with the same bare name makes that identity unsafe.
            if self.schema.is_none() && views.bare_keys.contains_key(&bare) {
                return None;
            }
            if self
                .schema
                .as_ref()
                .is_some_and(|schema| qualifier.as_ref() != Some(schema))
            {
                return None;
            }
            return self.relation(name);
        }
        if views.unique.contains_key(&bare) {
            return None;
        }
        self.relation(name)
    }

    /// Resolve only one known view and never treat a colliding table as a view.
    pub fn view_relation(&self, name: &str) -> Option<&CatalogView> {
        if self.views.is_empty() {
            return None;
        }
        let tables = self
            .relation_fallback
            .get_or_init(|| fallback::build(&self.model_tables));
        let normalized = names::normalize_table_name(name);
        let (qualifier, bare) = names::split_name(name);
        if qualifier.is_none() && tables.unique.contains_key(&bare) {
            return None;
        }
        if let Some(view) = self.views.get(&normalized) {
            let bare_table_in_schema =
                qualifier.as_ref() == self.schema.as_ref() && tables.bare_keys.contains_key(&bare);
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
