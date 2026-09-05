impl GraphFiles {
    /// Add one existing, explicitly requested file to the request graph.
    ///
    /// This grants authority only to the root target itself. Imports still
    /// resolve against visible paths, so ignored transitive files remain excluded.
    pub(crate) fn add_explicit_root(&mut self, path: &Path) -> bool {
        let path = crate::codebase::ts_resolver::normalize_path(path);
        if !path.is_file() {
            return false;
        }
        let mut changed = false;
        match self
            .all
            .binary_search_by(|candidate| candidate.as_os_str().cmp(path.as_os_str()))
        {
            Ok(index) => {
                if self.visible.get(index).copied() != Some(1) {
                    self.visible[index] = 1;
                    changed = true;
                }
            }
            Err(index) => {
                std::sync::Arc::make_mut(&mut self.all).insert(index, path.clone());
                self.visible.insert(index, 1);
                if let Ok(canonical) = path.canonicalize() {
                    self.canonical_visible.insert_if_built(
                        crate::codebase::ts_resolver::normalize_path(&canonical),
                        path.clone(),
                    );
                }
                changed = true;
            }
        }
        // A demand plan may leave an unrequested runner config visible for import resolution
        // while excluding it from eager graph parsing. An explicit query restores that ordinary
        // source file to the indexable universe even though it was already visible.
        if is_indexable(&path) && !self.indexable.contains(&path) {
            let indexable = std::sync::Arc::make_mut(&mut self.indexable);
            indexable.push(path);
            indexable.sort_by(|left, right| left.as_os_str().cmp(right.as_os_str()));
            changed = true;
        }
        if changed {
            self.canonical_visible.bump_universe();
            self.scoped_visible.take();
        }
        changed
    }

    pub(crate) fn contains_visible(&self, path: &Path) -> bool {
        self.visible_index(path).is_some()
    }

    fn visible_index(&self, path: &Path) -> Option<usize> {
        self.all
            .binary_search_by(|candidate| candidate.as_os_str().cmp(path.as_os_str()))
            .ok()
            .filter(|&index| self.visible.get(index).copied() == Some(1))
    }

    pub(crate) fn visible_path(&self, path: &Path) -> Option<&Path> {
        if let Some(index) = self.visible_index(path) {
            return Some(self.all[index].as_path());
        }
        let canonical = crate::codebase::ts_resolver::normalize_path(&path.canonicalize().ok()?);
        if let Some(index) = self.visible_index(&canonical) {
            return Some(self.all[index].as_path());
        }
        let original = self
            .canonical_visible
            .get(&self.all, &self.visible, &canonical)?;
        self.visible_index(&original)
            .map(|index| self.all[index].as_path())
    }

    pub(crate) fn iter_visible(&self) -> impl Iterator<Item = &PathBuf> {
        self.all
            .iter()
            .zip(self.visible.iter())
            .filter(|(_, flag)| **flag == 1)
            .map(|(path, _)| path)
    }

    pub(crate) fn visible_len(&self) -> usize {
        self.visible.iter().filter(|flag| **flag == 1).count()
    }

    /// Already-normalized visible paths as a membership set.
    pub(crate) fn visible_path_set(&self) -> crate::fx::PathSet {
        self.iter_visible().cloned().collect()
    }
}

impl crate::codebase::ts_resolver::VisiblePathLookup for GraphFiles {
    fn contains_visible(&self, path: &Path) -> bool {
        // Exact membership only. Canonical remapping belongs in `visible_path`;
        // resolver probes must not canonicalize every miss the way HashSet did not.
        GraphFiles::contains_visible(self, path)
    }

    fn visible_cache_key(&self) -> Vec<PathBuf> {
        self.iter_visible().cloned().collect()
    }

    fn normalized_visible(&self) -> std::sync::Arc<crate::fx::PathSet> {
        std::sync::Arc::clone(self.scoped_visible.get_or_init(|| {
            std::sync::Arc::new(
                crate::codebase::ts_resolver::normalized_visible_path_set(
                    self.iter_visible().cloned(),
                ),
            )
        }))
    }

    fn visible_len(&self) -> usize {
        GraphFiles::visible_len(self)
    }
}
