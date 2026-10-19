use super::local_name;
use oxc_allocator::Allocator;
use oxc_ast::ast::{ModuleExportName, Statement};
use oxc_parser::Parser;
use oxc_span::SourceType;

#[test]
fn export_names_cover_identifier_and_string_forms() {
    let allocator = Allocator::default();
    let source = "export { name as exported };\nexport { \"raw\" as other } from \"./m\";\n";
    let parsed = Parser::new(&allocator, source, SourceType::mjs()).parse();
    assert!(parsed.diagnostics.is_empty(), "{:?}", parsed.diagnostics);
    let mut saw_name = false;
    let mut saw_string = false;
    for statement in &parsed.program.body {
        let specifiers = match statement {
            Statement::ExportNamedDeclaration(export) => &export.specifiers,
            Statement::ExportFromDeclaration(export) => &export.specifiers,
            _ => continue,
        };
        for specifier in specifiers {
            // The exported side of `as` is an identifier name. A string local
            // only appears on `export { "..." as name } from`.
            if let ModuleExportName::IdentifierName(ident) = &specifier.exported {
                assert_eq!(local_name(&specifier.exported), Some(ident.name.as_str()));
                saw_name = true;
            }
            if let ModuleExportName::StringLiteral(_) = &specifier.local {
                assert_eq!(local_name(&specifier.local), None);
                saw_string = true;
            }
        }
    }
    assert!(saw_name, "{:?}", parsed.program);
    assert!(saw_string, "{:?}", parsed.program);
}
