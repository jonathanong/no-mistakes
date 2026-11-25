use super::support::{fixtures, load_catalog, Database};

#[test]
fn catalog_retains_ordinary_inheritance_separately_from_partitions() {
    let Some(database) = Database::create("inheritance") else {
        return;
    };
    database.load(&fixtures().join("inheritance.sql"));
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("catalog.json");
    let output = database.generate("family", None, &path);
    assert!(output.status.success(), "{:?}", output);
    let catalog: serde_json::Value =
        serde_json::from_slice(&std::fs::read(&path).unwrap()).unwrap();
    assert_eq!(
        catalog["tables"]["child"]["inheritedFrom"],
        serde_json::json!(["parent", "shared.other_parent"])
    );
    assert!(catalog["tables"]["parent"].get("inheritedFrom").is_none());
    assert_eq!(catalog["tables"]["parent"]["hasDescendants"], true);
    assert_eq!(catalog["tables"]["child"]["hasDescendants"], false);
    assert_eq!(catalog["tables"]["partitioned"]["hasDescendants"], true);
    let shared = database.catalog("shared", None);
    assert_eq!(shared["tables"]["other_parent"]["hasDescendants"], true);
    let partition = &catalog["tables"]["partitioned_child"];
    assert_eq!(partition["partitionOf"], "partitioned");
    assert!(partition.get("inheritedFrom").is_none());
    let loaded = load_catalog(directory.path(), "catalog.json");
    assert_eq!(
        loaded.table("child").unwrap().inherited_from,
        ["parent", "shared.other_parent"]
    );
    let ordering = database.catalog("family", Some("ordering"));
    assert!(ordering["tables"]["child"].get("inheritedFrom").is_none());
    assert!(ordering["tables"]["child"].get("hasDescendants").is_none());
}
