use super::{ArgumentEffect, ScopeVisitor};
use crate::codebase::postgres::embedded::EmbeddedSqlKind;
use crate::codebase::ts_source::unwrap_ts_wrappers;
use oxc_ast::ast::{CallExpression, Expression};

impl ScopeVisitor<'_> {
    pub(in crate::codebase::postgres::embedded::walk) fn begin_argument_invocation(
        &self,
        call: &CallExpression<'_>,
    ) -> Option<(usize, usize)> {
        (self.argument_frames > 0
            && matches!(
                unwrap_ts_wrappers(&call.callee),
                Expression::ArrowFunctionExpression(_) | Expression::FunctionExpression(_)
            ))
        .then_some((self.argument_effects.len(), self.scopes.len()))
    }

    pub(in crate::codebase::postgres::embedded::walk) fn finish_argument_invocation(
        &mut self,
        invocation: Option<(usize, usize)>,
    ) {
        if let Some((start, owner)) = invocation {
            let caller = self.function_scopes.last().copied().unwrap_or(0);
            for effect in &mut self.argument_effects[start..] {
                if effect.owner == owner {
                    // An invoked closure executes captured mutations. Merely
                    // declaring a deeper callback still does not execute it.
                    effect.owner = caller;
                    effect.state = None;
                }
            }
        }
    }

    pub(in crate::codebase::postgres::embedded::walk) fn record_argument_append_effects(
        &mut self,
        appends: &[(String, Option<u32>, Option<usize>)],
        position: u32,
    ) {
        if self.argument_frames == 0 || self.variant_paths.is_empty() {
            return;
        }
        for (name, identity, _) in appends {
            let state = self
                .lookup(name)
                .filter(|binding| {
                    binding.kind != EmbeddedSqlKind::Dynamic || binding.variants.is_some()
                })
                .map(|mut binding| {
                    if let Some(versions) = &mut binding.variants {
                        for version in versions {
                            version.enumerated = true;
                        }
                    }
                    binding
                });
            self.argument_effects.push(ArgumentEffect {
                owner: self.function_scopes.last().copied().unwrap_or(0),
                name: name.clone(),
                identity: *identity,
                position,
                state,
            });
        }
    }

    pub(in crate::codebase::postgres::embedded::walk) fn record_argument_helper_effects(
        &mut self,
        effects: &[(String, Option<u32>)],
        position: u32,
    ) {
        if self.argument_frames > 0 && !self.variant_paths.is_empty() {
            for (name, identity) in effects {
                self.argument_effects.push(ArgumentEffect {
                    owner: self.function_scopes.last().copied().unwrap_or(0),
                    name: name.clone(),
                    identity: *identity,
                    position,
                    state: None,
                });
            }
        }
    }
}
