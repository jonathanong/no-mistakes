use super::recover::failure::keywords::{reserved, type_reserved};
use sqlparser::ast::{
    AccessExpr, AssignmentTarget, Expr, Ident, ObjectName, ObjectNamePart, Statement, Subscript,
    TableAlias, TableFactor, Visit, Visitor,
};
use std::ops::ControlFlow;

pub(super) fn valid(value: &impl Visit) -> bool {
    value
        .visit(&mut QueryIdentifiers {
            array_parents: Vec::new(),
        })
        .is_continue()
}

struct QueryIdentifiers {
    array_parents: Vec<bool>,
}

impl Visitor for QueryIdentifiers {
    type Break = ();

    fn pre_visit_statement(&mut self, statement: &Statement) -> ControlFlow<()> {
        let valid = match statement {
            Statement::CreateTable(table) => table
                .columns
                .iter()
                .all(|column| valid_ident(&column.name) && valid_type(&column.data_type)),
            Statement::Insert(insert) => insert.columns.iter().all(valid_name),
            Statement::Update(update) => {
                update
                    .assignments
                    .iter()
                    .all(|assignment| match &assignment.target {
                        AssignmentTarget::ColumnName(name) => valid_name(name),
                        AssignmentTarget::Tuple(names) => names.iter().all(valid_name),
                    })
            }
            _ => true,
        };
        if valid {
            ControlFlow::Continue(())
        } else {
            ControlFlow::Break(())
        }
    }

    fn pre_visit_query(&mut self, query: &sqlparser::ast::Query) -> ControlFlow<()> {
        if query
            .with
            .as_ref()
            .is_some_and(|with| with.cte_tables.iter().any(|cte| !valid_alias(&cte.alias)))
        {
            ControlFlow::Break(())
        } else {
            ControlFlow::Continue(())
        }
    }

    fn pre_visit_expr(&mut self, expr: &Expr) -> ControlFlow<()> {
        let is_array = matches!(expr, Expr::Array(_));
        // PostgreSQL permits bare brackets only as dimensions inside ARRAY[...].
        if matches!(expr, Expr::Array(array) if !array.named)
            && !self.array_parents.last().copied().unwrap_or(false)
        {
            return ControlFlow::Break(());
        }
        if matches!(expr, Expr::Cast { data_type, .. } if !valid_type(data_type)) {
            return ControlFlow::Break(());
        }
        // sqlparser accepts a slice stride, but PostgreSQL array targets do not.
        if matches!(expr, Expr::CompoundFieldAccess { access_chain, .. }
            if access_chain.iter().any(|access| matches!(access,
                AccessExpr::Subscript(Subscript::Slice { stride: Some(_), .. }))))
        {
            return ControlFlow::Break(());
        }
        let first = match expr {
            Expr::Identifier(ident) => Some(ident),
            Expr::CompoundIdentifier(parts) => parts.first(),
            _ => None,
        };
        if first.is_some_and(|ident| !valid_expression_ident(ident)) {
            return ControlFlow::Break(());
        }
        self.array_parents.push(is_array);
        ControlFlow::Continue(())
    }

    fn post_visit_expr(&mut self, _expr: &Expr) -> ControlFlow<()> {
        let _ = self.array_parents.pop();
        ControlFlow::Continue(())
    }

    fn pre_visit_relation(&mut self, relation: &ObjectName) -> ControlFlow<()> {
        if relation.0.first().is_some_and(
            |part| matches!(part, ObjectNamePart::Identifier(ident) if !valid_ident(ident)),
        ) {
            ControlFlow::Break(())
        } else {
            ControlFlow::Continue(())
        }
    }

    fn pre_visit_table_factor(&mut self, factor: &TableFactor) -> ControlFlow<()> {
        let alias = match factor {
            TableFactor::Table { alias, .. }
            | TableFactor::Derived { alias, .. }
            | TableFactor::TableFunction { alias, .. }
            | TableFactor::Function { alias, .. }
            | TableFactor::UNNEST { alias, .. } => alias.as_ref(),
            _ => None,
        };
        if alias.is_some_and(|alias| !valid_alias(alias)) {
            ControlFlow::Break(())
        } else {
            ControlFlow::Continue(())
        }
    }
}

fn valid_type(data_type: &sqlparser::ast::DataType) -> bool {
    use sqlparser::ast::{ArrayElemTypeDef, DataType};
    match data_type {
        DataType::Custom(name, _) => name.0.first().is_some_and(|part|
            matches!(part, ObjectNamePart::Identifier(ident) if !ident.value.is_empty()
                && (ident.quote_style == Some('"') || ident.quote_style.is_none() && !type_reserved(&ident.value)))),
        DataType::Array(ArrayElemTypeDef::SquareBracket(element, _) | ArrayElemTypeDef::Qualified(element, _)) => valid_type(element),
        DataType::Array(_) => false,
        _ => true,
    }
}

fn valid_ident(ident: &Ident) -> bool {
    !ident.value.is_empty()
        && (ident.quote_style == Some('"')
            || ident.quote_style.is_none() && !reserved(&ident.value))
}

fn valid_name(name: &ObjectName) -> bool {
    name.0
        .first()
        .is_none_or(|part| matches!(part, ObjectNamePart::Identifier(ident) if valid_ident(ident)))
}

fn valid_expression_ident(ident: &Ident) -> bool {
    valid_ident(ident)
        || [
            "USER",
            "CURRENT_USER",
            "CURRENT_ROLE",
            "SESSION_USER",
            "SYSTEM_USER",
            "CURRENT_DATE",
            "CURRENT_TIME",
            "CURRENT_TIMESTAMP",
            "LOCALTIME",
            "LOCALTIMESTAMP",
        ]
        .iter()
        .any(|name| ident.value.eq_ignore_ascii_case(name))
}

fn valid_alias(alias: &TableAlias) -> bool {
    valid_ident(&alias.name)
        && alias.columns.iter().all(|column| valid_ident(&column.name))
        && (alias.columns.is_empty()
            || alias
                .columns
                .iter()
                .all(|column| column.data_type.is_some())
                == alias
                    .columns
                    .iter()
                    .any(|column| column.data_type.is_some()))
}
