use super::CheckFactPlan;
use crate::codebase::ts_source::facts::ArgumentProjection;

#[test]
fn argument_demand_merges_while_preserving_sparse_and_full_consumers() {
    let a = ArgumentProjection::exact(vec!["selected".into()]);
    let b = ArgumentProjection::exact(vec!["ignored".into()]);
    let mut plan = CheckFactPlan::default();
    plan.require_call_arguments(None);
    assert!(!plan.graph.call_arguments);
    plan.require_call_arguments(Some(a.clone()));
    plan.require_call_arguments(Some(b.clone()));
    assert!(plan.graph_context.argument_projection.covers(&a));
    assert!(plan.graph_context.argument_projection.covers(&b));
    assert!(plan.graph_context.argument_projection.is_exact());

    let imports_only = CheckFactPlan {
        imports: true,
        ..Default::default()
    };
    let mut retained = plan.clone();
    retained.include(imports_only.clone());
    assert!(retained.graph_context.argument_projection.is_exact());

    let mut introduced = imports_only;
    introduced.include(plan.clone());
    assert!(introduced.graph_context.argument_projection.covers(&a));
    assert!(introduced.graph_context.argument_projection.covers(&b));

    let mut broad = CheckFactPlan::default();
    broad.require_call_arguments(Some(ArgumentProjection::default()));
    plan.include(broad);
    assert!(!plan.graph_context.argument_projection.is_exact());
}
