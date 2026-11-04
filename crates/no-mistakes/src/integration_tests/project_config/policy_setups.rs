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
        .enumerate()
        .map(|(index, raw)| {
            let declaration = policy
                .setup_declaration_paths
                .get(index)
                .map(|path| root.join(path))
                .unwrap_or_else(|| declaration.clone());
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

#[cfg(test)]
mod tests {
    #[test]
    fn merged_setup_items_keep_their_individual_fragment_declarations() {
        let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../test-cases/config-v2/split-provenance/fixture/setup-items");
        let root = crate::codebase::ts_resolver::normalize_path(&root);
        let config = crate::config::v2::load_v2_config(&root, None).unwrap();
        let dependencies = super::dependencies(&root, &config.tests.vitest.projects["unit"]);
        assert_eq!(dependencies.len(), 2);
        for (dependency, name) in dependencies.iter().zip(["first", "second"]) {
            assert_eq!(
                dependency.declaration_path,
                root.join(format!(".no-mistakes/tests/{name}.yml"))
            );
            assert_eq!(
                dependency.resolved_path.as_ref(),
                Some(&crate::codebase::ts_resolver::normalize_path(
                    &root.join(format!("setup/{name}.ts"))
                ))
            );
            assert!(dependency
                .trigger_paths
                .contains(&dependency.declaration_path));
            assert_eq!(dependency.resolution_base, root);
        }
    }
}
