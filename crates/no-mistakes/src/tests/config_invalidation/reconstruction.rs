use super::super::diff_parser::{DiffFile, DiffFileStatus, HunkLineKind};
use anyhow::{Context, Result};
use std::fs;
use std::path::Path;

pub(super) fn reconstruct_diff_sources(
    diff: &DiffFile,
) -> Result<(Option<String>, Option<String>)> {
    match &diff.status {
        DiffFileStatus::Added => {
            let expected = apply_unified_hunks("", diff, false)?;
            let after = validated_endpoint_source(&diff.path, expected, "after", diff)?;
            Ok((None, Some(after)))
        }
        DiffFileStatus::Deleted => {
            let expected = apply_unified_hunks("", diff, true)?;
            let before = validated_endpoint_source(&diff.path, expected, "before", diff)?;
            Ok((Some(before), None))
        }
        DiffFileStatus::Modified => {
            let checkout = fs::read_to_string(&diff.path)
                .with_context(|| format!("reading diff checkout file {}", diff.path.display()))?;
            reconstruct_modified_sources(&checkout, diff)
                .map(|(before, after)| (Some(before), Some(after)))
        }
        DiffFileStatus::Renamed => reconstruct_renamed_sources(diff),
    }
}

fn validated_endpoint_source(
    path: &Path,
    expected: String,
    side: &str,
    diff: &DiffFile,
) -> Result<String> {
    let Some(checkout) = read_optional(path)? else {
        return Ok(expected);
    };
    if patch_side_matches(&checkout, &expected) {
        return Ok(checkout);
    }
    anyhow::bail!(
        "checkout file {} does not match the unified diff {side} side for {}",
        path.display(),
        diff.path.display()
    )
}

fn patch_side_matches(checkout: &str, expected: &str) -> bool {
    let checkout = checkout.replace("\r\n", "\n");
    let expected = expected.replace("\r\n", "\n");
    checkout == expected
        || checkout.strip_suffix('\n') == Some(expected.as_str())
        || expected.strip_suffix('\n') == Some(checkout.as_str())
}

fn reconstruct_modified_sources(checkout: &str, diff: &DiffFile) -> Result<(String, String)> {
    match apply_unified_hunks(checkout, diff, true) {
        Ok(before) => Ok((before, checkout.to_string())),
        Err(reverse_error) => match apply_unified_hunks(checkout, diff, false) {
            Ok(after) => Ok((checkout.to_string(), after)),
            Err(forward_error) => anyhow::bail!(
                "unified diff for {} applies to neither checkout side (reverse: {reverse_error}; forward: {forward_error})",
                diff.path.display()
            ),
        },
    }
}

fn reconstruct_renamed_sources(diff: &DiffFile) -> Result<(Option<String>, Option<String>)> {
    let old_path = diff
        .old_path
        .as_ref()
        .context("renamed unified diff is missing its old path")?;
    if let Some(after) = read_optional(&diff.path)? {
        let before = apply_unified_hunks(&after, diff, true)?;
        return Ok((Some(before), Some(after)));
    }
    if let Some(before) = read_optional(old_path)? {
        let after = apply_unified_hunks(&before, diff, false)?;
        return Ok((Some(before), Some(after)));
    }
    anyhow::bail!(
        "neither side of renamed unified diff exists in the checkout: {} -> {}",
        old_path.display(),
        diff.path.display()
    )
}

fn read_optional(path: &Path) -> Result<Option<String>> {
    match fs::read_to_string(path) {
        Ok(source) => Ok(Some(source)),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(error) => {
            Err(error).with_context(|| format!("reading diff checkout file {}", path.display()))
        }
    }
}

pub(super) fn apply_unified_hunks(source: &str, diff: &DiffFile, reverse: bool) -> Result<String> {
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
