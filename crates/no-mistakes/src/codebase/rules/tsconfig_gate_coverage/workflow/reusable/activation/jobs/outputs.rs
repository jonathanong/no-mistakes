use crate::codebase::rules::tsconfig_gate_coverage::workflow::{
    conditions::{resolve_static_interpolations, EnvironmentState, InputState, StaticValue},
    reusable::model::ActivationScan,
};
use serde_yaml::Value;
use std::collections::{BTreeMap, BTreeSet};

pub(super) fn merge_reusable_outputs(
    aggregate: &mut Option<BTreeMap<String, StaticValue>>,
    scan: &ActivationScan,
) {
    if scan.failed || scan.indeterminate {
        *aggregate = Some(BTreeMap::new());
        return;
    }
    let Some(outputs) = aggregate else {
        *aggregate = Some(scan.outputs.clone());
        return;
    };
    let names = outputs
        .keys()
        .chain(scan.outputs.keys())
        .cloned()
        .collect::<BTreeSet<_>>();
    for name in names {
        if outputs.get(&name) != scan.outputs.get(&name) {
            outputs.insert(name, StaticValue::Unknown);
        }
    }
}

pub(super) fn static_step_job_outputs(
    job: &Value,
    inputs: &InputState,
    environment: &EnvironmentState,
) -> BTreeMap<String, StaticValue> {
    job.get("outputs")
        .and_then(Value::as_mapping)
        .into_iter()
        .flatten()
        .filter_map(|(name, value)| {
            Some((
                name.as_str()?.to_lowercase(),
                StaticValue::String(resolve_static_interpolations(
                    value.as_str()?,
                    inputs,
                    environment,
                )?),
            ))
        })
        .collect()
}

pub(super) fn merge_step_job_outputs(
    aggregate: &mut Option<BTreeMap<String, StaticValue>>,
    outputs: BTreeMap<String, StaticValue>,
) {
    let Some(existing) = aggregate else {
        *aggregate = Some(outputs);
        return;
    };
    let names = existing
        .keys()
        .chain(outputs.keys())
        .cloned()
        .collect::<BTreeSet<_>>();
    for name in names {
        if existing.get(&name) != outputs.get(&name) {
            existing.insert(name, StaticValue::Unknown);
        }
    }
}

#[cfg(test)]
#[path = "outputs_tests.rs"]
mod tests;
