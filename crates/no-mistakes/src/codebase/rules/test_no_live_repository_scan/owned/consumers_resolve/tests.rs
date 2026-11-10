use super::{after_miss, apply, note_dropped, Choice, Gate};
use crate::codebase::ts_resolver::{ImportResolver, TsConfig};
use crate::codebase::ts_source::facts::TsFileFacts;
use crate::codebase::ts_source::module_facts::arguments::PathProvenance;
use crate::codebase::ts_source::module_facts::TypeScriptModuleFacts;
use crate::codebase::workspaces::IndexedWorkspaceMap;
use crate::fx::PathSet;
use std::path::Path;
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
