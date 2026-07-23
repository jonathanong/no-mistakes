use super::{Extends, Options};
use crate::integration_tests::types::VitestSetupDependency;
use std::collections::BTreeMap;
use std::path::PathBuf;

pub(super) fn merge_options(root: &Options, project: Options) -> Options {
    Options {
        name: project.name.or_else(|| root.name.clone()),
        root: project.root.or_else(|| root.root.clone()),
        include: project.include.or_else(|| root.include.clone()),
        exclude: combine(root.exclude.clone(), project.exclude),
        setup_files: inherit_setup_files(
            matches!(project.extends.as_ref(), Some(Extends::True))
                .then(|| root.setup_files.clone())
                .flatten(),
            project.setup_files,
        ),
        global_setup: inherit_setup_files(
            matches!(project.extends.as_ref(), Some(Extends::True))
                .then(|| root.global_setup.clone())
                .flatten(),
            project.global_setup,
        ),
        extends: project.extends,
        nested_test_scope: project.nested_test_scope,
        standalone_config: project.standalone_config,
        standalone_config_path: project.standalone_config_path,
        config_base: project.config_base.or_else(|| root.config_base.clone()),
    }
}

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

fn combine(left: Option<Vec<String>>, right: Option<Vec<String>>) -> Option<Vec<String>> {
    let mut values = left.unwrap_or_default();
    values.extend(right.unwrap_or_default());
    (!values.is_empty()).then_some(values)
}
