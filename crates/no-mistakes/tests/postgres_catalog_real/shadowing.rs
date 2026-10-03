use super::support::{fixtures, load_catalog, Database};

fn shadow_database(label: &str) -> Option<Database> {
    let database = Database::create(label)?;
    database.load(&fixtures().join("shadowing.sql"));
    Some(database)
}

#[test]
fn an_exclusion_constraint_backs_its_index() {
    let Some(database) = shadow_database("exclusion") else {
        return;
    };
    for coverage in [None, Some("ordering")] {
        let catalog = database.catalog("shadow_demo", coverage);
        let indexes = &catalog["tables"]["rooms"]["indexes"];
        // The index behind EXCLUDE is named by the constraint, like a primary key or unique constraint.
        assert_eq!(indexes["rooms_no_overlap"]["constraintBacked"], true);
        assert_eq!(indexes["rooms_pkey"]["constraintBacked"], true);
        assert_eq!(indexes["rooms_during_idx"]["constraintBacked"], false);
    }
}

#[test]
fn an_enum_is_keyed_by_the_type_a_column_renders() {
    let Some(database) = shadow_database("shadowed_enum") else {
        return;
    };
    let catalog = database.catalog("shadow_demo", None);
    let columns = &catalog["tables"]["rooms"]["columns"];
    // The enum named `text` is not visible once pg_catalog is searched first, so it renders
    // qualified; `tone` is visible and stays bare. pg_catalog's own text is unaffected.
    assert_eq!(columns["kinds"]["dataType"], "shadow_demo.text[]");
    assert_eq!(columns["tones"]["dataType"], "tone[]");
    assert_eq!(columns["title"]["dataType"], "text");
    let enums = catalog["enums"].as_object().unwrap();
    assert_eq!(
        enums.keys().map(String::as_str).collect::<Vec<_>>(),
        ["empty_kind", "shadow_demo.text", "tone"]
    );
    // Every enum column's element type finds its enum under the key the generator wrote.
    for column in ["kinds", "tones"] {
        let rendered = columns[column]["dataType"].as_str().unwrap();
        let element = rendered.strip_suffix("[]").unwrap();
        assert!(enums.contains_key(element), "{column}: {element}");
    }
    assert_eq!(enums["shadow_demo.text"]["values"][1], "live");
}

#[test]
fn only_a_trigger_that_fires_in_a_normal_session_is_a_catalog_trigger() {
    let Some(database) = shadow_database("triggers") else {
        return;
    };
    let catalog = database.catalog("shadow_demo", None);
    let triggers: Vec<&str> = catalog["tables"]["rooms"]["triggers"]
        .as_object()
        .unwrap()
        .keys()
        .map(String::as_str)
        .collect();
    // rooms_off is disabled and rooms_replica fires only for the replica role.
    assert_eq!(triggers, ["rooms_always", "rooms_live"]);
    let all = database.query(
        "SELECT count(*) FROM pg_trigger WHERE tgrelid = 'shadow_demo.rooms'::regclass \
         AND NOT tgisinternal",
    );
    assert_eq!(all.trim(), "4");
}

#[test]
fn an_enum_with_no_labels_has_an_empty_value_list_the_reader_accepts() {
    let Some(database) = shadow_database("empty_enum") else {
        return;
    };
    let catalog = database.catalog("shadow_demo", None);
    assert_eq!(
        catalog["enums"]["empty_kind"]["values"],
        serde_json::json!([])
    );
    // The loader requires an array of strings, so a null would be rejected here.
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("catalog.json");
    assert!(database
        .generate("shadow_demo", None, &path)
        .status
        .success());
    let loaded = load_catalog(directory.path(), "catalog.json");
    assert!(loaded.enums().any(|enum_type| enum_type.values.is_empty()));
}
