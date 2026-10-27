use super::{fixture_root, SchemaCatalog};
use crate::codebase::postgres::{TriggerEvent, TriggerTiming};

#[test]
fn catalog_trigger_fanout_stays_outside_direct_target_row_bounds() {
    let root = fixture_root();
    let files = [
        root.join("trigger-direct-scope.json"),
        root.join("sql/referential-actions.sql"),
    ];
    let sources = crate::codebase::rules::source_store_for_files(&files);
    let catalog = SchemaCatalog::load(&root, "trigger-direct-scope.json", &sources).unwrap();
    let trigger = &catalog.table("accounts").unwrap().triggers[0];
    assert_eq!(trigger.timing, TriggerTiming::After);
    assert_eq!(
        trigger.events,
        vec![TriggerEvent::Update, TriggerEvent::Delete]
    );
    assert!(trigger.for_each_row);
    assert_eq!(trigger.function, "fn_account_fanout");
    assert!(catalog.functions().any(|function| function
        .definition
        .contains("UPDATE orders SET account_id = NULL")));
}
