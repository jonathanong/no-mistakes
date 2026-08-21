use super::order::cmp_sql_rel;
use super::redundancy::{describe_index, is_redundant_prefix, LiveIndex};
use super::{sql_rel, CompiledOptions, RuleFinding, RULE_ID};
use crate::codebase::check_facts::CheckFactPlan;
use crate::codebase::postgres::{collect_postgres_facts, SqlDropIndexMetadata, SqlSchemaFileFacts};
use crate::codebase::ts_source::SourceStore;
use anyhow::Context;
use std::cmp::Ordering;
use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

struct LiveDrop {
    file: String,
    name: String,
    line: usize,
}

pub(super) fn scan(
    root: &Path,
    opts: &CompiledOptions,
    files: &[PathBuf],
    sources: &SourceStore,
) -> anyhow::Result<Vec<RuleFinding>> {
    let facts = collect_postgres_facts(
        root,
        sources,
        files,
        &CheckFactPlan {
            postgres_schema: true,
            ..CheckFactPlan::default()
        },
        &opts.schema,
        &Default::default(),
    )
    .context(format!("{RULE_ID} failed to collect PostgreSQL facts"))?;
    let index_drops = named_drops(root, &facts.schema, |file| &file.dropped_indexes);
    let table_drops = named_drops(root, &facts.schema, |file| &file.dropped_tables);
    let indexes = live_indexes(root, &facts.schema, &index_drops, &table_drops);
    let mut used = BTreeSet::new();
    let mut findings = Vec::new();
    for table_indexes in indexes.values() {
        findings.extend(scan_table(table_indexes, opts, sources, &mut used));
    }
    findings.extend(stale_allowlist(opts, &used));
    Ok(findings)
}

fn scan_table(
    table_indexes: &[LiveIndex<'_>],
    opts: &CompiledOptions,
    sources: &SourceStore,
    used: &mut BTreeSet<String>,
) -> Vec<RuleFinding> {
    let mut findings = Vec::new();
    for shorter in table_indexes {
        let Some(name) = shorter.index.name.as_deref() else {
            continue;
        };
        let Some(longer) = table_indexes
            .iter()
            .find(|candidate| is_redundant_prefix(shorter, candidate))
        else {
            continue;
        };
        let source = sources
            .read_path(shorter.path)
            .map(|source| source.to_string())
            .unwrap_or_default();
        if directive_on_line(&source, shorter.index.line, &opts.allow_directive) {
            continue;
        }
        let key = format!("{}.{}", shorter.index.table_name, name);
        if opts.allowed_indexes.contains(&key) {
            used.insert(key);
            continue;
        }
        findings.push(finding(
            &shorter.rel,
            shorter.index.line.max(1),
            format!(
                "{key}: redundant index — its columns are a strict prefix of {} with the same predicate; drop it",
                describe_index(longer.index)
            ),
            &key,
        ));
    }
    findings
}

fn live_indexes<'a>(
    root: &Path,
    schema: &'a [SqlSchemaFileFacts],
    index_drops: &[LiveDrop],
    table_drops: &[LiveDrop],
) -> BTreeMap<String, Vec<LiveIndex<'a>>> {
    let mut indexes = BTreeMap::<String, Vec<LiveIndex<'a>>>::new();
    for file in schema {
        let rel = sql_rel(root, &file.path);
        for index in &file.indexes {
            if dropped_later(index_drops, index.name.as_deref(), &rel, index.line)
                || dropped_later(
                    table_drops,
                    Some(index.table_name.as_str()),
                    &rel,
                    index.line,
                )
            {
                continue;
            }
            indexes
                .entry(index.table_name.clone())
                .or_default()
                .push(LiveIndex {
                    rel: rel.clone(),
                    path: &file.path,
                    index,
                });
        }
    }
    indexes
}

fn named_drops(
    root: &Path,
    schema: &[SqlSchemaFileFacts],
    names: impl Fn(&SqlSchemaFileFacts) -> &[SqlDropIndexMetadata],
) -> Vec<LiveDrop> {
    schema
        .iter()
        .flat_map(|file| {
            let rel = sql_rel(root, &file.path);
            names(file).iter().map(move |drop| LiveDrop {
                file: rel.clone(),
                name: drop.name.clone(),
                line: drop.line,
            })
        })
        .collect()
}

fn dropped_later(drops: &[LiveDrop], name: Option<&str>, file: &str, line: usize) -> bool {
    let Some(name) = name else {
        return false;
    };
    drops
        .iter()
        .any(|drop| drop.name == name && is_later(drop, file, line))
}

fn is_later(drop: &LiveDrop, index_file: &str, index_line: usize) -> bool {
    match cmp_sql_rel(&drop.file, index_file) {
        Ordering::Greater => true,
        Ordering::Equal => drop.line > index_line,
        Ordering::Less => false,
    }
}

pub(super) fn directive_on_line(source: &str, line: usize, directive: &str) -> bool {
    !directive.is_empty()
        && line != 0
        && source
            .lines()
            .nth(line - 1)
            .is_some_and(|text| text.contains("--") && text.contains(directive))
}

fn stale_allowlist(opts: &CompiledOptions, used: &BTreeSet<String>) -> Vec<RuleFinding> {
    opts.allowed_indexes
        .iter()
        .filter(|entry| !used.contains(*entry))
        .map(|entry| {
            finding(
                entry,
                1,
                format!("stale postgres-redundant-index allowedIndexes entry: {entry}"),
                entry,
            )
        })
        .collect()
}

fn finding(file: &str, line: usize, message: String, target: &str) -> RuleFinding {
    RuleFinding {
        rule: RULE_ID.to_string(),
        file: file.to_string(),
        line,
        message,
        import: None,
        target: Some(target.to_string()),
    }
}
