use super::super::{select_exprs::target_identity, select_item_expr};
use crate::fx::FxHashSet;
use crate::idents::{ident_key, unwrap_expr};
use sqlparser::ast::{Expr, GroupByExpr, Select, SelectItem, Value};

#[derive(Default)]
pub(in crate::statements::keyed_lookups) struct GroupingTargets<'a> {
    pub(in crate::statements::keyed_lookups) all: Vec<&'a Expr>,
    pub(in crate::statements::keyed_lookups) required: FxHashSet<Expr>,
}

impl<'a> GroupingTargets<'a> {
    pub(in crate::statements::keyed_lookups) fn new(select: &'a Select) -> Self {
        let GroupByExpr::Expressions(expressions, _) = &select.group_by else {
            return Self::default();
        };
        if expressions.is_empty() {
            return Self::default();
        }
        let known_outputs: Vec<_> = select
            .projection
            .iter()
            .map_while(select_item_expr)
            .collect();
        let alias_outputs = if select.from.is_empty()
            && select
                .projection
                .iter()
                .all(|item| matches!(item, SelectItem::ExprWithAlias { .. }))
        {
            select.projection.as_slice()
        } else {
            &[]
        };
        collect(expressions, &known_outputs, alias_outputs)
    }
}

fn collect<'a>(
    expressions: &'a [Expr],
    outputs: &[&'a Expr],
    aliases: &'a [SelectItem],
) -> GroupingTargets<'a> {
    let mut targets = GroupingTargets::default();
    for expression in expressions {
        let expr = unwrap_expr(expression);
        match expr {
            Expr::GroupingSets(sets) => {
                let mut alternatives = sets.iter().map(|set| collect(set, outputs, aliases));
                if let Some(mut common) = alternatives.next() {
                    for alternative in alternatives {
                        common
                            .required
                            .retain(|expr| alternative.required.contains(expr));
                        common.all.extend(alternative.all);
                    }
                    targets.required.extend(common.required);
                    targets.all.extend(common.all);
                }
            }
            Expr::Rollup(sets) | Expr::Cube(sets) => {
                for set in sets {
                    targets.all.extend(collect(set, outputs, aliases).all);
                }
            }
            Expr::Tuple(members) => {
                let tuple = collect(members, outputs, aliases);
                targets.required.extend(tuple.required);
                targets.all.extend(tuple.all);
            }
            _ => {
                if let Some(resolved) = resolve_target(expr, outputs, aliases) {
                    if matches!(unwrap_expr(resolved), Expr::Value(_)) {
                        targets.required.insert(target_identity(resolved));
                    }
                    targets.all.push(resolved);
                }
            }
        }
    }
    targets
}

fn resolve_target<'a>(
    expr: &'a Expr,
    outputs: &[&'a Expr],
    alias_outputs: &'a [SelectItem],
) -> Option<&'a Expr> {
    if let Expr::Value(value) = expr {
        if let Value::Number(number, _) = &value.value {
            let index = number.parse::<usize>().ok()?.checked_sub(1)?;
            return outputs.get(index).copied();
        }
    }
    if let Expr::Identifier(ident) = expr {
        let name = ident_key(ident);
        let mut aliases = alias_outputs.iter().filter_map(|item| match item {
            SelectItem::ExprWithAlias { expr, alias } if ident_key(alias) == name => Some(expr),
            _ => None,
        });
        if let Some(first) = aliases.next() {
            if aliases.next().is_none() {
                return Some(first);
            }
        }
    }
    Some(expr)
}
