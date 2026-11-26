use crate::fx::{FxHashMap, FxHashSet};
use oxc_ast::{ast::*, AstKind};
use oxc_semantic::Semantic;
use oxc_syntax::symbol::SymbolId;

pub(super) fn unique_receivers(semantic: &Semantic<'_>) -> FxHashSet<SymbolId> {
    let mut groups: FxHashMap<String, Vec<SymbolId>> = FxHashMap::default();
    for node in semantic.nodes().iter() {
        let AstKind::ImportDeclaration(import) = node.kind() else {
            continue;
        };
        if import.import_kind.is_type() {
            continue;
        }
        let literal = crate::ast::js_literal(&import.source);
        let module = literal.strip_prefix("node:").unwrap_or(&literal);
        if !matches!(module, "fs" | "fs/promises" | "fs-extra") {
            continue;
        }
        for specifier in import.specifiers.iter().flatten() {
            let binding = match specifier {
                ImportDeclarationSpecifier::ImportDefaultSpecifier(specifier) => &specifier.local,
                ImportDeclarationSpecifier::ImportNamespaceSpecifier(specifier) => &specifier.local,
                ImportDeclarationSpecifier::ImportSpecifier(specifier)
                    if module == "fs"
                        && !specifier.import_kind.is_type()
                        && matches!(&specifier.imported, ModuleExportName::IdentifierName(name) if name.name == "promises") =>
                {
                    &specifier.local
                }
                _ => continue,
            };
            groups
                .entry(
                    if module == "fs-extra" {
                        "fs-extra"
                    } else {
                        "fs"
                    }
                    .into(),
                )
                .or_default()
                .extend(binding.symbol_id.get());
        }
    }
    groups
        .into_values()
        .filter_map(|symbols| (symbols.len() == 1).then(|| symbols[0]))
        .collect()
}
