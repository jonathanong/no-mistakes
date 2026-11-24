use super::{
    policy::{relation_matches, Mode},
    CompiledOptions, RuleFinding, CHECKS, RULE_ID,
};
use crate::codebase::{
    check_facts::SqlFactContext,
    postgres::{EmbeddedSqlCall, SchemaCatalog},
    ts_source::{relative_slash_path, SourceStore},
};
use std::{
    collections::{BTreeMap, BTreeSet},
    path::{Path, PathBuf},
};
mod ddl;
mod output;
mod rows;
mod sequences;
mod settings;
mod statements;
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
    let enabled = CHECKS
        .iter()
        .any(|check| options.mode(check) != Mode::Ignore);
    let catalog = options
        .schema_catalog_path
        .as_deref()
        .filter(|_| {
            enabled
                && files
                    .iter()
                    .any(|path| crate::codebase::dependencies::extract::is_indexable(path))
        })
        .map(|path| facts.prepared().postgres_ordering_catalog(path))
        .transpose()?;
    let triggers = triggers::Triggers::new(catalog);
    let all: anyhow::Result<Vec<_>> = files
        .par_iter()
        .filter(|path| crate::codebase::dependencies::extract::is_indexable(path))
        .map(|path| scan_file(root, options, path, sources, facts, &triggers, catalog))
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
    catalog: Option<&SchemaCatalog>,
) -> anyhow::Result<crate::codebase::rules::PostgresFindings> {
    let mut output = Output::new(options, facts);
    let embedded = facts.prepared().embedded_sql(path, &options.embedded)?;
    let statements = facts
        .prepared()
        .postgres_statements(path, Some(&options.embedded))?;
    let file = relative_slash_path(root, path);
    let source = super::super::read_language_source(sources, path).unwrap_or_default();
    options.checked_files.insert(file.clone());
    let enabled = |check: &str| options.mode(check) != Mode::Ignore;
    if !CHECKS.iter().any(|check| enabled(check)) {
        return Ok(output.into_findings());
    }
    let check_written = enabled(CHECKS[0]);
    let check_outside = enabled(CHECKS[3]);
    let grouped = statements::group(statements);
    let unresolved = statements::unresolved_transactions(&embedded.calls, &grouped);
    let catalog_invalidations =
        catalog.map(|_| statements::CatalogInvalidations::collect(&embedded.calls, &grouped));
    sequences::scan(
        &mut output,
        (&file, &source),
        &embedded.calls,
        &grouped,
        &unresolved,
        &options.embedded,
        catalog.zip(catalog_invalidations.as_ref()),
    );
    let exemptions = ddl::exemptions(&embedded.calls, &grouped, &unresolved, catalog);
    let trigger_disabled = settings::trigger_disabled_transactions(&embedded.calls, &grouped);
    let mut writes = writes::Writes::default();
    let mut trigger_relations: BTreeMap<u32, BTreeSet<String>> = BTreeMap::new();
    for (index, call) in embedded.calls.iter().enumerate() {
        if let Some(id) = call.transaction_id.or(call.caller_transaction_id) {
            if let Some(versions) = grouped.get(&index) {
                if versions.len() == 1
                    && call.execution_unconditional
                    && (call.executor_transaction_known || call.execution_awaited)
                {
                    if call.executor_transaction_known && !trigger_disabled.contains(&id) {
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
                let self_contained = access.self_contained_locks.get(&lock_index);
                if !call.executor_transaction_known {
                    let lock_catalog = statements::CatalogInvalidations::for_lock(
                        catalog.zip(catalog_invalidations.as_ref()),
                        call,
                        lock.relation.as_deref(),
                        &unresolved,
                    );
                    rows::check_membership(
                        &mut output,
                        &site,
                        lock,
                        rows::same_statement(call, lock, access, lock_index, lock_catalog),
                        &writes,
                        &unresolved,
                    );
                } else if let Some(id) = call.transaction_id {
                    if rows::same_statement(call, lock, access, lock_index, None) {
                        continue;
                    }
                    let written_by_trigger = lock.relation.as_deref().is_some_and(|relation| {
                        trigger_relations.get(&id).is_some_and(|relations| {
                            relations.contains(triggers.canonical(relation))
                        })
                    });
                    rows::check_transaction(
                        &mut output,
                        &site,
                        lock,
                        &writes,
                        id,
                        unresolved.contains(&id),
                        written_by_trigger,
                    );
                } else if self_contained.is_none() {
                    output.emit(&site, CHECKS[3], lock.relation.as_deref(), None, "row lock in autocommit is released immediately; move the lock and write into the same transaction");
                }
            }
        }
    }
    Ok(output.into_findings())
}
