use super::{compile_sql_include, matches_sql_include, read_source};
use crate::codebase::postgres::embedded::{EmbeddedSqlCall, EmbeddedSqlFileFacts, EmbeddedSqlKind};
use crate::codebase::postgres::statement_facts::SqlStatementFileFacts;
use crate::codebase::postgres::statements::extract_sql_statement_facts;
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
) -> Result<Vec<SqlStatementFileFacts>, PostgresFactError> {
    let globs = compile_sql_include(&schema_options.sql_include)?;
    let mut facts = files
        .par_iter()
        .filter(|path| matches_sql_include(root, path, &globs))
        .map(|path| {
            let source = read_source(path, sources)?;
            let mut file = extract_sql_statement_facts(&source);
            file.path = path.clone();
            Ok(file)
        })
        .collect::<Result<Vec<_>, _>>()?;
    for file in embedded {
        facts.extend(embedded_call_facts(file));
    }
    facts.sort_by(|left, right| left.path.cmp(&right.path));
    Ok(facts)
}

pub(crate) fn embedded_call_facts(file: &EmbeddedSqlFileFacts) -> Vec<SqlStatementFileFacts> {
    file.calls
        .iter()
        .filter_map(|call| {
            let sql = call.sql_text.as_deref()?;
            let mut facts = extract_sql_statement_facts(sql);
            if call.kind == EmbeddedSqlKind::Dynamic {
                // Recovered interpolation text can prove OFFSET syntax, while
                // other rules keep their existing dynamic-SQL failure policy.
                facts = SqlStatementFileFacts {
                    offset_uses: facts.offset_uses,
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
            .map(|index| call.sql_source_positions[index].source_line as usize)
            .unwrap_or_else(|| line.saturating_add(shift))
    };
    for offset in &mut facts.offset_uses {
        offset.line = source_line(offset.line, offset.column);
    }
    for insert in &mut facts.inserts {
        insert.line = source_line(insert.line, 1);
    }
    for select in &mut facts.selects {
        select.line = source_line(select.line, 1);
        for relation in &mut select.relations {
            relation.line = source_line(relation.line, 1);
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
}

#[cfg(test)]
mod tests;
