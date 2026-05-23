fn build_entry(
    abs_path: &Path,
    root: &Path,
    tsconfig: &TsConfig,
    include: Include,
    kind_filter: Option<&KindFilter>,
) -> Result<FileEntry> {
    let source =
        std::fs::read_to_string(abs_path).context(format!("reading {}", abs_path.display()))?;
    let is_tsx = matches!(
        abs_path.extension().and_then(|s| s.to_str()),
        Some("tsx") | Some("jsx")
    );
    let symbols: FileSymbols = extract_symbols(&source, is_tsx)
        .context(format!("extracting symbols from {}", abs_path.display()))?;

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
            .map(|e| resolve_export(e, abs_path, root, tsconfig))
            .collect()
    } else {
        Vec::new()
    };

    let imports = if want_imports {
        symbols
            .imports
            .into_iter()
            .map(|i| resolve_named_import(i, abs_path, root, tsconfig))
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

fn resolve_export(e: Export, abs_path: &Path, root: &Path, tsconfig: &TsConfig) -> ResolvedExport {
    let resolved = if let ExportKind::ReExport { source, .. } = &e.kind {
        resolve_import(source, abs_path, tsconfig).map(|abs| make_relative(&abs, root))
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
    tsconfig: &TsConfig,
) -> ResolvedImport {
    let resolved =
        resolve_import(&i.source, abs_path, tsconfig).map(|abs| make_relative(&abs, root));
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
