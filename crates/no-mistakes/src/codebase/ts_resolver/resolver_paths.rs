impl<'a> ImportResolver<'a> {
    fn try_queue_compatibility_path(&self, base: &Path) -> Option<PathBuf> {
        const QUEUE_EXTENSIONS: &[&str] =
            &["mts", "ts", "tsx", "mjs", "js", "jsx", "cjs", "cts"];
        let base = normalize_path(base);
        if is_queue_source(&base) && self.path_is_file(&base) {
            return Some(base);
        }
        for extension in QUEUE_EXTENSIONS {
            let with_extension = base.with_extension(extension);
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

fn is_queue_source(path: &Path) -> bool {
matches!(
    path.extension().and_then(|extension| extension.to_str()),
    Some("mts" | "ts" | "tsx" | "mjs" | "js" | "jsx" | "cjs" | "cts")
)
}
