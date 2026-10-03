use super::support::{fixtures, Database};

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
        ["shadow_demo.text", "tone"]
    );
    // Every enum column's element type finds its enum under the key the generator wrote.
    for column in ["kinds", "tones"] {
        let rendered = columns[column]["dataType"].as_str().unwrap();
        let element = rendered.strip_suffix("[]").unwrap();
        assert!(enums.contains_key(element), "{column}: {element}");
    }
    assert_eq!(enums["shadow_demo.text"]["values"][1], "live");
}
