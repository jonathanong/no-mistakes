use crate::config::v2::schema::TestProjectPolicy;
use crate::integration_tests::types::{VitestSetupDependency, VitestSetupField};
use std::collections::BTreeSet;
use std::path::Path;

/// Project policies resolve setup paths from the repository, not a runner root.
/// Keep missing candidates as impact triggers without reading a second inventory.
pub(crate) fn dependencies(root: &Path, policy: &TestProjectPolicy) -> Vec<VitestSetupDependency> {
    let declaration = root.join(
        policy
            .declaration_path
            .as_deref()
            .unwrap_or(Path::new(".no-mistakes.yml")),
    );
    policy
        .setup_files
        .iter()
        .map(|raw| {
            let relative = Path::new(raw);
            let path = crate::codebase::ts_resolver::normalize_path(&root.join(relative));
            let specifier = format!(
                "./{}",
                crate::codebase::ts_source::relative_slash_path(root, &path)
            );
            VitestSetupDependency {
                field: VitestSetupField::SetupFiles,
                specifier: Some(specifier),
                needs_final_catalog_reparse: false,
                unresolved_config_extends: None,
                config_extends_provenance: false,
                resolved_path: Some(path.clone()),
                resolution_base: root.to_path_buf(),
                declaration_path: declaration.clone(),
                declaration_line: 0,
                trigger_paths: BTreeSet::from([declaration.clone(), path.clone()]),
                resolver_candidate_paths: BTreeSet::from([path]),
                conservative_specifiers: BTreeSet::new(),
                transitive_trigger_paths: BTreeSet::new(),
            }
        })
        .collect()
}
