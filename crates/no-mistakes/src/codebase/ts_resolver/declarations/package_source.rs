use super::*;

pub(crate) fn resolve_source(
    specifier: &str,
    importer: &Path,
    consumer_root: &Path,
    sources: &SourceStore,
) -> Option<PathBuf> {
    let (name, subpath, _) = package_name(specifier)?;
    let importer = lookup_context(specifier, importer, consumer_root)?;
    for directory in importer.parent()?.ancestors() {
        let root = directory.join("node_modules").join(&name);
        if !installed(&root).ok()? {
            continue;
        }
        let manifest = sources.parse_json_path(&root.join("package.json")).ok()?;
        manifest.get("exports")?;
        if let Some(path) = entry(
            &root,
            &subpath,
            consumer_root,
            mode::ConditionMode::for_importer(&importer),
            sources,
        )
        .ok()
        .flatten()
        {
            return Some(path);
        }
        let targets = exports::source_targets(
            manifest.get("exports")?,
            &subpath,
            mode::ConditionMode::for_importer(&importer),
        )
        .ok()?;
        let mut found = None;
        for target in targets {
            if !target.known {
                return None;
            }
            let path = contained(&root, &target.path, true).ok()?;
            SourceStore::read_prepared_or_open(Some(sources), &path).ok()?;
            if found.as_ref().is_some_and(|prior| prior != &path) {
                return None;
            }
            found = Some(path);
        }
        return found;
    }
    None
}
