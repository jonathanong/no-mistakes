use super::*;

pub(super) fn removable_keys(select: &Select) -> Option<Vec<&Expr>> {
    let GroupByExpr::Expressions(expressions, modifiers) = &select.group_by else {
        return None;
    };
    if !modifiers.is_empty() {
        return None;
    }
    if let [expr] = expressions.as_slice() {
        if let Expr::GroupingSets(sets) = bare(expr) {
            let [set] = sets.as_slice() else { return None };
            return Some(set.iter().collect());
        }
    }
    (!expressions.iter().any(complex)).then(|| expressions.iter().collect())
}

pub(super) fn preserves_key(
    select: &Select,
    projected: &[&Expr],
    catalog: &super::super::scope::CatalogScope<'_>,
) -> bool {
    let GroupByExpr::Expressions(expressions, modifiers) = &select.group_by else {
        return false;
    };
    if !modifiers.is_empty() {
        return false;
    }
    if !expressions.iter().any(complex) {
        return true;
    }
    let ([projected], [grouping]) = (projected, expressions.as_slice()) else {
        return false;
    };
    let (sets, subtotal) = match bare(grouping) {
        Expr::GroupingSets(sets) => (sets, false),
        Expr::Rollup(sets) | Expr::Cube(sets) => (sets, true),
        _ => return false,
    };
    let [set] = sets.as_slice() else { return false };
    let [key] = set.as_slice() else { return false };
    bare(key) == bare(projected) && (!subtotal || nonnull(select, key, catalog))
}

fn complex(expr: &Expr) -> bool {
    matches!(
        bare(expr),
        Expr::GroupingSets(_) | Expr::Rollup(_) | Expr::Cube(_)
    )
}

fn nonnull(select: &Select, key: &Expr, catalog: &super::super::scope::CatalogScope<'_>) -> bool {
    let [from] = select.from.as_slice() else {
        return false;
    };
    if !from.joins.is_empty() {
        return false;
    }
    let Some(relation) = walk::relation(&from.relation) else {
        return false;
    };
    let Some(key) = super::owned(key, &relation, true) else {
        return false;
    };
    let (_, name) = column(&key).expect("owned grouping keys are columns");
    catalog.column_is_not_null(&relation.name, &name)
        || walk::predicates(select)
            .iter()
            .any(|predicate| match bare(predicate) {
                Expr::IsNotNull(expr) => super::owned(expr, &relation, true).as_ref() == Some(&key),
                Expr::BinaryOp {
                    left,
                    op: BinaryOperator::Eq,
                    right,
                } => {
                    super::owned(left, &relation, true).as_ref() == Some(&key)
                        || super::owned(right, &relation, true).as_ref() == Some(&key)
                }
                _ => false,
            })
}
