// Included into `data_pw_query` via `include!`; shares that module's imports.
// File discovery and per-file selector-attribute scanning.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum FileKind {
    Source,
    Test,
}

fn build_globset(patterns: &[String]) -> Result<GlobSet> {
    let mut builder = GlobSetBuilder::new();
    for pattern in patterns {
        let glob = GlobBuilder::new(pattern.trim_start_matches("./"))
            .literal_separator(false)
            .build()?;
        builder.add(glob);
    }
    Ok(builder.build()?)
}

/// Apply the scanner's existing match and skip-descent semantics to the shared
/// ignore-aware candidate list.
fn discover_files_from_visible_paths(
    root: &Path,
    files: &[PathBuf],
    extra_skip: &[String],
) -> Vec<PathBuf> {
    files
        .iter()
        .filter(|path| {
            path.strip_prefix(root)
                .is_ok_and(|rel| !rel_path_under_skip_dir(rel, extra_skip))
        })
        .filter(|path| has_source_extension(path))
        // `WalkDir`'s default (non-link-following) file type never reports a
        // symlink as a file, so match that here rather than `Path::is_file`,
        // which follows the link.
        .filter(|path| std::fs::symlink_metadata(path).is_ok_and(|metadata| metadata.is_file()))
        .cloned()
        .collect()
}

fn has_source_extension(path: &Path) -> bool {
    let ext = path.extension().and_then(|ext| ext.to_str()).unwrap_or("");
    SOURCE_EXTENSIONS.contains(&ext)
}

/// True if any ancestor directory component of `rel` (a path relative to the
/// discovery root, as returned by `git ls-files`) is a skip directory. This
/// mirrors [`is_skip_dir`]'s per-directory-entry check during a live
/// filesystem walk, where descent stops at the first skip dir: a match at
/// any depth disqualifies the file.
fn rel_path_under_skip_dir(rel: &Path, extra_skip: &[String]) -> bool {
    rel.parent()
        .into_iter()
        .flat_map(Path::components)
        .any(|component| {
            let name = component.as_os_str().to_str().unwrap_or_default();
            is_skip_dir_name(name, extra_skip)
        })
}

/// Skip dot-directories and the usual build artifacts, plus any directory named
/// in the configured `filesystem.skip_directories`.
fn is_skip_dir_name(name: &str, extra_skip: &[String]) -> bool {
    name.starts_with('.')
        || matches!(
            name,
            "node_modules" | "target" | "dist" | "build" | "coverage"
        )
        || extra_skip.iter().any(|skip| skip == name)
}

/// Inputs shared across every per-file scan in a run.
struct ScanConfig<'a> {
    value: &'a str,
    regex: &'a regex::Regex,
    roots: &'a [String],
    test_globs: &'a GlobSet,
    test_exclude_globs: &'a GlobSet,
    /// `None` when `selectorInclude` is unset (no extra source restriction).
    selector_include_globs: Option<&'a GlobSet>,
    exclude_globs: &'a GlobSet,
}

fn scan_file(path: &Path, rel: &str, scan: &ScanConfig) -> Vec<(FileKind, DataPwHit)> {
    let matches_test = scan.test_globs.is_match(rel);
    if matches_test {
        // Test files are filtered only by `testExclude`; `selectorExclude` is a
        // source-scanning setting and must not drop legitimate test usages.
        if scan.test_exclude_globs.is_match(rel) {
            return Vec::new();
        }
    } else {
        let in_source_root =
            scan.roots.is_empty() || scan.roots.iter().any(|root| path_in_root(rel, root));
        let included = scan
            .selector_include_globs
            .is_none_or(|globs| globs.is_match(rel));
        if !in_source_root || !included || scan.exclude_globs.is_match(rel) {
            return Vec::new();
        }
    }
    let is_test = matches_test;
    let kind = if is_test {
        FileKind::Test
    } else {
        FileKind::Source
    };
    let Ok(source) = std::fs::read_to_string(path) else {
        return Vec::new();
    };
    let mut hits = Vec::new();
    for (index, line) in source.lines().enumerate() {
        for caps in scan.regex.captures_iter(line) {
            let attribute = &caps["attr"];
            let matched = caps
                .name("dq")
                .or_else(|| caps.name("sq"))
                .map(|m| m.as_str())
                .unwrap_or("");
            if matched == scan.value {
                hits.push((
                    kind,
                    DataPwHit {
                        file: rel.to_string(),
                        line: index + 1,
                        attribute: attribute.to_string(),
                    },
                ));
            }
        }
    }
    hits
}

/// Whether `rel` lives under directory prefix `root` (e.g. `app` matches
/// `app/page.tsx` but not `apply.ts`).
fn path_in_root(rel: &str, root: &str) -> bool {
    rel == root || (rel.starts_with(root) && rel.as_bytes().get(root.len()) == Some(&b'/'))
}
