use crate::codebase::rules::tsconfig_gate_coverage::workflow::{
    conditions::StaticValue, reusable::model::ActivationScan,
};
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

pub(super) fn merge_fail_fast_failure_projects(
    aggregate: &mut Option<BTreeSet<String>>,
    projects: BTreeSet<String>,
) {
    match aggregate {
        Some(failed_projects) => {
            failed_projects.retain(|project| projects.contains(project));
        }
        None => *aggregate = Some(projects),
    }
}

pub(super) fn retain_fail_fast_projects(
    projects: &mut BTreeSet<String>,
    failed_projects: Option<BTreeSet<String>>,
    instance_count: usize,
) {
    if instance_count > 1 {
        if let Some(failed_projects) = failed_projects {
            *projects = failed_projects;
        }
    }
}
