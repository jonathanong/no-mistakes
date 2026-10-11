use crate::config::v2::schema::TestProjectPolicy;
use crate::integration_tests::types::{VitestSetupDependency, VitestSetupField};
use std::collections::BTreeSet;
use std::path::{Component, Path};

/// Project policies resolve setup paths from the repository, not a runner root.
/// Keep missing candidates as impact triggers without reading a second inventory.
pub(crate) fn dependencies(root: &Path, policy: &TestProjectPolicy) -> Vec<VitestSetupDependency> {
    policy
        .setup_files
        .iter()
        .filter_map(|raw| {
            let relative = Path::new(raw);
            if raw.is_empty()
                || relative.is_absolute()
                || !relative
                    .components()
                    .all(|part| matches!(part, Component::Normal(_) | Component::CurDir))
            {
                return None;
            }
            let path = crate::codebase::ts_resolver::normalize_path(&root.join(relative));
            let specifier = format!(
                "./{}",
                crate::codebase::ts_source::relative_slash_path(root, &path)
            );
            let declaration = root.join(".no-mistakes.yml");
            Some(VitestSetupDependency {
                field: VitestSetupField::SetupFiles,
                specifier: Some(specifier),
                needs_final_catalog_reparse: false,
                unresolved_config_extends: None,
                config_extends_provenance: false,
                resolved_path: Some(path.clone()),
                resolution_base: root.to_path_buf(),
                declaration_path: declaration.clone(),
                declaration_line: 0,
                trigger_paths: BTreeSet::from([declaration, path.clone()]),
                resolver_candidate_paths: BTreeSet::from([path]),
                conservative_specifiers: BTreeSet::new(),
                transitive_trigger_paths: BTreeSet::new(),
            })
        })
        .collect()
}
