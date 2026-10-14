use super::*;

impl FlowFileIndex {
    pub(in super::super) fn factory_binding(&self, scope: usize, name: &str) -> bool {
        self.factory_value(
            &ValueExpression::Named {
                name: name.to_string(),
                scope: Some(scope),
                offset: u32::MAX,
                context: None,
            },
            &mut fx_set(),
        )
    }
    fn factory_value(
        &self,
        value: &ValueExpression,
        visiting: &mut FxHashSet<(usize, String)>,
    ) -> bool {
        match value {
            ValueExpression::Call(_, _) => true,
            ValueExpression::Member(value, _) | ValueExpression::Awaited(value) => {
                self.factory_value(value, visiting)
            }
            ValueExpression::Default(value, fallback) => {
                self.factory_value(value, visiting) || self.factory_value(fallback, visiting)
            }
            ValueExpression::Named {
                name,
                scope: Some(scope),
                ..
            } => {
                let base = name.split_once('.').map_or(name.as_str(), |(base, _)| base);
                if !visiting.insert((*scope, base.to_string())) {
                    return false;
                }
                scoped_name(&self.bindings, *scope, base).is_some_and(|binding| {
                    binding.invalidated_at.is_none() && self.factory_value(&binding.value, visiting)
                })
            }
            _ => false,
        }
    }
}
