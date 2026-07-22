fn build_entry_from_symbols(
    abs_path: &Path,
    root: &Path,
    resolver: &dyn crate::codebase::ts_resolver::ImportResolution,
    remapper: &crate::codebase::ts_source::FrozenPathRemapper,
    symbols: FileSymbols,
    include: Include,
    kind_filter: Option<&KindFilter>,
) -> Result<FileEntry> {
    let want_exports = matches!(include, Include::Exports | Include::Both);
    let want_imports = matches!(include, Include::Imports | Include::Both);

    let exports = if want_exports {
        symbols
            .exports
            .into_iter()
            .filter(|e| match kind_filter {
                Some(kf) => kf.matches_export(&e.kind),
                None => true,
            })
            .map(|e| resolve_export(e, abs_path, root, resolver, remapper))
            .collect()
    } else {
        Vec::new()
    };

    let imports = if want_imports {
        symbols
            .imports
            .into_iter()
            .map(|i| resolve_named_import(i, abs_path, root, resolver, remapper))
            .collect()
    } else {
        Vec::new()
    };

    let rel_path = make_relative(abs_path, root);

    Ok(FileEntry {
        rel_path,
        exports,
        imports,
    })
}

fn resolve_export(
    e: Export,
    abs_path: &Path,
    root: &Path,
    resolver: &dyn crate::codebase::ts_resolver::ImportResolution,
    remapper: &crate::codebase::ts_source::FrozenPathRemapper,
) -> ResolvedExport {
    let resolved = if let ExportKind::ReExport { source, .. } = &e.kind {
        resolver
            .resolve(source, abs_path)
            .map(|abs| make_relative(&remapper.remap(&abs), root))
    } else {
        None
    };
    ResolvedExport {
        name: e.name,
        kind: e.kind,
        line: e.line,
        resolved,
    }
}

fn resolve_named_import(
    i: NamedImport,
    abs_path: &Path,
    root: &Path,
    resolver: &dyn crate::codebase::ts_resolver::ImportResolution,
    remapper: &crate::codebase::ts_source::FrozenPathRemapper,
) -> ResolvedImport {
    let resolved = resolver
        .resolve(&i.source, abs_path)
        .map(|abs| make_relative(&remapper.remap(&abs), root));
    ResolvedImport {
        source: i.source,
        imported: i.imported,
        local: i.local,
        line: i.line,
        is_type_only: i.is_type_only,
        resolved,
    }
}

fn make_relative(abs: &Path, root: &Path) -> PathBuf {
    abs.strip_prefix(root).unwrap_or(abs).to_path_buf()
}
