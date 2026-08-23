use crate::integration_tests::types::ConfigProject;
use std::path::Path;

pub(super) fn assert_named_import_member_ownership(root: &Path, projects: &[ConfigProject]) {
    let setup = |name: &str| {
        &projects
            .iter()
            .find(|project| project.policy_name.as_deref() == Some(name))
            .unwrap_or_else(|| panic!("missing project {name}"))
            .vitest_setup
    };
    let named_member = setup("named-import-member");
    assert_eq!(named_member.len(), 5, "{named_member:#?}");
    assert_eq!(
        named_member
            .iter()
            .map(|dependency| dependency.specifier.as_deref())
            .collect::<Vec<_>>(),
        [
            Some("../shared-setup/named-member.ts"),
            Some("../shared-setup/named-member-source.ts"),
            Some("../shared-setup/named-member-imported.ts"),
            Some("../shared-setup/named-member-star.ts"),
            Some("../shared-setup/named-member-commonjs.ts"),
        ]
    );
    assert_eq!(
        named_member
            .iter()
            .filter_map(|dependency| dependency.resolved_path.as_ref())
            .map(|path| path.file_name().unwrap().to_string_lossy().to_string())
            .collect::<Vec<_>>(),
        [
            "named-member.ts",
            "named-member-source.ts",
            "named-member-imported.ts",
            "named-member-star.ts",
            "named-member-commonjs.ts",
        ]
    );
    assert!(named_member
        .iter()
        .all(|dependency| dependency.resolution_base == root.join("named-member-owner")));
    assert_eq!(
        named_member[0].declaration_path,
        root.join("config/named-member-setups.ts")
    );
    assert!(named_member[0]
        .trigger_paths
        .contains(&root.join("config/named-member-setups.ts")));
    assert!(named_member[0]
        .resolver_candidate_paths
        .contains(&root.join("shared-setup/named-member.ts")));
    assert!(named_member[1]
        .trigger_paths
        .contains(&root.join("config/named-member-source-reexport.ts")));
    assert!(named_member[1]
        .trigger_paths
        .contains(&root.join("config/named-member-source-leaf.ts")));
    assert_eq!(
        named_member[4].declaration_path,
        root.join("config/named-member-commonjs.cjs")
    );
    assert!(named_member[2]
        .trigger_paths
        .contains(&root.join("config/named-member-imported-reexport.ts")));
    assert!(named_member[3]
        .trigger_paths
        .contains(&root.join("config/named-member-star-barrel.ts")));

    let named_member_cycle = setup("named-import-member-cycle");
    assert_eq!(named_member_cycle.len(), 1);
    assert_eq!(named_member_cycle[0].specifier, None);
    for helper in ["named-member-cycle-a.ts", "named-member-cycle-b.ts"] {
        assert!(named_member_cycle[0]
            .trigger_paths
            .contains(&root.join("config").join(helper)));
    }
}
