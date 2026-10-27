use super::alias::catalog_workspace_name;
use crate::codebase::workspaces::{CatalogHit, CatalogIndex, WorkspacePackage};
use std::collections::BTreeMap;
use std::path::PathBuf;

fn workspace() -> crate::codebase::workspaces::WorkspaceMap {
    crate::codebase::workspaces::WorkspaceMap::from_packages(vec![WorkspacePackage {
        name: "@acme/domain".to_string(),
        dir: PathBuf::from("/repo/packages/domain"),
        entry: None,
        exports: None,
        imports: None,
    }])
}

fn catalogs(specifier: &str, base: &str) -> CatalogIndex {
    let mut default_entries = BTreeMap::new();
    default_entries.insert(
        "@acme/domain".to_string(),
        CatalogHit {
            specifier: specifier.to_string(),
            base_dir: PathBuf::from(base),
        },
    );
    CatalogIndex {
        default_entries,
        named: BTreeMap::new(),
    }
}

fn resolved(protocol: &str, specifier: &str, base: &str) -> Option<String> {
    catalog_workspace_name(
        "@acme/domain",
        protocol,
        &workspace(),
        &catalogs(specifier, base),
    )
}

#[test]
fn catalog_protocols_match_direct_workspace_dependencies() {
    assert_eq!(
        resolved("catalog:", "workspace:*", "/repo"),
        Some("@acme/domain".to_string())
    );
    assert_eq!(
        resolved("catalog:", "workspace:~", "/repo"),
        Some("@acme/domain".to_string())
    );
    assert_eq!(
        resolved("catalog:", "workspace:^", "/repo"),
        Some("@acme/domain".to_string())
    );
    assert_eq!(
        resolved("catalog:default", "workspace:*", "/repo"),
        Some("@acme/domain".to_string())
    );
    assert_eq!(
        resolved("catalog:", "workspace:../packages/domain", "/repo/pnpm"),
        Some("@acme/domain".to_string())
    );
    assert_eq!(
        resolved("catalog:", "file:../packages/domain", "/repo/pnpm"),
        Some("@acme/domain".to_string())
    );
    assert_eq!(
        resolved("catalog:", "link:./domain", "/repo/packages"),
        Some("@acme/domain".to_string())
    );
    assert_eq!(resolved("catalog:", "^1.0.0", "/repo"), None);
    assert_eq!(resolved("catalog:", "workspace:^1.2.3", "/repo"), None);
    assert_eq!(resolved("catalog:", "file:/etc/domain", "/repo"), None);
    assert_eq!(
        resolved("catalog:", "link:https://example.test", "/repo"),
        None
    );
    assert_eq!(resolved("catalog:foo/bar", "workspace:*", "/repo"), None);
    assert_eq!(resolved("catalog:foo bar", "workspace:*", "/repo"), None);
    assert_eq!(resolved("catalog:foo\\bar", "workspace:*", "/repo"), None);
    assert_eq!(
        catalog_workspace_name(
            "@acme/missing",
            "catalog:",
            &workspace(),
            &catalogs("workspace:*", "/repo")
        ),
        None
    );
    let mut named = BTreeMap::new();
    named.insert(
        "@acme/domain".to_string(),
        CatalogHit {
            specifier: "workspace:^".to_string(),
            base_dir: PathBuf::from("/repo"),
        },
    );
    let index = CatalogIndex {
        default_entries: BTreeMap::new(),
        named: BTreeMap::from([("legacy".to_string(), named)]),
    };
    assert_eq!(
        catalog_workspace_name("@acme/domain", "catalog:legacy", &workspace(), &index),
        Some("@acme/domain".to_string())
    );
    assert_eq!(
        catalog_workspace_name("@acme/domain", "catalog:missing", &workspace(), &index),
        None
    );
}
