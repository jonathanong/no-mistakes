use super::prepared_plan::revisions::RevisionSources;
use super::Warning;
use no_mistakes::codebase::lockfile::{self, LockfileDiff};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

mod helpers;
pub(super) use helpers::find_git_root;
use helpers::{push_validation_warning, record_lockfile_diff};

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

        let Some(manager) = lockfile::detect_planning_manager(basename) else {
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
        if let Err(error) = lockfile::validate::validate_for_planning(manager, &new_content) {
            push_validation_warning(root, file, error, &mut warnings);
            fallback_triggered = true;
            continue;
        }
        let new_packages = lockfile::parse_lockfile(manager, &new_content);

        match revisions.base_name() {
            Some(base) => match revisions.read_base(file) {
                Some(old) => {
                    if let Err(error) = lockfile::validate::validate_for_planning(manager, &old) {
                        push_validation_warning(root, file, error, &mut warnings);
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
                    record_lockfile_diff(
                        file,
                        manager,
                        &old,
                        &new_content,
                        &new_packages,
                        &mut diff_by_lockfile,
                        &mut pnpm_importer_paths,
                    );
                }
                None => {
                    if !revisions.base_ref_exists() {
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
                        record_lockfile_diff(
                            file,
                            manager,
                            "",
                            &new_content,
                            &new_packages,
                            &mut diff_by_lockfile,
                            &mut pnpm_importer_paths,
                        );
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
