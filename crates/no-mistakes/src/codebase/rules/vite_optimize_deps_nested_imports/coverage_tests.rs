use super::include_config::{parse_vite_config, ParsedConfig};
use super::include_source::{load, parse_include_source, SourceList};
use super::IncludeSource;
use crate::codebase::rules::source_store_for_files;
use oxc_ast::ast::{BindingPattern, Declaration, Statement, VariableDeclarationKind};
use std::path::Path;

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
fn an_imported_include_keeps_literal_entries_beside_a_spread() {
    let source = "\
const extra = [];
export default { optimizeDeps: { include: [\"widgets > items\", ...extra, \"widgets > orders\"] } };
";
    match parse_vite_config(Path::new("vite.config.ts"), source, true) {
        ParsedConfig::Includes(entries) => {
            let texts: Vec<_> = entries.iter().map(|entry| entry.text.as_str()).collect();
            assert_eq!(texts, vec!["widgets > items", "widgets > orders"]);
        }
        ParsedConfig::Unanalyzable { reason, .. } => panic!("{reason}"),
    }
    match parse_vite_config(Path::new("vite.config.ts"), source, false) {
        ParsedConfig::Unanalyzable { reason, .. } => {
            assert!(reason.contains("not a static array"));
        }
        ParsedConfig::Includes(_) => {
            panic!("spread include stays unanalyzable without includeSources")
        }
    }
}
