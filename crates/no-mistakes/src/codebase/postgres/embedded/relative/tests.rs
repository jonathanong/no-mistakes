use super::super::{
    extract_embedded_sql_from_source, EmbeddedSqlCall, EmbeddedSqlFileFacts, EmbeddedSqlOptions,
};
use super::{
    package_name, package_root_for_specifier, project_relative_scoped_facts, PendingRelativeCall,
    PendingRelativeScope, PendingRelativeSpan, RelativeScopedCandidate,
};
use crate::codebase::ts_resolver::normalize_path;
use crate::codebase::ts_source::{discover_visible_paths, FileInventory, SourceStore};
use crate::codebase::workspaces::load_indexed_from_source_store;
use std::path::{Path, PathBuf};
use std::sync::Arc;

fn call(sql: &str) -> EmbeddedSqlCall {
    EmbeddedSqlCall {
        sql_text: Some(sql.to_string()),
        callee: "tx".to_string(),
        ..EmbeddedSqlCall::default()
    }
}

fn candidate(
    specifier: &str,
    name: &str,
    factory: bool,
    type_import: bool,
) -> RelativeScopedCandidate {
    RelativeScopedCandidate {
        specifier: specifier.to_string(),
        imported_name: name.to_string(),
        local_name: name.to_string(),
        factory,
        type_import,
    }
}

fn facts(calls: Vec<EmbeddedSqlCall>, pending: PendingRelativeScope) -> EmbeddedSqlFileFacts {
    EmbeddedSqlFileFacts {
        path: PathBuf::from("file.ts"),
        executor_bindings: Vec::new(),
        call_spans: vec![(0, 0); calls.len()],
        source_comment_spans: Vec::new(),
        calls,
        fragments: Vec::new(),
        fragment_variants: Vec::new(),
        fragment_sites: Vec::new(),
        matched_factory_names: Vec::new(),
        matched_type_names: Vec::new(),
        pending_relative: pending,
    }
}

fn sqls(facts: &EmbeddedSqlFileFacts) -> Vec<String> {
    facts
        .calls
        .iter()
        .filter_map(|call| call.sql_text.clone())
        .collect()
}

fn project(facts: &mut EmbeddedSqlFileFacts, root: Option<&Path>, resolved: Option<PathBuf>) {
    project_relative_scoped_facts(facts, root, |_| resolved.clone());
}

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

#[test]
fn an_unresolved_specifier_and_empty_or_unknown_owners_drop_the_call() {
    let root = PathBuf::from("/pkg");
    let inside = PathBuf::from("/pkg/src/transaction.ts");
    let mut facts = facts(
        Vec::new(),
        PendingRelativeScope {
            candidates: vec![
                candidate("../missing", "openTransaction", true, false),
                candidate("../transaction", "TxExecutor", false, true),
            ],
            calls: vec![
                PendingRelativeCall {
                    seq: 0,
                    owners: vec![0],
                    call: call("unresolved"),
                },
                PendingRelativeCall {
                    seq: 1,
                    owners: Vec::new(),
                    call: call("no-owner"),
                },
                PendingRelativeCall {
                    seq: 2,
                    owners: vec![9],
                    call: call("out-of-range"),
                },
                PendingRelativeCall {
                    seq: 3,
                    owners: vec![1, 9],
                    call: call("kept-by-sibling"),
                },
            ],
            ..PendingRelativeScope::default()
        },
    );
    project_relative_scoped_facts(&mut facts, Some(&root), |specifier| {
        (specifier == "../transaction").then(|| inside.clone())
    });
    assert_eq!(sqls(&facts), ["kept-by-sibling"]);
    assert!(facts.matched_factory_names.is_empty());
    assert_eq!(facts.matched_type_names, ["TxExecutor"]);
}

#[test]
fn promoted_calls_merge_by_sequence_and_a_mismatch_appends() {
    let root = PathBuf::from("/pkg");
    let inside = Some(PathBuf::from("/pkg/a.ts"));
    let mut ordered = facts(
        vec![call("second"), call("fourth")],
        PendingRelativeScope {
            candidates: vec![candidate("./a", "openTransaction", true, false)],
            calls: vec![
                PendingRelativeCall {
                    seq: 0,
                    owners: vec![0],
                    call: call("first"),
                },
                PendingRelativeCall {
                    seq: 2,
                    owners: vec![0],
                    call: call("third"),
                },
            ],
            confirmed_order: vec![1, 3],
            ..PendingRelativeScope::default()
        },
    );
    project(&mut ordered, Some(&root), inside.clone());
    assert_eq!(sqls(&ordered), ["first", "second", "third", "fourth"]);

    let mut tied = facts(
        vec![call("confirmed")],
        PendingRelativeScope {
            candidates: vec![candidate("./a", "openTransaction", false, false)],
            calls: vec![PendingRelativeCall {
                seq: 0,
                owners: vec![0],
                call: call("same-seq"),
            }],
            confirmed_order: vec![0],
            ..PendingRelativeScope::default()
        },
    );
    project(&mut tied, Some(&root), inside.clone());
    assert_eq!(sqls(&tied), ["confirmed", "same-seq"]);
    assert!(tied.matched_factory_names.is_empty());

    let mut mismatched = facts(
        vec![call("confirmed")],
        PendingRelativeScope {
            candidates: vec![candidate("./a", "openTransaction", true, false)],
            calls: vec![PendingRelativeCall {
                seq: 0,
                owners: vec![0],
                call: call("appended"),
            }],
            confirmed_order: Vec::new(),
            ..PendingRelativeScope::default()
        },
    );
    project(&mut mismatched, Some(&root), inside);
    assert_eq!(sqls(&mismatched), ["confirmed", "appended"]);
}

#[test]
fn nothing_promoted_leaves_confirmed_calls_in_place() {
    let mut facts = facts(
        vec![call("confirmed")],
        PendingRelativeScope {
            candidates: vec![candidate("../transaction", "openTransaction", true, false)],
            calls: Vec::new(),
            confirmed_order: vec![0],
            ..PendingRelativeScope::default()
        },
    );
    project(
        &mut facts,
        Some(Path::new("/pkg")),
        Some(PathBuf::from("/other/transaction.ts")),
    );
    assert_eq!(sqls(&facts), ["confirmed"]);
    assert!(facts.matched_factory_names.is_empty());
}

#[test]
fn empty_candidates_clear_pending_without_reading_the_resolver() {
    let mut facts = facts(
        vec![call("confirmed")],
        PendingRelativeScope {
            spans: vec![PendingRelativeSpan {
                owners: vec![0],
                name: "tx".to_string(),
                start: 0,
                end: 4,
            }],
            ..PendingRelativeScope::default()
        },
    );
    project_relative_scoped_facts(&mut facts, None, |_| panic!("resolver"));
    assert_eq!(sqls(&facts), ["confirmed"]);
    assert_eq!(facts.pending_relative, PendingRelativeScope::default());
}

fn embedded_fixture(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/postgres-facts/embedded")
        .join(name)
}

fn scoped(specifier: &str) -> EmbeddedSqlOptions {
    EmbeddedSqlOptions::configured(specifier, &[])
        .with_scoped_executors(&["openTransaction".into()], &["TxExecutor".into()])
}

fn extract(name: &str, options: &EmbeddedSqlOptions) -> EmbeddedSqlFileFacts {
    let path = embedded_fixture(name);
    let source = std::fs::read_to_string(&path).expect("fixture");
    extract_embedded_sql_from_source(&path, &source, options)
}

fn bodies(facts: &EmbeddedSqlFileFacts) -> Vec<String> {
    facts
        .calls
        .iter()
        .filter_map(|call| call.sql_text.as_deref())
        .map(|sql| sql.trim_start_matches("SELECT id FROM ").to_string())
        .collect()
}

#[test]
fn relative_imports_stay_unresolved_until_projection() {
    let pending = extract("scoped-relative-pending.ts", &scoped("@example/db"));
    assert!(pending.calls.is_empty());
    assert!(pending.matched_factory_names.is_empty());
    assert!(pending.matched_type_names.is_empty());
    assert_eq!(pending.pending_relative.candidates.len(), 2);
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
            "relative_plain",
            "relative_typed",
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
fn projecting_a_saved_relative_import_keeps_only_files_inside_the_package() {
    let mut pending = extract("scoped-relative-pending.ts", &scoped("@example/db"));
    let root = PathBuf::from("/virtual/db");
    project_relative_scoped_facts(&mut pending, Some(&root), |specifier| {
        assert_eq!(specifier, "../transaction");
        Some(root.join("src/transaction.ts"))
    });
    assert_eq!(
        bodies(&pending),
        [
            "relative_pending",
            "relative_plain",
            "relative_typed",
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

fn fixture_root() -> PathBuf {
    normalize_path(&PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(
        "../../test-cases/rules/postgres-lock-ordering/fixture/fail-scoped-relative-executors",
    ))
}

fn workspace(root: &Path) -> crate::codebase::workspaces::IndexedWorkspaceMap {
    let files = discover_visible_paths(root);
    let sources = SourceStore::new(Arc::new(FileInventory::from_paths(&files)));
    load_indexed_from_source_store(root, &sources).expect("workspace")
}
