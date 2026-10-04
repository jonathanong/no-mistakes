use super::{compile_sql_include, matches_sql_include, read_source};
use crate::codebase::postgres::embedded::{EmbeddedSqlCall, EmbeddedSqlFileFacts, EmbeddedSqlKind};
use crate::codebase::postgres::statement_facts::SqlStatementFileFacts;
use crate::codebase::postgres::statements::{
    extract_sql_statement_facts_with_bounds,
    extract_sql_statement_facts_with_recovered_placeholders,
};
use crate::codebase::postgres::types::{PostgresFactError, PostgresSchemaOptions};
use crate::codebase::ts_source::SourceStore;
use rayon::prelude::*;
use std::path::{Path, PathBuf};

pub(super) fn collect(
    root: &Path,
    sources: &SourceStore,
    files: &[PathBuf],
    schema_options: &PostgresSchemaOptions,
    embedded: &[EmbeddedSqlFileFacts],
    collect_bounds: bool,
) -> Result<Vec<SqlStatementFileFacts>, PostgresFactError> {
    let globs = compile_sql_include(&schema_options.sql_include)?;
    let mut facts = files
        .par_iter()
        .filter(|path| matches_sql_include(root, path, &globs))
        .map(|path| {
            let source = read_source(path, sources)?;
            let mut file = extract_sql_statement_facts_with_bounds(&source, collect_bounds);
            file.path = path.clone();
            Ok(file)
        })
        .collect::<Result<Vec<_>, _>>()?;
    for file in embedded {
        facts.extend(embedded_call_facts(file, collect_bounds));
    }
    facts.sort_by(|left, right| left.path.cmp(&right.path));
    Ok(facts)
}

pub(crate) fn embedded_call_facts(
    file: &EmbeddedSqlFileFacts,
    collect_bounds: bool,
) -> Vec<SqlStatementFileFacts> {
    file.calls
        .iter()
        .filter_map(|call| {
            let sql = call.sql_text.as_deref()?;
            let mut facts = extract_sql_statement_facts_with_recovered_placeholders(
                sql,
                collect_bounds,
                &call.recovered_placeholder_positions,
            );
            if call.kind == EmbeddedSqlKind::Dynamic {
                // Recovered interpolation text can prove OFFSET syntax and write
                // targets. Other rules keep their existing dynamic-SQL failure policy.
                facts = SqlStatementFileFacts {
                    offset_uses: facts.offset_uses,
                    writes: facts.writes,
                    ..Default::default()
                };
            }
            facts.path = file.path.clone();
            rebase_embedded_lines(&mut facts, call);
            Some(facts)
        })
        .collect()
}

fn rebase_embedded_lines(facts: &mut SqlStatementFileFacts, call: &EmbeddedSqlCall) {
    let base = match call.kind {
        EmbeddedSqlKind::Inline => call.line,
        _ => call.declaration_line.unwrap_or(call.line),
    } as usize;
    facts.origin_line = base;
    let shift = base.saturating_sub(1);
    let source_line = |line: usize, column: usize| {
        call.sql_source_positions
            .partition_point(|position| {
                (position.sql_line as usize, position.sql_column as usize) <= (line, column)
            })
            .checked_sub(1)
            .map(|index| {
                let position = &call.sql_source_positions[index];
                position.source_line as usize + line - position.sql_line as usize
            })
            .unwrap_or_else(|| line.saturating_add(shift))
    };
    for offset in &mut facts.offset_uses {
        offset.line = source_line(offset.line, offset.column);
    }
    for write in &mut facts.writes {
        write.line = call.line.max(1) as usize;
    }
    for insert in &mut facts.inserts {
        insert.line = source_line(insert.line, 1);
    }
    for select in &mut facts.selects {
        select.line = source_line(select.line, 1);
        for relation in &mut select.relations {
            relation.line = source_line(relation.line, 1);
        }
        for (line, column) in select
            .not_in_subqueries
            .iter_mut()
            .zip(&select.not_in_columns)
        {
            *line = source_line(*line, *column);
        }
        for count in &mut select.count_existence_checks {
            count.line = source_line(count.line, count.column);
        }
        for exists in &mut select.exists_set_operations {
            exists.line = source_line(exists.line, exists.column);
        }
        for star in &mut select.star_projections {
            star.line = source_line(star.line, 1);
        }
        for column in &mut select.column_uses {
            column.line = source_line(column.line, 1);
        }
    }
    for use_ in &mut facts.mutation_column_uses {
        use_.line = source_line(use_.line, 1);
    }
    for star in &mut facts.returning_stars {
        star.line = source_line(star.line, 1);
    }
    for relation in facts
        .updates
        .iter_mut()
        .chain(facts.deletes.iter_mut())
        .flatten()
    {
        relation.line = source_line(relation.line, 1);
    }
    for trigger in &mut facts.triggers {
        trigger.line = source_line(trigger.line, 1);
    }
    for limit in &mut facts.limit_uses {
        limit.line = source_line(limit.line, limit.column);
    }
    for sweep in &mut facts.sweeps {
        sweep.line = source_line(sweep.line, sweep.column);
    }
    for bound in &mut facts.bounds {
        bound.map_lines(&source_line);
    }
    if let Some(lifecycle) = &mut facts.lifecycle {
        for bound in &mut lifecycle.raw_bounds {
            bound.map_lines(&source_line);
        }
    }
}

#[cfg(test)]
mod tests;
