use globset::GlobSet;
use std::path::Path;

use super::PackageLayoutSpec;

pub(super) fn has_extension(filename: &str, ext: &str) -> bool {
    Path::new(filename)
        .extension()
        .and_then(|e| e.to_str())
        .is_some_and(|e| e == ext.trim_start_matches('.'))
}

pub(super) fn is_md(name: &str) -> bool {
    name.ends_with(".md")
}

pub(super) fn check_root_file(
    name: &str,
    spec: &PackageLayoutSpec,
    globs: &GlobSet,
    path: &str,
) -> Option<String> {
    if spec.allowed_root_files.iter().any(|f| f == name) || globs.is_match(name) || is_md(name) {
        None
    } else {
        Some(format!(
            "{path}: root-level file must be in allowedRootFiles, a test file, or a .md file"
        ))
    }
}

pub(super) fn check_one_deep(
    subdir: &str,
    file: &str,
    spec: &PackageLayoutSpec,
    test_dir: &str,
    globs: &GlobSet,
    path: &str,
) -> Option<String> {
    if subdir == test_dir {
        if globs.is_match(file) {
            None
        } else {
            Some(format!(
                "{path}: files in {test_dir}/ must match test file patterns"
            ))
        }
    } else if spec.allowed_subdirs.iter().any(|d| d == subdir) {
        if has_extension(file, &spec.source_extension) {
            None
        } else {
            Some(format!(
                "{path}: files in {subdir}/ must have extension {}",
                spec.source_extension
            ))
        }
    } else {
        Some(format!(
            "{path}: subdirectory {subdir}/ is not allowed (allowedSubdirs: [{}])",
            spec.allowed_subdirs.join(", ")
        ))
    }
}

pub(super) fn check_two_deep(
    subdir: &str,
    subsubdir: &str,
    file: &str,
    spec: &PackageLayoutSpec,
    test_dir: &str,
    globs: &GlobSet,
    path: &str,
) -> Option<String> {
    if spec.allowed_subdirs.iter().any(|d| d == subdir) && subsubdir == test_dir {
        if globs.is_match(file) {
            None
        } else {
            Some(format!(
                "{path}: files in {subdir}/{test_dir}/ must match test file patterns"
            ))
        }
    } else {
        Some(format!(
            "{path}: nested subdirectories beyond one level are not allowed"
        ))
    }
}
