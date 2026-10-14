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
