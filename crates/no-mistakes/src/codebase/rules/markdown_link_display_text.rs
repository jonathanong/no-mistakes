use super::RuleFinding;
use crate::codebase::ts_source::{byte_offset_to_line, relative_slash_path};
use crate::config::v2::NoMistakesConfig;
use anyhow::Result;
use rayon::prelude::*;
use serde::Deserialize;
use std::path::{Path, PathBuf};

mod parser;
use parser::{inline_links_outside_code, strip_fenced_code, InlineLink};

pub const RULE_ID: &str = "markdown-link-display-text";

const DEFAULT_EXTENSIONS: &[&str] = &[".md"];

#[derive(Deserialize, Default)]
#[serde(default, rename_all = "camelCase")]
pub(crate) struct Options {
    pub(crate) extensions: Vec<String>,
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
                .filter(|p| super::file_allowed_by_roots_and_skip(root, &skip, p, &target_roots))
                .cloned()
                .collect();
            let files = super::path_filter::filter_rule_files(root, config, rule, &files)?;
            scan(root, &opts, &files)
        })
        .collect();
    let mut findings: Vec<RuleFinding> = all?.into_iter().flatten().collect();
    super::sort_findings(&mut findings);
    Ok(findings)
}

fn scan(root: &Path, opts: &Options, files: &[PathBuf]) -> Result<Vec<RuleFinding>> {
    let extensions = effective_extensions(opts);
    let mut findings: Vec<RuleFinding> = files
        .par_iter()
        .flat_map(|path| check_file(root, path, &extensions))
        .collect();
    super::sort_findings(&mut findings);
    Ok(findings)
}

fn effective_extensions(opts: &Options) -> Vec<&str> {
    if opts.extensions.is_empty() {
        DEFAULT_EXTENSIONS.to_vec()
    } else {
        opts.extensions.iter().map(String::as_str).collect()
    }
}

fn check_file(root: &Path, path: &Path, extensions: &[&str]) -> Vec<RuleFinding> {
    let rel = relative_slash_path(root, path);
    if !extensions.iter().any(|ext| rel.ends_with(ext)) {
        return Vec::new();
    }
    let Ok(source) = std::fs::read_to_string(path) else {
        return Vec::new();
    };
    let fenced = strip_fenced_code(&source);
    inline_links_outside_code(&source)
        .into_iter()
        .filter_map(|link| finding_for_link(&rel, &fenced, link, extensions))
        .collect()
}

fn finding_for_link(
    file: &str,
    source: &str,
    link: InlineLink,
    extensions: &[&str],
) -> Option<RuleFinding> {
    let text = link.text.replace('`', "");
    if !looks_like_md_filename(&text, extensions) || is_non_local_href(&link.href) {
        return None;
    }
    let basename = href_basename(&link.href)?;
    if basename == text {
        return None;
    }
    Some(RuleFinding {
        rule: RULE_ID.to_string(),
        file: file.to_string(),
        line: byte_offset_to_line(source, link.offset) as usize,
        message: format!(
            "{file}: link text \"{text}\" does not match target basename \"{basename}\""
        ),
        import: Some(text),
        target: Some(basename),
    })
}

fn looks_like_md_filename(text: &str, extensions: &[&str]) -> bool {
    extensions.iter().any(|extension| text.ends_with(extension))
        && !text.is_empty()
        && !text
            .chars()
            .any(|ch| ch == '/' || ch == '\\' || ch.is_whitespace())
}

fn href_basename(href: &str) -> Option<String> {
    let bare = href_destination(href);
    let before_fragment = bare.split('#').next().unwrap_or_default();
    let before_query = before_fragment.split('?').next().unwrap_or_default();
    if before_query.ends_with('/') {
        return None;
    }
    before_query
        .rsplit('/')
        .next()
        .filter(|basename| !basename.is_empty())
        .map(ToString::to_string)
}

fn is_non_local_href(href: &str) -> bool {
    let bare = href_destination(href);
    bare.starts_with('#')
        || bare.starts_with("http://")
        || bare.starts_with("https://")
        || bare.starts_with("mailto:")
        || bare.starts_with("//")
}

fn href_destination(value: &str) -> &str {
    let trimmed = value.trim();
    if let Some(rest) = trimmed.strip_prefix('<') {
        if let Some(end) = rest.find('>') {
            &rest[..end]
        } else {
            trimmed
        }
    } else {
        trimmed.split_ascii_whitespace().next().unwrap_or_default()
    }
}

#[cfg(test)]
mod tests;
