use crate::codebase::postgres::statements::SqlFactSite;
use crate::codebase::postgres::{EmbeddedSqlCall, EmbeddedSqlKind, SqlStatementFileFacts};

pub(super) fn rebase_embedded_lines(
    facts: &mut SqlStatementFileFacts,
    call: &EmbeddedSqlCall,
    variant: bool,
) {
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
    let mut locations = facts.variant_locations.take();
    if let Some(locations) = &mut locations {
        let sql = call.sql_text.as_deref().unwrap_or_default();
        for position in locations.positions.values_mut() {
            position.source_line = source_line(position.sql_line, position.sql_column);
            position.source_offset =
                call.source_offset_at_sql_position(sql, position.sql_line, position.sql_column);
        }
    }
    let physical_line = |site: SqlFactSite, line: usize, column: usize| {
        locations
            .as_ref()
            .and_then(|locations| locations.position(site))
            .map_or_else(
                || source_line(line, column),
                |position| position.source_line,
            )
    };
    for (i, setting) in facts.setting_uses.iter_mut().enumerate() {
        setting.line = physical_line(SqlFactSite::Setting(i), setting.line, 1);
    }
    for (i, function) in facts.function_calls.iter_mut().enumerate() {
        function.line = physical_line(SqlFactSite::Function(i), function.line, function.column);
    }
    for (i, statement) in facts.statement_kinds.iter_mut().enumerate() {
        statement.line = physical_line(SqlFactSite::StatementKind(i), statement.line, 1);
    }
    for (i, offset) in facts.offset_uses.iter_mut().enumerate() {
        offset.line = physical_line(SqlFactSite::Offset(i), offset.line, offset.column);
    }
    for (i, write) in facts.writes.iter_mut().enumerate() {
        write.line = if variant {
            physical_line(SqlFactSite::Write(i), write.line, 1)
        } else {
            call.line.max(1) as usize
        };
    }
    for (i, insert) in facts.inserts.iter_mut().enumerate() {
        insert.line = physical_line(SqlFactSite::Insert(i), insert.line, 1);
    }
    for (i, select) in facts.selects.iter_mut().enumerate() {
        select.line = physical_line(SqlFactSite::Select(i), select.line, 1);
        for (j, relation) in select.relations.iter_mut().enumerate() {
            relation.line = physical_line(SqlFactSite::Relation(i, j), relation.line, 1);
        }
        for (j, (line, column)) in select
            .not_in_subqueries
            .iter_mut()
            .zip(&select.not_in_columns)
            .enumerate()
        {
            *line = physical_line(SqlFactSite::NotIn(i, j), *line, *column);
        }
        for (j, count) in select.count_existence_checks.iter_mut().enumerate() {
            count.line = physical_line(SqlFactSite::Count(i, j), count.line, count.column);
        }
        for (j, exists) in select.exists_set_operations.iter_mut().enumerate() {
            exists.line = physical_line(SqlFactSite::Exists(i, j), exists.line, exists.column);
        }
        for (j, star) in select.star_projections.iter_mut().enumerate() {
            star.line = physical_line(SqlFactSite::Star(i, j), star.line, 1);
        }
        for (j, column) in select.column_uses.iter_mut().enumerate() {
            column.line = physical_line(SqlFactSite::Column(i, j), column.line, 1);
        }
    }
    for (i, use_) in facts.mutation_column_uses.iter_mut().enumerate() {
        use_.line = physical_line(SqlFactSite::MutationColumn(i), use_.line, 1);
    }
    for (i, star) in facts.returning_stars.iter_mut().enumerate() {
        star.line = physical_line(SqlFactSite::ReturningStar(i), star.line, 1);
    }
    for (i, group) in facts.updates.iter_mut().enumerate() {
        for (j, relation) in group.iter_mut().enumerate() {
            relation.line = physical_line(SqlFactSite::UpdateRelation(i, j), relation.line, 1);
        }
    }
    for (i, group) in facts.deletes.iter_mut().enumerate() {
        for (j, relation) in group.iter_mut().enumerate() {
            relation.line = physical_line(SqlFactSite::DeleteRelation(i, j), relation.line, 1);
        }
    }
    rebase_source_lines(facts, &source_line);
    facts.variant_locations = locations;
}

fn rebase_source_lines(
    facts: &mut SqlStatementFileFacts,
    source_line: &impl Fn(usize, usize) -> usize,
) {
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
        bound.map_lines(source_line);
    }
    if let Some(lifecycle) = &mut facts.lifecycle {
        for bound in &mut lifecycle.raw_bounds {
            bound.map_lines(source_line);
        }
    }
}
