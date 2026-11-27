use super::super::statement_keyword_position;
use crate::{SqlPolicyClause, SqlPolicyClauseUse};
use sqlparser::{
    ast::{
        AlterTableOperation, CascadeOption, DropBehavior, ObjectType, ReferentialAction, Spanned,
        Statement,
    },
    tokenizer::TokenWithSpan,
};

pub(super) fn used(statement: &Statement, tokens: &[TokenWithSpan]) -> Option<SqlPolicyClauseUse> {
    let (keyword, anchor) = match statement {
        Statement::Drop {
            object_type:
                ObjectType::View
                | ObjectType::MaterializedView
                | ObjectType::Table
                | ObjectType::Type
                | ObjectType::Schema
                | ObjectType::Index
                | ObjectType::Sequence,
            cascade: true,
            names,
            ..
        } => ("DROP", names.first()?.span().start),
        Statement::DropExtension(extension)
            if extension.cascade_or_restrict == Some(ReferentialAction::Cascade) =>
        {
            ("DROP", extension.names.first()?.span.start)
        }
        Statement::DropDomain(domain) if domain.drop_behavior == Some(DropBehavior::Cascade) => {
            ("DROP", domain.name.span().start)
        }
        Statement::DropTrigger(trigger)
            if trigger.table_name.is_some()
                && trigger.option == Some(ReferentialAction::Cascade) =>
        {
            ("DROP", trigger.trigger_name.span().start)
        }
        Statement::DropFunction(function)
            if function.drop_behavior == Some(DropBehavior::Cascade) =>
        {
            ("DROP", function.func_desc.first()?.name.span().start)
        }
        Statement::AlterTable(table)
            if table.operations.iter().any(|operation| {
                matches!(
                    operation,
                    AlterTableOperation::DropColumn { drop_behavior, .. }
                        | AlterTableOperation::DropConstraint { drop_behavior, .. }
                        if *drop_behavior == Some(DropBehavior::Cascade)
                )
            }) =>
        {
            ("ALTER", table.name.span().start)
        }
        Statement::Truncate(truncate) if truncate.cascade == Some(CascadeOption::Cascade) => {
            ("TRUNCATE", truncate.table_names.first()?.name.span().start)
        }
        _ => return None,
    };
    let position = statement_keyword_position(tokens, keyword, anchor);
    Some(SqlPolicyClauseUse {
        clause: SqlPolicyClause::DropCascade,
        line: position.line.max(1) as usize,
        column: position.column.max(1) as usize,
    })
}
