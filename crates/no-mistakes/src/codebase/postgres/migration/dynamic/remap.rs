use crate::codebase::postgres::types::SqlSchemaFileFacts;

pub(in super::super) fn remap_fact_lines(
    facts: &mut SqlSchemaFileFacts,
    dynamic: &super::DynamicSql,
) {
    for event in &mut facts.table_events {
        let mut order = dynamic.source_order.clone();
        order.extend(event.source_order());
        *event.source_order_mut() = order;
        *event.line_mut() = dynamic.source_line(event.line());
    }
    for index in &mut facts.indexes {
        index.line = dynamic.source_line(index.line);
    }
    for index in &mut facts.dropped_indexes {
        index.line = dynamic.source_line(index.line);
    }
    for table in &mut facts.dropped_tables {
        table.line = dynamic.source_line(table.line);
    }
    for key in &mut facts.foreign_keys {
        key.line = dynamic.source_line(key.line);
    }
    for column in &mut facts.add_columns {
        column.line = dynamic.source_line(column.line);
    }
    for constraint in &mut facts.unnamed_constraints {
        constraint.line = dynamic.source_line(constraint.line);
    }
    for setting in &mut facts.setting_uses {
        setting.line = dynamic.source_line(setting.line);
    }
    for statement in &mut facts.statement_kinds {
        statement.line = dynamic.source_line(statement.line);
    }
    for constraint in &mut facts.not_valid_constraints {
        constraint.line = dynamic.source_line(constraint.line);
    }
    for constraint in &mut facts.validated_constraints {
        constraint.line = dynamic.source_line(constraint.line);
    }
    for identifier in &mut facts.declared_identifiers {
        identifier.line = dynamic.source_line(identifier.line);
    }
}
