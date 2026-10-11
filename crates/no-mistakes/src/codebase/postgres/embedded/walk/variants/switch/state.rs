use super::{ScopeVisitor, Snapshot, SwitchStatement};

pub(super) fn join(
    visitor: &mut ScopeVisitor<'_>,
    prior: Option<&Snapshot>,
    prior_paths: &[Vec<(u64, u32)>],
    current: &Snapshot,
    current_paths: &[Vec<(u64, u32)>],
    id: u64,
) -> Snapshot {
    if let Some(prior) = prior {
        let paths = visitor.variant_paths.clone();
        let left = changed_paths(visitor, prior, current, prior_paths);
        let right = changed_paths(visitor, current, prior, current_paths);
        visitor.variant_paths = paths;
        // Unequal bindings already carry their actual entry/fallthrough paths.
        // Equal bindings stay independent of this unrelated switch decision.
        visitor.join_variants(&left, &right, id, (0, 0));
    } else {
        visitor.restore_variants(current);
    }
    visitor.variant_snapshot()
}

fn changed_paths(
    visitor: &mut ScopeVisitor<'_>,
    snapshot: &Snapshot,
    other: &Snapshot,
    paths: &[Vec<(u64, u32)>],
) -> Snapshot {
    visitor.variant_paths = paths.to_vec();
    snapshot
        .iter()
        .zip(other)
        .map(|(scope, other)| {
            scope
                .iter()
                .map(|(name, values)| {
                    let values = if Some(values) == other.get(name) {
                        values.clone()
                    } else {
                        values
                            .clone()
                            .and_then(|values| visitor.with_variant_paths(values))
                    };
                    (name.clone(), values)
                })
                .collect()
        })
        .collect()
}

pub(super) fn select(visitor: &mut ScopeVisitor<'_>, statement: &SwitchStatement<'_>) {
    if let Some(selected) = super::super::conditions::selected_case(statement) {
        let id = u64::from(statement.span.start);
        for scope in &mut visitor.scopes {
            for state in scope.values_mut() {
                if let Some(values) = &mut state.variants {
                    values.retain(|value| {
                        !value
                            .choices
                            .iter()
                            .any(|(other, arm)| *other == id && *arm != selected)
                    });
                    for value in values {
                        value.enumerated |= value.choices.iter().any(|(other, _)| *other == id);
                    }
                }
            }
        }
    }
}
