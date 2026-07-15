use super::RuleFinding;
use crate::codebase::ts_source::relative_slash_path;
use crate::config::v2::NoMistakesConfig;
use anyhow::{Context, Result};
use globset::{GlobBuilder, GlobSet, GlobSetBuilder};
use rayon::prelude::*;
use serde::Deserialize;
use std::path::{Path, PathBuf};

pub const RULE_ID: &str = "banned-paths";

#[derive(Deserialize, Default)]
#[serde(default, rename_all = "camelCase")]
pub(crate) struct Options {
    pub(crate) banned_paths: Vec<BannedPath>,
}

#[derive(Deserialize, Default)]
#[serde(default, rename_all = "camelCase")]
pub(crate) struct BannedPath {
    pub(crate) glob: String,
    pub(crate) message: String,
}

struct CompiledBan<'a> {
    def: &'a BannedPath,
    globset: GlobSet,
}

pub(crate) fn check_with_files(
    root: &Path,
    config: &NoMistakesConfig,
    all_files: &[PathBuf],
) -> Result<Vec<RuleFinding>> {
    let all: Result<Vec<Vec<RuleFinding>>> = config
        .rule_applications(RULE_ID)
        .into_par_iter()
        .map(|rule| -> Result<Vec<RuleFinding>> {
            let opts: Options = rule.rule_options();
            let target_roots = super::target_roots(root, config, rule);
            let skip = super::skip_dir_set(config);
            let files: Vec<PathBuf> = all_files
                .iter()
                .filter(|p| {
                    if rule.applies_to_repository() && p.starts_with(root) {
                        true
                    } else {
                        super::file_allowed_by_roots_and_skip(root, &skip, p, &target_roots)
                    }
                })
                .cloned()
                .collect();
            let files = super::path_filter::filter_rule_files(root, config, rule, &files)?;
            scan(root, &opts, &files, &target_roots)
        })
        .collect();
    let mut findings: Vec<RuleFinding> = all?.into_iter().flatten().collect();
    super::sort_findings(&mut findings);
    Ok(findings)
}

fn scan(
    root: &Path,
    opts: &Options,
    files: &[PathBuf],
    target_roots: &[PathBuf],
) -> Result<Vec<RuleFinding>> {
    let compiled = compile_bans(opts)?;
    let mut findings: Vec<RuleFinding> = files
        .par_iter()
        .flat_map(|path| check_file(root, path, &compiled, target_roots))
        .collect();
    super::sort_findings(&mut findings);
    Ok(findings)
}

fn compile_bans(opts: &Options) -> Result<Vec<CompiledBan<'_>>> {
    opts.banned_paths
        .iter()
        .map(|def| {
            let mut builder = GlobSetBuilder::new();
            let pattern = escape_literal_route_brackets(&def.glob);
            add_glob(&mut builder, &pattern)?;
            Ok(CompiledBan {
                def,
                globset: builder.build()?,
            })
        })
        .collect()
}

fn add_glob(builder: &mut GlobSetBuilder, pattern: &str) -> Result<()> {
    builder.add(
        GlobBuilder::new(pattern.trim_start_matches("./"))
            .literal_separator(true)
            .build()
            .with_context(|| format!("banned-paths contains invalid glob `{pattern}`"))?,
    );
    Ok(())
}

fn escape_literal_route_brackets(pattern: &str) -> String {
    let mut escaped = String::with_capacity(pattern.len());
    let chars: Vec<char> = pattern.chars().collect();
    let mut index = 0;
    while index < chars.len() {
        if chars[index] == '[' && (index == 0 || chars[index - 1] == '/') {
            let segment_end = chars[index..]
                .iter()
                .position(|ch| *ch == '/')
                .map(|offset| index + offset)
                .unwrap_or(chars.len());
            if chars[index..segment_end].contains(&']') {
                for ch in &chars[index..segment_end] {
                    match ch {
                        '[' => escaped.push_str("[[]"),
                        ']' => escaped.push_str("[]]"),
                        _ => escaped.push(*ch),
                    }
                }
                index = segment_end;
                continue;
            }
        }
        escaped.push(chars[index]);
        index += 1;
    }
    escaped
}

fn check_file(
    root: &Path,
    path: &Path,
    bans: &[CompiledBan<'_>],
    target_roots: &[PathBuf],
) -> Vec<RuleFinding> {
    let rel = relative_slash_path(root, path);
    bans.iter()
        .filter(|ban| {
            ban.globset.is_match(&rel)
                || target_roots
                    .iter()
                    .filter(|target_root| *target_root != root && path.starts_with(target_root))
                    .any(|target_root| ban.globset.is_match(relative_slash_path(target_root, path)))
        })
        .map(|ban| RuleFinding {
            rule: RULE_ID.to_string(),
            file: rel.clone(),
            line: 1,
            message: if ban.def.message.is_empty() {
                format!("{rel}: path is banned by `{}`", ban.def.glob)
            } else {
                ban.def.message.clone()
            },
            import: None,
            target: Some(ban.def.glob.clone()),
        })
        .collect()
}

#[cfg(test)]
mod tests;
