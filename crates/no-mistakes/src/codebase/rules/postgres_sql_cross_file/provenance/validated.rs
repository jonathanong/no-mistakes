use super::*;
use crate::fx::{FxHashMap, FxHashSet};
use std::path::PathBuf;

#[derive(Default)]
pub(crate) struct IdentifierReturns {
    calls: FxHashMap<PathBuf, FxHashMap<(u32, u32), Vec<usize>>>,
    used_sources: std::cell::RefCell<FxHashSet<usize>>,
}

#[cfg(test)]
#[path = "validated_usage_tests.rs"]
mod tests;

impl IdentifierReturns {
    pub(crate) fn insert(&mut self, file: PathBuf, span: (u32, u32), sources: Vec<usize>) {
        self.calls.entry(file).or_default().insert(span, sources);
    }

    pub(crate) fn source_used(&self, index: usize) -> bool {
        self.used_sources.borrow().contains(&index)
    }

    fn proves(&self, file: &Path, span: (u32, u32)) -> bool {
        let Some(sources) = self.calls.get(file).and_then(|calls| calls.get(&span)) else {
            return false;
        };
        if !sources.is_empty() {
            self.used_sources
                .borrow_mut()
                .extend(sources.iter().copied());
        }
        true
    }
}

impl<'a> Bindings<'a> {
    pub(crate) fn imported_collection_safe(
        &self,
        file: &Path,
        name: &str,
        compute: impl FnOnce() -> bool,
    ) -> bool {
        let key = (file.to_path_buf(), name.to_string());
        if let Some(safe) = self.collection_safety.borrow().get(&key) {
            return *safe;
        }
        let safe = compute();
        self.collection_safety.borrow_mut().insert(key, safe);
        safe
    }
    pub(crate) fn with_identifier_returns(mut self, returns: &'a IdentifierReturns) -> Self {
        self.identifier_returns = Some(returns);
        self
    }

    pub(crate) fn identifier_return(&self, file: &Path, call: &ValueCall) -> bool {
        self.identifier_returns
            .is_some_and(|returns| returns.proves(file, (call.offset, call.end)))
    }

    pub(crate) fn identifier_value(
        &self,
        file: &Path,
        value: &ValueExpression,
        members: &[String],
    ) -> bool {
        members.is_empty() && self.return_value(file, value, &mut FxHashSet::default())
    }

    fn return_value(
        &self,
        file: &Path,
        value: &ValueExpression,
        seen: &mut FxHashSet<(usize, String, u32)>,
    ) -> bool {
        match value {
            ValueExpression::Call(start, end) => self
                .identifier_returns
                .is_some_and(|returns| returns.proves(file, (*start, *end))),
            ValueExpression::Awaited(inner) => self.return_value(file, inner, seen),
            ValueExpression::Named {
                name,
                scope,
                offset,
                ..
            } => {
                let scope = scope.unwrap_or(0);
                self.value(file, scope, name, *offset)
                    .is_some_and(|binding| {
                        binding.immutable
                            && seen.insert((scope, name.clone(), binding.declared_at))
                            && self.return_value(file, &binding.value, seen)
                    })
            }
            _ => false,
        }
    }
}
