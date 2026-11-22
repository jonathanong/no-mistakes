impl<'a> ScopedImportResolver<'a> {
    fn importer_selection(&self, importing_file: &ScopedPath) -> Option<usize> {
        if let Some(selection) = self.importer_selections.selections.get(importing_file) {
            return *selection;
        }
        let importing_file = normalize_path(importing_file);
        match self
            .importer_selections
            .selections
            .entry(importing_file.clone())
        {
            ScopedEntry::Occupied(entry) => *entry.get(),
            ScopedEntry::Vacant(entry) => {
                self.importer_selections
                    .builds
                    .fetch_add(1, ScopedOrdering::Relaxed);
                let selection = self.catalog.resolver_scope_index_for(&importing_file);
                entry.insert(selection);
                selection
            }
        }
    }

    #[cfg(any(test, feature = "test-instrumentation"))]
    pub(crate) fn importer_selection_build_count(&self) -> usize {
        self.importer_selections
            .builds
            .load(ScopedOrdering::Relaxed)
    }

    fn fixed_resolver_for(&self, importing_file: &ScopedPath) -> Option<&ImportResolver<'a>> {
        let resolver = self.fixed_resolver.as_ref()?;
        let Some((root, config_dir)) = &self.automatic_fixed_roots else {
            return Some(resolver);
        };
        let within = |path: &ScopedPath| path.starts_with(root) || path.starts_with(config_dir);
        let prepared_alias = self
            .session
            .and_then(|session| session.prepared_visible_alias(importing_file));
        let applies = match prepared_alias {
            Some(Some(alias)) => within(&alias),
            Some(None) => within(&normalize_path(importing_file)),
            None => {
                if let Some(session) = self.session {
                    session.record_work("resolver.fixed_importer_canonicalizations", 1);
                }
                match importing_file.canonicalize() {
                    Ok(path) => within(&normalize_path(&path)),
                    Err(_) => within(&normalize_path(importing_file)),
                }
            }
        };
        applies.then_some(resolver)
    }
}
