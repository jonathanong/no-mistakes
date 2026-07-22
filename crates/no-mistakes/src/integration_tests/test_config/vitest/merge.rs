use crate::integration_tests::types::VitestSetupDependency;
use std::collections::BTreeMap;
use std::path::PathBuf;

pub(super) fn inherit_setup_files(
    inherited: Option<Vec<VitestSetupDependency>>,
    local: Option<Vec<VitestSetupDependency>>,
) -> Option<Vec<VitestSetupDependency>> {
    let setups = inherited
        .into_iter()
        .flatten()
        .chain(local.into_iter().flatten())
        .collect::<Vec<_>>();
    (!setups.is_empty()).then_some(setups)
}

pub(super) fn dedupe_resolved_setups(setups: &mut Vec<VitestSetupDependency>) {
    let mut retained = Vec::<VitestSetupDependency>::new();
    let mut resolved_indices = BTreeMap::<PathBuf, usize>::new();
    for setup in setups.drain(..) {
        let Some(identity) = setup.resolved_path.clone() else {
            retained.push(setup);
            continue;
        };
        if let Some(index) = resolved_indices.get(&identity).copied() {
            let existing = &mut retained[index];
            existing.trigger_paths.extend(setup.trigger_paths);
            existing
                .resolver_candidate_paths
                .extend(setup.resolver_candidate_paths);
            existing
                .transitive_trigger_paths
                .extend(setup.transitive_trigger_paths);
        } else {
            resolved_indices.insert(identity, retained.len());
            retained.push(setup);
        }
    }
    *setups = retained;
}
