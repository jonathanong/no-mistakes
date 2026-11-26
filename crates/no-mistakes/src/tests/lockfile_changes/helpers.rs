use super::super::Warning;
use no_mistakes::codebase::lockfile::{self, LockfileDiff, PackageManager, ResolvedPackage};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

pub(super) fn record_lockfile_diff(
    file: &Path,
    manager: PackageManager,
    old_content: &str,
    new_content: &str,
    new_packages: &[ResolvedPackage],
    diff_by_lockfile: &mut Vec<(PathBuf, LockfileDiff)>,
    pnpm_importer_paths: &mut BTreeMap<PathBuf, BTreeMap<String, Vec<String>>>,
) {
    let old_packages = lockfile::parse_lockfile(manager, old_content);
    let mut lf_diff = lockfile::diff(&old_packages, new_packages);
    if manager == PackageManager::Pnpm {
        let changed_names: Vec<String> = lf_diff.all_changed_names().map(str::to_string).collect();
        pnpm_importer_paths.insert(
            file.to_path_buf(),
            lockfile::pnpm::impact_importer_paths(old_content, new_content, &changed_names),
        );
        lf_diff.changed = lockfile::pnpm::impact_names(old_content, new_content, changed_names);
    }
    if !lf_diff.is_empty() {
        diff_by_lockfile.push((file.to_path_buf(), lf_diff));
    }
}

pub(super) fn push_validation_warning(
    root: &Path,
    file: &Path,
    error: lockfile::validate::LockfileValidationError,
    warnings: &mut Vec<Warning>,
) {
    let rel = crate::tests::plan::relative_path(root, file);
    warnings.push(Warning {
        r#type: error.kind.to_string(),
        message: format!(
            "`{rel}` has {} at a compared revision; full-suite selection requires global fallback opt-in",
            error.description
        ),
        file: rel,
        line: None,
    });
}

/// `Ok(None)` reports that `dir` is not inside a Git repository (or the
/// probe otherwise failed for a non-timeout reason); a deadline timeout
/// propagates as `Err` instead, so a strict caller like
/// `git_diff::classify_git_diff_failure` can tell the two apart. Callers
/// that are infallible by design (`analyze_lockfile_changes`) collapse
/// both back into their own fallback via `.ok().flatten()`.
pub(in crate::tests) fn find_git_root(dir: &Path) -> std::io::Result<Option<PathBuf>> {
    let mut command = std::process::Command::new("git");
    command
        .args(["rev-parse", "--show-toplevel"])
        .current_dir(dir);
    let output = match crate::invocation::command_output(&mut command) {
        Ok(output) => output,
        Err(error) if error.kind() == std::io::ErrorKind::TimedOut => return Err(error),
        Err(_) => return Ok(None),
    };
    if !output.status.success() {
        return Ok(None);
    }
    Ok(String::from_utf8(output.stdout)
        .ok()
        .map(|s| PathBuf::from(s.trim())))
}
