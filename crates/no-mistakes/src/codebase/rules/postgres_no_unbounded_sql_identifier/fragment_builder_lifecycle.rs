use super::*;
use crate::codebase::dependencies::extract::ValueBinding;

impl Proof<'_, '_> {
    pub(super) fn lifecycle(&mut self, binding: &ValueBinding) -> bool {
        let calls: Vec<_> = self
            .flow
            .calls
            .iter()
            .filter(|call| self.rooted_at(&call.callee, binding, &mut FxHashSet::default()))
            .collect();
        calls.iter().all(|call| {
            matches!(&call.callee, ValueExpression::Member(base, method)
                if method == "append" && self.append_receiver(base, binding))
                && self.expression(&binding.value, true)
                && self.append_argument(call)
        })
    }

    fn rooted_at(
        &self,
        value: &ValueExpression,
        binding: &ValueBinding,
        active: &mut FxHashSet<(u32, u32)>,
    ) -> bool {
        match value {
            ValueExpression::Named { name, scope, .. } => {
                name == &binding.name && scope.unwrap_or(0) == binding.scope
            }
            ValueExpression::Member(base, _) => self.rooted_at(base, binding, active),
            ValueExpression::Call(start, end) if active.insert((*start, *end)) => self
                .calls
                .get(&(*start, *end))
                .is_some_and(|call| self.rooted_at(&call.callee, binding, active)),
            _ => false,
        }
    }

    fn append_receiver(&self, value: &ValueExpression, binding: &ValueBinding) -> bool {
        match value {
            ValueExpression::Named { name, scope, .. } => {
                name == &binding.name && scope.unwrap_or(0) == binding.scope
            }
            ValueExpression::Call(start, end) => {
                self.calls.get(&(*start, *end)).is_some_and(|call| {
                    matches!(&call.callee, ValueExpression::Member(base, method)
                    if method == "append" && self.append_receiver(base, binding))
                })
            }
            _ => false,
        }
    }
}
