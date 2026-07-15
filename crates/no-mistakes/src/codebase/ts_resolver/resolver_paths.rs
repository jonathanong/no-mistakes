impl<'a> ImportResolver<'a> {
    fn resolve_uncached(&self, specifier: &str, importing_file: &Path) -> Option<PathBuf> {
        let is_relative = match self.policy {
            ImportResolutionPolicy::Standard => {
                specifier.starts_with("./") || specifier.starts_with("../")
            }
            ImportResolutionPolicy::QueueCompatibility { .. } => specifier.starts_with('.'),
        };
        if is_relative {
            let dir = importing_file.parent()?;
            return self.try_path(&dir.join(specifier));
        }

        for idx in &self.alias_order {
            let (pattern, replacements) = &self.tsconfig.paths[*idx];
            if let Some(capture) = match_alias(pattern, specifier) {
                for replacement in replacements {
                    let resolved = replacement.replace('*', &capture);
                    let base = self
                        .tsconfig
                        .base_url
                        .as_ref()
                        .unwrap_or(&self.tsconfig.paths_dir)
                        .join(&resolved);
                    if let Some(path) = self.try_path(&base) {
                        return Some(path);
                    }
                }
            }
        }

        if let Some(base_url) = &self.tsconfig.base_url {
            if let Some(path) = self.try_path(&base_url.join(specifier)) {
                return Some(path);
            }
        }

        if let ImportResolutionPolicy::QueueCompatibility { root } = self.policy {
            return self.try_path(&root.join(specifier));
        }

        None
    }

    /// Try `base` as-is, then with each known extension appended, then as an index file.
    fn try_path(&self, base: &Path) -> Option<PathBuf> {
        if matches!(
            self.policy,
            ImportResolutionPolicy::QueueCompatibility { .. }
        ) {
            return self.try_queue_compatibility_path(base);
        }
        let base = normalize_path(base);
        if has_explicit_extension(&base) {
            // NodeNext/ESM: for an emitted `.js`/`.mjs`/`.cjs` specifier:
            // 1. TypeScript source (`.ts`/`.tsx`/`.mts`/`.cts`) — highest priority.
            // 2. Literal file — takes precedence over `.jsx` and declarations.
            // 3. `.jsx`/`.d.*` fallbacks — only when the literal file is absent.
            if let Some(source) = self.try_emitted_source(&base) {
                return Some(source);
            }
            if self.path_is_file(&base) {
                return Some(base);
            }
            return self.try_emitted_fallback(&base);
        }
        if self.path_is_file(&base) {
            return Some(base);
        }
        let path = base.to_string_lossy();

        for extension in EXTENSIONS {
            let candidate = PathBuf::from(format!("{path}{extension}"));
            if self.path_exists(&candidate) {
                return Some(candidate);
            }
        }

        for extension in EXTENSIONS {
            let candidate = base.join(format!("index{extension}"));
            if self.path_exists(&candidate) {
                return Some(candidate);
            }
        }

        None
    }

    fn try_queue_compatibility_path(&self, base: &Path) -> Option<PathBuf> {
        const QUEUE_EXTENSIONS: &[&str] = &["mts", "ts", "tsx", "mjs", "js", "jsx", "cjs", "cts"];
        let base = normalize_path(base);
        if is_queue_source(&base) && self.path_is_file(&base) {
            return Some(base);
        }
        for extension in QUEUE_EXTENSIONS {
            let with_extension = if is_queue_source(&base) {
                base.with_extension(extension)
            } else {
                let mut path = base.as_os_str().to_os_string();
                path.push(format!(".{extension}"));
                PathBuf::from(path)
            };
            if self.path_is_file(&with_extension) {
                return Some(with_extension);
            }
            let index = base.join(format!("index.{extension}"));
            if self.path_is_file(&index) {
                return Some(index);
            }
        }
        None
    }

    /// Resolve an emitted `.js`/`.mjs`/`.cjs` specifier to its TypeScript source.
    /// Only checks source files (`.ts`, `.tsx`, `.mts`, `.cts`); does not return
    /// declarations or `.jsx`. Call `try_emitted_fallback` for those after the
    /// literal-file check.
    fn try_emitted_source(&self, base: &Path) -> Option<PathBuf> {
        let extension = base.extension().and_then(|extension| extension.to_str())?;
        let stem = self.stem_str(base, extension);
        emitted_source_candidates(extension)
            .iter()
            .find_map(|extension| {
                let candidate = PathBuf::from(format!("{stem}{extension}"));
                self.path_exists(&candidate).then_some(candidate)
            })
    }

    /// Resolve an emitted `.js`/`.mjs`/`.cjs` specifier to `.jsx` or a
    /// declaration file. Only called when neither a TypeScript source nor the
    /// literal file was found, so the literal always wins over these fallbacks.
    fn try_emitted_fallback(&self, base: &Path) -> Option<PathBuf> {
        let extension = base.extension().and_then(|extension| extension.to_str())?;
        let stem = self.stem_str(base, extension);
        emitted_fallback_candidates(extension)
            .iter()
            .find_map(|extension| {
                let candidate = PathBuf::from(format!("{stem}{extension}"));
                self.path_exists(&candidate).then_some(candidate)
            })
    }

    fn stem_str(&self, base: &Path, extension: &str) -> String {
        let path = base.to_string_lossy();
        path[..path.len() - extension.len() - 1].to_string()
    }

    fn path_exists(&self, path: &Path) -> bool {
        self.visible
            .map(|visible| visible.contains(path))
            .unwrap_or_else(|| path.exists())
    }

    fn path_is_file(&self, path: &Path) -> bool {
        self.visible
            .map(|visible| visible.contains(path))
            .unwrap_or_else(|| path.is_file())
    }
}

fn is_queue_source(path: &Path) -> bool {
    matches!(
        path.extension().and_then(|extension| extension.to_str()),
        Some("mts" | "ts" | "tsx" | "mjs" | "js" | "jsx" | "cjs" | "cts")
    )
}
