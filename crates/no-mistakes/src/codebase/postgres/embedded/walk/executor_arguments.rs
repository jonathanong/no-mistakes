use super::{resolve, BindingState, ScopeVisitor};
use crate::codebase::postgres::embedded::{first_call_argument, EmbeddedSqlKind};
use crate::codebase::ts_source::unwrap_ts_wrappers;
use oxc_ast::ast::{CallExpression, Expression};
use oxc_span::GetSpan;
mod effects;

pub(super) struct ArgumentEffect {
    owner: usize,
    name: String,
    identity: Option<u32>,
    position: u32,
    state: Option<BindingState>,
}

pub(super) struct ArgumentSnapshot {
    confirmed: Option<usize>,
    pending: Option<usize>,
    start: usize,
    owner: usize,
    direct: bool,
    first_end: u32,
    targets: Vec<(String, BindingState)>,
}

impl ScopeVisitor<'_> {
    pub(super) fn begin_executor_arguments(
        &mut self,
        call: &CallExpression<'_>,
        before: (usize, usize),
    ) -> Option<ArgumentSnapshot> {
        let confirmed = (self.calls.len() > before.0).then_some(before.0);
        let pending = (self.pending_calls.len() > before.1).then_some(before.1);
        if confirmed.is_none() && pending.is_none() || call.arguments.len() < 2 {
            return None;
        }
        let argument = first_call_argument(call)?;
        let targets = self
            .builder_result_names(argument)
            .into_iter()
            .filter_map(|name| {
                let binding = self.lookup(&name)?;
                (binding.builder_identity.is_some()
                    || binding
                        .variants
                        .as_ref()
                        .is_some_and(|values| values.iter().any(|value| value.fragment)))
                .then_some((name, binding))
            })
            .collect::<Vec<_>>();
        if targets.is_empty() {
            return None;
        }
        self.argument_frames += 1;
        Some(ArgumentSnapshot {
            confirmed,
            pending,
            start: self.argument_effects.len(),
            owner: self.function_scopes.last().copied().unwrap_or(0),
            direct: matches!(unwrap_ts_wrappers(argument), Expression::Identifier(_)),
            first_end: argument.span().end,
            targets,
        })
    }

    pub(super) fn finish_executor_arguments(
        &mut self,
        call: &CallExpression<'_>,
        snapshot: ArgumentSnapshot,
    ) {
        let mut changed = None;
        let mut changed_name = None;
        let mut opaque = false;
        for effect in &self.argument_effects[snapshot.start..] {
            if effect.owner != snapshot.owner || effect.position < snapshot.first_end {
                continue;
            }
            for (name, original) in &snapshot.targets {
                let same_object = original
                    .builder_identity
                    .map_or(effect.name == *name, |id| effect.identity == Some(id));
                if same_object {
                    // Re-evaluating a conditional/helper first argument could
                    // select another object after a later argument changed its guard.
                    opaque |= !snapshot.direct
                        || effect.state.is_none()
                        || original.builder_identity.is_none() && original.condition_key.is_none();
                    changed = effect.state.clone();
                    changed_name = Some((effect.name.as_str(), original.builder_identity));
                }
            }
        }
        if let Some((name, identity)) = changed_name {
            if let Some(binding) = self.lookup(name).filter(|binding| {
                binding.builder_identity == identity
                    && (binding.kind != EmbeddedSqlKind::Dynamic || binding.variants.is_some())
            }) {
                // The final state contains the joined taken/not-taken paths.
                // A per-branch append snapshot alone is not the executed set.
                changed = Some(binding);
            } else {
                opaque = true;
            }
        }
        // A historical alias with lost identity cannot certify the actual
        // object, but its invalidation still forbids a stale finite snapshot.
        for (name, original) in &snapshot.targets {
            if original.builder_identity.is_none()
                && self.argument_effects[snapshot.start..]
                    .iter()
                    .any(|effect| {
                        effect.owner == snapshot.owner && effect.position >= snapshot.first_end
                    })
                && self.lookup(name).is_some_and(|binding| {
                    binding.builder_effect_revision != original.builder_effect_revision
                })
            {
                opaque = true;
            }
        }
        let original = snapshot.confirmed.map_or_else(
            || self.pending_calls[snapshot.pending.unwrap()].call.clone(),
            |index| self.calls[index].clone(),
        );
        let replacement = if opaque {
            let mut dynamic = original;
            dynamic.kind = EmbeddedSqlKind::Dynamic;
            dynamic.sql_text = None;
            dynamic.variants.clear();
            dynamic.sql_source_positions.clear();
            dynamic.recovered_placeholder_positions.clear();
            dynamic
        } else if let Some(binding) = changed {
            resolve::executor::executor_call_from_binding(self, call, original.callee, &binding)
        } else {
            original
        };
        if let Some(index) = snapshot.confirmed {
            self.calls[index] = replacement;
        } else {
            self.pending_calls[snapshot.pending.unwrap()].call = replacement;
        }
        self.argument_frames -= 1;
        if self.argument_frames == 0 {
            self.argument_effects.clear();
        }
    }
}
