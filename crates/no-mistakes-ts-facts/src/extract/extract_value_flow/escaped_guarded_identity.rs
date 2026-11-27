use super::*;

impl ImportCollector {
    pub(super) fn array_identities(
        &self,
        fields: &FxHashMap<String, ValueExpression>,
    ) -> FxHashMap<String, Identity> {
        fields
            .iter()
            .filter_map(|(slot, value)| {
                self.array_identity(value)
                    .map(|identity| (slot.clone(), identity))
            })
            .collect()
    }
    pub(super) fn array_identity(&self, value: &ValueExpression) -> Option<Identity> {
        if let ValueExpression::Either(left, right) = value {
            let left = self.array_identity(left)?;
            return (self.array_identity(right)? == left).then_some(left);
        }
        let ValueExpression::Named {
            name,
            scope,
            offset,
            context,
        } = value
        else {
            return None;
        };
        let scope = scope.unwrap_or(0);
        let binding = (scope, name.clone());
        let stable = (scope == 0 && self.imported_bindings.contains(name))
            || self
                .receiver_escapes
                .stable_bindings
                .get(&scope)
                .is_some_and(|names| names.contains(name));
        let parameter = if stable {
            None
        } else if let Some(parameter) = self
            .value_parameter_indexes
            .get(&scope)
            .and_then(|names| names.get(name))
            .and_then(|indexes| indexes.first())
            .and_then(|index| self.value_flow.parameters.get(*index))
        {
            Some(self.guarded_parameter_epoch(&binding, parameter.owner, *offset, *context)?)
        } else {
            None
        };
        let root = if stable || parameter.is_some() {
            None
        } else {
            Some(
                self.mutable_receiver_version(&binding, (*offset, *context))?
                    .known_identity()?,
            )
        };
        Some(Identity {
            binding,
            root,
            parameter,
        })
    }
}
