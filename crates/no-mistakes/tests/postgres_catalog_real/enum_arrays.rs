use super::support::{check, copy_tree, fixtures, Database};

#[test]
fn external_array_elements_retain_enum_metadata_without_scalar_references() {
    let Some(database) = Database::create("external_enum_arrays") else {
        return;
    };
    let fixture = fixtures().join("external-enum-arrays");
    database.load(&fixture.join("schema.sql"));
    let project = tempfile::tempdir().unwrap();
    copy_tree(&fixture, project.path());
    let output = database.generate(
        "array_demo",
        Some("complete"),
        &project.path().join("schema.json"),
    );
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let catalog: serde_json::Value =
        serde_json::from_slice(&std::fs::read(project.path().join("schema.json")).unwrap())
            .unwrap();
    assert_eq!(
        catalog["tables"]["accounts"]["columns"]["priorities"]["dataType"],
        "shared.array_priority[]"
    );
    assert_eq!(
        catalog["enums"]["shared.array_priority"]["values"],
        serde_json::json!(["low", "high"])
    );
    assert!(catalog["enums"].get("shared.unused_priority").is_none());
    assert!(catalog["enums"].get("shared.domain_priority").is_none());
    let (output, findings) = check(project.path(), &project.path().join(".no-mistakes.yml"));
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(findings.is_empty(), "{findings:#?}");
}
