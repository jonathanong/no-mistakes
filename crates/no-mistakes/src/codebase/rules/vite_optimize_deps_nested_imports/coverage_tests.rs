use super::closure::runtime_imports;
use super::include_config::{parse_vite_config, ParsedConfig};
use super::include_source::{load, parse_include_source, SourceList};
use super::IncludeSource;
use crate::codebase::rules::source_store_for_files;
use crate::codebase::ts_resolver::{self, TsConfig};
use oxc_ast::ast::{BindingPattern, Declaration, Statement, VariableDeclarationKind};
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

fn unanalyzable_reason(source: &str, export_name: &str) -> String {
    match parse_include_source(Path::new("nested-includes.ts"), source, export_name) {
        SourceList::Includes(_) => panic!("expected an unanalyzable include source"),
        SourceList::Unanalyzable { reason, .. } => reason,
    }
}

#[test]
fn include_sources_reject_blank_names_unreadable_files_and_parser_gaps() {
    let sources = source_store_for_files(&[]);
    let root = Path::new("/check");
    let mut includes = Vec::new();
    let mut findings = Vec::new();
    let mut known = true;
    load(
        root,
        &sources,
        &[
            IncludeSource {
                module: String::new(),
                export: "nestedIncludes".to_string(),
            },
            IncludeSource {
                module: "nested-includes.ts".to_string(),
                export: String::new(),
            },
            IncludeSource {
                module: "missing-includes.ts".to_string(),
                export: "nestedIncludes".to_string(),
            },
        ],
        &mut includes,
        &mut findings,
        &mut known,
    );
    assert!(!known);
    assert!(includes.is_empty());
    assert_eq!(findings.len(), 3);
    assert!(findings[0].message.contains("module and an export name"));
    assert!(findings[2].message.contains("cannot read source"));

    assert!(
        unanalyzable_reason("export const nestedIncludes = [", "nestedIncludes")
            .contains("cannot parse include source")
    );
    assert!(unanalyzable_reason(
        "export function nestedIncludes() { return []; }\n",
        "nestedIncludes"
    )
    .contains("does not export const"));
    assert!(unanalyzable_reason(
        "export let nestedIncludes = [\"widgets > items\"];\n",
        "nestedIncludes"
    )
    .contains("does not export const"));
    assert!(unanalyzable_reason(
        "export const { nestedIncludes } = { nestedIncludes: [\"widgets > items\"] };\n",
        "nestedIncludes"
    )
    .contains("does not export const"));
    assert!(unanalyzable_reason(
        "export const other = [\"widgets > items\"];\n",
        "nestedIncludes"
    )
    .contains("does not export const"));
    assert!(
        unanalyzable_reason("export const nestedIncludes = [1];\n", "nestedIncludes")
            .contains("not a static array of strings")
    );
    let declare_const = "export declare const nestedIncludes: string[];\n";
    assert!(unanalyzable_reason(declare_const, "nestedIncludes").contains("does not export const"));
    crate::ast::with_program(
        Path::new("nested-includes.ts"),
        declare_const,
        |program, _| {
            let Statement::ExportDeclaration(export) = &program.body[0] else {
                panic!("declare const is not an export");
            };
            let Declaration::VariableDeclaration(declaration) = &export.declaration else {
                panic!("declare const is not a variable");
            };
            assert_eq!(declaration.kind, VariableDeclarationKind::Const);
            let declarator = &declaration.declarations[0];
            assert!(matches!(
                declarator.id,
                BindingPattern::BindingIdentifier(_)
            ));
            assert!(declarator.init.is_none());
        },
    )
    .unwrap();
}

#[test]
fn const_and_satisfies_wrappers_keep_a_static_string_array() {
    for source in [
        "export const deps = ['a > b', 'x > y'] as const;\n",
        "export const deps = ['a > b' /* kept */, 'x > y'] as const;\n",
        "export const deps = ['a > b', 'x > y'] satisfies readonly string[];\n",
        "export const deps = ['a > b', 'x > y'] as const satisfies readonly string[];\n",
    ] {
        match parse_include_source(Path::new("deps.ts"), source, "deps") {
            SourceList::Includes(entries) => {
                let texts: Vec<_> = entries.iter().map(|entry| entry.text.as_str()).collect();
                assert_eq!(texts, vec!["a > b", "x > y"], "{source}");
                assert!(entries
                    .iter()
                    .all(|entry| entry.child == "b" || entry.child == "y"));
            }
            SourceList::Unanalyzable { reason, .. } => panic!("{source}: {reason}"),
        }
    }
    let source = "\
export default { optimizeDeps: { include: [\"widgets > items/shape\"] as const } };
";
    match parse_vite_config(Path::new("vite.config.ts"), source, false) {
        ParsedConfig::Includes(entries) => {
            assert_eq!(entries.len(), 1);
            assert_eq!(entries[0].child, "items/shape");
            assert_eq!(entries[0].parent, "widgets");
        }
        ParsedConfig::Unanalyzable { reason, .. } => panic!("{reason}"),
    }
}

#[test]
fn an_imported_include_drops_a_non_static_array() {
    let source = "\
const extra = [];
export default { optimizeDeps: { include: [\"widgets > items\", ...extra] } };
";
    match parse_vite_config(Path::new("vite.config.ts"), source, true) {
        ParsedConfig::Includes(entries) => assert!(entries.is_empty()),
        ParsedConfig::Unanalyzable { reason, .. } => panic!("{reason}"),
    }
}

#[test]
fn closure_skips_dependency_directories_and_imports_outside_the_package() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/rules/vite-optimize-deps-nested-imports/fixture/closure-edges");
    let dir = root.join("packages/widgets");
    assert!(root.join("outside.ts").is_file());
    let sources = source_store_for_files(&[]);
    let manifest = serde_json::json!({
        "name": "widgets",
        "exports": {
            ".": "./index.ts",
            "./*": "./*.ts"
        }
    });
    let entry = dir.join("index.ts");
    let resolved = ts_resolver::resolve_import("../../outside.ts", &entry, &TsConfig::default())
        .expect("outside import resolves");
    assert!(!ts_resolver::normalize_path(&resolved).starts_with(&dir));
    let imports = runtime_imports(
        &root,
        &dir,
        &manifest,
        "widgets",
        &sources,
        &[
            dir.join("node_modules/items/index.ts"),
            PathBuf::from("packages/widgets/index.ts"),
        ],
        &BTreeSet::new(),
    )
    .unwrap();
    assert!(imports.is_empty());
    let skipped = runtime_imports(
        &root,
        &dir,
        &serde_json::json!({
            "name": "widgets",
            "main": "node_modules/items/index.ts"
        }),
        "widgets",
        &sources,
        &[],
        &BTreeSet::new(),
    )
    .unwrap();
    assert!(skipped.is_empty());
}
