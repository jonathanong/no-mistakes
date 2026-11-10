use super::{after_miss, apply, note_dropped, Choice, Gate};
use crate::codebase::ts_resolver::{ImportResolver, TsConfig, VisiblePathLookup};
use crate::codebase::ts_source::facts::TsFileFacts;
use crate::codebase::ts_source::module_facts::arguments::PathProvenance;
use crate::codebase::ts_source::module_facts::{ModuleDiagnostic, TypeScriptModuleFacts};
use crate::codebase::workspaces::{IndexedWorkspaceMap, WorkspacePackage};
use crate::fx::PathSet;
use std::path::{Path, PathBuf};
use std::sync::Arc;

#[test]
fn a_readable_relative_is_followed_when_classify_misses() {
    let config = TsConfig::default();
    let workspace = IndexedWorkspaceMap::default();
    let visible = PathSet::default();
    let resolver = ImportResolver::new(&config);
    let gate = Gate {
        resolver: &resolver,
        workspace: &workspace,
        visible: &visible,
        resolver_name: "private",
    };
    let importer = Path::new(env!("CARGO_MANIFEST_DIR")).join("Cargo.toml");
    assert!(matches!(
        after_miss(&gate, &importer, "./Cargo.toml", false),
        Choice::Follow(_)
    ));
    assert!(matches!(
        after_miss(&gate, &importer, "./no-such-scan-target.mts", false),
        Choice::Unread(reason) if reason.contains("did not resolve")
    ));
    assert!(matches!(
        after_miss(&gate, &importer, "@scope/missing", true),
        Choice::Unread(reason) if reason.contains("workspace")
    ));
}

#[test]
fn apply_follows_a_readable_file_and_names_a_missing_specifier() {
    let config = TsConfig::default();
    let workspace = IndexedWorkspaceMap::default();
    let visible = PathSet::default();
    let resolver = ImportResolver::new(&config);
    let gate = Gate {
        resolver: &resolver,
        workspace: &workspace,
        visible: &visible,
        resolver_name: "private",
    };
    let importer = Path::new(env!("CARGO_MANIFEST_DIR")).join("Cargo.toml");
    let mut provenance = PathProvenance::default();
    provenance.unsafe_modules.insert("./Cargo.toml".to_string());
    provenance
        .unsafe_modules
        .insert("./no-such-scan-target.mts".to_string());
    let facts = TsFileFacts {
        module_bindings: Some(Arc::new(TypeScriptModuleFacts::default())),
        ..TsFileFacts::default()
    };
    let mut consumed = super::super::Consumed::default();
    apply(&gate, &mut consumed, &importer, &facts, &provenance);
    assert!(consumed.contains(&importer));
    assert_eq!(consumed.unknown_sites().len(), 1);
}

#[test]
fn dropped_path_facts_do_not_record_an_unread_module() {
    let mut consumed = super::super::Consumed::default();
    let facts = TsFileFacts {
        module_bindings: Some(Arc::new(TypeScriptModuleFacts::default())),
        ..TsFileFacts::default()
    };
    let file = Path::new("/repo/src/fn.mts");
    note_dropped(&mut consumed, file, &facts);
    assert!(consumed.contains(file));
    assert!(consumed.unknown_sites().is_empty());
}

#[test]
fn a_recognized_package_entry_is_followed_when_the_file_exists() {
    let cargo = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("Cargo.toml");
    let workspace = IndexedWorkspaceMap::from_packages(vec![WorkspacePackage {
        name: "@scope/pkg".to_string(),
        dir: PathBuf::from("/repo/packages/pkg"),
        entry: Some(cargo.clone()),
        exports: None,
        imports: None,
    }]);
    let config = TsConfig::default();
    let resolver = ImportResolver::new(&config);
    let visible = PathSet::default();
    let gate = Gate {
        resolver: &resolver,
        workspace: &workspace,
        visible: &visible,
        resolver_name: "private",
    };
    assert!(matches!(
        after_miss(&gate, &cargo, "@scope/pkg", true),
        Choice::Follow(path) if path == cargo
    ));
}

#[test]
fn dropped_reasons_name_missing_facts_and_diagnostics() {
    assert_eq!(
        super::dropped_reason(&TsFileFacts::default()),
        "path facts were dropped"
    );
    let empty = TsFileFacts {
        module_bindings: Some(Arc::new(TypeScriptModuleFacts::default())),
        ..TsFileFacts::default()
    };
    assert_eq!(super::dropped_reason(&empty), "path facts were dropped");
    let diagnosed = TsFileFacts {
        module_bindings: Some(Arc::new(TypeScriptModuleFacts {
            diagnostics: vec![
                ModuleDiagnostic {
                    kind: "unsupported".to_string(),
                    message: "Non-literal or unsupported require specifier".to_string(),
                    span: None,
                },
                ModuleDiagnostic {
                    kind: "unsupported".to_string(),
                    message: "Shadowed require is not a module loader".to_string(),
                    span: None,
                },
            ],
            ..TypeScriptModuleFacts::default()
        })),
        ..TsFileFacts::default()
    };
    assert_eq!(
        super::dropped_reason(&diagnosed),
        "Non-literal or unsupported require specifier; Shadowed require is not a module loader"
    );
}

#[test]
fn files_on_disk_sees_a_real_file_and_an_empty_visible_set() {
    let disk = super::FilesOnDisk;
    let cargo = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("Cargo.toml");
    assert!(disk.contains_visible(&cargo));
    assert!(!disk.contains_visible(Path::new("/no/such/scan-resolve-target.mts")));
    assert_eq!(disk.visible_len(), 0);
    assert!(disk.visible_cache_key().is_empty());
}

#[test]
fn dropped_path_facts_trace_when_resolve_debug_is_set() {
    let previous = std::env::var_os("NO_MISTAKES_SCAN_RESOLVE_DEBUG");
    std::env::set_var("NO_MISTAKES_SCAN_RESOLVE_DEBUG", "1");
    let mut consumed = super::super::Consumed::default();
    let facts = TsFileFacts {
        module_bindings: Some(Arc::new(TypeScriptModuleFacts::default())),
        ..TsFileFacts::default()
    };
    let file = Path::new("/repo/src/fn.mts");
    note_dropped(&mut consumed, file, &facts);
    match previous {
        Some(value) => std::env::set_var("NO_MISTAKES_SCAN_RESOLVE_DEBUG", value),
        None => std::env::remove_var("NO_MISTAKES_SCAN_RESOLVE_DEBUG"),
    }
    assert!(consumed.contains(file));
    assert!(consumed.unknown_sites().is_empty());
}
