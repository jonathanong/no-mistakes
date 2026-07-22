//! Request-scoped comparison for `.no-mistakes.yml` changes.
//!
//! A changed configuration must remain a conservative full-suite trigger when
//! we cannot read both complete revisions. When both revisions are available,
//! however, only the requested framework is invalidated.

use super::changed_files::ChangedFiles;
use super::diff_parser::{DiffFile, DiffFileStatus, HunkLineKind};
use super::{PlanArgs, TestFramework};
use anyhow::{Context, Result};
use no_mistakes::config::v2::schema::{NoMistakesConfig, TestPlanFrameworkConfig};
use std::fs;
use std::path::{Path, PathBuf};

pub(crate) struct ConfigInvalidation {
    before: NoMistakesConfig,
    after: NoMistakesConfig,
    trigger_file: PathBuf,
}

impl ConfigInvalidation {
    pub(crate) fn framework_changed(&self, framework: TestFramework) -> bool {
        framework_test_plan(&self.before, framework) != framework_test_plan(&self.after, framework)
            || framework_tests(&self.before, framework) != framework_tests(&self.after, framework)
            || framework_trigger_projects(&self.before, framework)
                != framework_trigger_projects(&self.after, framework)
    }

    pub(crate) fn trigger(&self) -> (String, PathBuf) {
        let relative = self
            .trigger_file
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or(".no-mistakes.yml");
        (
            format!("Global configuration file changed: {relative}"),
            self.trigger_file.clone(),
        )
    }
}

/// Returns `None` when no effective v2 configuration was changed. Returns an
/// error for a changed config whose pair cannot be reconstructed: callers must
/// conservatively retain the global fallback in that case.
pub(crate) fn compare_changed_config(
    args: &PlanArgs,
    root: &Path,
    collected: &ChangedFiles,
) -> Result<Option<ConfigInvalidation>> {
    let Some(trigger_file) = changed_config_path(args, root, collected) else {
        return Ok(None);
    };

    let (before_source, after_source) = if let Some(base) = args.base.as_deref() {
        sources_from_git(root, base, args.head.as_deref(), args.config.as_deref())?
    } else if !collected.diff_files.is_empty() {
        sources_from_diff(root, args.config.as_deref(), &collected.diff_files)?
    } else {
        anyhow::bail!("configuration change has no revision or unified diff to compare")
    };

    let before = parse_endpoint(before_source)?;
    let after = parse_endpoint(after_source)?;
    Ok(Some(ConfigInvalidation {
        before,
        after,
        trigger_file,
    }))
}

fn config_candidates(args: &PlanArgs, root: &Path) -> Vec<PathBuf> {
    if let Some(config) = args.config.as_deref() {
        return vec![normalize(root, config)];
    }
    [".no-mistakes.yml", ".no-mistakes.yaml"]
        .into_iter()
        .map(|name| normalize(root, Path::new(name)))
        .collect()
}

pub(crate) fn changed_config_path(
    args: &PlanArgs,
    root: &Path,
    collected: &ChangedFiles,
) -> Option<PathBuf> {
    let candidates = config_candidates(args, root);
    collected
        .files
        .iter()
        .find(|path| {
            candidates
                .iter()
                .any(|candidate| same_path(path, candidate))
        })
        .cloned()
        .or_else(|| {
            collected.diff_files.iter().find_map(|diff| {
                candidates
                    .iter()
                    .find(|candidate| {
                        same_path(&diff.path, candidate)
                            || diff
                                .old_path
                                .as_ref()
                                .is_some_and(|old| same_path(old, candidate))
                    })
                    .cloned()
            })
        })
}

fn sources_from_git(
    root: &Path,
    base: &str,
    head: Option<&str>,
    explicit: Option<&Path>,
) -> Result<(Option<ConfigSource>, Option<ConfigSource>)> {
    let head = head.unwrap_or("HEAD");
    let merge_base = run_git(root, &["merge-base", base, head])?;
    let merge_base = merge_base.trim();
    if merge_base.is_empty() {
        anyhow::bail!("git merge-base returned an empty revision")
    }
    Ok((
        git_endpoint(root, merge_base, explicit)?,
        git_endpoint(root, head, explicit)?,
    ))
}

fn git_endpoint(
    root: &Path,
    revision: &str,
    explicit: Option<&Path>,
) -> Result<Option<ConfigSource>> {
    let candidates = endpoint_candidates(root, explicit);
    let mut found = Vec::new();
    for path in candidates {
        let relative = path
            .strip_prefix(root)
            .with_context(|| format!("config path {} is outside --root", path.display()))?;
        if let Some(source) = git_show(root, revision, relative)? {
            found.push(ConfigSource { path, source });
        }
    }
    match found.len() {
        0 => Ok(None),
        1 => Ok(found.pop()),
        _ => anyhow::bail!("multiple .no-mistakes config files exist at revision {revision}"),
    }
}

fn sources_from_diff(
    root: &Path,
    explicit: Option<&Path>,
    diff_files: &[DiffFile],
) -> Result<(Option<ConfigSource>, Option<ConfigSource>)> {
    let candidates = endpoint_candidates(root, explicit);
    let mut before = Vec::new();
    let mut after = Vec::new();
    for candidate in candidates {
        if let Some(source) = diff_side_source(&candidate, diff_files, DiffSide::Before)? {
            before.push(source);
        }
        if let Some(source) = diff_side_source(&candidate, diff_files, DiffSide::After)? {
            after.push(source);
        }
    }
    if before.len() > 1 || after.len() > 1 {
        anyhow::bail!("multiple .no-mistakes config files are present in unified diff endpoints")
    }
    Ok((before.pop(), after.pop()))
}

fn endpoint_candidates(root: &Path, explicit: Option<&Path>) -> Vec<PathBuf> {
    explicit.map_or_else(
        || {
            [".no-mistakes.yml", ".no-mistakes.yaml"]
                .into_iter()
                .map(|name| normalize(root, Path::new(name)))
                .collect()
        },
        |path| vec![normalize(root, path)],
    )
}

#[derive(Clone, Copy)]
enum DiffSide {
    Before,
    After,
}

struct ConfigSource {
    path: PathBuf,
    source: String,
}

fn diff_side_source(
    candidate: &Path,
    diff_files: &[DiffFile],
    side: DiffSide,
) -> Result<Option<ConfigSource>> {
    let matching = diff_files
        .iter()
        .filter(|diff| {
            same_path(&diff.path, candidate)
                || diff
                    .old_path
                    .as_ref()
                    .is_some_and(|old| same_path(old, candidate))
        })
        .collect::<Vec<_>>();
    if matching.len() > 1 {
        anyhow::bail!(
            "multiple unified diff entries match {}",
            candidate.display()
        )
    }
    let Some(diff) = matching.first() else {
        return fs::read_to_string(candidate)
            .map(|source| {
                Some(ConfigSource {
                    path: candidate.to_path_buf(),
                    source,
                })
            })
            .or_else(|error| {
                if error.kind() == std::io::ErrorKind::NotFound {
                    Ok(None)
                } else {
                    Err(error.into())
                }
            });
    };

    let is_new_path = same_path(&diff.path, candidate);
    match (diff.status.clone(), side, is_new_path) {
        (DiffFileStatus::Added, DiffSide::After, true) => {
            let source = read_post_diff_file(&diff.path, diff.status == DiffFileStatus::Deleted)?;
            Ok(Some(ConfigSource {
                path: candidate.to_path_buf(),
                source,
            }))
        }
        (DiffFileStatus::Added, DiffSide::Before, true) => Ok(None),
        (DiffFileStatus::Added, _, false) => Ok(None),
        (DiffFileStatus::Deleted, DiffSide::Before, _) => {
            let after = read_post_diff_file(&diff.path, diff.status == DiffFileStatus::Deleted)?;
            let source = apply_unified_hunks(&after, diff, true)?;
            Ok(Some(ConfigSource {
                path: candidate.to_path_buf(),
                source,
            }))
        }
        (DiffFileStatus::Deleted, DiffSide::After, _) => Ok(None),
        (DiffFileStatus::Renamed, DiffSide::After, true) => {
            let source = read_post_diff_file(&diff.path, false)?;
            Ok(Some(ConfigSource {
                path: candidate.to_path_buf(),
                source,
            }))
        }
        (DiffFileStatus::Renamed, DiffSide::Before, false) => {
            let after = read_post_diff_file(&diff.path, diff.status == DiffFileStatus::Deleted)?;
            let source = apply_unified_hunks(&after, diff, true)?;
            Ok(Some(ConfigSource {
                path: candidate.to_path_buf(),
                source,
            }))
        }
        (DiffFileStatus::Renamed, _, _) => Ok(None),
        (DiffFileStatus::Modified, DiffSide::After, true) => {
            let source = read_post_diff_file(&diff.path, false)?;
            Ok(Some(ConfigSource {
                path: candidate.to_path_buf(),
                source,
            }))
        }
        (DiffFileStatus::Modified, DiffSide::Before, true) => {
            let after = read_post_diff_file(&diff.path, false)?;
            let source = apply_unified_hunks(&after, diff, true)?;
            Ok(Some(ConfigSource {
                path: candidate.to_path_buf(),
                source,
            }))
        }
        (DiffFileStatus::Modified, _, false) => Ok(None),
    }
}

fn read_post_diff_file(path: &Path, deleted: bool) -> Result<String> {
    if deleted {
        return Ok(String::new());
    }
    fs::read_to_string(path).with_context(|| format!("reading patched file {}", path.display()))
}

fn apply_unified_hunks(source: &str, diff: &DiffFile, reverse: bool) -> Result<String> {
    if diff.hunks.is_empty() {
        if diff.status == DiffFileStatus::Renamed {
            // Git may represent a content-identical rename without hunks.
            return Ok(source.to_string());
        }
        anyhow::bail!(
            "unified diff for {} has no reconstructable hunks",
            diff.path.display()
        )
    }
    let trailing_newline = source.ends_with('\n');
    let mut lines = source
        .strip_suffix('\n')
        .unwrap_or(source)
        .split('\n')
        .filter(|line| !(source.is_empty() && line.is_empty()))
        .map(str::to_string)
        .collect::<Vec<_>>();
    let mut offset: isize = 0;
    for hunk in &diff.hunks {
        let (start, expected_count, replacement_count) = if reverse {
            (hunk.new_start, hunk.new_count, hunk.old_count)
        } else {
            (hunk.old_start, hunk.old_count, hunk.new_count)
        };
        let start = start.saturating_sub(1) as isize + offset;
        let start = usize::try_from(start).context("unified diff hunk starts before file")?;
        let expected = hunk
            .lines
            .iter()
            .filter(|(kind, _)| {
                matches!(kind, HunkLineKind::Context)
                    || (!reverse && matches!(kind, HunkLineKind::Removed))
                    || (reverse && matches!(kind, HunkLineKind::Added))
            })
            .map(|(_, line)| line.clone())
            .collect::<Vec<_>>();
        let replacement = hunk
            .lines
            .iter()
            .filter(|(kind, _)| {
                matches!(kind, HunkLineKind::Context)
                    || (!reverse && matches!(kind, HunkLineKind::Added))
                    || (reverse && matches!(kind, HunkLineKind::Removed))
            })
            .map(|(_, line)| line.clone())
            .collect::<Vec<_>>();
        if expected.len() != expected_count || replacement.len() != replacement_count {
            anyhow::bail!("unified diff hunk counts do not match its body")
        }
        let end = start
            .checked_add(expected.len())
            .context("unified diff hunk overflows")?;
        if lines.get(start..end) != Some(expected.as_slice()) {
            anyhow::bail!("unified diff hunk does not apply exactly")
        }
        lines.splice(start..end, replacement);
        offset += replacement_count as isize - expected_count as isize;
    }
    let mut result = lines.join("\n");
    if trailing_newline {
        result.push('\n');
    }
    Ok(result)
}

fn parse_endpoint(source: Option<ConfigSource>) -> Result<NoMistakesConfig> {
    let Some(source) = source else {
        return Ok(NoMistakesConfig::default());
    };
    crate::config::v2::discover::parse_v2_config_quiet(&source.source, &source.path)
}

fn git_show(root: &Path, revision: &str, path: &Path) -> Result<Option<String>> {
    let spec = format!("{revision}:{}", path.to_string_lossy().replace('\\', "/"));
    let output = std::process::Command::new("git")
        .args(["show", &spec])
        .current_dir(root)
        .output()
        .context("running git show for configuration comparison")?;
    if output.status.success() {
        return Ok(Some(String::from_utf8(output.stdout)?));
    }
    if output.status.code() == Some(128) {
        return Ok(None);
    }
    anyhow::bail!(
        "git show failed: {}",
        String::from_utf8_lossy(&output.stderr)
    )
}

fn run_git(root: &Path, args: &[&str]) -> Result<String> {
    let output = std::process::Command::new("git")
        .args(args)
        .current_dir(root)
        .output()
        .context("running git for configuration comparison")?;
    if !output.status.success() {
        anyhow::bail!(
            "git command failed: {}",
            String::from_utf8_lossy(&output.stderr)
        )
    }
    Ok(String::from_utf8(output.stdout)?)
}

fn normalize(root: &Path, path: &Path) -> PathBuf {
    let path = if path.is_absolute() {
        path.to_path_buf()
    } else {
        root.join(path)
    };
    no_mistakes::codebase::ts_resolver::normalize_path(&path)
}

fn same_path(left: &Path, right: &Path) -> bool {
    no_mistakes::codebase::ts_resolver::normalize_path(left)
        == no_mistakes::codebase::ts_resolver::normalize_path(right)
}

fn framework_test_plan(
    config: &NoMistakesConfig,
    framework: TestFramework,
) -> TestPlanFrameworkConfig {
    let mut plan = match framework {
        TestFramework::Dotnet => config.test_plan.dotnet.clone(),
        TestFramework::Playwright => config.test_plan.playwright.clone(),
        TestFramework::Vitest => config.test_plan.vitest.clone(),
        TestFramework::Swift => config.test_plan.swift.clone(),
    };
    // Compatibility warning bookkeeping is not behavior.
    plan.deprecated_dependencies_key = false;
    plan
}

fn framework_tests(config: &NoMistakesConfig, framework: TestFramework) -> serde_json::Value {
    match framework {
        TestFramework::Dotnet => serde_json::to_value(&config.tests.dotnet),
        TestFramework::Playwright => serde_json::to_value(&config.tests.playwright),
        TestFramework::Vitest => serde_json::to_value(&config.tests.vitest),
        TestFramework::Swift => serde_json::to_value(&config.tests.swift),
    }
    .expect("test configuration must serialize")
}

fn framework_trigger_projects(
    config: &NoMistakesConfig,
    framework: TestFramework,
) -> Vec<(&str, Option<&no_mistakes::config::v2::schema::Project>)> {
    let plan = match framework {
        TestFramework::Dotnet => &config.test_plan.dotnet,
        TestFramework::Playwright => &config.test_plan.playwright,
        TestFramework::Vitest => &config.test_plan.vitest,
        TestFramework::Swift => &config.test_plan.swift,
    };
    plan.full_suite_triggers
        .projects
        .keys()
        .map(|name| (name.as_str(), config.projects.get(name)))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tests::diff_parser::parse_unified_diff;

    #[test]
    fn reverses_positioned_hunks_against_the_on_disk_version() {
        let diff = parse_unified_diff(
            "diff --git a/.no-mistakes.yml b/.no-mistakes.yml\n--- a/.no-mistakes.yml\n+++ b/.no-mistakes.yml\n@@ -1,3 +1,3 @@\n tests:\n   vitest:\n-    configs: old.ts\n+    configs: new.ts\n",
        );
        assert_eq!(
            apply_unified_hunks("tests:\n  vitest:\n    configs: new.ts\n", &diff[0], true)
                .unwrap(),
            "tests:\n  vitest:\n    configs: old.ts\n"
        );
    }

    #[test]
    fn ignores_deprecated_marker_but_compares_the_selected_framework_only() {
        let mut before = NoMistakesConfig::default();
        before.test_plan.vitest.deprecated_dependencies_key = true;
        let after = NoMistakesConfig::default();
        let invalidation = ConfigInvalidation {
            before,
            after,
            trigger_file: PathBuf::from(".no-mistakes.yml"),
        };
        assert!(!invalidation.framework_changed(TestFramework::Vitest));
        assert!(!invalidation.framework_changed(TestFramework::Playwright));
    }

    #[test]
    fn content_identical_rename_needs_no_hunks_to_reconstruct_its_before_side() {
        let diff = parse_unified_diff(
            "diff --git a/.no-mistakes.yml b/.no-mistakes.yaml\nsimilarity index 100%\nrename from .no-mistakes.yml\nrename to .no-mistakes.yaml\n",
        );
        assert_eq!(
            apply_unified_hunks("tests: {}\n", &diff[0], true).unwrap(),
            "tests: {}\n"
        );
    }
}
