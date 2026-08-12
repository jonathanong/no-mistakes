use super::*;
use crate::codebase::rules::tsconfig_gate_coverage::workflow::conditions::{
    expression_bool, StaticBool,
};

#[test]
fn strategy_configuration_values_resolve_after_needs_outputs() {
    let job: Value = serde_yaml::from_str(
        "needs: setup\nstrategy:\n  fail-fast: '${{ fromJSON(needs.setup.outputs.fail_fast) }}'\n  max-parallel: '${{ needs.setup.outputs.parallel }}'\n  matrix: {target: [one, two]}",
    )
    .unwrap();
    let jobs =
        serde_yaml::Mapping::from_iter([(Value::String("typecheck".to_string()), job.clone())]);
    let states = JobStates::new(&jobs, &InputState::new()).unwrap();
    let outputs = BTreeMap::from([(
        "setup".to_string(),
        BTreeMap::from([
            (
                "fail_fast".to_string(),
                StaticValue::String("false".to_string()),
            ),
            ("parallel".to_string(), StaticValue::String("1".to_string())),
        ]),
    )]);
    let inputs = states
        .inputs_with_results_for(
            "typecheck",
            &job,
            &BTreeSet::new(),
            &BTreeSet::new(),
            &BTreeSet::from(["setup".to_string()]),
            &outputs,
        )
        .unwrap();

    assert_eq!(inputs.len(), 2);
    assert!(inputs.iter().all(|inputs| {
        expression_bool("!strategy.fail-fast && strategy.max-parallel == 1", inputs)
            == StaticBool::True
    }));
}
