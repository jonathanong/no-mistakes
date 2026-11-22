use super::scan::{sql_helper, FileInventory, Occurrence};
use super::CompiledOptions;
use crate::codebase::check_facts::SqlFactContext;
use crate::codebase::postgres::statements::SqlFactSite;
use crate::codebase::postgres::{
    EmbeddedSqlCall, EmbeddedSqlKind, EmbeddedSqlSourcePosition, SqlByteOrigins,
    SqlStatementFileFacts,
};
use anyhow::Result;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
mod gaps;

type VersionSites = BTreeMap<(Option<PathBuf>, Option<usize>, Occurrence), usize>;

pub(super) fn collect(
    options: &CompiledOptions,
    path: &Path,
    facts: &dyn SqlFactContext,
    row: &mut FileInventory,
) -> Result<()> {
    let embedded = facts.embedded_sql(path, &options.embedded)?;
    let mut statements = facts.postgres_statements(path, Some(&options.embedded))?;
    for call in &embedded.calls {
        let version_count = if call.variants.is_empty() {
            usize::from(call.sql_text.is_some())
        } else {
            call.variants.len()
        };
        // Preparation emits one row per SQL-bearing version in call order.
        let (versions, remaining) = statements.split_at(version_count);
        statements = remaining;
        if call.variants.is_empty()
            && (versions.is_empty() || call.kind == EmbeddedSqlKind::Dynamic)
        {
            row.gaps
                .push((call.line as usize, "executor SQL could not be resolved"));
            continue;
        }
        let mut sites = BTreeMap::new();
        let mut gaps = gaps::AlternativeGaps::default();
        for (variant_index, statement) in versions.iter().enumerate() {
            if !gaps.record(call, variant_index, statement) {
                for (site, count) in version_sites(options, call, variant_index, statement) {
                    let previous = sites.entry(site).or_insert(0);
                    *previous = (*previous).max(count);
                }
            }
        }
        gaps.append_to(&mut row.gaps);
        for ((_, _, site), count) in sites {
            row.sites.extend(std::iter::repeat_n(site, count));
        }
    }
    debug_assert!(statements.is_empty());
    Ok(())
}

fn version_sites(
    options: &CompiledOptions,
    call: &EmbeddedSqlCall,
    variant_index: usize,
    statement: &SqlStatementFileFacts,
) -> VersionSites {
    let mut sites = BTreeMap::new();
    if !statement.lock_inventory.is_empty() {
        let positions = Positions::new(call, variant_index);
        for lock in &statement.lock_inventory {
            let (source_file, line, column) = positions.source(lock.line, lock.column);
            let site = Occurrence {
                line,
                column,
                category: lock.category.clone(),
                fingerprint: lock.fingerprint.clone(),
            };
            *sites.entry((source_file, None, site)).or_insert(0) += 1;
        }
    }
    for (index, function) in statement.function_calls.iter().enumerate() {
        let Some(mut helper) = sql_helper(options, function) else {
            continue;
        };
        let imported = statement
            .variant_locations
            .as_ref()
            .and_then(|locations| locations.position(SqlFactSite::Function(index)))
            .filter(|position| position.source_path.is_some());
        let source_line = imported.and_then(|position| {
            helper.line = call.line as usize;
            helper.column = position.source_offset.unwrap_or(0);
            (helper.column == 0).then_some(position.source_line)
        });
        let source_file = imported.and_then(|position| position.source_path.clone());
        *sites.entry((source_file, source_line, helper)).or_insert(0) += 1;
    }
    sites
}

struct Positions<'a> {
    sql: &'a str,
    ascii: bool,
    line_starts: Vec<usize>,
    offsets: &'a SqlByteOrigins,
    positions: &'a [EmbeddedSqlSourcePosition],
    call_line: usize,
    base_line: usize,
}

impl<'a> Positions<'a> {
    fn new(call: &'a EmbeddedSqlCall, variant_index: usize) -> Self {
        let variant = call
            .variants
            .get(variant_index)
            .filter(|_| !call.mirrors_single_variant());
        let (sql, offsets, positions, base_line) = if let Some(variant) = variant {
            (
                variant.sql_text.as_str(),
                &variant.sql_source_offsets,
                variant.sql_source_positions.as_slice(),
                variant.line,
            )
        } else {
            let base = if call.kind == EmbeddedSqlKind::Inline {
                call.line
            } else {
                call.declaration_line.unwrap_or(call.line)
            };
            (
                call.sql_text.as_deref().unwrap_or_default(),
                &call.sql_source_offsets,
                call.sql_source_positions.as_slice(),
                base,
            )
        };
        Self {
            sql,
            ascii: sql.is_ascii(),
            line_starts: std::iter::once(0)
                .chain(sql.match_indices('\n').map(|(offset, _)| offset + 1))
                .collect(),
            offsets,
            positions,
            call_line: call.line as usize,
            base_line: base_line as usize,
        }
    }

    fn source(&self, line: usize, column: usize) -> (Option<PathBuf>, usize, usize) {
        let start = self.positions.partition_point(|position| {
            (position.sql_line as usize, position.sql_column as usize) <= (line, column)
        });
        let position = start
            .checked_sub(1)
            .and_then(|index| self.positions.get(index));
        let path = position.and_then(|position| position.source_path.clone());
        let source_line = if path.is_some() {
            self.call_line
        } else if let Some(position) = position {
            position.source_line as usize + line.saturating_sub(position.sql_line as usize)
        } else {
            self.base_line.saturating_add(line.saturating_sub(1))
        };
        let offset = self
            .byte_offset(line, column)
            .and_then(|offset| self.offsets.get(offset));
        (
            path,
            source_line,
            offset.map_or(column, |offset| offset as usize),
        )
    }

    fn byte_offset(&self, line: usize, column: usize) -> Option<usize> {
        let start = *self.line_starts.get(line.checked_sub(1)?)?;
        let column = column.checked_sub(1)?;
        if self.ascii {
            return start.checked_add(column);
        }
        self.sql
            .get(start..)?
            .char_indices()
            .nth(column)
            .map(|(offset, _)| start + offset)
    }
}
