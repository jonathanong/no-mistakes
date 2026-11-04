use super::*;
use anyhow::Context;
mod diff_source;
#[cfg(test)]
pub(super) use diff_source::diff_side_source;
pub(super) use diff_source::{ConfigSource, DiffSide};

pub(super) fn sources_from_git(
    root: &Path,
    base: &str,
    head: Option<&str>,
    explicit: Option<&Path>,
) -> Result<(Vec<ConfigSource>, Vec<ConfigSource>)> {
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

fn git_endpoint(root: &Path, revision: &str, explicit: Option<&Path>) -> Result<Vec<ConfigSource>> {
    let mut candidates = endpoint_candidates(root, explicit);
    if explicit.is_none() {
        let output = std::process::Command::new("git")
            .args([
                "ls-tree",
                "-r",
                "--name-only",
                "-z",
                revision,
                "--",
                ".no-mistakes",
            ])
            .current_dir(root)
            .output()
            .context("listing historical configuration fragments")?;
        if !output.status.success() {
            anyhow::bail!("cannot list configuration at revision {revision}")
        }
        for path in output
            .stdout
            .split(|byte| *byte == 0)
            .filter(|path| !path.is_empty())
        {
            let path = root.join(std::str::from_utf8(path)?);
            if crate::config::v2::discover::classify_fragment(root, &path)?.is_some() {
                candidates.push(path);
            }
        }
    }
    let mut found = Vec::new();
    for path in candidates {
        let relative = path
            .strip_prefix(root)
            .with_context(|| format!("config path {} is outside --root", path.display()))?;
        if let Some(source) = git_show(root, revision, relative)? {
            found.push(ConfigSource { path, source });
        }
    }
    Ok(found)
}

pub(super) fn sources_from_diff(
    root: &Path,
    explicit: Option<&Path>,
    diff_files: &[DiffFile],
    visible: &[PathBuf],
    sources: &crate::codebase::ts_source::SourceStore,
) -> Result<(Vec<ConfigSource>, Vec<ConfigSource>)> {
    let mut candidates = endpoint_candidates(root, explicit);
    if explicit.is_none() {
        candidates.extend(
            visible
                .iter()
                .filter(|path| path.starts_with(root.join(".no-mistakes")))
                .cloned(),
        );
        for diff in diff_files {
            candidates.extend(
                std::iter::once(&diff.path)
                    .chain(diff.old_path.iter())
                    .filter(|path| path.starts_with(root.join(".no-mistakes")))
                    .cloned(),
            );
        }
    }
    candidates.sort();
    candidates.dedup();
    let mut before = Vec::new();
    let mut after = Vec::new();
    for candidate in candidates {
        if explicit.is_none()
            && candidate.starts_with(root.join(".no-mistakes"))
            && crate::config::v2::discover::classify_fragment(root, &candidate)?.is_none()
        {
            continue;
        }
        if let Some(source) = diff_source::diff_side_source_with_store(
            &candidate,
            diff_files,
            DiffSide::Before,
            Some(sources),
        )? {
            before.push(source);
        }
        if let Some(source) = diff_source::diff_side_source_with_store(
            &candidate,
            diff_files,
            DiffSide::After,
            Some(sources),
        )? {
            after.push(source);
        }
    }
    Ok((before, after))
}

fn endpoint_candidates(root: &Path, explicit: Option<&Path>) -> Vec<PathBuf> {
    explicit.map_or_else(
        || automatic_config_candidates(root),
        |path| vec![normalize(root, path)],
    )
}

pub(super) fn parse_endpoint(
    root: &Path,
    sources: Vec<ConfigSource>,
    explicit: bool,
) -> Result<NoMistakesConfig> {
    let mut documents = Vec::new();
    for source in sources {
        let section = if explicit {
            None
        } else {
            crate::config::v2::discover::classify_fragment(root, &source.path)?
        };
        documents.push(crate::config::v2::discover::ConfigDocument {
            path: source.path,
            section,
            source: source.source,
        });
    }
    documents.sort_by(|a, b| {
        a.section
            .is_some()
            .cmp(&b.section.is_some())
            .then(a.path.cmp(&b.path))
    });
    if documents.iter().filter(|doc| doc.section.is_none()).count() > 1 {
        anyhow::bail!("multiple .no-mistakes config files are present in configuration endpoints")
    }
    crate::config::v2::discover::parse_sources_quiet(root, &documents)
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
