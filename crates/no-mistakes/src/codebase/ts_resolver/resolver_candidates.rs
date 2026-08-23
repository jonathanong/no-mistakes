impl<'a> ImportResolver<'a> {
    /// Return every local path this resolver policy may resolve to, without
    /// checking whether the path currently exists. Callers that need to
    /// account for deleted files can use this instead of reimplementing
    /// aliases, `baseUrl`, emitted-extension, index, and queue-compatibility
    /// semantics.
    pub(crate) fn resolution_candidates(
        &self,
        specifier: &str,
        importing_file: &Path,
    ) -> std::collections::BTreeSet<PathBuf> {
        let specifier_path = Path::new(specifier);
        if specifier_path.is_absolute() {
            return self.path_resolution_candidates(specifier_path);
        }
        let is_relative = match self.policy {
            ImportResolutionPolicy::Standard => {
                specifier.starts_with("./") || specifier.starts_with("../")
            }
            ImportResolutionPolicy::QueueCompatibility { .. } => specifier.starts_with('.'),
        };
        if is_relative {
            return importing_file
                .parent()
                .map(|dir| self.path_resolution_candidates(&dir.join(specifier)))
                .unwrap_or_default();
        }

        let mut candidates = std::collections::BTreeSet::new();
        for idx in &self.alias_order {
            let (pattern, replacements) = &self.tsconfig().paths[*idx];
            let Some(capture) = match_alias(pattern, specifier) else {
                continue;
            };
            for replacement in replacements {
                let resolved = replacement.replace('*', &capture);
                let base = self
                    .tsconfig()
                    .base_url
                    .as_ref()
                    .unwrap_or(&self.tsconfig().paths_dir)
                    .join(resolved);
                candidates.extend(self.path_resolution_candidates(&base));
            }
        }
        if let Some(base_url) = &self.tsconfig().base_url {
            candidates.extend(self.path_resolution_candidates(&base_url.join(specifier)));
        }
        if let ImportResolutionPolicy::QueueCompatibility { root } = self.policy {
            candidates.extend(self.path_resolution_candidates(&root.join(specifier)));
        }
        candidates
    }

    fn path_resolution_candidates(&self, base: &Path) -> std::collections::BTreeSet<PathBuf> {
        let base = normalize_path(base);
        if matches!(
            self.policy,
            ImportResolutionPolicy::QueueCompatibility { .. }
        ) {
            return self.queue_path_resolution_candidates(&base);
        }
        let mut candidates = std::collections::BTreeSet::from([base.clone()]);
        if has_explicit_extension(&base) {
            let extension = base
                .extension()
                .and_then(|extension| extension.to_str())
                .expect("an explicit extension is valid UTF-8");
            let stem = self.stem_str(&base, extension);
            candidates.extend(
                emitted_source_candidates(extension)
                    .iter()
                    .map(|extension| PathBuf::from(format!("{stem}{extension}"))),
            );
            candidates.extend(
                emitted_fallback_candidates(extension)
                    .iter()
                    .map(|extension| PathBuf::from(format!("{stem}{extension}"))),
            );
            return candidates;
        }

        let path = base.to_string_lossy();
        for extension in EXTENSIONS {
            candidates.insert(PathBuf::from(format!("{path}{extension}")));
            candidates.insert(base.join(format!("index{extension}")));
        }
        candidates
    }

    fn queue_path_resolution_candidates(&self, base: &Path) -> std::collections::BTreeSet<PathBuf> {
        const QUEUE_EXTENSIONS: &[&str] = &[
            "mts", "ts", "tsx", "mjs", "js", "jsx", "cjs", "cts",
        ];
        let mut candidates = std::collections::BTreeSet::new();
        if is_queue_source(base) {
            candidates.insert(base.to_path_buf());
        }
        for extension in QUEUE_EXTENSIONS {
            let with_extension = if is_queue_source(base) {
                base.with_extension(extension)
            } else {
                let mut path = base.as_os_str().to_os_string();
                path.push(format!(".{extension}"));
                PathBuf::from(path)
            };
            candidates.insert(with_extension);
            candidates.insert(base.join(format!("index.{extension}")));
        }
        candidates
    }
}
