pub struct ImportResolver<'a> {
    tsconfig: &'a TsConfig,
    visible: Option<&'a HashSet<PathBuf>>,
    alias_order: Vec<usize>,
    cache_enabled: bool,
    cache: DashMap<ResolveKey, Option<PathBuf>>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct ResolveKey {
    importing_dir: PathBuf,
    specifier: String,
}

impl<'a> ImportResolver<'a> {
    pub fn new(tsconfig: &'a TsConfig) -> Self {
        let mut alias_order: Vec<usize> = (0..tsconfig.paths.len()).collect();
        alias_order.sort_by(|&a, &b| {
            let la = tsconfig.paths[a].0.len();
            let lb = tsconfig.paths[b].0.len();
            lb.cmp(&la).then(a.cmp(&b))
        });

        Self {
            tsconfig,
            visible: None,
            alias_order,
            cache_enabled: true,
            cache: DashMap::new(),
        }
    }

    pub fn with_visible(mut self, visible: &'a HashSet<PathBuf>) -> Self {
        self.visible = Some(visible);
        self
    }

    pub fn without_cache(mut self) -> Self {
        self.cache_enabled = false;
        self
    }

    /// Returns `true` if `specifier` matches any configured tsconfig path
    /// alias pattern, regardless of whether the target exists on disk. Used by
    /// `resolve-check` to flag a configured alias whose target is missing as a
    /// real error rather than an external/bare specifier.
    pub fn matches_alias(&self, specifier: &str) -> bool {
        self.tsconfig
            .paths
            .iter()
            .any(|(pattern, _)| match_alias(pattern, specifier).is_some())
    }

    pub fn resolve(&self, specifier: &str, importing_file: &Path) -> Option<PathBuf> {
        if !self.cache_enabled {
            return self.resolve_uncached(specifier, importing_file);
        }

        let importing_dir = importing_file.parent().map(normalize_path)?;
        let key = ResolveKey {
            importing_dir,
            specifier: specifier.to_string(),
        };

        if let Some(cached) = self.cache.get(&key) {
            return cached.clone();
        }
        let resolved = self.resolve_uncached(specifier, importing_file);
        self.cache.insert(key, resolved.clone());
        resolved
    }

    fn resolve_uncached(&self, specifier: &str, importing_file: &Path) -> Option<PathBuf> {
        if specifier.starts_with("./") || specifier.starts_with("../") {
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
                    if let Some(p) = self.try_path(&base) {
                        return Some(p);
                    }
                }
            }
        }

        if let Some(base_url) = &self.tsconfig.base_url {
            if let Some(p) = self.try_path(&base_url.join(specifier)) {
                return Some(p);
            }
        }

        None
    }

    /// Try `base` as-is, then with each known extension appended, then as an index file.
    fn try_path(&self, base: &Path) -> Option<PathBuf> {
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
        let s = base.to_string_lossy();

        for ext in EXTENSIONS {
            let candidate = PathBuf::from(format!("{}{}", s, ext));
            if self.path_exists(&candidate) {
                return Some(candidate);
            }
        }

        for ext in EXTENSIONS {
            let candidate = base.join(format!("index{}", ext));
            if self.path_exists(&candidate) {
                return Some(candidate);
            }
        }

        None
    }

    /// Resolve an emitted `.js`/`.mjs`/`.cjs` specifier to its TypeScript source.
    /// Only checks source files (`.ts`, `.tsx`, `.mts`, `.cts`); does not return
    /// declarations or `.jsx`. Call `try_emitted_fallback` for those after the
    /// literal-file check.
    fn try_emitted_source(&self, base: &Path) -> Option<PathBuf> {
        let extension = base.extension().and_then(|ext| ext.to_str())?;
        let stem = self.stem_str(base, extension);
        emitted_source_candidates(extension).iter().find_map(|ext| {
            let candidate = PathBuf::from(format!("{stem}{ext}"));
            self.path_exists(&candidate).then_some(candidate)
        })
    }

    /// Resolve an emitted `.js`/`.mjs`/`.cjs` specifier to `.jsx` or a
    /// declaration file. Only called when neither a TypeScript source nor the
    /// literal file was found, so the literal always wins over these fallbacks.
    fn try_emitted_fallback(&self, base: &Path) -> Option<PathBuf> {
        let extension = base.extension().and_then(|ext| ext.to_str())?;
        let stem = self.stem_str(base, extension);
        emitted_fallback_candidates(extension).iter().find_map(|ext| {
            let candidate = PathBuf::from(format!("{stem}{ext}"));
            self.path_exists(&candidate).then_some(candidate)
        })
    }

    fn stem_str(&self, base: &Path, extension: &str) -> String {
        let s = base.to_string_lossy();
        s[..s.len() - extension.len() - 1].to_string()
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

/// Resolve `.` and `..` components without touching the filesystem.
pub fn normalize_path(path: &Path) -> PathBuf {
    use std::path::Component;
    let mut parts: Vec<Component> = Vec::new();
    for c in path.components() {
        match c {
            Component::CurDir => {}
            Component::ParentDir => {
                if matches!(parts.last(), Some(Component::Normal(_))) {
                    parts.pop();
                } else {
                    parts.push(c);
                }
            }
            other => parts.push(other),
        }
    }
    parts.iter().collect()
}

/// Try to match `specifier` against `pattern` (which may contain a single `*`).
/// Returns `Some(capture)` where `capture` is what the `*` matched, or `""` for exact.
fn match_alias(pattern: &str, specifier: &str) -> Option<String> {
    if let Some(star) = pattern.find('*') {
        let prefix = &pattern[..star];
        let suffix = &pattern[star + 1..];
        if specifier.starts_with(prefix) && specifier.ends_with(suffix) {
            let cap_end = specifier.len() - suffix.len();
            let cap_start = prefix.len();
            return (cap_start <= cap_end).then(|| specifier[cap_start..cap_end].to_string());
        }
        None
    } else if specifier == pattern {
        Some(String::new())
    } else {
        None
    }
}

