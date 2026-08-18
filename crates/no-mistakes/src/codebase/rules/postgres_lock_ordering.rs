use super::path_filter::GlobMatcher;
use super::RuleFinding;
use crate::codebase::check_facts::CheckFactPlan;
use crate::codebase::postgres::{
    collect_postgres_facts, extract_locking_select_metadata, EmbeddedSqlOptions,
    PostgresSchemaOptions,
};
use crate::codebase::ts_source::relative_slash_path;
use crate::config::v2::NoMistakesConfig;
use anyhow::{Context, Result};
use rayon::prelude::*;
use serde::Deserialize;
use std::path::{Path, PathBuf};

pub const RULE_ID: &str = "postgres-lock-ordering";

const DEFAULT_SAFE_DIRECTIVE: &str = "deadlock-safe";
const DIRECTIVE_LOOKBACK: usize = 200;
const UNPARSEABLE_TARGET: &str = "unparseable";
const LOCK_ORDERING_TARGET: &str = "lock-ordering";

#[derive(Deserialize, Default)]
#[serde(default, rename_all = "camelCase")]
pub(crate) struct Options {
    pub(crate) include: Vec<String>,
    pub(crate) exclude: Vec<String>,
    pub(crate) import_specifier: String,
    pub(crate) executor_names: Vec<String>,
    pub(crate) safe_directive: String,
}

struct CompiledOptions {
    include: GlobMatcher,
    exclude: GlobMatcher,
    embedded: EmbeddedSqlOptions,
    safe_directive: String,
}

impl CompiledOptions {
    fn includes(&self, rel: &str) -> bool {
        (self.include.is_empty() || self.include.is_match(rel))
            && (self.exclude.is_empty() || !self.exclude.is_match(rel))
    }
}

pub(crate) fn check_with_files(
    root: &Path,
    config: &NoMistakesConfig,
    all_files: &[PathBuf],
) -> Result<Vec<RuleFinding>> {
    let sources = super::source_store_for_files(all_files);
    check_with_files_and_sources(root, config, all_files, &sources)
}

pub(crate) fn check_with_files_and_sources(
    root: &Path,
    config: &NoMistakesConfig,
    all_files: &[PathBuf],
    sources: &crate::codebase::ts_source::SourceStore,
) -> Result<Vec<RuleFinding>> {
    let all: Result<Vec<Vec<RuleFinding>>> = config
        .rule_applications(RULE_ID)
        .into_par_iter()
        .map(|rule| -> Result<Vec<RuleFinding>> {
            let opts: Options = rule.rule_options();
            let compiled = compile_options(&opts)?;
            let target_roots = super::target_roots(root, config, rule);
            let skip = super::skip_dir_set(config);
            let files: Vec<PathBuf> = all_files
                .iter()
                .filter(|path| {
                    super::file_allowed_by_roots_and_skip(root, &skip, path, &target_roots)
                })
                .cloned()
                .collect();
            let files = super::path_filter::filter_rule_files(root, config, rule, &files)?;
            let files: Vec<PathBuf> = files
                .into_iter()
                .filter(|path| compiled.includes(&relative_slash_path(root, path)))
                .collect();
            scan_with_sources(root, &compiled, &files, sources)
        })
        .collect();
    let mut findings: Vec<RuleFinding> = all?.into_iter().flatten().collect();
    super::sort_findings(&mut findings);
    Ok(findings)
}

fn compile_options(opts: &Options) -> Result<CompiledOptions> {
    let include = GlobMatcher::new(&opts.include, &format!("{RULE_ID} include"))?;
    let exclude = GlobMatcher::new(&opts.exclude, &format!("{RULE_ID} exclude"))?;
    let defaults = EmbeddedSqlOptions::default();
    Ok(CompiledOptions {
        include,
        exclude,
        embedded: EmbeddedSqlOptions {
            import_specifier: if opts.import_specifier.is_empty() {
                defaults.import_specifier
            } else {
                opts.import_specifier.clone()
            },
            executor_names: if opts.executor_names.is_empty() {
                defaults.executor_names
            } else {
                opts.executor_names.clone()
            },
        },
        safe_directive: if opts.safe_directive.is_empty() {
            DEFAULT_SAFE_DIRECTIVE.to_string()
        } else {
            opts.safe_directive.clone()
        },
    })
}

fn scan_with_sources(
    root: &Path,
    opts: &CompiledOptions,
    files: &[PathBuf],
    sources: &crate::codebase::ts_source::SourceStore,
) -> Result<Vec<RuleFinding>> {
    let facts = collect_postgres_facts(
        root,
        sources,
        files,
        &CheckFactPlan {
            embedded_sql: true,
            ..CheckFactPlan::default()
        },
        &PostgresSchemaOptions::default(),
        &opts.embedded,
    )
    .with_context(|| format!("{RULE_ID} failed to collect embedded SQL facts"))?;
    let mut findings = Vec::new();
    for file in facts.embedded {
        let rel = relative_slash_path(root, &file.path);
        let source = super::read_source(sources, &file.path).unwrap_or_default();
        for call in &file.calls {
            findings.extend(findings_for_call(&rel, &source, call, opts));
        }
    }
    super::sort_findings(&mut findings);
    Ok(findings)
}

fn findings_for_call(
    file: &str,
    source: &str,
    call: &crate::codebase::postgres::EmbeddedSqlCall,
    opts: &CompiledOptions,
) -> Vec<RuleFinding> {
    let Some(sql) = call.sql_text.as_deref() else {
        return Vec::new();
    };
    if !contains_for_update(sql) {
        return Vec::new();
    }
    if has_safe_directive(source, call.line, sql, &opts.safe_directive) {
        return Vec::new();
    }
    match extract_locking_select_metadata(sql) {
        Err(_) => vec![finding(
            file,
            call.line,
            unparseable_message(file, call.line, &opts.safe_directive),
            UNPARSEABLE_TARGET,
        )],
        Ok(locks) => {
            if locks.iter().any(|lock| {
                lock.has_multi_row_predicate && !lock.has_order_by && !lock.skips_locked_rows
            }) {
                vec![finding(
                    file,
                    call.line,
                    lock_ordering_message(file, call.line, &opts.safe_directive),
                    LOCK_ORDERING_TARGET,
                )]
            } else {
                Vec::new()
            }
        }
    }
}

fn contains_for_update(sql: &str) -> bool {
    sql.to_ascii_lowercase().contains("for update")
}

fn has_safe_directive(source: &str, line: u32, sql: &str, directive: &str) -> bool {
    if directive.is_empty() {
        return false;
    }
    comment_contains_directive(&lookback_window(source, line), directive)
        || comment_contains_directive(sql, directive)
}

fn lookback_window(source: &str, line: u32) -> String {
    let offset = call_offset(source, line);
    let start = floor_char_boundary(source, offset.saturating_sub(DIRECTIVE_LOOKBACK));
    let end = floor_char_boundary(source, offset);
    source.get(start..end).unwrap_or("").to_string()
}

fn call_offset(source: &str, line: u32) -> usize {
    let line_start = line_start_offset(source, line);
    let haystack = source.get(line_start..).unwrap_or("");
    let rel = haystack
        .to_ascii_lowercase()
        .find("for update")
        .unwrap_or(0);
    line_start + rel
}

fn line_start_offset(source: &str, line: u32) -> usize {
    if line <= 1 {
        return 0;
    }
    source
        .match_indices('\n')
        .nth(line.saturating_sub(2) as usize)
        .map(|(idx, _)| idx + 1)
        .unwrap_or(source.len())
}

fn floor_char_boundary(source: &str, index: usize) -> usize {
    if index >= source.len() {
        return source.len();
    }
    if source.is_char_boundary(index) {
        return index;
    }
    (0..index)
        .rev()
        .find(|idx| source.is_char_boundary(*idx))
        .unwrap_or(0)
}

fn comment_contains_directive(text: &str, directive: &str) -> bool {
    let bytes = text.as_bytes();
    let mut index = 0;
    while index < bytes.len() {
        if bytes[index] == b'/' && bytes.get(index + 1) == Some(&b'*') {
            let start = index + 2;
            let rest = &text[start..];
            if let Some(end) = rest.find("*/") {
                if rest[..end].contains(directive) {
                    return true;
                }
                index = start + end + 2;
                continue;
            }
            return rest.contains(directive);
        }
        if bytes[index] == b'-' && bytes.get(index + 1) == Some(&b'-') {
            let start = index + 2;
            let rest = &text[start..];
            let end = rest.find('\n').unwrap_or(rest.len());
            if rest[..end].contains(directive) {
                return true;
            }
            index = start + end;
            continue;
        }
        index += 1;
    }
    false
}

fn finding(file: &str, line: u32, message: String, target: &str) -> RuleFinding {
    RuleFinding {
        rule: RULE_ID.to_string(),
        file: file.to_string(),
        line: line as usize,
        message,
        import: None,
        target: Some(target.to_string()),
    }
}

fn lock_ordering_message(file: &str, line: u32, directive: &str) -> String {
    format!(
        "{file}:{line}: multi-row FOR UPDATE without ORDER BY or SKIP LOCKED can deadlock (ABBA); add ORDER BY, use SKIP LOCKED, or add a `{directive}` comment"
    )
}

fn unparseable_message(file: &str, line: u32, directive: &str) -> String {
    format!(
        "{file}:{line}: keep FOR UPDATE SQL parseable so lock ordering can be checked, or add a `{directive}` comment"
    )
}

#[cfg(test)]
mod tests;
