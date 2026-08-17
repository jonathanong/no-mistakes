use super::extract::{self, extract_set_with_sources};
use super::extraction_completeness;
use super::{comparison, Options, RuleFinding, RULE_ID};
use crate::codebase::dependencies::graph::TsFactLookup;
use crate::config::v2::schema::RuleDef;
use crate::config::v2::NoMistakesConfig;
use anyhow::Result;
use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

pub(super) struct ScanInput<'a> {
    pub(super) root: &'a Path,
    pub(super) config: &'a NoMistakesConfig,
    pub(super) rule: &'a RuleDef,
    pub(super) opts: &'a Options,
    pub(super) files: &'a [PathBuf],
    pub(super) target_roots: &'a [PathBuf],
    pub(super) sources: &'a crate::codebase::ts_source::SourceStore,
    pub(super) facts: Option<&'a dyn TsFactLookup>,
}

pub(super) fn scan(input: ScanInput<'_>) -> Result<Vec<RuleFinding>> {
    let ScanInput {
        root,
        config,
        rule,
        opts,
        files,
        target_roots,
        sources,
        facts,
    } = input;
    let skip = super::super::skip_dir_set(config);
    let mut has_path_regex = false;
    for spec in &opts.sets {
        if spec.kind == extract::PATH_REGEX_CAPTURE {
            has_path_regex = true;
            break;
        }
    }
    let path_files = if has_path_regex {
        extract::path_regex_capture_files(root, config, rule, sources, &skip, target_roots, files)?
    } else {
        Vec::new()
    };
    let mut sets = BTreeMap::new();
    for spec in &opts.sets {
        if spec.name.is_empty() {
            continue;
        }
        let extract_files = if spec.kind == extract::PATH_REGEX_CAPTURE {
            &path_files
        } else {
            files
        };
        sets.insert(
            spec.name.clone(),
            extract_set_with_sources(root, spec, extract_files, target_roots, sources, facts)?,
        );
    }

    let mut findings = sets
        .values()
        .flat_map(|set| {
            set.issues.iter().map(|issue| RuleFinding {
                rule: RULE_ID.to_string(),
                file: issue.file.clone(),
                line: issue.line,
                message: issue.message.clone(),
                import: None,
                target: issue.target.clone(),
            })
        })
        .collect::<Vec<_>>();
    let incomplete_sets = sets
        .iter()
        .filter(|(_, set)| extraction_completeness::has_unsuppressed_issues(root, set, sources))
        .map(|(name, _)| name.as_str())
        .collect::<BTreeSet<_>>();
    for comparison in &opts.comparisons {
        let (Some(left), Some(right)) = (sets.get(&comparison.left), sets.get(&comparison.right))
        else {
            continue;
        };
        // An incomplete extraction cannot answer a set comparison soundly.
        // Suppressed extraction issues are intentionally not incomplete: the
        // static values retained by the extractor can still be compared, and
        // the shared suppression pass will remove the issue itself later.
        if incomplete_sets.contains(comparison.left.as_str())
            || incomplete_sets.contains(comparison.right.as_str())
        {
            continue;
        }
        comparison::compare(left, right, comparison, &mut findings);
    }
    findings.sort_by(|a, b| a.file.cmp(&b.file).then(a.message.cmp(&b.message)));
    Ok(findings)
}
