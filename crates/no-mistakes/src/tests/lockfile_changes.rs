use super::prepared_plan::revisions::RevisionSources;
use super::Warning;
use no_mistakes::codebase::lockfile::{self, LockfileDiff};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

#[derive(Clone)]
pub(crate) struct LockfileAnalysis {
    pub diff_by_lockfile: Vec<(PathBuf, LockfileDiff)>,
    pub pnpm_importer_paths: BTreeMap<PathBuf, BTreeMap<String, Vec<String>>>,
    pub warnings: Vec<Warning>,
    pub fallback_triggered: bool,
}

pub(crate) fn analyze_lockfile_changes(
    root: &Path,
    all_files: &[PathBuf],
    revisions: &RevisionSources,
) -> LockfileAnalysis {
    let mut diff_by_lockfile = Vec::new();
    let mut pnpm_importer_paths = BTreeMap::new();
    let mut warnings = Vec::new();
    let mut fallback_triggered = false;

    for file in all_files {
        let basename = file.file_name().and_then(|n| n.to_str()).unwrap_or("");

        if lockfile::is_binary_lockfile(basename) {
            let rel = crate::tests::plan::relative_path(root, file);
            warnings.push(Warning {
                r#type: "lockfile-binary-unsupported".to_string(),
                message: format!(
                    "`{}` is a binary lockfile and cannot be analyzed for package changes; full-suite selection requires global fallback opt-in",
                    rel
                ),
                file: rel,
                line: None,
            });
            fallback_triggered = true;
            continue;
        }

        let Some(manager) = lockfile::detect_manager(basename) else {
            continue;
        };

        if revisions.base_name().is_none() && !revisions.is_diff_only() {
            let rel = crate::tests::plan::relative_path(root, file);
            warnings.push(Warning {
                r#type: "lockfile-no-baseline".to_string(),
                message: format!(
                    "Could not determine old content of `{rel}`. Provide `--base` to enable targeted lockfile analysis; full-suite selection requires global fallback opt-in."
                ),
                file: rel,
                line: None,
            });
            fallback_triggered = true;
            continue;
        }

        let new_content = if let Some(head) = revisions.head_name() {
            match revisions.read_after(file) {
                Some(content) => content,
                None => {
                    let rel = crate::tests::plan::relative_path(root, file);
                    if !revisions.head_ref_exists() {
                        warnings.push(Warning {
                            r#type: "lockfile-no-baseline".to_string(),
                            message: format!(
                                "Could not read `{}` at head ref `{}`; full-suite selection requires global fallback opt-in",
                                rel, head
                            ),
                            file: rel,
                            line: None,
                        });
                        fallback_triggered = true;
                        continue;
                    }
                    // Valid head ref but file deleted at head — treat new content as empty
                    std::sync::Arc::from("")
                }
            }
        } else if revisions.is_diff_only() {
            // In diff-only mode (--diff/--diff-stdin/etc.) without --head, the working tree
            // may still be at the base. Reading from disk would compare base-vs-base and miss
            // the lockfile change; fall back instead of producing a bogus empty diff.
            let rel = crate::tests::plan::relative_path(root, file);
            warnings.push(Warning {
                r#type: "lockfile-no-baseline".to_string(),
                message: format!(
                    "Could not determine new content of `{}` in diff-only mode. Provide `--head` to enable targeted lockfile analysis; full-suite selection requires global fallback opt-in.",
                    rel
                ),
                file: rel,
                line: None,
            });
            fallback_triggered = true;
            continue;
        } else {
            revisions.read_after_or_empty(file)
        };
        if manager == lockfile::PackageManager::Pnpm {
            if let Err(error) = lockfile::pnpm::validate_for_planning(&new_content) {
                let rel = crate::tests::plan::relative_path(root, file);
                let (kind, description) = match error {
                    lockfile::pnpm::PnpmValidationError::Malformed => {
                        ("lockfile-pnpm-malformed", "malformed pnpm YAML")
                    }
                    lockfile::pnpm::PnpmValidationError::UnsupportedSchema => (
                        "lockfile-pnpm-unsupported-schema",
                        "unsupported pnpm lockfile schema",
                    ),
                };
                warnings.push(Warning {
                    r#type: kind.to_string(),
                    message: format!(
                        "`{rel}` has {description} at a compared revision; full-suite selection requires global fallback opt-in"
                    ),
                    file: rel,
                    line: None,
                });
                fallback_triggered = true;
                continue;
            }
        }
        let new_packages = lockfile::parse_lockfile(manager, &new_content);

        match revisions.base_name() {
            Some(base) => match revisions.read_base(file) {
                Some(old) => {
                    if let Some(error) = (manager == lockfile::PackageManager::Pnpm)
                        .then(|| {
                            lockfile::pnpm::validate_for_planning(&old)
                                .and_then(|()| lockfile::pnpm::validate_for_planning(&new_content))
                        })
                        .and_then(Result::err)
                    {
                        let rel = crate::tests::plan::relative_path(root, file);
                        let (kind, description) = match error {
                            lockfile::pnpm::PnpmValidationError::Malformed => {
                                ("lockfile-pnpm-malformed", "malformed pnpm YAML")
                            }
                            lockfile::pnpm::PnpmValidationError::UnsupportedSchema => (
                                "lockfile-pnpm-unsupported-schema",
                                "unsupported pnpm lockfile schema",
                            ),
                        };
                        warnings.push(Warning {
                            r#type: kind.to_string(),
                            message: format!(
                                "`{rel}` has {description} at a compared revision; full-suite selection requires global fallback opt-in"
                            ),
                            file: rel,
                            line: None,
                        });
                        fallback_triggered = true;
                        continue;
                    }
                    if manager == lockfile::PackageManager::Pnpm {
                        let sections = lockfile::pnpm::changed_unmodeled_installation_sections(
                            &old,
                            &new_content,
                        );
                        if !sections.is_empty() {
                            let rel = crate::tests::plan::relative_path(root, file);
                            warnings.push(Warning {
                                r#type: "lockfile-pnpm-unmodeled-installation-section".to_string(),
                                message: format!(
                                    "`{rel}` changed pnpm installation section(s) {}; full-suite selection requires global fallback opt-in",
                                    sections.join(", ")
                                ),
                                file: rel,
                                line: None,
                            });
                            fallback_triggered = true;
                            continue;
                        }
                    }
                    let old_packages = lockfile::parse_lockfile(manager, &old);
                    let mut lf_diff = lockfile::diff(&old_packages, &new_packages);
                    if manager == lockfile::PackageManager::Pnpm {
                        let changed_names: Vec<String> =
                            lf_diff.all_changed_names().map(str::to_string).collect();
                        pnpm_importer_paths.insert(
                            file.clone(),
                            lockfile::pnpm::impact_importer_paths(
                                &old,
                                &new_content,
                                &changed_names,
                            ),
                        );
                        lf_diff.changed =
                            lockfile::pnpm::impact_names(&old, &new_content, changed_names);
                    }
                    if !lf_diff.is_empty() {
                        diff_by_lockfile.push((file.clone(), lf_diff));
                    }
                }
                None => {
                    if !revisions.base_ref_exists() {
                        // Invalid base ref — cannot determine what changed
                        let rel = crate::tests::plan::relative_path(root, file);
                        warnings.push(Warning {
                            r#type: "lockfile-no-baseline".to_string(),
                            message: format!(
                                "Could not read `{}` at base ref `{}`; full-suite selection requires global fallback opt-in",
                                rel, base
                            ),
                            file: rel,
                            line: None,
                        });
                        fallback_triggered = true;
                    } else {
                        // Valid base ref but file not at base — newly added lockfile;
                        // treat baseline as empty so all packages are seen as added.
                        let old_packages = lockfile::parse_lockfile(manager, "");
                        let mut lf_diff = lockfile::diff(&old_packages, &new_packages);
                        if manager == lockfile::PackageManager::Pnpm {
                            let changed_names: Vec<String> =
                                lf_diff.all_changed_names().map(str::to_string).collect();
                            pnpm_importer_paths.insert(
                                file.clone(),
                                lockfile::pnpm::impact_importer_paths(
                                    "",
                                    &new_content,
                                    &changed_names,
                                ),
                            );
                            lf_diff.changed =
                                lockfile::pnpm::impact_names("", &new_content, changed_names);
                        }
                        if !lf_diff.is_empty() {
                            diff_by_lockfile.push((file.clone(), lf_diff));
                        }
                    }
                }
            },
            None => {
                let rel = crate::tests::plan::relative_path(root, file);
                warnings.push(Warning {
                r#type: "lockfile-no-baseline".to_string(),
                message: format!(
                        "Could not determine old content of `{}`. Provide `--base` to enable targeted lockfile analysis; full-suite selection requires global fallback opt-in.",
                        rel
                    ),
                    file: rel,
                    line: None,
                });
                fallback_triggered = true;
            }
        }
    }

    LockfileAnalysis {
        diff_by_lockfile,
        pnpm_importer_paths,
        warnings,
        fallback_triggered,
    }
}

/// `Ok(None)` reports that `dir` is not inside a Git repository (or the
/// probe otherwise failed for a non-timeout reason); a deadline timeout
/// propagates as `Err` instead, so a strict caller like
/// `git_diff::classify_git_diff_failure` can tell the two apart. Callers
/// that are infallible by design (`analyze_lockfile_changes`) collapse
/// both back into their own fallback via `.ok().flatten()`.
pub(super) fn find_git_root(dir: &Path) -> std::io::Result<Option<PathBuf>> {
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
