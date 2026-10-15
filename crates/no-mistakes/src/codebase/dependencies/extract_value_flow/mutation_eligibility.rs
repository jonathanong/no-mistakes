use super::*;

impl ImportCollector {
    pub(in super::super) fn first_projected_receiver(
        &mut self,
        key: &(usize, String),
        root_read: bool,
        #[cfg(feature = "test-instrumentation")] binding_visits: &mut usize,
    ) -> Option<usize> {
        self.value_binding_indexes
            .get(&key.0)
            .and_then(|names| names.get(key.1.as_str()))
            .map_or(&[][..], Vec::as_slice)
            .iter()
            .position(|index| {
                #[cfg(feature = "test-instrumentation")]
                {
                    *binding_visits += 1;
                }
                let value = &self.value_flow.bindings[*index].value;
                !matches!(value, ValueExpression::Named { name, .. } if !name.contains('.'))
                    || (root_read && self.references_mutable_receiver(value))
            })
    }
}

impl ImportCollector {
    pub(in super::super) fn projected_root_version<R: nested::RootCoordinate>(
        &mut self,
        key: &(usize, String),
        root_read: R,
    ) -> Option<mutable_origins::RootVersion> {
        root_read.read().and_then(|read| {
            #[cfg(feature = "test-instrumentation")]
            {
                self.value_flow.reference_root_probes += 1;
            }
            self.mutable_receiver_version(key, read)
        })
    }
}

impl ImportCollector {
    pub(in super::super) fn finish_projected_receiver<R: nested::RootCoordinate>(
        &mut self,
        traversal: &mut receiver_traversal::ReceiverTraversal,
        key: &(usize, String),
        root_read: R,
        assigned: bool,
    ) {
        // RHS exits own their separate receipt-qualified active stack.
        if assigned {
            traversal.exit_assigned(key);
            return;
        }
        if root_read.read().is_none() {
            traversal.exit_ordinary(key);
            return;
        }
        if root_read.read().is_some() && traversal.exit_single_version(key) {
            return;
        }
        let version = self.projected_root_version(key, root_read);
        traversal.exit(key, version);
    }
}
