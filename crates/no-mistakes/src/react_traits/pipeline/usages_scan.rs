fn run_usages_from_visible(
    root: &Path,
    target: &str,
    scan_targets: &[String],
    include: &UsagesInclude,
    file_config: &crate::react_traits::report::types::FileConfig,
    visible_paths: &[PathBuf],
) -> Result<UsagesReport> {
    let root = crate::codebase::ts_source::normalize_discovery_path(root);
    let (path_part, symbol) = split_target(target);
    let candidate = if Path::new(path_part).is_absolute() {
        PathBuf::from(path_part)
    } else {
        root.join(path_part)
    };
    if !candidate.exists() {
        anyhow::bail!("target file not found: {}", candidate.display());
    }
    // `exists()` above guarantees this resolves; `?` keeps the rare race as an error.
    let target_abs = candidate.canonicalize()?;

    let visible_files = visible_paths
        .iter()
        .map(|path| crate::codebase::ts_resolver::normalize_path(path))
        .collect::<HashSet<_>>();
    let files = discover_react_files_from_visible(&root, file_config, scan_targets, visible_paths)?;
    let hits: Vec<FileHit> = files
        .par_iter()
        .filter_map(|file| {
            analyze_one(file, &root, &target_abs, symbol.as_deref(), &visible_files).ok()
        })
        .collect();

    let mut callsites = Vec::new();
    let mut importer_files = BTreeSet::new();
    for hit in hits {
        callsites.extend(hit.callsites);
        if let Some(file) = hit.importer {
            importer_files.insert(file);
        }
    }
    callsites.sort_by(|a, b| (a.file.as_str(), a.line).cmp(&(b.file.as_str(), b.line)));

    let stories = include
        .stories
        .then(|| filter_importers(&importer_files, is_story));
    let tests = include
        .tests
        .then(|| filter_importers(&importer_files, is_test));
    let prop_types = include.prop_types.then(|| prop_type_names(&candidate));

    Ok(UsagesReport {
        target: UsagesTarget {
            file: relative_string(&root, &candidate),
            symbol,
        },
        callsites,
        stories,
        tests,
        prop_types,
    })
}

fn analyze_one(
    file: &Path,
    root: &Path,
    target_abs: &Path,
    symbol: Option<&str>,
    visible_files: &HashSet<PathBuf>,
) -> Result<FileHit> {
    let source = std::fs::read_to_string(file)?;
    ast::with_program(file, &source, |program, _src| {
        let import_table = build_import_table_from_visible(file, program, visible_files);
        let importer = import_table
            .values()
            .any(|entry| {
                same_path(&entry.resolved_path, target_abs)
                    && importer_symbol_matches(&entry.exported_name, symbol)
            })
            .then(|| relative_string(root, file));

        let callsites = collect_jsx_callsites(program, &import_table, &file.to_path_buf(), &source)
            .into_iter()
            .filter(|c| {
                same_path(&c.resolved_path, target_abs)
                    && callsite_symbol_matches(&c.exported_name, symbol)
            })
            .map(|c| Callsite {
                file: relative_string(root, file),
                line: c.line,
                component: c.exported_name,
                props: c.props,
                has_spread: c.has_spread,
            })
            .collect();

        FileHit {
            callsites,
            importer,
        }
    })
}
