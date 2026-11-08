impl ExternalStarProjection<'_> {
    fn has_external_star(&self, path: &Path, active: &mut FxHashSet<PathBuf>) -> bool {
        if let Some(cached) = self.demand.get(path) {
            return *cached;
        }
        if !active.insert(path.to_path_buf()) {
            return false;
        }
        let result = self
            .facts
            .get_ts_facts(path)
            .and_then(|facts| facts.symbols.as_ref())
            .is_some_and(|symbols| {
                symbols.exports.iter().any(|export| {
                    let ExportKind::ReExport { source, imported } = &export.kind else {
                        return false;
                    };
                    if export.name != "*" || imported != "*" {
                        return false;
                    }
                    let classification = self.resolver.classify_import(
                        source,
                        path,
                        self.workspace,
                        self.input.graph_files,
                    );
                    if classification.resolver_path().is_none()
                        && self.resolver.matches_paths_alias(source, path)
                    {
                        return false;
                    }
                    if let Some(target) = classification.preferred_path() {
                        return self
                            .input
                            .graph_files
                            .visible_path(target)
                            .is_some_and(|target| self.has_external_star(target, active));
                    }
                    !source.starts_with(['.', '/', '#'])
                        && !self.workspace.recognizes_specifier_from(source, path)
                        && !self.resolver.matches_paths_alias(source, path)
                        && self.resolver.resolution_candidates(source, path).is_empty()
                })
            });
        active.remove(path);
        if result || active.is_empty() {
            self.demand.insert(path.to_path_buf(), result);
        }
        result
    }
}

impl ExternalStarProjection<'_> {
    fn target(&self, from: &Path, source: &str, external: bool) -> Option<(PathBuf, bool)> {
        if external {
            if matches!(
                from.extension().and_then(|extension| extension.to_str()),
                Some("js" | "mjs" | "cjs")
            ) && (source.starts_with("./") || source.starts_with("../"))
            {
                let context = crate::codebase::ts_resolver::declarations::package_import_context(
                    source,
                    from,
                    self.input.root,
                )?;
                let from = context.as_path();
                let resolved = self.resolver.resolve(source, from).or_else(|| {
                    let path =
                        crate::codebase::ts_resolver::normalize_path(&from.parent()?.join(source));
                    matches!(
                        path.extension().and_then(|extension| extension.to_str()),
                        Some("js" | "mjs" | "cjs")
                    )
                    .then(|| self.sources.read_path(&path).ok().map(|_| path))
                    .flatten()
                });
                if let Some(path) = resolved {
                    return Some((path, true));
                }
            }
            return self
                .declarations
                .resolve(source, from)
                .map(|path| (path, true));
        }
        let classification =
            self.resolver
                .classify_import(source, from, self.workspace, self.input.graph_files);
        if classification.resolver_path().is_none()
            && self.resolver.matches_paths_alias(source, from)
        {
            return None;
        }
        if let Some(path) = classification.preferred_path() {
            return self
                .input
                .graph_files
                .visible_path(path)
                .map(|path| (path.to_path_buf(), false));
        }
        if source.starts_with(['.', '/', '#'])
            || self.workspace.recognizes_specifier_from(source, from)
            || self.resolver.matches_paths_alias(source, from)
            || !self.resolver.resolution_candidates(source, from).is_empty()
        {
            return None;
        }
        crate::codebase::ts_resolver::declarations::resolve_package_export_source(
            source,
            from,
            self.input.root,
            self.sources,
        )
        .map(|path| (path, true))
    }
}
