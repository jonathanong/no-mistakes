use super::*;
use crate::config::v2::{EffectiveRuleAnalysis, Effort};

fn config() -> NoMistakesConfig {
    serde_yaml::from_str(include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../test-cases/rules/postgres-no-unbounded-sql-identifier/fixture/issue-3073-imported-guards/.no-mistakes.yml"))).unwrap()
}

#[test]
fn value_import_selector_demand_is_only_for_enabled_max_identifier_analysis() {
    let enabled = config();
    let plan = graph_plan(&enabled).unwrap();
    assert!(plan.value_import_selectors);
    assert!(!plan.symbol_import_selectors);
    assert!(!plan.ts_fact_plan().symbols && !plan.ts_fact_plan().module_bindings);
    let mut low = config();
    low.rules[0].resolved_analysis = Some(EffectiveRuleAnalysis {
        effort: Effort::Low,
        ..Default::default()
    });
    assert!(graph_plan(&low).is_none());
    let mut other = config();
    other.rules[0].rule = "postgres-no-mutable-sql-registry".into();
    assert!(!graph_plan(&other).unwrap().value_import_selectors);
    low.rules.push(other.rules[0].clone());
    assert!(!graph_plan(&low).unwrap().value_import_selectors);
    let mut disabled = config();
    disabled.rules[0].enabled = false;
    disabled.rules.push(other.rules[0].clone());
    assert!(!graph_plan(&disabled).unwrap().value_import_selectors);
}
