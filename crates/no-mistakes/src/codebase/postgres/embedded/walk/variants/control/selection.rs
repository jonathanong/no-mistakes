use super::{ScopeVisitor, Snapshot};

pub(super) fn select(visitor: &mut ScopeVisitor<'_>, selected: &Snapshot, other: &Snapshot) {
    visitor.restore_variants(selected);
    for (index, scope) in visitor.scopes.iter_mut().enumerate() {
        for (name, state) in scope {
            let selected = selected.get(index).and_then(|scope| scope.get(name));
            let other = other.get(index).and_then(|scope| scope.get(name));
            if selected != other {
                if let Some(values) = &mut state.variants {
                    for value in values {
                        value.enumerated = true;
                    }
                }
            }
        }
    }
}
