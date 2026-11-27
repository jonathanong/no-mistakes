use super::*;

impl ReceiverTraversal {
    pub(in super::super) fn enter_ordinary_owned_path<'a>(
        &'a mut self,
        key: &(usize, String),
        path: ValueMutationPath,
        reads: Vec<nested::ReceiverRead>,
    ) -> Option<state::OwnedEntry<'a>> {
        self.ordinary.enter_owned_path(key, path, reads)
    }

    pub(in super::super) fn enter_owned_path<'a>(
        &'a mut self,
        key: &(usize, String),
        path: ValueMutationPath,
        reads: Vec<nested::ReceiverRead>,
        version: Option<mutable_origins::RootVersion>,
    ) -> Option<state::OwnedEntry<'a>> {
        #[cfg(feature = "test-instrumentation")]
        {
            self.version_dispatches += 1;
        }
        let state = match version {
            Some(version) => self
                .mutable
                .get_or_insert_with(|| MutableTraversal {
                    first: (version, TraversalState::default()),
                    other: fx_map(),
                })
                .state(version),
            None => &mut self.ordinary,
        };
        state.enter_owned_path(key, path, reads)
    }
}

impl ReceiverTraversal {
    pub(in super::super) fn enter_assigned_owned_path<'a>(
        &'a mut self,
        key: &(usize, String),
        path: ValueMutationPath,
        reads: Vec<nested::ReceiverRead>,
        version: Option<mutable_origins::RootVersion>,
        receipts: assigned_receiver_origins::Receipts,
    ) -> Option<state::OwnedEntry<'a>> {
        self.enter_assigned(key, &path, reads, version, receipts)
            .map(|reads| (reads, std::borrow::Cow::Owned(path)))
    }
}
