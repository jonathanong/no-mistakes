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
        decl: &oxc::ast::ast::ExportDefaultDeclaration<'a>,
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
                    oxc::ast::ast::ObjectPropertyKind::ObjectProperty(prop) => {
                        Some(self.entry_from_expression(&prop.value))
                    }
                    oxc::ast::ast::ObjectPropertyKind::SpreadProperty(_) => None,
                })
                .collect();
            self.container = Some(RawContainer {
                kind: "container-object".to_string(),
                entries,
            });
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
}

fn slice(source: &str, start: u32, end: u32) -> String {
    source
        .get(start as usize..end as usize)
        .unwrap_or("")
        .to_string()
}

/// `(key, display)` for a call's callee. `key` groups calls; `display` is the
/// verbatim callee text.
fn callee_key(callee: &Expression<'_>, source: &str) -> Option<(String, String)> {
    match callee {
        Expression::Identifier(ident) => Some((ident.name.to_string(), ident.name.to_string())),
        Expression::StaticMemberExpression(member) => {
            let display = slice(source, member.span.start, member.span.end);
            Some((member.property.name.to_string(), display))
        }
        _ => None,
    }
}

fn argument_import(
    arg: &Argument<'_>,
    imports: &HashMap<String, EntryImport>,
) -> Option<EntryImport> {
    expression_import(arg.as_expression()?, imports)
}

fn expression_import(
    expr: &Expression<'_>,
    imports: &HashMap<String, EntryImport>,
) -> Option<EntryImport> {
    match expr {
        Expression::Identifier(ident) => imports.get(ident.name.as_str()).cloned(),
        Expression::NewExpression(new) => new_expression_import(new, imports),
        Expression::ArrowFunctionExpression(_) | Expression::FunctionExpression(_) => {
            dynamic_import(expr)
        }
        _ => None,
    }
}

fn new_expression_import(
    new: &NewExpression<'_>,
    imports: &HashMap<String, EntryImport>,
) -> Option<EntryImport> {
    if let Expression::Identifier(ident) = &new.callee {
        return imports.get(ident.name.as_str()).cloned();
    }
    None
}

/// Find a `() => import("...")` dynamic import inside an expression.
fn dynamic_import(expr: &Expression<'_>) -> Option<EntryImport> {
    let mut finder = ImportExprFinder { specifier: None };
    match expr {
        Expression::ArrowFunctionExpression(arrow) => finder.visit_function_body(&arrow.body),
        Expression::FunctionExpression(function) => {
            if let Some(body) = &function.body {
                finder.visit_function_body(body);
            }
        }
        _ => {}
    }
    finder.specifier.map(|specifier| EntryImport {
        specifier,
        symbol: None,
        local: String::new(),
        kind: "dynamic".to_string(),
    })
}

struct ImportExprFinder {
    specifier: Option<String>,
}

impl<'a> Visit<'a> for ImportExprFinder {
    fn visit_import_expression(&mut self, import: &ImportExpression<'a>) {
        if self.specifier.is_none() {
            if let Expression::StringLiteral(literal) = &import.source {
                self.specifier = Some(literal.value.as_str().to_string());
            }
        }
        walk::walk_import_expression(self, import);
    }
}
