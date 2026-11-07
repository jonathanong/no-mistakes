use super::*;
use crate::fx::{FxHashMap, FxHashSet};
use std::path::PathBuf;

pub(crate) type IdentifierReturns = FxHashMap<PathBuf, FxHashSet<(u32, u32)>>;

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
            .and_then(|returns| returns.get(file))
            .is_some_and(|returns| returns.contains(&(call.offset, call.end)))
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
                .and_then(|returns| returns.get(file))
                .is_some_and(|returns| returns.contains(&(*start, *end))),
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
