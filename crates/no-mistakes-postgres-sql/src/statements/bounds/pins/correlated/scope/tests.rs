use super::*;

#[test]
fn snapshots_share_records_without_seeing_later_names_or_tables() {
    let mut scope = Scope::default();
    assert!(!scope.relations.contains(&vec!["accounts".to_string()]));
    scope.relations.insert(vec!["accounts".into()]);
    scope.columns.insert("initial_column".into());
    scope.whole_rows.insert("accounts".into());
    scope.tables.push("accounts".into());
    let earlier = scope.clone();
    // Identity locks the shallow snapshot path, not merely equal output.
    assert!(Rc::ptr_eq(
        scope.relations.entries.as_ref().unwrap(),
        earlier.relations.entries.as_ref().unwrap(),
    ));
    assert!(Rc::ptr_eq(
        scope.columns.entries.as_ref().unwrap(),
        earlier.columns.entries.as_ref().unwrap(),
    ));
    assert!(Rc::ptr_eq(
        scope.tables.entries.as_ref().unwrap(),
        earlier.tables.entries.as_ref().unwrap(),
    ));
    assert!(Rc::ptr_eq(
        scope.whole_rows.entries.as_ref().unwrap(),
        earlier.whole_rows.entries.as_ref().unwrap()
    ));
    scope.whole_rows.insert("later".into());
    assert!(!earlier.whole_rows.contains("later"));
    assert!(earlier.whole_rows.contains("accounts"));
    scope
        .relations
        .extend([vec!["later".into()], vec!["accounts".into()]]);
    scope.columns.extend(["id".into(), "initial_column".into()]);
    scope.tables.push("invoices".into());
    scope.foreign = true;
    assert!(!earlier.relations.contains(&vec!["later".to_string()]));
    assert!(earlier.relations.contains(&vec!["accounts".to_string()]));
    assert!(scope.relations.contains(&vec!["later".to_string()]));
    assert!(earlier.columns.contains("initial_column"));
    assert!(!earlier.columns.contains("id"));
    let mut reads = vec![SqlBareRead {
        column: "id".into(),
        tables: Vec::new(),
    }];
    earlier.resolve_reads(&mut reads);
    assert_eq!(reads[0].tables, ["accounts"]);
    scope.resolve_reads(&mut reads);
    assert!(reads.is_empty());
}

#[test]
fn empty_snapshot_keeps_its_empty_visibility_after_owner_appends() {
    let mut scope = Scope::default();
    let earlier = scope.clone();
    scope.relations.insert(vec!["later".into()]);
    scope.columns.insert("id".into());
    scope.tables.push("accounts".into());
    assert!(!earlier.relations.contains(&vec!["later".to_string()]));
    let mut reads = vec![SqlBareRead {
        column: "id".into(),
        tables: vec!["nested".into(), "nested".into()],
    }];
    earlier.resolve_reads(&mut reads);
    assert_eq!(reads[0].tables, ["nested"]);
    scope.resolve_reads(&mut reads);
    assert!(reads.is_empty());
}

#[test]
fn joined_namespaces_keep_internal_visibility_without_resurrecting_hidden_children() {
    let mut scope = Scope::default();
    scope.relations.insert(vec!["preceding".into()]);
    scope.whole_rows.insert("preceding".into());
    scope.record_base_table("preceding".into(), false);
    let before = scope.clone();
    scope.relations.insert(vec!["after_snapshot".into()]);
    let isolated = before.relations.isolated();
    assert!(isolated.contains(&vec!["preceding".into()]));
    assert!(!isolated.contains(&vec!["after_snapshot".into()]));
    let outside = scope.begin_join();
    scope.relations.insert(vec!["hidden".into()]);
    scope.whole_rows.insert("hidden".into());
    scope.record_base_table("hidden".into(), false);
    let internal = scope.clone();
    scope.finish_join(outside);
    scope.relations.insert(vec!["later".into()]);
    scope.whole_rows.insert("later".into());
    scope.record_base_table("later".into(), false);
    assert!(!scope.relations.contains(&vec!["hidden".into()]));
    assert!(!scope.whole_rows.contains("hidden"));
    assert!(internal.relations.contains(&vec!["hidden".into()]));
    assert!(internal.whole_rows.contains("hidden"));
    assert!(!internal.relations.contains(&vec!["later".into()]));
    assert_eq!(scope.qualified_candidates().tables, ["preceding", "later"]);
    assert_eq!(
        internal.qualified_candidates().tables,
        ["preceding", "hidden"]
    );
    assert_eq!(scope.tables.visible(), ["preceding", "hidden", "later"]);
}
