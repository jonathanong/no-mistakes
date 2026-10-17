impl ImportCollector {
    pub(super) fn classify_receiver_successor(
        &mut self,
        key: &(usize, String),
        root_read: Option<nested::RootRead>,
        terminal_allowed: bool,
    ) -> (bool, Option<mutable_origins::RootVersion>) {
        #[cfg(all(test, feature = "test-instrumentation"))]
        let old_lookup = mutable_origins::fusion_tests::OLD.with(std::cell::Cell::get);
        #[cfg(not(all(test, feature = "test-instrumentation")))]
        let old_lookup = false;
        #[cfg(all(test, feature = "test-instrumentation"))]
        let old_terminal = old_lookup
            && terminal_allowed
            && root_read.is_some()
            && self.mutation_reference_is_terminal(key);
        #[cfg(not(all(test, feature = "test-instrumentation")))]
        let old_terminal = false;
        // Preserve the child's imported-terminal fast path without a negative
        // root lookup. Cache false too: a value successor must be checked once.
        let imported_terminal = (!old_lookup
            && terminal_allowed
            && root_read.is_some()
            && self.imported_receiver_without_lexical(key))
        .then(|| self.mutation_reference_has_no_value_successor(key));
        let version = root_read
            .filter(|_| !old_terminal && imported_terminal != Some(true))
            .and_then(|read| {
                #[cfg(feature = "test-instrumentation")]
                {
                    self.value_flow.reference_root_probes += 1;
                }
                self.mutable_receiver_version(key, read)
            });
        let terminal = if old_lookup {
            old_terminal
        } else {
            imported_terminal.unwrap_or_else(|| {
                terminal_allowed
                    && root_read.is_some()
                    && version.is_none()
                    && self.mutation_reference_has_no_value_successor(key)
            })
        };
        (terminal, if terminal { None } else { version })
    }
    fn imported_receiver_without_lexical(&self, key: &(usize, String)) -> bool {
        key.0 == 0
            && self.imported_bindings.contains(&key.1)
            && !self.has_lexical_binding_at(0, &key.1)
    }
    pub(super) fn mutation_reference_has_no_value_successor(&self, key: &(usize, String)) -> bool {
        #[cfg(all(test, feature = "test-instrumentation"))]
        mutable_origins::fusion_tests::DESCRIPTORS.with(|n| n.set(n.get() + 1));
        self.value_binding_indexes
            .get(&key.0)
            .and_then(|names| names.get(key.1.as_str()))
            .is_none_or(Vec::is_empty)
            && self
                .value_reference_links
                .as_ref()
                .and_then(|links| links.get(&key.0))
                .and_then(|names| names.get(key.1.as_str()))
                .is_none_or(|links| links.is_empty())
    }
    #[cfg(all(test, feature = "test-instrumentation"))]
    pub(super) fn mutation_reference_is_terminal(&self, key: &(usize, String)) -> bool {
        self.mutation_reference_has_no_value_successor(key)
            && (self.imported_receiver_without_lexical(key)
                || !self.has_mutable_receiver_binding(key.0, &key.1))
    }
}
