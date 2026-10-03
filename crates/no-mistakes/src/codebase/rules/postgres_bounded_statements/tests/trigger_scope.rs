use super::{fixture_root, offenders, SchemaCatalog};
use crate::codebase::postgres::{extract_sql_statement_facts, TriggerEvent, TriggerTiming};

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
    let facts = extract_sql_statement_facts(&sources.read_path(&files[1]).unwrap());
    // Trigger side effects can touch every child row but do not widen direct target bounds.
    for fact in &facts.bounds[..3] {
        assert!(offenders(fact, &catalog).is_empty());
    }
    assert_eq!(offenders(&facts.bounds[3], &catalog).len(), 1);
}
