use super::*;

pub(super) fn add_max_lines_work<'a>(
    root: &Path,
    config: &'a NoMistakesConfig,
    all_files: &[PathBuf],
    work: &mut BTreeMap<PathBuf, RustWork<'a>>,
) -> Result<()> {
    for rule in config.rule_applications(RUST_MAX_LINES_PER_FILE) {
        let opts: MaxLinesOptions = rule.try_rule_options()?;
        let files =
            matching_rust_files(root, config, rule, all_files, &opts.excludes, &opts.roots)?;
        for path in files {
            let limit = if rust_max_lines_per_file::is_test_file(root, &path) {
                opts.test_max
                    .unwrap_or(rust_max_lines_per_file::DEFAULT_TEST_MAX)
            } else {
                opts.src_max
                    .unwrap_or(rust_max_lines_per_file::DEFAULT_SRC_MAX)
            };
            let entry = work.entry(path).or_default();
            entry.applications.push((rule, Some(limit)));
            if !entry.max_limits.contains(&limit) {
                entry.max_limits.push(limit);
            }
        }
    }
    Ok(())
}

pub(super) fn add_inline_tests_work<'a>(
    root: &Path,
    config: &'a NoMistakesConfig,
    all_files: &[PathBuf],
    work: &mut BTreeMap<PathBuf, RustWork<'a>>,
) -> Result<()> {
    for rule in config.rule_applications(RUST_NO_INLINE_TESTS) {
        let opts: MaxLinesOptions = rule.try_rule_options()?;
        let files =
            matching_rust_files(root, config, rule, all_files, &opts.excludes, &opts.roots)?;
        for path in files
            .into_iter()
            .filter(|path| !rust_max_lines_per_file::is_test_file(root, path))
        {
            let entry = work.entry(path).or_default();
            entry.applications.push((rule, None));
            entry.inline_tests = true;
        }
    }
    Ok(())
}

pub(super) fn add_inline_allows_work<'a>(
    root: &Path,
    config: &'a NoMistakesConfig,
    all_files: &[PathBuf],
    work: &mut BTreeMap<PathBuf, RustWork<'a>>,
) -> Result<()> {
    for rule in config.rule_applications(RUST_NO_INLINE_ALLOWS) {
        let opts: MaxLinesOptions = rule.try_rule_options()?;
        let files =
            matching_rust_files(root, config, rule, all_files, &opts.excludes, &opts.roots)?;
        for path in files
            .into_iter()
            .filter(|path| !rust_max_lines_per_file::is_test_file(root, path))
        {
            let entry = work.entry(path).or_default();
            entry.applications.push((rule, None));
            entry.inline_allows = true;
        }
    }
    Ok(())
}

fn matching_rust_files(
    root: &Path,
    config: &NoMistakesConfig,
    rule: &crate::config::v2::schema::RuleDef,
    all_files: &[PathBuf],
    excludes: &[String],
    option_roots: &Option<Vec<PathBuf>>,
) -> Result<Vec<PathBuf>> {
    let target_roots = super::super::target_roots(root, config, rule);
    let roots = normalize_roots(root, &target_roots, option_roots);
    let skip = super::super::skip_dir_set(config);
    let files: Vec<PathBuf> = all_files
        .iter()
        .filter(|path| {
            super::super::file_allowed_by_roots_and_skip(root, &skip, path, &roots)
                && path
                    .extension()
                    .and_then(|extension| extension.to_str())
                    .is_some_and(|extension| extension == "rs")
                && !is_excluded(root, path, excludes)
        })
        .cloned()
        .collect();
    super::super::path_filter::filter_rule_files(root, config, rule, &files)
}

pub(super) fn normalize_roots(
    root: &Path,
    target_roots: &[PathBuf],
    option_roots: &Option<Vec<PathBuf>>,
) -> Vec<PathBuf> {
    option_roots
        .as_deref()
        .map(|roots| {
            roots
                .iter()
                .map(|path| {
                    if path.is_absolute() {
                        path.clone()
                    } else {
                        root.join(path)
                    }
                })
                .collect()
        })
        .unwrap_or_else(|| target_roots.to_vec())
        .into_iter()
        .map(|root| crate::codebase::ts_resolver::normalize_path(&root))
        .collect()
}

pub(super) fn is_excluded(root: &Path, path: &Path, excludes: &[String]) -> bool {
    let rel = path.strip_prefix(root).unwrap_or(path).to_string_lossy();
    excludes
        .iter()
        .any(|exclude| rel.contains(exclude.as_str()))
}
