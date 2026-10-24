use super::locate::{Located, Locator, MAX_REEXPORT_HOPS};
use super::{has_candidates, project};
use crate::codebase::check_facts::CheckFileFacts;
use crate::codebase::postgres::{
    extract_embedded_sql_from_program, EmbeddedSqlKind, EmbeddedSqlOptions,
};
use crate::codebase::ts_resolver::normalize_path;
use crate::codebase::ts_source::facts::{
    collect_file_facts_from_program, TsFactContext, TsFactPlan,
};
use crate::codebase::ts_source::{discover_visible_paths, FileIdMap};
use crate::diagnostics::{with_observer, InvocationObserver};
use crate::fx::FxHashSet;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

fn variant_root(name: &str) -> PathBuf {
    normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../test-cases/rules/postgres-required-predicates/variants")
            .join(name),
    )
}

/// Per-file facts exactly as the request pass would collect them, one parse per file.
fn prepared(name: &str) -> FileIdMap<CheckFileFacts> {
    let mut paths = discover_visible_paths(&variant_root(name).join("src"));
    paths.retain(|path| path.extension().is_some_and(|extension| extension == "mts"));
    paths.sort();
    let options = EmbeddedSqlOptions::configured("@example/db", &[]);
    paths
        .into_iter()
        .map(|path| {
            let source = std::fs::read_to_string(&path).expect("fixture source");
            let file = crate::ast::with_program(&path, &source, |program, _| CheckFileFacts {
                ts: Arc::new(collect_file_facts_from_program(
                    &path,
                    TsFactPlan {
                        imports: true,
                        module_bindings: true,
                        ..Default::default()
                    },
                    &TsFactContext::default(),
                    &source,
                    program,
                    None,
                    None,
                )),
                embedded_sql: vec![(
                    options.clone(),
                    extract_embedded_sql_from_program(&path, program, &source, &options),
                )],
                ..Default::default()
            })
            .expect("parse");
            (path, file)
        })
        .collect()
}

/// Resolves relative specifiers against the fixture's own files.
fn resolver<'a>(
    files: &FileIdMap<CheckFileFacts>,
    calls: &'a Mutex<BTreeMap<(PathBuf, String), u32>>,
) -> impl FnMut(&str, &Path) -> Option<PathBuf> + 'a {
    let known = files.keys().cloned().collect::<FxHashSet<_>>();
    move |specifier, from| {
        *calls
            .lock()
            .unwrap()
            .entry((from.to_path_buf(), specifier.to_string()))
            .or_default() += 1;
        let target = normalize_path(&from.parent()?.join(specifier));
        known.contains(&target).then_some(target)
    }
}

fn run(name: &str) -> FileIdMap<CheckFileFacts> {
    let mut files = prepared(name);
    let calls = Mutex::new(BTreeMap::new());
    let resolve = resolver(&files, &calls);
    project(&mut files, resolve);
    files
}

fn caller_calls(
    files: &FileIdMap<CheckFileFacts>,
) -> Vec<crate::codebase::postgres::EmbeddedSqlCall> {
    let (_, caller) = files
        .iter()
        .find(|(path, _)| path.ends_with("src/caller.mts"))
        .expect("caller");
    caller.embedded_sql[0].1.calls.clone()
}

#[test]
fn module_const_sql_inlines_literals_and_leaves_computed_dynamic() {
    let calls = caller_calls(&run("module-const-sql"));
    let mut analyzed = Vec::new();
    let mut unknown = 0;
    for call in &calls {
        let sql = call.sql_text.clone().unwrap_or_default();
        if call.is_unanalyzable() {
            unknown += 1;
            assert!(!sql.contains('\0'), "{sql}");
            continue;
        }
        assert!(sql.contains("orders"), "{call:?}");
        analyzed.push(sql);
    }
    assert_eq!(unknown, 13, "{calls:#?}");
    assert!(analyzed
        .iter()
        .any(|sql| sql == "SELECT id FROM orders WHERE account_id = $1"));
    assert!(analyzed
        .iter()
        .any(|sql| sql == "SELECT id, account_id FROM orders WHERE account_id = $1"));
    assert!(analyzed
        .iter()
        .any(|sql| sql == "SELECT id FROM orders WHERE true AND account_id = $1"));
    assert!(analyzed
        .iter()
        .any(|sql| sql == "SELECT id FROM orders WHERE true"));
}

#[test]
fn a_constant_hole_without_its_import_record_is_not_inlined() {
    // The walk already recorded the hole. Clearing the import makes the name
    // look local, which the filler must refuse instead of splicing a guess.
    let mut files = prepared("module-const-sql");
    let caller = files
        .keys()
        .find(|path| path.ends_with("src/caller.mts"))
        .expect("caller")
        .clone();
    Arc::make_mut(&mut files.get_mut(&caller).expect("caller").ts)
        .imported_bindings
        .clear();
    let calls = Mutex::new(BTreeMap::new());
    let resolve = resolver(&files, &calls);
    project(&mut files, resolve);
    let call = caller_calls(&files)
        .into_iter()
        .find(|call| call.line == 56)
        .expect("imported const");
    assert!(call.is_unanalyzable(), "{call:?}");
}

#[test]
fn a_shared_export_is_located_once_however_many_calls_import_it() {
    let mut files = prepared("imported-shared-export");
    assert!(has_candidates(&files));
    let calls = Mutex::new(BTreeMap::new());
    let resolve = resolver(&files, &calls);
    let observer = InvocationObserver::new(true);
    with_observer(Some(Arc::clone(&observer)), || project(&mut files, resolve));
    let sites: usize = files
        .values()
        .map(|file| file.embedded_sql[0].1.pending_imported.len())
        .sum();
    assert_eq!(sites, 3, "three call sites import the same export");
    assert_eq!(
        observer.snapshot().work["postgres.embedded.helper_resolutions"],
        1,
        "one (module, export) pair is located once, not once per call site"
    );
    let calls = calls.into_inner().unwrap();
    assert!(
        calls.values().all(|count| *count == 1),
        "each (importer, specifier) resolves once: {calls:?}"
    );
    let owner = variant_root("imported-shared-export").join("src/visibility-sql.mts");
    for (path, file) in &files {
        for call in &file.embedded_sql[0].1.calls {
            assert_eq!(call.kind, EmbeddedSqlKind::Composed, "{}", path.display());
            assert!(call
                .sql_text
                .as_deref()
                .unwrap()
                .contains("FROM view_active_plans"));
            assert!(call
                .sql_source_positions
                .iter()
                .any(|position| position.source_path.as_deref() == Some(owner.as_path())));
        }
    }
}

#[test]
fn the_request_pass_parses_each_file_once_and_locates_each_export_once() {
    use crate::codebase::check_facts::{collect_check_facts, CheckFactPlan};
    let saved = variant_root("imported-shared-export");
    let fixture = crate::test_support::materialize_saved_fixture(&saved);
    let root = normalize_path(fixture.path());
    let paths = ["first", "second", "index", "visibility-sql"]
        .map(|name| normalize_path(&root.join(format!("src/{name}.mts"))));
    let options = EmbeddedSqlOptions::configured("@example/db", &[]);
    let observer = InvocationObserver::new(true);
    crate::ast::begin_parse_count(&root);
    let facts = with_observer(Some(Arc::clone(&observer)), || {
        collect_check_facts(
            &root,
            paths.to_vec(),
            CheckFactPlan {
                embedded_sql: true,
                embedded_sql_options: vec![options.clone()],
                ..Default::default()
            },
        )
    });
    let counts = crate::ast::finish_parse_count(&root);
    for path in &paths {
        assert_eq!(counts.get(path), Some(&1), "{counts:#?}");
    }
    assert_eq!(
        observer.snapshot().work["postgres.embedded.helper_resolutions"],
        1,
        "three call sites, one (module, export) resolution"
    );
    for path in &paths[..2] {
        let calls = &facts.embedded_sql(path, &options).expect("facts").calls;
        assert!(!calls.is_empty());
        assert!(calls
            .iter()
            .all(|call| call.kind == EmbeddedSqlKind::Composed));
    }
}

#[test]
fn each_reexport_form_resolves_to_the_declaring_module() {
    let files = run("imported-reexport-forms");
    let expected = [
        ("d", "diamond: two star paths, one declaration"),
        ("e", "an explicit re-export outranks a star"),
        ("x", "a renamed default export"),
        ("r", "an import re-exported by name"),
    ];
    let calls = caller_calls(&files);
    assert_eq!(calls.len(), expected.len());
    for (call, (alias, why)) in calls.iter().zip(expected) {
        let sql = call.sql_text.as_deref().unwrap_or_default();
        assert!(
            sql.contains(&format!("FROM view_active_plans {alias} ")),
            "{why}: {sql}"
        );
        assert_eq!(call.kind, EmbeddedSqlKind::Composed, "{why}");
    }
}

#[test]
fn the_reexport_depth_cap_is_inclusive() {
    assert_eq!(MAX_REEXPORT_HOPS, 8);
    for (name, resolved) in [
        ("imported-barrel-depth-cap", true),
        ("imported-barrel-past-depth-cap", false),
    ] {
        let calls = caller_calls(&run(name));
        assert_eq!(calls.len(), 1, "{name}");
        assert_eq!(
            calls[0].kind == EmbeddedSqlKind::Composed,
            resolved,
            "{name}: {:?}",
            calls[0]
        );
    }
}

#[test]
fn unproven_links_leave_every_call_dynamic() {
    for name in [
        "imported-barrel-cycle",
        "imported-barrel-ambiguous",
        "imported-barrel-past-depth-cap",
        "imported-unresolved",
        "imported-nonliteral-arg",
    ] {
        let calls = caller_calls(&run(name));
        assert!(!calls.is_empty(), "{name}");
        for call in calls {
            assert_eq!(call.kind, EmbeddedSqlKind::Dynamic, "{name}: {call:?}");
            assert!(
                call.sql_text
                    .as_deref()
                    .is_none_or(|sql| !sql.contains("view_active_plans")),
                "{name}: {call:?}"
            );
        }
    }
}

#[test]
fn a_namespace_reexport_marker_is_never_followed() {
    let mut files = prepared("imported-barrel");
    let index = files
        .keys()
        .find(|path| path.ends_with("src/helpers/index.mts"))
        .cloned()
        .expect("barrel");
    let facts = Arc::make_mut(&mut files.get_mut(&index).expect("facts").ts);
    // `export * as ownerVisibleSql from ...` shadows the name with a namespace object.
    facts
        .exported_bindings
        .push(crate::codebase::dependencies::extract::ExportedBinding {
            specifier: Some("../visibility/owner-sql.mts".to_string()),
            local: "*".to_string(),
            exported: "ownerVisibleSql".to_string(),
        });
    let calls = Mutex::new(BTreeMap::new());
    let resolve = resolver(&files, &calls);
    project(&mut files, resolve);
    let calls = caller_calls(&files);
    assert_eq!(calls[0].kind, EmbeddedSqlKind::Dynamic, "{:?}", calls[0]);
}

#[test]
fn a_re_exported_namespace_import_is_never_followed() {
    use crate::codebase::dependencies::extract::{
        ExportedBinding, ImportedBinding, ImportedBindingKind,
    };
    let mut files = prepared("imported-barrel");
    let index = files
        .keys()
        .find(|path| path.ends_with("src/helpers/index.mts"))
        .cloned()
        .expect("barrel");
    let facts = Arc::make_mut(&mut files.get_mut(&index).expect("facts").ts);
    // `import * as ownerVisibleSql ...; export { ownerVisibleSql }`
    facts.imported_bindings.push(ImportedBinding {
        specifier: "../visibility/owner-sql.mts".to_string(),
        local: "ownerVisibleSql".to_string(),
        imported: "*".to_string(),
        kind: ImportedBindingKind::Namespace,
        is_type_only: false,
    });
    facts.exported_bindings.push(ExportedBinding {
        specifier: None,
        local: "ownerVisibleSql".to_string(),
        exported: "ownerVisibleSql".to_string(),
    });
    let calls = Mutex::new(BTreeMap::new());
    let resolve = resolver(&files, &calls);
    project(&mut files, resolve);
    assert_eq!(caller_calls(&files)[0].kind, EmbeddedSqlKind::Dynamic);
}

#[test]
fn a_call_whose_import_binding_is_missing_stays_dynamic() {
    let mut files = prepared("imported-literal-identifiers");
    let caller = files
        .keys()
        .find(|path| path.ends_with("src/caller.mts"))
        .cloned()
        .expect("caller");
    let facts = Arc::make_mut(&mut files.get_mut(&caller).expect("facts").ts);
    facts.imported_bindings.clear();
    let calls = Mutex::new(BTreeMap::new());
    let resolve = resolver(&files, &calls);
    project(&mut files, resolve);
    assert_eq!(caller_calls(&files)[0].kind, EmbeddedSqlKind::Dynamic);
    assert!(calls.into_inner().unwrap().is_empty(), "nothing to resolve");
}

#[test]
fn a_re_export_from_an_unresolved_module_is_unprovable() {
    let mut files = prepared("imported-barrel-ambiguous");
    let index = files
        .keys()
        .find(|path| path.ends_with("src/index.mts"))
        .cloned()
        .expect("barrel");
    let facts = Arc::make_mut(&mut files.get_mut(&index).expect("facts").ts);
    facts
        .star_reexport_specifiers
        .push("./gone.mts".to_string());
    let calls = Mutex::new(BTreeMap::new());
    let mut locator = Locator::new(&files, resolver(&files, &calls));
    // The missing module might declare the name, so it is not `Absent`.
    assert_eq!(locator.locate(&index, "neverDeclared"), Located::Unknown);
}

#[test]
fn the_locator_distinguishes_absent_from_unprovable() {
    let files = prepared("imported-barrel-ambiguous");
    let calls = Mutex::new(BTreeMap::new());
    let mut locator = Locator::new(&files, resolver(&files, &calls));
    let src = variant_root("imported-barrel-ambiguous").join("src");
    let index = src.join("index.mts");
    // Two different declarations behind `export *` make the name ambiguous.
    assert_eq!(locator.locate(&index, "visibleSql"), Located::Unknown);
    assert_eq!(locator.locate(&index, "neverDeclared"), Located::Absent);
    // `export *` never forwards a default export.
    assert_eq!(locator.locate(&index, "default"), Located::Absent);
    assert_eq!(
        locator.locate(&src.join("one.mts"), "visibleSql"),
        Located::Found(src.join("one.mts"), "visibleSql".to_string())
    );
    // A module outside the scanned files cannot be proven either way.
    assert_eq!(
        locator.locate(Path::new("/nowhere/outside.mts"), "visibleSql"),
        Located::Unknown
    );
    assert_eq!(
        locator.target("./one.mts", &index),
        Some(src.join("one.mts"))
    );
    assert_eq!(locator.target("./missing.mts", &index), None);
}

#[test]
fn a_file_without_pending_helpers_is_not_a_candidate() {
    let files = prepared("same-file-literal-identifiers");
    assert!(!has_candidates(&files));
}

#[path = "namespace_tests.rs"]
mod namespace_tests;
