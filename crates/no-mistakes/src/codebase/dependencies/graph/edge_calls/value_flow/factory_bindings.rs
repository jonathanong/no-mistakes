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

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn default_factory_and_alias_cycles_are_classified_conservatively() {
        let mut index = FlowFileIndex::default();
        let fallback = ValueExpression::Default(
            std::sync::Arc::new(ValueExpression::Unknown),
            std::sync::Arc::new(ValueExpression::Call(1, 2)),
        );
        assert!(index.factory_value(&fallback, &mut fx_set()));
        let binding = ValueBinding {
            name: "cycle".to_string(), scope: 0, declared_at: 0,
            invalidated_at: None, mutation_contexts: fx_set(),
            value: ValueExpression::Named {
                name: "cycle.member".to_string(), scope: Some(0), offset: 0, context: None,
            },
        };
        index.bindings = index_scoped_names([((0, "cycle".to_string()), binding)]);
        assert!(!index.factory_binding(0, "cycle"));
    }
}
