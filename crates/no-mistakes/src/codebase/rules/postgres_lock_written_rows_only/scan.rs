use super::{
    policy::{relation_matches, Mode},
    CompiledOptions, RuleFinding, CHECKS, RULE_ID,
};
use crate::codebase::{
    check_facts::SqlFactContext,
    postgres::{EmbeddedSqlCall, SqlStatementFileFacts},
    ts_source::{relative_slash_path, SourceStore},
};
use std::{
    collections::{BTreeMap, BTreeSet},
    path::{Path, PathBuf},
};
mod ddl;
mod output;
mod rows;
mod settings;
mod triggers;
mod writes;
use output::{Output, Site};
use rayon::prelude::*;

pub(super) fn scan(
    root: &Path,
    options: &CompiledOptions,
    files: &[PathBuf],
    sources: &SourceStore,
    facts: &dyn SqlFactContext,
) -> anyhow::Result<crate::codebase::rules::PostgresFindings> {
    let catalog = options
        .schema_catalog_path
        .as_deref()
        .map(|path| facts.prepared().postgres_schema_catalog(path))
        .transpose()?;
    let triggers = triggers::Triggers::new(catalog);
    let all: anyhow::Result<Vec<_>> = files
        .par_iter()
        .filter(|path| crate::codebase::dependencies::extract::is_indexable(path))
        .map(|path| scan_file(root, options, path, sources, facts, &triggers))
        .collect();
    let mut output = Output::new(options, facts);
    for findings in all? {
        output.extend(findings);
    }
    Ok(output.finish())
}
fn scan_file(
    root: &Path,
    options: &CompiledOptions,
    path: &Path,
    sources: &SourceStore,
    facts: &dyn SqlFactContext,
    triggers: &triggers::Triggers<'_>,
) -> anyhow::Result<crate::codebase::rules::PostgresFindings> {
    let mut output = Output::new(options, facts);
    let embedded = facts.prepared().embedded_sql(path, &options.embedded)?;
    let statements = facts
        .prepared()
        .postgres_statements(path, Some(&options.embedded))?;
    let file = relative_slash_path(root, path);
    let source = super::super::read_language_source(sources, path).unwrap_or_default();
    options.checked_files.insert(file.clone());
    let enabled =
        |check: &str| options.checks.get(check).copied().unwrap_or_default() != Mode::Ignore;
    if !CHECKS.iter().any(|check| enabled(check)) {
        return Ok(output.into_findings());
    }
    let catalog = options
        .schema_catalog_path
        .as_deref()
        .map(|path| facts.prepared().postgres_ordering_catalog(path))
        .transpose()?;
    let check_written = enabled(CHECKS[0]);
    let check_outside = enabled(CHECKS[3]);
    let mut grouped: BTreeMap<usize, Vec<&SqlStatementFileFacts>> = BTreeMap::new();
    for (index, statement) in statements.iter().filter_map(|statement| {
        statement
            .variant_locations
            .as_ref()
            .map(|locations| (locations.call_index, statement))
    }) {
        grouped.entry(index).or_default().push(statement);
    }
    let unresolved: BTreeSet<_> = embedded
        .calls
        .iter()
        .enumerate()
        .filter(|(index, call)| {
            (!call.executor_transaction_known || call.is_unanalyzable())
                || grouped.get(index).is_none_or(|versions| {
                    versions.len() != 1
                        || versions.iter().any(|version| {
                            (version.parse_failed && !version.lock_writes.ddl_recovery_complete)
                                || version
                                    .lock_writes
                                    .writes
                                    .iter()
                                    .any(|write| write.relation.is_none())
                                || version.lock_writes.transaction_boundary
                                || version.lock_writes.relation_identity_changed
                                || version
                                    .setting_uses
                                    .iter()
                                    .any(|setting| setting.name == "search_path")
                                || (!call.execution_unconditional
                                    && !version.lock_writes.writes.is_empty())
                        })
                })
        })
        .filter_map(|(_, call)| call.transaction_id)
        .collect();
    let exemptions = ddl::exemptions(&embedded.calls, &grouped, &unresolved, catalog);
    let trigger_disabled = settings::trigger_disabled_transactions(&embedded.calls, &grouped);
    let mut writes = writes::Writes::default();
    let mut trigger_relations: BTreeMap<u32, BTreeSet<String>> = BTreeMap::new();
    for (index, call) in embedded.calls.iter().enumerate() {
        if let Some(id) = call.transaction_id {
            if let Some(versions) = grouped.get(&index) {
                if versions.len() == 1
                    && call.execution_unconditional
                    && call.executor_transaction_known
                {
                    if !trigger_disabled.contains(&id) {
                        trigger_relations
                            .entry(id)
                            .or_default()
                            .extend(triggers.relations(&versions[0].lock_writes.trigger_writes));
                    }
                    for write in &versions[0].lock_writes.writes {
                        writes.insert(id, write, call);
                    }
                }
            }
        }
    }
    for (index, call) in embedded.calls.iter().enumerate() {
        if call.helper_definition {
            continue;
        }
        let site = Site {
            file: &file,
            source: &source,
            call,
        };
        let Some(versions) = grouped.get(&index).filter(|versions| {
            !call.is_unanalyzable()
                && !versions.iter().any(|version| {
                    version.parse_failed && !version.lock_writes.ddl_recovery_complete
                })
        }) else {
            output.unresolved(&site, "executed SQL cannot be recovered completely; use static SQL so lock scope and writes can be checked");
            continue;
        };
        for version in versions {
            let access = &version.lock_writes;
            for (lock, relation) in access.table_locks.iter().enumerate() {
                if exemptions.contains(&(index, lock)) {
                    continue;
                }
                output.emit(
                    &site,
                    CHECKS[1],
                    relation.as_deref(),
                    None,
                    "LOCK TABLE widens contention; lock only the row being written",
                );
            }
            for keys in &access.advisory_keys {
                if let Some(key) = rows::constant_key(keys, call) {
                    output.emit(&site, CHECKS[2], None, Some(&key), "constant advisory key serializes unrelated writes; use a key derived from the written row");
                }
            }
            for (lock_index, lock) in access.locks.iter().enumerate() {
                if !rows::enabled_for(call, check_written, check_outside) {
                    continue;
                }
                if !call.executor_transaction_known {
                    output.unresolved(&site, "transaction membership is unresolved; acquire the lock through a configured transaction factory");
                } else if let Some(id) = call.transaction_id {
                    if unresolved.contains(&id) {
                        output.unresolved(&site, "transaction SQL is unresolved; make every write statically visible before comparing the lock with its write set");
                    } else if let Some(relation) = &lock.relation {
                        if trigger_relations.get(&id).is_some_and(|relations| {
                            relations.contains(triggers.canonical(relation))
                        }) {
                            continue;
                        }
                        let (candidates, ambiguous) = writes.for_relation(id, relation);
                        rows::check(&mut output, &site, lock, candidates, relation, ambiguous);
                    } else {
                        output.unresolved(
                            &site,
                            "locked relation is unresolved; lock a single explicit base relation",
                        );
                    }
                } else if !access.self_contained_locks.contains(&lock_index) {
                    output.emit(&site, CHECKS[3], lock.relation.as_deref(), None, "row lock in autocommit is released immediately; move the lock and write into the same transaction");
                }
            }
        }
    }
    Ok(output.into_findings())
}
