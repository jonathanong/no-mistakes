use super::*;

type Fields = std::sync::Arc<FxHashMap<String, ValueExpression>>;
#[derive(Clone, PartialEq, Eq)]
struct Identity {
    binding: (usize, String),
    root: Option<super::super::mutable_origins::RootIdentity>,
}
pub(super) struct Summary {
    at: u32,
    guarded: bool,
    width: usize,
    seed: Fields,
    common: Option<FxHashMap<String, Identity>>,
}

impl ImportCollector {
    pub(in super::super::super) fn note_predeclared_array_binding(
        &mut self,
        pattern: &BindingPattern<'_>,
        kind: VariableDeclarationKind,
    ) {
        if self.collect_value_flow
            && matches!(
                kind,
                VariableDeclarationKind::Const
                    | VariableDeclarationKind::Using
                    | VariableDeclarationKind::AwaitUsing
            )
        {
            self.note_stable_array_binding(pattern);
        }
    }

    pub(in super::super::super) fn note_stable_array_binding(
        &mut self,
        pattern: &BindingPattern<'_>,
    ) {
        for name in binding_names(pattern) {
            let scope = self
                .callee_binding_scope(&name)
                .unwrap_or(self.current_lexical_scope_id());
            self.receiver_escapes
                .stable_bindings
                .entry(scope)
                .or_default()
                .insert(name);
        }
    }
    pub(super) fn note_guarded_array(
        &mut self,
        scope: usize,
        name: &str,
        at: u32,
        fields: &Fields,
        guarded: bool,
    ) {
        let previous = self
            .receiver_escapes
            .guarded_summaries
            .entry(scope)
            .or_default()
            .remove(name);
        let common = if guarded {
            let incoming = self.array_identities(fields);
            let mut common = previous.map_or_else(fx_map, |summary| {
                summary
                    .common
                    .unwrap_or_else(|| self.array_identities(&summary.seed))
            });
            common.retain(|slot, identity| incoming.get(slot) == Some(identity));
            Some(common)
        } else {
            None
        };
        let width = fields
            .keys()
            .filter_map(|key| key.parse::<usize>().ok())
            .map(|slot| slot + 1)
            .max()
            .unwrap_or(0);
        self.receiver_escapes
            .guarded_summaries
            .entry(scope)
            .or_default()
            .insert(
                name.to_owned(),
                Summary {
                    at,
                    guarded,
                    width,
                    seed: std::sync::Arc::clone(fields),
                    common,
                },
            );
    }
    fn array_identities(
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
    fn array_identity(&self, value: &ValueExpression) -> Option<Identity> {
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
        let root = if stable {
            None
        } else {
            Some(
                self.mutable_receiver_version(&binding, (*offset, *context))?
                    .known_identity()?,
            )
        };
        Some(Identity { binding, root })
    }
}

/// The ordinary entries API remains unchanged; only escaped spread arguments
/// distinguish branch-exclusive literal slots from guaranteed later pushes.
pub(super) fn entries(
    collector: &ImportCollector,
    value: &ValueExpression,
    before: u32,
) -> Option<Vec<(usize, ValueExpression, bool)>> {
    let baseline = super::escaped_array_entries::array_entries(collector, value, before)?;
    let summary = match value {
        ValueExpression::Named { name, scope, .. } => collector
            .receiver_escapes
            .guarded_summaries
            .get(&scope.unwrap_or(0))
            .and_then(|names| names.get(name))
            .filter(|summary| summary.at <= before && summary.guarded),
        _ => None,
    };
    Some(
        baseline
            .into_iter()
            .map(|(slot, value)| {
                let possible = summary.is_some_and(|summary| {
                    slot < summary.width
                        && !summary
                            .common
                            .as_ref()
                            .is_some_and(|common| common.contains_key(&slot.to_string()))
                });
                (slot, value, possible)
            })
            .collect(),
    )
}
