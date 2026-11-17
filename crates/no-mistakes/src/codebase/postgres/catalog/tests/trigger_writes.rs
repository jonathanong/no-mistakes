use super::super::TriggerEvent;

#[test]
fn trigger_effects_preserve_events_targets_and_conflict_updates() {
    let catalog = super::load_fixture("trigger-writes.json").unwrap();
    let effects = catalog.trigger_write_effects();
    assert_eq!(effects.len(), 3);
    let touch = effects
        .iter()
        .find(|effect| effect.events.contains(&TriggerEvent::Insert))
        .unwrap();
    assert_eq!(touch.source_relation, "orders");
    assert_eq!(touch.events, [TriggerEvent::Insert, TriggerEvent::Update]);
    assert_eq!(touch.update_columns, ["total"]);
    let writes: Vec<_> = touch
        .writes
        .iter()
        .map(|write| (write.relation.as_str(), write.event, write.columns.clone()))
        .collect();
    assert_eq!(
        writes,
        vec![
            (
                "accounts",
                TriggerEvent::Update,
                vec!["balance".to_string()]
            ),
            ("audit", TriggerEvent::Insert, vec![]),
            ("audit", TriggerEvent::Update, vec!["count".to_string()]),
            ("expired", TriggerEvent::Delete, vec![]),
        ]
    );
    let repeated = effects
        .iter()
        .find(|effect| effect.source_relation == "other_orders")
        .unwrap();
    assert_eq!(repeated.events, [TriggerEvent::Delete]);
    assert_eq!(repeated.writes, touch.writes);
    assert_eq!(effects[0].writes[0].relation, "accounts");
    assert!(std::ptr::eq(effects, catalog.trigger_write_effects()));
}

#[test]
fn trigger_functions_preserve_schema_identity_and_catalog_schema_fallback() {
    let catalog = super::load_fixture("trigger-writes-qualified.json").unwrap();
    let effects = catalog.trigger_write_effects();
    let targets: Vec<_> = effects
        .iter()
        .map(|effect| {
            (
                effect.source_relation.as_str(),
                effect.writes[0].relation.as_str(),
            )
        })
        .collect();
    assert_eq!(
        targets,
        [
            ("audit_orders", "audit_accounts"),
            ("orders", "accounts"),
            ("same_orders", "accounts"),
        ]
    );
    assert_eq!(
        catalog.table("orders").unwrap().triggers[0].function,
        "touch"
    );
}

#[test]
fn unsupported_trigger_bodies_and_ambiguous_functions_supply_no_inferred_writes() {
    let catalog = super::load_fixture("trigger-writes-coverage.json").unwrap();
    let effects = catalog.trigger_write_effects();
    assert_eq!(effects.len(), 1, "{effects:?}");
    assert_eq!(effects[0].source_relation, "source_upsert_nothing");
    assert_eq!(effects[0].events, [TriggerEvent::Insert]);
    assert_eq!(effects[0].writes.len(), 1);
    assert_eq!(effects[0].writes[0].relation, "audit");
    assert_eq!(effects[0].writes[0].event, TriggerEvent::Insert);
    assert!(effects[0].writes[0].columns.is_empty());
}
