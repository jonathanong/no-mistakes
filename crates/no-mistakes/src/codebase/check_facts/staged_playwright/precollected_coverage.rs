use super::super::CheckFactPlan;
use crate::codebase::ts_source::facts::TsFactMap;

pub(super) fn assert_covers(facts: &TsFactMap, plan: &CheckFactPlan) {
    assert!(
        facts.is_empty() || facts.plan().covers(plan.graph),
        "precollected graph facts must cover the staged graph plan"
    );
    assert!(
        facts.is_empty()
            || !plan.graph.call_arguments
            || facts.covers_argument_projection(&plan.graph_context.argument_projection),
        "precollected argument facts must cover the staged graph plan"
    );
}
