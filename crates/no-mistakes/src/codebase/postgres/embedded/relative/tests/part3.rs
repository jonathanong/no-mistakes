use super::*;

#[test]
fn relative_imports_stay_unresolved_until_projection() {
    let pending = extract("scoped-relative-pending.ts", &scoped("@example/db"));
    assert!(pending.calls.is_empty());
    assert!(pending.matched_factory_names.is_empty());
    assert!(pending.matched_type_names.is_empty());
    assert_eq!(pending.pending_relative.candidates.len(), 2);
    assert_eq!(pending.pending_relative.calls.len(), 9);
    assert!(pending.pending_relative.candidates.iter().any(|candidate| {
        candidate.imported_name == "openTransaction" && candidate.factory && !candidate.type_import
    }));
    assert!(pending.pending_relative.candidates.iter().any(|candidate| {
        candidate.imported_name == "TxExecutor" && !candidate.factory && candidate.type_import
    }));
    let recorded: Vec<_> = pending
        .pending_relative
        .calls
        .iter()
        .filter_map(|call| call.call.sql_text.as_deref())
        .map(|sql| sql.trim_start_matches("SELECT id FROM "))
        .collect();
    assert_eq!(
        recorded,
        [
            "relative_pending",
            "relative_member_factory",
            "relative_member_alias",
            "relative_plain",
            "relative_typed",
            "relative_member_type",
            "relative_member_type_alias",
            "relative_union",
            "relative_inline"
        ]
    );
    let both = EmbeddedSqlOptions::configured("@example/db", &[]).with_scoped_executors(
        &["openTransaction".into()],
        &["openTransaction".into(), "TxExecutor".into()],
    );
    let flagged = extract("scoped-relative-pending.ts", &both);
    assert!(flagged.pending_relative.candidates.iter().any(|candidate| {
        candidate.imported_name == "openTransaction" && candidate.factory && candidate.type_import
    }));
}

#[test]
fn relative_member_escapes_retain_unanalyzable_pending_calls() {
    let mut pending = extract("relative-member-escape.ts", &scoped("@example/db"));
    assert_eq!(pending.pending_relative.calls.len(), 3);
    assert!(pending.pending_relative.calls[..2]
        .iter()
        .all(|call| call.call.is_unanalyzable()));
    assert_eq!(
        pending.pending_relative.calls[2].call.sql_text.as_deref(),
        Some("SELECT direct relative")
    );
    let root = PathBuf::from("/virtual/db");
    project_relative_scoped_facts(&mut pending, Some(&root), |specifier| {
        assert_eq!(specifier, "../transaction");
        Some(root.join("src/transaction.ts"))
    });
    assert_eq!(bodies(&pending), ["SELECT direct relative"]);
    assert_eq!(pending.calls.len(), 3);
}

#[test]
fn projecting_a_saved_relative_import_keeps_only_files_inside_the_package() {
    let mut pending = extract("scoped-relative-pending.ts", &scoped("@example/db"));
    let root = PathBuf::from("/virtual/db");
    project_relative_scoped_facts(&mut pending, Some(&root), |specifier| {
        assert_eq!(specifier, "../transaction");
        Some(root.join("src/transaction.ts"))
    });
    assert_eq!(pending.calls.len(), 9);
    assert_eq!(
        bodies(&pending),
        [
            "relative_pending",
            "relative_member_factory",
            "relative_member_alias",
            "relative_plain",
            "relative_typed",
            "relative_member_type",
            "relative_member_type_alias",
            "relative_union",
            "relative_inline"
        ]
    );
    assert_eq!(pending.matched_factory_names, ["openTransaction"]);
    assert_eq!(pending.matched_type_names, ["TxExecutor"]);

    let mut outside = extract("scoped-relative-pending.ts", &scoped("@example/db"));
    project(
        &mut outside,
        Some(&root),
        Some(PathBuf::from("/virtual/other/transaction.ts")),
    );
    assert!(outside.calls.is_empty());
    assert!(outside.matched_factory_names.is_empty());
}

#[test]
fn an_empty_specifier_scans_relative_imports_immediately() {
    let facts = extract("scoped-relative-any-module.ts", &scoped(""));
    assert_eq!(bodies(&facts), ["relative_any"]);
    assert!(facts.pending_relative.candidates.is_empty());
    assert_eq!(facts.matched_factory_names, ["openTransaction"]);
}

#[test]
fn subpath_imports_stay_confirmed_and_carry_no_relative_candidates() {
    let facts = extract("scoped-subpath-imports.ts", &scoped("@example/db"));
    assert_eq!(
        bodies(&facts),
        ["subpath_factory", "subpath_type", "subpath_inline_type"]
    );
    assert!(facts.pending_relative.candidates.is_empty());
    assert!(facts.pending_relative.calls.is_empty());
}

#[test]
fn mixed_confirmed_and_relative_calls_keep_source_order() {
    let options = &scoped("@example/db");
    let extracted = extract("scoped-relative-mixed.ts", options);
    assert_eq!(bodies(&extracted), ["confirmed_first"]);
    assert_eq!(extracted.pending_relative.confirmed_order.len(), 1);
    assert_eq!(extracted.pending_relative.calls.len(), 1);

    let mut kept = extracted.clone();
    project(
        &mut kept,
        Some(Path::new("/pkg")),
        Some(PathBuf::from("/pkg/transaction.ts")),
    );
    assert_eq!(bodies(&kept), ["confirmed_first", "relative_second"]);

    let mut dropped = extracted;
    project(
        &mut dropped,
        Some(Path::new("/pkg")),
        Some(PathBuf::from("/other/transaction.ts")),
    );
    assert_eq!(bodies(&dropped), ["confirmed_first"]);
}
