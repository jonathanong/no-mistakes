use super::super::Scope;
use crate::codebase::postgres::statements::{
    SqlBoundInputMode, SqlBoundItemKind, SqlBoundQuery, SqlPinSource,
};
use std::{collections::BTreeSet, rc::Rc};

impl Scope {
    pub(in super::super) fn mark_pool(&self, name: &str) {
        let Some(bodies) = &self.bodies else {
            return;
        };
        let mut frame = Some(self.frame.as_ref());
        while let Some(current) = frame {
            if let Some(bound) = current.ctes.get(name) {
                bodies
                    .borrow_mut()
                    .pools
                    .insert(Rc::as_ptr(bound) as usize, Rc::clone(bound));
                return;
            }
            frame = current.parent.as_deref();
        }
    }

    pub(in super::super) fn record_pool_reference(
        &self,
        bound: &SqlBoundQuery,
        at: impl FnOnce() -> (usize, usize),
    ) {
        let Some(bodies) = &self.bodies else {
            return;
        };
        let mut bodies = bodies.borrow_mut();
        if bodies
            .pools
            .contains_key(&(bound as *const SqlBoundQuery as usize))
        {
            let at = at();
            if at != (1, 1) {
                bodies.references.insert(at);
            }
        }
    }
}

#[cfg(test)]
mod tests;

pub(super) fn project(
    query: &SqlBoundQuery,
    references: &BTreeSet<(usize, usize)>,
) -> SqlBoundQuery {
    let mut projected = query.clone();
    if !references.is_empty() {
        mask(&mut projected, references);
    }
    projected
}

fn mask(query: &mut SqlBoundQuery, references: &BTreeSet<(usize, usize)>) {
    for item in &mut query.items {
        if let SqlBoundItemKind::Query(inner) = &mut item.kind {
            if references.contains(&(item.line, item.column)) {
                // Skip this inherited read obligation while retaining its column provenance.
                inner.input_mode = SqlBoundInputMode::Skipped;
            } else {
                mask(inner, references);
            }
        }
        for pin in &mut item.pins {
            if let SqlPinSource::Query(inner)
            | SqlPinSource::ReadQuery(inner)
            | SqlPinSource::ArrayQuery(inner) = &mut pin.source
            {
                mask(inner, references);
            }
        }
    }
}
