use super::super::predicates::Projection;

pub(in crate::codebase::postgres::statements) struct Policy<'a, P> {
    pub(super) positions: super::super::value::PlaceholderPositions<'a>,
    pub(super) projection: &'a P,
}

impl<P> Copy for Policy<'_, P> {}
impl<P> Clone for Policy<'_, P> {
    fn clone(&self) -> Self {
        *self
    }
}

pub(in crate::codebase::postgres::statements) fn policy<'a, P: Projection>(
    positions: super::super::value::PlaceholderPositions<'a>,
    projection: &'a P,
) -> Policy<'a, P> {
    Policy {
        positions,
        projection,
    }
}
