use super::support::{fixtures, load_catalog, Database};
use serde_json::Value;

fn edge_database(label: &str) -> Option<Database> {
    let database = Database::create(label)?;
    database.load(&fixtures().join("edges.sql"));
    Some(database)
}

fn keys(value: &Value) -> Vec<&str> {
    value
        .as_object()
        .unwrap()
        .keys()
        .map(String::as_str)
        .collect()
}

fn count(database: &Database, statement: &str) -> usize {
    database.query(statement).trim().parse().unwrap()
}

#[test]
fn partition_children_are_excluded_and_only_the_parent_carries_the_key() {
    let Some(database) = edge_database("partitions") else {
        return;
    };
    let tables = ["\"Order Items\"", "deferred", "event_refs", "events"];
    let complete = database.catalog("edge_demo", None);
    assert_eq!(keys(&complete["tables"]), tables);
    // One relation-selection policy: ordering coverage selects exactly the same tables.
    let ordering = database.catalog("edge_demo", Some("ordering"));
    assert_eq!(keys(&ordering["tables"]), tables);
    assert_eq!(ordering["coverage"], "ordering");
    assert_eq!(complete["coverage"], "complete");

    let events = &complete["tables"]["events"];
    assert_eq!(events["relationKind"], "partitioned table");
    assert_eq!(events["physicalPartition"]["key"], "RANGE (created_at)");
    // The index, trigger and constraints cloned onto each leaf are not separate entries.
    assert_eq!(keys(&events["indexes"]), ["events_pkey", "events_tenant"]);
    assert_eq!(keys(&events["triggers"]), ["events_touch"]);
    assert_eq!(
        events["primaryKey"]["columns"],
        serde_json::json!(["id", "created_at"])
    );
    assert_eq!(
        count(
            &database,
            "SELECT count(*) FROM pg_trigger WHERE tgname = 'events_touch'"
        ),
        3
    );
    // A foreign key that references the partitioned table is one entry, not one per partition.
    let foreign_keys = &complete["tables"]["event_refs"]["foreignKeys"];
    assert_eq!(keys(foreign_keys), ["event_refs_event_id_event_at_fkey"]);
    assert_eq!(count(&database, "SELECT count(*) FROM pg_constraint WHERE conrelid = 'edge_demo.event_refs'::regclass AND contype = 'f'"), 3);
}

#[test]
fn extension_owned_objects_are_excluded() {
    let Some(database) = edge_database("extensions") else {
        return;
    };
    let catalog = database.catalog("edge_demo", None);
    let extension_functions = "SELECT count(*) FROM pg_proc p JOIN pg_depend d ON d.objid = p.oid \
        AND d.deptype = 'e' WHERE p.pronamespace = 'edge_demo'::regnamespace";
    assert!(
        count(&database, extension_functions) > 40,
        "the extensions own many functions"
    );
    let functions = keys(&catalog["functions"]);
    assert!(!functions
        .iter()
        .any(|key| key.contains("citext") || key.contains("pg_stat") || key.contains("member")));
    assert!(functions.contains(&"touch()"));
    assert_eq!(keys(&catalog["enums"]), ["\"Mood\"", "tone"]);
    assert_eq!(keys(&catalog["views"]), ["order_items_m", "order_items_v"]);
    assert!(!keys(&catalog["tables"]).contains(&"member_table"));
    for owned in ["member_table", "member_view", "pg_stat_statements"] {
        let exists = format!("SELECT count(*) FROM pg_class WHERE relname = '{owned}'");
        assert_eq!(
            count(&database, &exists),
            1,
            "{owned} exists but is extension-owned"
        );
    }
}

#[test]
fn internal_triggers_and_not_null_constraints_are_not_catalog_facts() {
    let Some(database) = edge_database("internal") else {
        return;
    };
    let catalog = database.catalog("edge_demo", None);
    assert!(
        count(
            &database,
            "SELECT count(*) FROM pg_trigger WHERE tgisinternal"
        ) > 4
    );
    for table in ["\"Order Items\"", "deferred", "event_refs"] {
        assert!(
            keys(&catalog["tables"][table]["triggers"]).is_empty(),
            "{table}"
        );
    }
    // PostgreSQL 18 stores NOT NULL as contype 'n'; nullability comes from the column instead.
    assert!(
        count(
            &database,
            "SELECT count(*) FROM pg_constraint WHERE contype = 'n'"
        ) > 4
    );
    let items = &catalog["tables"]["\"Order Items\""];
    assert_eq!(
        keys(&items["checkConstraints"]),
        ["\"Odd.Check\"", "\"Order Items_note_check\""]
    );
    assert_eq!(items["columns"]["Id"]["nullable"], false);
    assert_eq!(items["columns"]["Parent Id"]["nullable"], true);
    assert_eq!(
        items["checkConstraints"]["\"Odd.Check\""]["validated"],
        false
    );
    assert!(items["checkConstraints"]["\"Odd.Check\""]["definition"]
        .as_str()
        .unwrap()
        .ends_with("NOT VALID"));
    assert_eq!(
        items["checkConstraints"]["\"Order Items_note_check\""]["validated"],
        true
    );
}

#[test]
fn only_functions_and_procedures_with_distinct_stable_overload_keys() {
    let Some(database) = edge_database("functions") else {
        return;
    };
    let catalog = database.catalog("edge_demo", None);
    assert_eq!(
        keys(&catalog["functions"]),
        [
            "do_it(IN a integer)",
            "over(a integer)",
            "over(a text)",
            "over(a tone, b other.shade)",
            "touch()"
        ]
    );
    assert_eq!(
        count(
            &database,
            "SELECT count(*) FROM pg_proc WHERE proname = 'sum_it' AND prokind = 'a'"
        ),
        1
    );
    let procedure = catalog["functions"]["do_it(IN a integer)"]["definition"]
        .as_str()
        .unwrap();
    assert!(procedure.starts_with("CREATE OR REPLACE PROCEDURE edge_demo.do_it"));
    assert_eq!(catalog, database.catalog("edge_demo", None));
}

#[test]
fn other_relation_kinds_are_not_catalog_tables() {
    let Some(database) = edge_database("kinds") else {
        return;
    };
    let catalog = database.catalog("edge_demo", None);
    let listed = keys(&catalog["tables"]);
    for absent in [
        "plain_seq",
        "foreign_things",
        "member_table",
        "order_items_v",
    ] {
        assert!(!listed.contains(&absent), "{absent}");
    }
    assert_eq!(
        count(
            &database,
            "SELECT count(*) FROM pg_class WHERE relkind = 'S' AND relname = 'plain_seq'"
        ),
        1
    );
    assert_eq!(
        count(
            &database,
            "SELECT count(*) FROM pg_class WHERE relkind = 'f'"
        ),
        1
    );
    // TOAST tables are relations too, in their own schema, and never catalog tables.
    assert!(
        count(
            &database,
            "SELECT count(*) FROM pg_class WHERE relkind = 't'"
        ) > 0
    );
    assert!(keys(&database.catalog("pg_toast", None)["tables"]).is_empty());
}

#[test]
fn temporary_tables_are_not_catalog_tables() {
    let Some(database) = edge_database("temporary") else {
        return;
    };
    // Hold a session open so its temporary table is visible in pg_class while we generate.
    let mut holder = std::process::Command::new("psql")
        .args([
            "-X",
            "--no-password",
            "-q",
            "-c",
            "CREATE TEMP TABLE temp_probe (id integer)",
        ])
        .args(["-c", "SELECT pg_sleep(60)"])
        .arg(database.url())
        .spawn()
        .unwrap();
    let probe = "SELECT n.nspname FROM pg_class c JOIN pg_namespace n ON n.oid = c.relnamespace \
                 WHERE c.relname = 'temp_probe' AND c.relpersistence = 't'";
    let mut schema = String::new();
    for _ in 0..100 {
        schema = database.query(probe).trim().to_string();
        if !schema.is_empty() {
            break;
        }
        std::thread::sleep(std::time::Duration::from_millis(100));
    }
    let catalog = (!schema.is_empty()).then(|| database.catalog(&schema, None));
    holder.kill().unwrap();
    holder.wait().unwrap();
    assert!(
        schema.starts_with("pg_temp_"),
        "the temporary table never appeared"
    );
    assert!(keys(&catalog.unwrap()["tables"]).is_empty());
}

#[test]
fn types_render_relative_to_the_selected_schema() {
    let Some(database) = edge_database("types") else {
        return;
    };
    let catalog = database.catalog("edge_demo", None);
    let columns = &catalog["tables"]["\"Order Items\""]["columns"];
    let data_type = |column: &str| columns[column]["dataType"].as_str().unwrap();
    // A type defined in the selected schema renders unqualified and equals the enum's key.
    let enums = keys(&catalog["enums"]);
    assert_eq!(data_type("mood"), "\"Mood\"");
    assert_eq!(data_type("moods"), "\"Mood\"[]");
    assert_eq!((data_type("tone"), data_type("tones")), ("tone", "tone[]"));
    assert!(enums.contains(&data_type("mood")) && enums.contains(&data_type("tone")));
    assert_eq!(
        data_type("email"),
        "citext",
        "an extension type in the selected schema"
    );
    // Types from other schemas stay qualified.
    assert_eq!(
        (data_type("shade"), data_type("shades")),
        ("other.shade", "other.shade[]")
    );
    // Seen from the other schema, the same two types swap.
    let other = database.catalog("other", None);
    let consumers = &other["tables"]["consumers"]["columns"];
    assert_eq!(consumers["tone"]["dataType"], "edge_demo.tone");
    assert_eq!(consumers["shade"]["dataType"], "shade");
    assert_eq!(keys(&other["enums"]), ["shade"]);
}

#[test]
fn column_facts_comments_and_foreign_key_actions_are_observed() {
    let Some(database) = edge_database("facts") else {
        return;
    };
    let catalog = database.catalog("edge_demo", None);
    let items = &catalog["tables"]["\"Order Items\""];
    let column = |name: &str| &items["columns"][name];
    assert_eq!(column("total")["generated"], "virtual");
    assert_eq!(column("stored_total")["generated"], "stored");
    assert_eq!(
        column("total")["generatedExpression"],
        "length((\"Id\")::text)"
    );
    assert_eq!(column("total")["defaultExpression"], Value::Null);
    assert_eq!(column("ident")["identity"], "d");
    assert_eq!(column("moods")["defaultExpression"], "'{}'::\"Mood\"[]");
    assert_eq!(column("note")["comment"], "a note");
    assert_eq!(items["comment"], "it's quoted");
    let positions: Vec<_> = items["columns"]
        .as_object()
        .unwrap()
        .values()
        .map(|c| c["ordinalPosition"].as_u64().unwrap())
        .collect();
    assert_eq!(positions.iter().max(), Some(&13));
    assert_eq!(keys(&items["uniqueConstraints"]), ["\"Unique.Pair\""]);
    assert_eq!(
        items["uniqueConstraints"]["\"Unique.Pair\""]["columns"],
        serde_json::json!(["\"Id\"", "mood"])
    );
    assert_eq!(items["primaryKey"]["columns"], serde_json::json!(["Id"]));
    let parent = &items["foreignKeys"]["\"Order Items_Parent Id_fkey\""];
    assert_eq!(
        (parent["onDelete"].as_str(), parent["onUpdate"].as_str()),
        (Some("SET NULL"), Some("CASCADE"))
    );
    assert_eq!(parent["validated"], true);
    let loose = &items["foreignKeys"]["\"Loose Parent\""];
    assert_eq!(loose["columns"], serde_json::json!(["Parent Id", "mood"]));
    assert_eq!(
        loose["referencedColumns"],
        serde_json::json!(["Id", "mood"])
    );
    assert_eq!(loose["referencedTable"], "\"Order Items\"");
    assert_eq!(loose["validated"], false);
    assert_eq!(catalog["views"]["order_items_v"]["comment"], "a view");
    assert_eq!(catalog["views"]["order_items_m"]["materialized"], true);
    assert_eq!(catalog["views"]["order_items_m"]["comment"], Value::Null);
    assert_eq!(
        catalog["enums"]["tone"]["values"],
        serde_json::json!(["c", "b", "a"])
    );
    let mixed = &items["indexes"]["\"Mixed Index\""];
    assert_eq!(mixed["keys"][1]["column"], "Parent Id");
    assert_eq!(mixed["keys"][1]["expression"], "\"Parent Id\"");
}

#[test]
fn the_reader_derives_the_model_from_the_generated_catalog() {
    let Some(database) = edge_database("reader") else {
        return;
    };
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("catalog.json");
    assert!(database.generate("edge_demo", None, &path).status.success());
    let catalog = load_catalog(directory.path(), "catalog.json");
    assert_eq!(
        catalog.coverage(),
        no_mistakes::codebase::postgres::CatalogCoverage::Complete
    );
    // Quoted and mixed-case names round-trip through name normalization.
    for spelling in ["\"Order Items\"", "edge_demo.\"Order Items\""] {
        assert!(catalog.relation(spelling).is_some(), "{spelling}");
    }
    let items = catalog.table("\"Order Items\"").unwrap();
    assert_eq!(items.primary_key.as_deref(), Some(&["Id".to_string()][..]));
    assert!(items
        .columns
        .iter()
        .any(|column| column.name == "Parent Id"));
    let events = catalog.table("events").unwrap();
    let key = events.partition_key.as_ref().unwrap();
    assert_eq!(
        key.strategy,
        no_mistakes::codebase::postgres::PartitionStrategy::Range
    );
    let touch = &events.triggers[0];
    assert_eq!(
        (
            touch.update_columns.clone(),
            touch.for_each_row,
            touch.function.as_str()
        ),
        (vec!["tenant".to_string()], true, "touch")
    );
    let function = catalog
        .functions()
        .find(|function| function.key == "touch()")
        .unwrap();
    assert!(function.returns_trigger && function.language.as_deref() == Some("plpgsql"));
    assert!(function.body.as_deref().unwrap().contains("RETURN NEW"));
    let overloads = catalog
        .functions()
        .filter(|function| function.name == "over")
        .count();
    assert_eq!(overloads, 3);
    assert_eq!(
        catalog.enums().find(|e| e.name == "tone").unwrap().values,
        ["c", "b", "a"]
    );
    assert!(catalog
        .views()
        .any(|view| view.materialized && view.name == "order_items_m"));
    assert!(matches!(
        catalog.resolve_constraint("\"Order Items\"", "\"Unique.Pair\""),
        no_mistakes::codebase::postgres::ResolvedArbiter::Exact(_)
    ));
}
