use super::*;

#[test]
fn package_name_keeps_the_package_portion_only() {
    assert_eq!(package_name("@example/db"), Some("@example/db"));
    assert_eq!(package_name("@example/db/tx/open"), Some("@example/db"));
    assert_eq!(package_name("@example/db/"), Some("@example/db"));
    assert_eq!(package_name("db/sub"), Some("db"));
    assert_eq!(package_name("@example"), None);
    assert_eq!(package_name("@scope/"), None);
    assert_eq!(package_name("@/name"), None);
    assert_eq!(package_name("./rel"), None);
    assert_eq!(package_name("../rel"), None);
    assert_eq!(package_name("/abs"), None);
    assert_eq!(package_name(""), None);
}

#[test]
fn package_root_uses_the_workspace_name_before_resolution() {
    let root = fixture_root();
    let workspace = workspace(&root);
    let importer = root.join("packages/db/src/orders/lock.ts");
    let resolved = package_root_for_specifier("@example/db", &importer, &workspace, |_, _| None);
    let package = resolved.expect("named package");
    assert!(package.ends_with("packages/db"), "{}", package.display());
    assert!(
        package_root_for_specifier("@missing/pkg", &importer, &workspace, |_, _| None).is_none()
    );
}

#[test]
fn package_root_falls_back_to_the_resolved_file_ancestor() {
    let root = fixture_root();
    let workspace = workspace(&root);
    let importer = root.join("packages/other/src/orders/lock.ts");
    let inside = root.join("packages/db/src/transaction.ts");
    let resolved =
        package_root_for_specifier("@missing/pkg", &importer, &workspace, |spec, from| {
            assert_eq!(spec, "@missing/pkg");
            assert_eq!(from, importer);
            Some(inside.clone())
        });
    let package = resolved.expect("ancestor package");
    assert!(package.ends_with("packages/db"), "{}", package.display());
    let outside = package_root_for_specifier("@missing/pkg", &importer, &workspace, |_, _| {
        Some(root.join("packages/other/src/transaction.ts"))
    })
    .expect("other package");
    assert!(outside.ends_with("packages/other"), "{}", outside.display());
}

#[test]
fn an_unknown_package_root_drops_candidates_without_touching_calls() {
    let mut facts = facts(
        vec![call("kept")],
        PendingRelativeScope {
            candidates: vec![candidate("../transaction", "openTransaction", true, false)],
            calls: vec![PendingRelativeCall {
                seq: 1,
                owners: vec![0],
                call: call("dropped"),
            }],
            ..PendingRelativeScope::default()
        },
    );
    project(
        &mut facts,
        None,
        Some(PathBuf::from("/pkg/src/transaction.ts")),
    );
    assert_eq!(sqls(&facts), ["kept"]);
    assert!(facts.matched_factory_names.is_empty());
    assert_eq!(facts.pending_relative, PendingRelativeScope::default());
}

#[test]
fn a_resolved_file_inside_the_package_promotes_calls_and_names() {
    let root = PathBuf::from("/pkg");
    let mut facts = facts(
        vec![call("confirmed")],
        PendingRelativeScope {
            candidates: vec![candidate("../transaction", "openTransaction", true, true)],
            calls: vec![PendingRelativeCall {
                seq: 1,
                owners: vec![0],
                call: call("relative"),
            }],
            confirmed_order: vec![0],
            ..PendingRelativeScope::default()
        },
    );
    facts.matched_factory_names = vec!["openTransaction".to_string()];
    project(
        &mut facts,
        Some(&root),
        Some(PathBuf::from("/pkg/src/transaction.ts")),
    );
    assert_eq!(sqls(&facts), ["confirmed", "relative"]);
    assert_eq!(facts.matched_factory_names, ["openTransaction"]);
    assert_eq!(facts.matched_type_names, ["openTransaction"]);
    assert_eq!(facts.pending_relative, PendingRelativeScope::default());
}

#[test]
fn a_file_outside_the_package_or_equal_to_the_root_is_not_kept() {
    let root = PathBuf::from("/packages/db");
    for resolved in [
        PathBuf::from("/packages/db"),
        PathBuf::from("/packages/db-other/src/transaction.ts"),
        PathBuf::from("/packages/other/transaction.ts"),
    ] {
        let mut facts = facts(
            vec![call("confirmed")],
            PendingRelativeScope {
                candidates: vec![candidate("../transaction", "openTransaction", true, true)],
                calls: vec![PendingRelativeCall {
                    seq: 0,
                    owners: vec![0],
                    call: call("relative"),
                }],
                confirmed_order: vec![1],
                call_spans: Default::default(),
                spans: vec![PendingRelativeSpan {
                    owners: vec![0],
                    name: "tx".to_string(),
                    start: 0,
                    end: 1,
                }],
            },
        );
        project(&mut facts, Some(&root), Some(resolved));
        assert_eq!(sqls(&facts), ["confirmed"]);
        assert!(facts.matched_factory_names.is_empty());
        assert!(facts.matched_type_names.is_empty());
    }
}
