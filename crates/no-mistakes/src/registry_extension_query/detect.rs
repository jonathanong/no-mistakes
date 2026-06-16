// Included into `registry_extension_query` via `include!`; shares that module's
// imports. Holds the AST visitors and helpers that detect registry entries.

struct ImportCollector {
    imports: HashMap<String, EntryImport>,
    side_effects: Vec<String>,
}

impl<'a> Visit<'a> for ImportCollector {
    fn visit_import_declaration(&mut self, import: &ImportDeclaration<'a>) {
        let specifier = import.source.value.as_str().to_string();
        let Some(specifiers) = import.specifiers.as_deref() else {
            self.side_effects.push(specifier);
            return;
        };
        if specifiers.is_empty() {
            self.side_effects.push(specifier);
            return;
        }
        for spec in specifiers {
            let (symbol, local, kind) = match spec {
                ImportDeclarationSpecifier::ImportSpecifier(s) => (
                    s.imported.name().to_string(),
                    s.local.name.as_str().to_string(),
                    "static",
                ),
                ImportDeclarationSpecifier::ImportDefaultSpecifier(s) => (
                    "default".to_string(),
                    s.local.name.as_str().to_string(),
                    "default",
                ),
                ImportDeclarationSpecifier::ImportNamespaceSpecifier(s) => (
                    "*".to_string(),
                    s.local.name.as_str().to_string(),
                    "namespace",
                ),
            };
            self.imports.insert(
                local.clone(),
                EntryImport {
                    specifier: specifier.clone(),
                    symbol: Some(symbol),
                    local,
                    kind: kind.to_string(),
                },
            );
        }
    }
}

struct RawCall {
    key: String,
    registrant: String,
    line: usize,
    call_shape: String,
    entry_import: Option<EntryImport>,
}

struct RawContainer {
    kind: String,
    entries: Vec<RegistryEntry>,
}

struct BodyCollector<'a, 'i> {
    source: &'a str,
    imports: &'i HashMap<String, EntryImport>,
    calls: Vec<RawCall>,
    container: Option<RawContainer>,
}

impl<'a> Visit<'a> for BodyCollector<'a, '_> {
    fn visit_call_expression(&mut self, call: &CallExpression<'a>) {
        if let Some((key, registrant)) = callee_key(&call.callee, self.source) {
            let entry_import = call
                .arguments
                .iter()
                .find_map(|arg| argument_import(arg, self.imports));
            self.calls.push(RawCall {
                key,
                registrant,
                line: byte_offset_to_line(self.source, call.span.start as usize) as usize,
                call_shape: slice(self.source, call.span.start, call.span.end),
                entry_import,
            });
        }
        walk::walk_call_expression(self, call);
    }

    fn visit_export_default_declaration(
        &mut self,
        decl: &oxc_ast::ast::ExportDefaultDeclaration<'a>,
    ) {
        if let ExportDefaultDeclarationKind::ArrayExpression(array) = &decl.declaration {
            let entries = array
                .elements
                .iter()
                .filter_map(|element| element.as_expression())
                .map(|expr| self.entry_from_expression(expr))
                .collect();
            self.container = Some(RawContainer {
                kind: "container-array".to_string(),
                entries,
            });
        } else if let ExportDefaultDeclarationKind::ObjectExpression(object) = &decl.declaration {
            let entries = object
                .properties
                .iter()
                .filter_map(|property| match property {
                    oxc_ast::ast::ObjectPropertyKind::ObjectProperty(prop) => {
                        Some(self.entry_from_property(prop))
                    }
                    oxc_ast::ast::ObjectPropertyKind::SpreadProperty(_) => None,
                })
                .collect();
            self.container = Some(RawContainer {
                kind: "container-object".to_string(),
                entries,
            });
        } else {
            // e.g. `export default function setup() { registry.register(...) }`:
            // keep walking so register calls inside are still detected.
            walk::walk_export_default_declaration(self, decl);
        }
    }
}

impl BodyCollector<'_, '_> {
    fn entry_from_expression(&self, expr: &Expression<'_>) -> RegistryEntry {
        let span = expr.span();
        RegistryEntry {
            line: byte_offset_to_line(self.source, span.start as usize) as usize,
            entry_import: expression_import(expr, self.imports),
            call_shape: slice(self.source, span.start, span.end),
        }
    }

    /// Object-literal entry: keep the full `key: value` span as the shape while
    /// resolving the import from the value.
    fn entry_from_property(&self, property: &oxc_ast::ast::ObjectProperty<'_>) -> RegistryEntry {
        RegistryEntry {
            line: byte_offset_to_line(self.source, property.span.start as usize) as usize,
            entry_import: expression_import(&property.value, self.imports),
            call_shape: slice(self.source, property.span.start, property.span.end),
        }
    }
}
