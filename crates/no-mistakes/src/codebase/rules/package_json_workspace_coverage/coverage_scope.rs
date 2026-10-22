// Split out of `scan` so that function stays under the CRAP complexity gate.
use super::patterns::{
    config_finding, covered_workspace_dirs, package_directories, path_under_package_roots,
    unmatched_patterns, workspace_config_file,
};
use super::{Options, RuleFinding, RULE_ID};
use crate::codebase::ts_source::relative_slash_path;
use crate::codebase::workspaces;
use anyhow::Result;
use std::collections::HashSet;
use std::path::{Path, PathBuf};

pub(super) struct WorkspaceCoverage {
    pub(super) findings: Vec<RuleFinding>,
    pub(super) workspace_globs: Vec<String>,
    pub(super) workspace_dirs: HashSet<String>,
}

pub(super) fn resolve_workspace(
    root: &Path,
    files: &[PathBuf],
    all_files: &[PathBuf],
    sources: &crate::codebase::ts_source::SourceStore,
) -> Result<WorkspaceCoverage> {
    let mut findings = Vec::new();
    // A packages conflict is reported when the workspace file is in scope.
    // Coverage still uses that file's own list, so an omitted directory is
    // not hidden behind the conflict.
    let owner = workspace_config_file(root, all_files);
    let owner_path = crate::codebase::ts_resolver::normalize_path(&root.join(&owner));
    let owner_in_scope = files
        .iter()
        .any(|candidate| crate::codebase::ts_resolver::normalize_path(candidate) == owner_path);
    let root_packages_on_conflict =
        record_packages_conflict(root, sources, &owner, owner_in_scope, &mut findings)?;
    let workspace_globs = match &root_packages_on_conflict {
        Some(globs) => globs.clone(),
        None => workspaces::load_workspace_globs_from_source_store(root, sources)?,
    };
    let workspace_dirs = resolved_workspace_dirs(
        root,
        all_files,
        sources,
        &workspace_globs,
        root_packages_on_conflict.is_some(),
    )?;
    // Existence uses the unfiltered inventory. The finding is still owned by
    // the workspace file, and a narrowed include must not report it when that
    // file is outside the rule scope.
    if owner_in_scope {
        push_unmatched(root, all_files, &owner, &workspace_globs, &mut findings)?;
    }
    Ok(WorkspaceCoverage {
        findings,
        workspace_globs,
        workspace_dirs,
    })
}

fn record_packages_conflict(
    root: &Path,
    sources: &crate::codebase::ts_source::SourceStore,
    owner: &str,
    owner_in_scope: bool,
    findings: &mut Vec<RuleFinding>,
) -> Result<Option<Vec<String>>> {
    if owner != "pnpm-workspace.yaml" {
        return Ok(None);
    }
    let Some(resolved) = workspaces::resolve_pnpm_include_at_root(root, sources)? else {
        return Ok(None);
    };
    if resolved.packages_invalid {
        anyhow::bail!("pnpm-workspace.yaml packages field must be a list of strings");
    }
    if owner_in_scope {
        for issue in resolved
            .load_errors
            .iter()
            .filter(|issue| issue.affects("packages"))
        {
            findings.push(config_finding(owner, &issue.message));
        }
    }
    if resolved
        .duplicates
        .iter()
        .any(|setting| setting == "packages")
    {
        if owner_in_scope {
            findings.push(config_finding(
                owner,
                "setting \"packages\" is claimed by more than one pnpm source",
            ));
        }
        return Ok(Some(resolved.root_packages));
    }
    Ok(None)
}

fn resolved_workspace_dirs(
    root: &Path,
    all_files: &[PathBuf],
    sources: &crate::codebase::ts_source::SourceStore,
    workspace_globs: &[String],
    conflict: bool,
) -> Result<HashSet<String>> {
    if conflict {
        return covered_workspace_dirs(root, all_files, workspace_globs);
    }
    Ok(workspaces::load_from_source_store(root, sources)?
        .packages
        .iter()
        .map(|pkg| relative_slash_path(root, &pkg.dir))
        .collect())
}

fn push_unmatched(
    root: &Path,
    all_files: &[PathBuf],
    owner: &str,
    workspace_globs: &[String],
    findings: &mut Vec<RuleFinding>,
) -> Result<()> {
    for pattern in unmatched_patterns(workspace_globs, &package_directories(root, all_files))? {
        findings.push(config_finding(
            owner,
            &format!("workspace pattern \"{pattern}\" does not correspond to a package directory"),
        ));
    }
    Ok(())
}

pub(super) fn package_findings(
    root: &Path,
    opts: &Options,
    path: &Path,
    sources: &crate::codebase::ts_source::SourceStore,
    allowlist: &HashSet<String>,
    workspace_dirs: &HashSet<String>,
    covered_workspace_dirs: &HashSet<String>,
) -> Vec<RuleFinding> {
    if path.file_name().and_then(|name| name.to_str()) != Some("package.json") {
        return Vec::new();
    }
    let rel = relative_slash_path(root, path);
    if allowlist.contains(rel.as_str()) {
        return Vec::new();
    }
    let dir = path
        .parent()
        .expect("discovered package.json paths have a parent directory");
    let dir_rel = relative_slash_path(root, dir);
    if dir_rel.is_empty() || !path_under_package_roots(&dir_rel, &opts.package_roots) {
        return Vec::new();
    }
    if sources.parse_json_path(path).is_err() && opts.require_named_package {
        return Vec::new();
    }
    let mut findings = Vec::new();
    let unnamed = opts.require_named_package
        && package_name_with_sources(path, sources)
            .filter(|name| !name.is_empty())
            .is_none();
    if unnamed {
        findings.push(RuleFinding {
            rule: RULE_ID.to_string(),
            file: rel.clone(),
            line: 1,
            message: format!("{rel}: package.json must declare a name"),
            import: None,
            target: Some(dir_rel.clone()),
        });
    }
    let canonical = workspace_dirs.contains(&dir_rel);
    let loosely_covered = covered_workspace_dirs.contains(&dir_rel);
    if canonical || (loosely_covered && (unnamed || !opts.require_named_package)) {
        return findings;
    }
    findings.push(RuleFinding {
        rule: RULE_ID.to_string(),
        file: rel.clone(),
        line: 1,
        message: format!("{rel}: package directory is not covered by the workspace config"),
        import: None,
        target: Some(dir_rel),
    });
    findings
}

pub(super) fn package_name_with_sources(
    path: &Path,
    sources: &crate::codebase::ts_source::SourceStore,
) -> Option<String> {
    let json = sources.parse_json_path(path).ok()?;
    json.get("name")?.as_str().map(str::to_string)
}
