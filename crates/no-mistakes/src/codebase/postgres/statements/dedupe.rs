use super::SqlSelectFact;
use std::collections::HashSet;

/// A nested `EXISTS` can be reached from both an expression walk and the
/// enclosing SELECT walk; keep the first fact at each source position.
pub(super) fn exists_set_operations(selects: &mut [SqlSelectFact]) {
    let mut seen = HashSet::new();
    for select in selects {
        select
            .exists_set_operations
            .retain(|fact| seen.insert((fact.line, fact.column)));
    }
}

#[cfg(test)]
mod tests;
