use sqlparser::ast::{Expr, GroupByExpr, GroupByWithModifier, Select};

#[cfg(test)]
mod tests;

pub(super) fn has_group_by(select: &Select) -> bool {
    match &select.group_by {
        GroupByExpr::Expressions(exprs, modifiers) => {
            let distinct = has_distinct_group_by_modifier(exprs, modifiers);
            let mut empty_grouping_sets = 1usize;
            for expr in exprs.iter().skip(usize::from(distinct)) {
                let Some(multiplicity) = empty_grouping_set_multiplicity(expr, distinct) else {
                    return true;
                };
                empty_grouping_sets = empty_grouping_sets.saturating_mul(multiplicity);
            }
            for modifier in modifiers {
                match modifier {
                    GroupByWithModifier::GroupingSets(expr) => {
                        let Some(multiplicity) = empty_grouping_set_multiplicity(expr, distinct)
                        else {
                            return true;
                        };
                        empty_grouping_sets = empty_grouping_sets.saturating_mul(multiplicity);
                    }
                    _ => return true,
                }
            }
            empty_grouping_sets > 1
        }
        GroupByExpr::All(_) => true,
    }
}

fn has_distinct_group_by_modifier(exprs: &[Expr], modifiers: &[GroupByWithModifier]) -> bool {
    // sqlparser represents PostgreSQL's `GROUP BY DISTINCT` token here as an
    // identifier followed by a grouping-set expression.
    let Some(Expr::Identifier(ident)) = exprs.first() else {
        return false;
    };
    if ident.quote_style.is_some() || !ident.value.eq_ignore_ascii_case("distinct") {
        return false;
    }
    modifiers
        .iter()
        .any(|modifier| matches!(modifier, GroupByWithModifier::GroupingSets(_)))
}

fn empty_grouping_set_multiplicity(expr: &Expr, distinct: bool) -> Option<usize> {
    match expr {
        Expr::Tuple(items) if items.is_empty() => Some(1),
        Expr::Tuple(_) => None,
        Expr::GroupingSets(sets) => {
            if sets.iter().any(|set| !set.is_empty()) {
                None
            } else if distinct {
                Some(1)
            } else {
                Some(sets.len().max(1))
            }
        }
        Expr::Rollup(sets) => {
            if sets.iter().any(|set| !set.is_empty()) {
                None
            } else {
                // ROLLUP yields every prefix, including the empty prefix.
                Some(sets.len().saturating_add(1))
            }
        }
        Expr::Cube(sets) => {
            if sets.iter().any(|set| !set.is_empty()) {
                None
            } else {
                // Every subset is a grouping set; two suffice to distinguish a
                // scalar aggregate from a query that can return multiple rows.
                Some(if sets.is_empty() { 1 } else { 2 })
            }
        }
        _ => None,
    }
}
