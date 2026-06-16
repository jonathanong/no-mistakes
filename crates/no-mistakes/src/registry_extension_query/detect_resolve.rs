// Included into `registry_extension_query` via `include!`; shares that
// module's imports. Argument/callee import-resolution helpers.

fn slice(source: &str, start: u32, end: u32) -> String {
    source
        .get(start as usize..end as usize)
        .unwrap_or("")
        .to_string()
}

/// `(key, display)` for a call's callee. `key` groups calls and is the full
/// callee text so different registries that share a method name
/// (`alpha.register` vs `beta.register`) are not collapsed; `display` is the
/// same verbatim callee text reported as the registrant.
fn callee_key(callee: &Expression<'_>, source: &str) -> Option<(String, String)> {
    match callee {
        Expression::Identifier(ident) => Some((ident.name.to_string(), ident.name.to_string())),
        Expression::StaticMemberExpression(member) => {
            let display = slice(source, member.span.start, member.span.end);
            Some((display.clone(), display))
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
        // Imported factory call, e.g. `register(makeFoo())`: resolve the callee.
        Expression::CallExpression(call) => expression_import(&call.callee, imports),
        Expression::ArrowFunctionExpression(arrow) => dynamic_import(&arrow.body),
        Expression::FunctionExpression(function) => {
            function.body.as_deref().and_then(dynamic_import)
        }
        _ => None,
    }
}

fn new_expression_import(
    new: &NewExpression<'_>,
    imports: &HashMap<String, EntryImport>,
) -> Option<EntryImport> {
    match &new.callee {
        Expression::Identifier(ident) => imports.get(ident.name.as_str()).cloned(),
        // Namespace-import constructor, e.g. `new plugins.Foo()`: resolve the
        // namespace binding (`plugins`).
        Expression::StaticMemberExpression(member) => {
            if let Expression::Identifier(object) = &member.object {
                return imports.get(object.name.as_str()).cloned();
            }
            None
        }
        _ => None,
    }
}

/// Find a `() => import("...")` dynamic import inside a function body.
fn dynamic_import(body: &oxc_ast::ast::FunctionBody<'_>) -> Option<EntryImport> {
    let mut finder = ImportExprFinder { specifier: None };
    finder.visit_function_body(body);
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
