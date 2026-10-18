use super::*;

#[path = "object_assign_providers.rs"]
mod object_assign_providers;

impl ReceiverHistories {
    pub(in super::super) fn foreign_member_provider_references(
        &self,
        key: &(usize, String),
        path: &[Option<String>],
    ) -> Vec<(nested::ReceiverWrite, Option<CallableId>, (u32, u32))> {
        let Some(origins) = &self.assigned_origins else {
            return Vec::new();
        };
        let Some(id) = self.names.get(&key.0).and_then(|names| names.get(&key.1)) else {
            return Vec::new();
        };
        origins.foreign_references(self.root(*id), path)
    }

    pub(in super::super) fn has_rhs_reference_demand(&self, scope: usize, name: &str) -> bool {
        self.names
            .get(&scope)
            .and_then(|names| names.get(name))
            .is_some_and(|id| {
                self.proofs
                    .get(&self.root(*id))
                    .is_some_and(|proof| proof.acquired)
            })
    }

    #[cfg(feature = "test-instrumentation")]
    pub(in super::super) fn record_reference_counts(&self, facts: &mut ValueFlowFacts) {
        facts.reference_rhs_origin_queries = self.assigned_origin_queries();
        facts.reference_rhs_receipt_queries = self.rhs_receipt_queries();
        facts.reference_component_links = self.links;
        facts.reference_component_root_steps = self.root_steps.get();
    }

    #[cfg(feature = "test-instrumentation")]
    pub(in super::super) fn note_rhs_receipt_query(&self) {
        self.rhs_receipt_queries
            .set(self.rhs_receipt_queries.get() + 1);
    }
    #[cfg(feature = "test-instrumentation")]
    pub(in super::super) fn rhs_receipt_queries(&self) -> usize {
        self.rhs_receipt_queries.get()
    }

    pub(in super::super) fn has_assigned_origins(&self) -> bool {
        self.assigned_origins.is_some()
    }

    #[cfg(feature = "test-instrumentation")]
    pub(in super::super) fn assigned_origin_queries(&self) -> usize {
        self.assigned_origins
            .as_ref()
            .map_or(0, |origins| origins.queries.get())
    }
    pub(in super::super) fn assigned_origin_interested(
        &self,
        key: &(usize, &str),
        path: &[Option<String>],
        version: Option<mutable_origins::RootVersion>,
    ) -> bool {
        self.assigned_origins.as_ref().is_some_and(|origins| {
            self.names
                .get(&key.0)
                .and_then(|names| names.get(key.1))
                .is_some_and(|id| {
                    origins.interested(
                        self.root(*id),
                        path,
                        version.map(mutable_origins::RootVersion::identity),
                    )
                })
        })
    }
    pub(in super::super) fn assigned_destination_component(
        &mut self,
        key: &(usize, String),
    ) -> usize {
        let id = self.intern(key.0, &key.1);
        self.root(id)
    }
    pub(in super::super) fn record_assigned_origin(
        &mut self,
        component: usize,
        path: ValueMutationPath,
        value: ValueExpression,
        store: assigned_receiver_origins::Store<'_>,
    ) {
        let root = self.root(component);
        self.assigned_origins
            .get_or_insert_with(Default::default)
            .record(root, path, value, store);
    }
    pub(in super::super) fn assigned_origin(
        &self,
        key: &(usize, String),
        path: &[Option<String>],
        reads: &[nested::ReceiverRead],
        version: Option<mutable_origins::RootVersion>,
        canonical: &[Option<u32>],
    ) -> Option<assigned_receiver_origins::Selection<'_>> {
        let origins = self.assigned_origins.as_ref()?;
        let id = self.names.get(&key.0)?.get(&key.1)?;
        origins.selected(
            self.root(*id),
            version.map(mutable_origins::RootVersion::identity),
            path,
            reads,
            canonical,
        )
    }
}
