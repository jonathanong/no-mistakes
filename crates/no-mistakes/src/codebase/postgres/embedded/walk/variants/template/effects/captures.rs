use super::ScopeVisitor;
use crate::fx::FxHashSet;
use oxc_ast::ast::Expression;

#[derive(Default)]
pub(super) struct CapturedBuilders {
    targets: Vec<(String, Option<u32>)>,
}

impl CapturedBuilders {
    pub(super) fn extend(&mut self, visitor: &ScopeVisitor<'_>, expression: &Expression<'_>) {
        for name in visitor.builder_result_names(expression) {
            if let Some(binding) = visitor.lookup(&name) {
                if binding.builder_identity.is_some()
                    || binding
                        .variants
                        .as_ref()
                        .is_some_and(|values| values.iter().any(|value| value.fragment))
                {
                    let target = (name, binding.builder_identity);
                    if !self.targets.contains(&target) {
                        self.targets.push(target);
                    }
                }
            }
        }
    }

    pub(super) fn mutated_by(
        &self,
        visitor: &ScopeVisitor<'_>,
        expression: &Expression<'_>,
    ) -> bool {
        if self.targets.is_empty() {
            return false;
        }
        let effects = visitor.expression_builder_effects(expression);
        effects.uncertain
            || effects.targets.into_iter().any(|(name, identity)| {
                let aliases = alias_group(visitor, name);
                self.targets.iter().any(|(captured, captured_identity)| {
                    // A fresh replacement can retain old alias edges. Known
                    // identities prove whether it is the object captured here.
                    if let (Some(identity), Some(captured_identity)) = (identity, captured_identity)
                    {
                        identity == *captured_identity
                    } else {
                        aliases.contains(captured)
                    }
                })
            })
    }
}

fn alias_group(visitor: &ScopeVisitor<'_>, name: String) -> FxHashSet<String> {
    let mut pending = vec![name];
    let mut seen = FxHashSet::default();
    while let Some(name) = pending.pop() {
        if seen.insert(name.clone()) {
            if let Some(aliases) = visitor.builder_aliases.get(&name) {
                pending.extend(aliases.iter().cloned());
            }
        }
    }
    seen
}
