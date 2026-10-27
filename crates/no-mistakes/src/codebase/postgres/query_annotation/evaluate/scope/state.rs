use super::{Bindings, Scope};
use std::sync::{Arc, Weak};

/// A weak allocation identity keeps no bindings or callback AST alive.
/// Retaining the allocation prevents address reuse after the last scope drops.
#[derive(Debug)]
pub(in crate::codebase::postgres::query_annotation::evaluate) struct StateWeak(Weak<Bindings>);

impl Scope {
    pub(in crate::codebase::postgres::query_annotation::evaluate) fn state(&self) -> StateWeak {
        StateWeak(Arc::downgrade(&self.0))
    }
}

impl StateWeak {
    pub(in crate::codebase::postgres::query_annotation::evaluate) fn matches(
        &self,
        scope: &Scope,
    ) -> bool {
        std::ptr::eq(self.0.as_ptr(), Arc::as_ptr(&scope.0))
    }
}

#[cfg(test)]
mod tests;
