use super::*;
use crate::config::v2::NoMistakesConfig;
use crate::codebase::dependencies::extract::{
    CallTargetIdentity, FunctionCall, ImportedBinding, ImportedBindingKind, InvocationKind,
};
use crate::codebase::ts_source::facts::TsFileFacts;

fn local_call(identity: CallTargetIdentity, callee: &str) -> FunctionCall {
    FunctionCall {
        caller: Some("consumer".to_string()),
        syntactic_caller: Some("consumer".to_string()),
        callee: callee.to_string(),
        line: 1,
        offset: 0,
        is_callback: false,
        invocation: InvocationKind::Call,
        target_identity: identity,
        static_arg: None,
        static_cwd: None,
    }
}

#[test]
fn legacy_local_caller_matching_uses_binding_identity_not_only_the_callee_spelling() {
    let local_names = BTreeSet::from(["alias".to_string()]);
    let imported = TsFileFacts {
        imported_bindings: vec![ImportedBinding {
            specifier: "./source.mts".to_string(),
            local: "alias".to_string(),
            imported: "target".to_string(),
            kind: ImportedBindingKind::Named,
            is_type_only: false,
        }],
        ..TsFileFacts::default()
    };
    let unimported = TsFileFacts::default();

    assert!(legacy_call_matches_local_target(
        &local_call(CallTargetIdentity::ModuleExport, "alias.member"),
        &local_names,
        &imported,
    ));
    assert!(!legacy_call_matches_local_target(
        &local_call(CallTargetIdentity::Unknown, "alias"),
        &local_names,
        &imported,
    ));
    assert!(legacy_call_matches_local_target(
        &local_call(CallTargetIdentity::RepositoryFunction, "alias"),
        &local_names,
        &unimported,
    ));
    assert!(!legacy_call_matches_local_target(
        &local_call(CallTargetIdentity::Global, "alias"),
        &local_names,
        &unimported,
    ));
    assert!(!legacy_call_matches_local_target(
        &local_call(CallTargetIdentity::ModuleExport, "other"),
        &local_names,
        &imported,
    ));
}

#[test]
fn caller_entries_filters_export_nodes_and_non_file_nodes() {
    let root = Path::new("/repo");
    let source = PathBuf::from("/repo/src/source.mts");
    let consumer = PathBuf::from("/repo/src/consumer.mts");
    let test = PathBuf::from("/repo/src/consumer.test.mts");
    let export_node = NodeId::symbol(source, "parseDate");
    let entries = vec![
        NodeEntry {
            node: export_node.clone(),
            depth: 1,
            via: vec![EdgeKind::Import],
        },
        NodeEntry {
            node: NodeId::module("external"),
            depth: 1,
            via: vec![EdgeKind::Import],
        },
        NodeEntry {
            node: NodeId::symbol(consumer, "format"),
            depth: 1,
            via: vec![EdgeKind::Import],
        },
        NodeEntry {
            node: NodeId::file(test),
            depth: 1,
            via: vec![EdgeKind::TestOf],
        },
    ];
    let filter = TestFileFilter::new(root, &NoMistakesConfig::default());
    let export_nodes = BTreeSet::from([export_node]);
    let file_target_symbols = BTreeMap::new();
    let facts = TsFactMap::new();
    let context = CallerEntriesContext {
        root,
        test_filter: &filter,
        export_nodes: &export_nodes,
        file_target_symbols: &file_target_symbols,
        facts: &facts,
    };

    let production = caller_entries(&entries, &context, false, &[]);
    let tests = caller_entries(&entries, &context, true, &[]);

    assert_eq!(production.len(), 1);
    assert_eq!(production[0].file, "src/consumer.mts");
    assert_eq!(production[0].symbol.as_deref(), Some("format"));
    assert!(tests.is_empty());
}

#[test]
fn caller_entries_merges_duplicate_callers_and_sorts() {
    let root = Path::new("/repo");
    let filter = TestFileFilter::new(root, &NoMistakesConfig::default());
    let export_nodes = BTreeSet::new();
    let file_target_symbols = BTreeMap::new();
    let facts = TsFactMap::new();
    let context = CallerEntriesContext {
        root,
        test_filter: &filter,
        export_nodes: &export_nodes,
        file_target_symbols: &file_target_symbols,
        facts: &facts,
    };
    let entries = vec![
        NodeEntry {
            node: NodeId::symbol(PathBuf::from("/repo/src/b.mts"), "beta"),
            depth: 1,
            via: vec![EdgeKind::Import],
        },
        NodeEntry {
            node: NodeId::symbol(PathBuf::from("/repo/src/a.mts"), "alpha"),
            depth: 1,
            via: vec![EdgeKind::Import],
        },
        NodeEntry {
            node: NodeId::symbol(PathBuf::from("/repo/src/b.mts"), "beta"),
            depth: 1,
            via: vec![EdgeKind::Import],
        },
    ];

    let extra = vec![CallerEntry {
        file: "src/a.mts".to_string(),
        symbol: Some("alpha".to_string()),
        depth: 2,
        via: vec!["symbol"],
    }];
    let callers = caller_entries(&entries, &context, false, &extra);

    assert_eq!(callers.len(), 2);
    assert_eq!(callers[0].file, "src/a.mts");
    assert_eq!(callers[0].via, vec!["import", "symbol"]);
    assert_eq!(callers[1].file, "src/b.mts");
    assert_eq!(callers[1].depth, 1);
    assert_eq!(callers[1].via, vec!["import"]);
}

#[test]
fn file_entry_uses_symbol_checks_extracted_and_alias_member_uses() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/codebase-analysis/tests-impact-symbol/fixture");
    let root = crate::codebase::ts_resolver::normalize_path(&root);
    let facts = impact_test_support::signature_test_facts(&root);

    assert!(file_entry_uses_symbol(
        &root,
        "require-caller.mts",
        "parseDate",
        &facts,
    ));
    assert!(file_entry_uses_symbol(
        &root,
        "dynamic-import-caller.mts",
        "parseDate",
        &facts,
    ));
    assert!(file_entry_uses_symbol(
        &root,
        "dynamic-import-alias-caller.mts",
        "parseDate",
        &facts,
    ));
    assert!(!file_entry_uses_symbol(
        &root,
        "dynamic-import-unused.mts",
        "parseDate",
        &facts,
    ));
    assert!(!file_entry_uses_symbol(
        &root,
        "dynamic-import-shadowed-member.mts",
        "parseDate",
        &facts,
    ));
    assert!(!file_entry_uses_symbol(
        &root,
        "dynamic-import-other-export-name.mts",
        "parseDate",
        &facts,
    ));
    assert!(file_entry_uses_symbol(
        &root,
        "dynamic-import-chained-member-caller.mts",
        "parseDate",
        &facts,
    ));
    assert!(!file_entry_uses_symbol(
        &root,
        "missing-dynamic-import-caller.mts",
        "parseDate",
        &facts,
    ));
}

#[test]
fn symbol_aliases_collect_destructured_and_member_assignment_locals() {
    let aliases = dynamic_symbol_aliases_in_source(
        "const { parseDate: pd } = await import('./utils.mts');\n\
         const readDate = require('./utils.mts').parseDate;\n\
         return utils.parseDate;\n\
         assigned = utils.parseDate;\n\
         pd(value); readDate(value);",
        "parseDate",
    );

    assert!(aliases.contains("pd"));
    assert!(aliases.contains("readDate"));
    assert!(!aliases.contains("assigned"));
}

mod usage_helpers;
