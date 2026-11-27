use super::*;

impl CheckFactMap {
    pub(crate) fn postgres_mut(&mut self) -> &mut PreparedPostgresFacts {
        &mut self.postgres_map_mut().prepared
    }

    pub(crate) fn postgres_catalogs_mut(
        &mut self,
    ) -> &mut BTreeMap<String, Result<Arc<SchemaCatalog>, Arc<str>>> {
        &mut self.postgres_map_mut().catalogs
    }

    fn postgres_map_mut(&mut self) -> &mut PostgresMapFacts {
        let needs_owned = self
            .postgres_payload
            .as_ref()
            .is_none_or(|facts| Arc::strong_count(facts) != 1);
        if needs_owned {
            self.postgres_payload = Some(Arc::new(self.postgres_map().clone()));
        }
        Arc::get_mut(self.postgres_payload.as_mut().unwrap())
            .unwrap()
            .downcast_mut()
            .expect("Postgres request fact payload")
    }
}
