use super::expressions::*;
use sqlparser::ast::*;
pub(super) struct Relation {
    pub name: String,
    pub cte: Option<String>,
    pub alias: String,
}
pub(super) fn relation(factor: &TableFactor) -> Option<Relation> {
    match factor {
        TableFactor::Table {
            name,
            alias,
            args: None,
            sample: None,
            version: None,
            ..
        } if alias.as_ref().is_none_or(|alias| alias.columns.is_empty()) => {
            let table = name.to_string();
            Some(Relation {
                alias: alias
                    .as_ref()
                    .map(|alias| ident(&alias.name))
                    .unwrap_or_else(|| {
                        crate::codebase::postgres::idents::object_name_ident(name)
                            .map(ident)
                            .unwrap_or_default()
                    }),
                cte: (name.0.len() == 1)
                    .then(|| crate::codebase::postgres::idents::object_name_key(name)),
                name: table,
            })
        }
        _ => None,
    }
}
pub(super) fn on(join: &Join) -> Option<&Expr> {
    match &join.join_operator {
        JoinOperator::Join(JoinConstraint::On(expr))
        | JoinOperator::Inner(JoinConstraint::On(expr)) => Some(expr),
        _ => None,
    }
}
pub(super) fn table_columns(select: &Select) -> Option<Vec<Relation>> {
    let [from] = select.from.as_slice() else {
        return None;
    };
    let mut relations = vec![relation(&from.relation)?];
    for join in &from.joins {
        on(join)?;
        relations.push(relation(&join.relation)?);
    }
    Some(relations)
}
pub(super) fn predicates(select: &Select) -> Vec<&Expr> {
    let mut out = conjuncts(select.selection.as_ref());
    for from in &select.from {
        for join in &from.joins {
            out.extend(conjuncts(on(join)));
        }
    }
    out
}
