use super::super::{
    CatalogCheck, CatalogColumn, CatalogForeignKey, CatalogIndexInfo, CatalogIndexKey,
    CatalogUnique, GeneratedKind, PartitionKey, PartitionKeyElement, PartitionStrategy,
    RelationKind, ResolvedArbiter, TriggerEvent, TriggerTiming,
};
use super::load_fixture;

#[test]
fn full_snapshot_fills_the_read_only_model() {
    let catalog = load_fixture("full.json").unwrap();
    assert_eq!(
        catalog
            .tables()
            .map(|table| table.name.as_str())
            .collect::<Vec<_>>(),
        ["accounts", "events"]
    );
    assert!(catalog.table("missing").is_none());

    let accounts = catalog.table("accounts").unwrap();
    assert_eq!(accounts.relation_kind, RelationKind::Table);
    assert_eq!(accounts.comment.as_deref(), Some("customer accounts"));
    assert_eq!(
        accounts.primary_key.as_deref(),
        Some(["id".to_string()].as_slice())
    );
    assert_eq!(
        accounts.columns,
        vec![
            CatalogColumn {
                name: "id".to_string(),
                data_type: "uuid".to_string(),
                nullable: false,
                default_expression: None,
                generated: None,
                generated_expression: None,
                identity: None,
                comment: Some("primary key".to_string()),
                ordinal_position: 1,
            },
            CatalogColumn {
                name: "display_name".to_string(),
                data_type: "text".to_string(),
                nullable: true,
                default_expression: None,
                generated: None,
                generated_expression: None,
                identity: None,
                comment: None,
                ordinal_position: 2,
            },
            CatalogColumn {
                name: "is_active".to_string(),
                data_type: "boolean".to_string(),
                nullable: false,
                default_expression: Some("true".to_string()),
                generated: None,
                generated_expression: None,
                identity: None,
                comment: None,
                ordinal_position: 3,
            },
            CatalogColumn {
                name: "created_at".to_string(),
                data_type: "timestamp with time zone".to_string(),
                nullable: false,
                default_expression: None,
                generated: Some(GeneratedKind::Virtual),
                generated_expression: Some("uuid_extract_timestamp(id)".to_string()),
                identity: None,
                comment: None,
                ordinal_position: 4,
            },
        ]
    );
    assert_eq!(
        accounts.foreign_keys,
        vec![CatalogForeignKey {
            name: "accounts_owner_fk".to_string(),
            columns: vec!["owner_id".to_string()],
            referenced_table: "users".to_string(),
            referenced_columns: vec!["id".to_string()],
            on_delete: "set null".to_string(),
            on_update: "no action".to_string(),
            validated: false,
        }]
    );
    assert_eq!(
        accounts.check_constraints,
        vec![CatalogCheck {
            name: "accounts_active_check".to_string(),
            definition: "CHECK ((is_active = true))".to_string(),
            validated: true,
        }]
    );
    assert_eq!(
        accounts.unique_constraints,
        vec![CatalogUnique {
            name: "accounts_display_name_key".to_string(),
            columns: vec!["display_name".to_string()],
        }]
    );
    assert_eq!(
        accounts.indexes,
        vec![
            index(IndexExpect {
                name: "accounts_active_idx",
                unique: false,
                primary: false,
                constraint_backed: false,
                access_method: "btree",
                predicate: Some("is_active"),
                column: "is_active",
                definition: "CREATE INDEX accounts_active_idx ON accounts USING btree (is_active) WHERE is_active",
            }),
            index(IndexExpect {
                name: "accounts_display_name_hash",
                unique: true,
                primary: false,
                constraint_backed: false,
                access_method: "hash",
                predicate: None,
                column: "display_name",
                definition: "CREATE UNIQUE INDEX accounts_display_name_hash ON accounts USING hash (display_name)",
            }),
            index(IndexExpect {
                name: "accounts_pkey",
                unique: true,
                primary: true,
                constraint_backed: true,
                access_method: "btree",
                predicate: None,
                column: "id",
                definition: "CREATE UNIQUE INDEX accounts_pkey ON accounts USING btree (id)",
            }),
        ]
    );
    let trigger = &accounts.triggers[0];
    assert_eq!(trigger.name, "trigger_accounts_touch");
    assert_eq!(trigger.timing, TriggerTiming::Before);
    assert_eq!(trigger.events, vec![TriggerEvent::Update]);
    assert_eq!(trigger.update_columns, vec!["display_name".to_string()]);
    assert!(trigger.for_each_row);
    assert_eq!(trigger.function, "fn_touch");
    assert_eq!(trigger.arguments, vec!["updated_at".to_string()]);
    assert_eq!(trigger.when, None);
    assert!(trigger.matches(
        "fn_touch",
        TriggerTiming::Before,
        &[TriggerEvent::Update],
        true
    ));
    assert!(trigger.matches("fn_touch", TriggerTiming::Before, &[], true));
    assert!(!trigger.matches(
        "fn_touch",
        TriggerTiming::Before,
        &[TriggerEvent::Insert],
        true
    ));
    assert!(!trigger.matches(
        "fn_other",
        TriggerTiming::Before,
        &[TriggerEvent::Update],
        true
    ));
    assert!(!trigger.matches(
        "fn_touch",
        TriggerTiming::After,
        &[TriggerEvent::Update],
        true
    ));
    assert!(!trigger.matches(
        "fn_touch",
        TriggerTiming::Before,
        &[TriggerEvent::Update],
        false
    ));
    assert!(matches!(
        catalog.resolve_columns("accounts", &["id".to_string()], None),
        ResolvedArbiter::Exact(_)
    ));
    assert_eq!(
        catalog.resolve_columns("accounts", &["display_name".to_string()], None),
        ResolvedArbiter::Unresolved
    );

    let events = catalog.table("events").unwrap();
    assert_eq!(events.relation_kind, RelationKind::PartitionedTable);
    assert_eq!(events.primary_key, None);
    assert_eq!(events.columns, Vec::<CatalogColumn>::new());
    assert_eq!(
        events.partition_key,
        Some(PartitionKey {
            strategy: PartitionStrategy::Range,
            elements: vec![PartitionKeyElement::Column("account_id".to_string())],
        })
    );

    let functions = catalog.functions().collect::<Vec<_>>();
    assert_eq!(functions[0].key, "fn_touch");
    assert_eq!(functions[0].name, "fn_touch");
    assert_eq!(functions[0].signature, None);
    assert_eq!(functions[0].language.as_deref(), Some("plpgsql"));
    assert!(functions[0].returns_trigger);
    assert_eq!(
        functions[0].body.as_deref(),
        Some("\nBEGIN\n  NEW.updated_at = now();\n  RETURN NEW; -- $$\nEND\n")
    );
    assert!(functions[0].body.as_deref().unwrap().contains("$$"));
    assert_eq!(functions[1].key, "fn_touch(col text)");
    assert_eq!(functions[1].name, "fn_touch");
    assert_eq!(functions[1].signature.as_deref(), Some("col text"));
    assert_eq!(functions[1].language.as_deref(), Some("plpgsql"));
    assert!(functions[1].returns_trigger);
    assert_eq!(
        functions[1].body.as_deref(),
        Some("\nBEGIN\n  RETURN NEW;\nEND\n")
    );

    let enums = catalog.enums().collect::<Vec<_>>();
    assert_eq!(enums[0].name, "account_plans");
    assert_eq!(enums[0].values, ["free".to_string(), "pro".to_string()]);
    let views = catalog.views().collect::<Vec<_>>();
    assert_eq!(views[0].name, "account_counts");
    assert!(views[0].materialized);
    assert_eq!(views[0].definition, "SELECT count(*) AS n FROM accounts");
    assert_eq!(views[0].comment, None);
    assert_eq!(views[1].name, "active_accounts");
    assert!(!views[1].materialized);
    assert_eq!(views[1].comment.as_deref(), Some("active accounts"));
}

struct IndexExpect<'a> {
    name: &'a str,
    unique: bool,
    primary: bool,
    constraint_backed: bool,
    access_method: &'a str,
    predicate: Option<&'a str>,
    column: &'a str,
    definition: &'a str,
}

fn index(expect: IndexExpect<'_>) -> CatalogIndexInfo {
    CatalogIndexInfo {
        name: expect.name.to_string(),
        unique: expect.unique,
        primary: expect.primary,
        constraint_backed: expect.constraint_backed,
        access_method: expect.access_method.to_string(),
        keys: vec![CatalogIndexKey {
            column: Some(expect.column.to_string()),
            expression: expect.column.to_string(),
        }],
        predicate: expect.predicate.map(str::to_string),
        definition: expect.definition.to_string(),
    }
}
