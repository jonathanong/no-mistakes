use super::{spreads, CollectedExcludes, ExcludeArray, SpreadImport};
use crate::codebase::ts_source::unwrap_ts_wrappers;
use oxc_ast::ast::{
    CallExpression, Expression, ImportDeclarationSpecifier, ObjectExpression, ObjectProperty,
    Program, Statement,
};
use oxc_ast_visit::{walk, Visit};
use oxc_semantic::{Semantic, SymbolId};
use rustc_hash::{FxHashMap, FxHashSet};

pub(crate) fn collect_prepared<'a>(
    program: &'a Program<'a>,
    source: &str,
    semantic: &Semantic<'a>,
    selected: bool,
) -> CollectedExcludes {
    let exports = spreads::exports(program, semantic);
    if !selected {
        return CollectedExcludes {
            exports,
            ..Default::default()
        };
    }
    let imports = VitestImports::collect(program, semantic);
    let default_export = collect_default_export(program, source, &imports);
    let mut visitor = TestLiteralVisitor {
        imports: &imports,
        source,
        arrays: Vec::new(),
    };
    visitor.visit_program(program);
    CollectedExcludes {
        default_export,
        test_literals: visitor.arrays,
        exports,
    }
}

fn collect_default_export<'ast>(
    program: &'ast Program<'ast>,
    source: &str,
    imports: &VitestImports<'_, 'ast>,
) -> Vec<ExcludeArray> {
    let Some(root) = program.body.iter().find_map(|statement| {
        let Statement::ExportDefaultDeclaration(export) = statement else {
            return None;
        };
        imports.config_object(export.declaration.as_expression()?)
    }) else {
        return Vec::new();
    };
    let Some(test) = property(root, "test").and_then(|prop| literal_object(&prop.value)) else {
        return Vec::new();
    };
    let mut arrays = Vec::new();
    arrays.extend(imports.exclude_array(test, source));
    if let Some(projects) =
        property(test, "projects").and_then(|prop| match unwrap_ts_wrappers(&prop.value) {
            Expression::ArrayExpression(array) => Some(array),
            _ => None,
        })
    {
        arrays.extend(projects.elements.iter().filter_map(|entry| {
            let project = imports.config_object(entry.as_expression()?)?;
            let test = literal_object(&property(project, "test")?.value)?;
            imports.exclude_array(test, source)
        }));
    }
    arrays
}

struct TestLiteralVisitor<'a, 'ast> {
    imports: &'a VitestImports<'a, 'ast>,
    source: &'a str,
    arrays: Vec<ExcludeArray>,
}

impl<'a> Visit<'a> for TestLiteralVisitor<'_, 'a> {
    fn visit_object_expression(&mut self, object: &ObjectExpression<'a>) {
        if let Some(test) = property(object, "test").and_then(|prop| literal_object(&prop.value)) {
            if let Some(array) = self.imports.exclude_array(test, self.source) {
                self.arrays.push(array);
            }
        }
        walk::walk_object_expression(self, object);
    }
}

pub(super) fn property<'a>(
    object: &'a ObjectExpression<'a>,
    name: &str,
) -> Option<&'a ObjectProperty<'a>> {
    for property in object.properties.iter().rev() {
        // A later spread or dynamic key can replace the property being queried.
        let property = property.as_property()?;
        if property.key.static_name()? == name {
            return Some(property);
        }
    }
    None
}

fn literal_object<'a>(expression: &'a Expression<'a>) -> Option<&'a ObjectExpression<'a>> {
    match unwrap_ts_wrappers(expression) {
        Expression::ObjectExpression(object) => Some(object),
        _ => None,
    }
}

pub(super) struct VitestImports<'a, 'ast> {
    pub(super) semantic: &'a Semantic<'ast>,
    wrappers: FxHashSet<SymbolId>,
    pub(super) defaults: FxHashSet<SymbolId>,
    pub(super) relative: FxHashMap<SymbolId, SpreadImport>,
}

impl<'a, 'ast> VitestImports<'a, 'ast> {
    fn collect(program: &'ast Program<'ast>, semantic: &'a Semantic<'ast>) -> Self {
        let mut imports = Self {
            semantic,
            wrappers: FxHashSet::default(),
            defaults: FxHashSet::default(),
            relative: spreads::relative_imports(program),
        };
        for statement in &program.body {
            let Statement::ImportDeclaration(import) = statement else {
                continue;
            };
            if import.import_kind.is_type()
                || crate::ast::js_literal(&import.source).as_ref() != "vitest/config"
            {
                continue;
            }
            for specifier in import.specifiers.iter().flatten() {
                let ImportDeclarationSpecifier::ImportSpecifier(binding) = specifier else {
                    continue;
                };
                if binding.import_kind.is_type() {
                    continue;
                }
                let Some(symbol) = binding.local.symbol_id.get() else {
                    continue;
                };
                match crate::ast::module_export_name(&binding.imported).as_ref() {
                    "defineConfig" | "defineProject" => {
                        imports.wrappers.insert(symbol);
                    }
                    "configDefaults" => {
                        imports.defaults.insert(symbol);
                    }
                    _ => {}
                }
            }
        }
        imports
    }

    fn config_object(
        &self,
        expression: &'ast Expression<'ast>,
    ) -> Option<&'ast ObjectExpression<'ast>> {
        match unwrap_ts_wrappers(expression) {
            Expression::ObjectExpression(object) => Some(object),
            Expression::CallExpression(call) => self.wrapped_object(call),
            _ => None,
        }
    }

    fn wrapped_object(
        &self,
        call: &'ast CallExpression<'ast>,
    ) -> Option<&'ast ObjectExpression<'ast>> {
        let Expression::Identifier(callee) = unwrap_ts_wrappers(&call.callee) else {
            return None;
        };
        let [arg] = call.arguments.as_slice() else {
            return None;
        };
        if call.optional || !self.is_import(callee, &self.wrappers) {
            return None;
        }
        literal_object(arg.as_expression()?)
    }
}

pub(super) fn literal_glob<'a>(
    expression: &'a Expression<'a>,
) -> Option<std::borrow::Cow<'a, str>> {
    match unwrap_ts_wrappers(expression) {
        Expression::StringLiteral(value) => Some(crate::ast::js_literal(value)),
        Expression::TemplateLiteral(template) if template.expressions.is_empty() => {
            let quasi = template.quasis.first()?;
            quasi.value.cooked?;
            Some(crate::ast::template_quasi_text(quasi))
        }
        _ => None,
    }
}
