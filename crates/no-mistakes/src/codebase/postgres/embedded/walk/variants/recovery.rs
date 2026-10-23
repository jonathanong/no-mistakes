use super::super::ScopeVisitor;
use super::{bounded, expression, Recovered};

impl ScopeVisitor<'_> {
    pub(in crate::codebase::postgres::embedded::walk) fn with_append_loop(
        &mut self,
        start: u32,
        walk: impl FnOnce(&mut Self),
    ) {
        if self.head_calls_only {
            self.with_loop(walk);
            return;
        }
        let before: Vec<_> = self
            .scopes
            .iter()
            .enumerate()
            .flat_map(|(index, scope)| {
                scope
                    .iter()
                    .filter(|(_, binding)| binding.sql_builder && binding.variants.is_some())
                    .filter_map(move |(name, binding)| {
                        binding
                            .variants
                            .clone()
                            .map(|variants| (index, name.clone(), variants))
                    })
            })
            .collect();
        self.with_loop(walk);
        // Only outer builders are approximated; unrelated scalar bindings keep
        // their historical loop behavior, without copying their snapshots.
        for (index, name, before) in before {
            if let Some(binding) = self
                .scopes
                .get_mut(index)
                .and_then(|scope| scope.get_mut(&name))
            {
                let after = binding.variants.take();
                let changed = after
                    .as_ref()
                    .is_none_or(|after| !same_values(&before, after));
                let id = (1u64 << 63) | u64::from(start);
                let mut joined =
                    super::control::joined(Some(before), after, id, (1, 0), Some((id, start, 1)));
                // A loop can execute again after the one explored iteration.
                // Mark changed builders as non-exhaustive so later SQL checks
                // retain an unknown residual; unchanged builders remain exact.
                if changed {
                    if let Some(values) = &mut joined {
                        for value in values {
                            value.variants_truncated = true;
                        }
                    }
                }
                binding.variants = joined;
            }
        }
    }
    pub(in crate::codebase::postgres::embedded::walk) fn recover_variants(
        &self,
        expression: &oxc_ast::ast::Expression<'_>,
    ) -> Option<Vec<Recovered>> {
        if self.head_calls_only || self.loop_depth > 0 {
            return None;
        }
        self.recover_over_paths(|path| expression::recover(self, expression, 8, path))
    }
    pub(in crate::codebase::postgres::embedded::walk) fn recover_append_variants(
        &self,
        expression: &oxc_ast::ast::Expression<'_>,
    ) -> Option<Vec<Recovered>> {
        if self.head_calls_only {
            return None;
        }
        self.recover_over_paths(|path| expression::recover(self, expression, 8, path))
    }
    /// Alternatives of an executor call whose argument is ``${b.text}`` of a
    /// fully recovered builder (see `template::builder_text`).
    pub(in crate::codebase::postgres::embedded::walk) fn recover_builder_text(
        &self,
        call: &oxc_ast::ast::CallExpression<'_>,
    ) -> Option<Vec<Recovered>> {
        if self.head_calls_only || self.loop_depth > 0 {
            return None;
        }
        let (template, builder) = super::template::builder_text::shape(call)?;
        self.recover_over_paths(|path| {
            super::template::builder_text::recover(self, template, builder, 8, path)
        })
    }
    fn recover_over_paths(
        &self,
        recover: impl Fn(&[(u64, u32)]) -> Option<Vec<Recovered>>,
    ) -> Option<Vec<Recovered>> {
        let mut recovered = Vec::new();
        for path in &self.variant_paths {
            let values = recover(path)?;
            recovered.extend(values.into_iter().map(|mut value| {
                for choice in path {
                    if !value.choices.contains(choice) {
                        value.choices.push(*choice);
                    }
                }
                value
            }));
        }
        bounded(recovered)
    }
    pub(in crate::codebase::postgres::embedded::walk) fn with_variant_paths(
        &self,
        values: Vec<Recovered>,
    ) -> Option<Vec<Recovered>> {
        let paths: Vec<Recovered> = self
            .variant_paths
            .iter()
            .map(|choices| {
                let mut value = Recovered::empty(1);
                value.choices = choices.clone();
                value
            })
            .collect();
        // Choice-only suffixes do not change the physical mapping.
        let mut result = Vec::new();
        for value in values {
            for path in &paths {
                let path: &Recovered = path;
                if value.choices.iter().any(|(id, arm)| {
                    path.choices
                        .iter()
                        .any(|(other, branch)| id == other && arm != branch)
                }) {
                    continue;
                }
                let mut value = value.clone();
                for choice in &path.choices {
                    if !value.choices.contains(choice) {
                        value.choices.push(*choice);
                    }
                }
                result.push(value);
            }
        }
        bounded(result)
    }
}

fn same_values(left: &[Recovered], right: &[Recovered]) -> bool {
    left.len() == right.len()
        && left
            .iter()
            .all(|value| right.iter().any(|other| value.same_value(other)))
}
