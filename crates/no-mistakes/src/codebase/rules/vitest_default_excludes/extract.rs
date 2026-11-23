use super::{CollectedExcludes, ExcludeArray};
use crate::codebase::ts_source::{line_number, unwrap_ts_wrappers};
use oxc_ast::ast::{
    ArrayExpressionElement, CallExpression, Expression, ImportDeclarationSpecifier,
    ObjectExpression, ObjectProperty, Program, Statement,
};
use oxc_ast_visit::{walk, Visit};
use oxc_span::GetSpan;
use rustc_hash::FxHashSet;

pub(crate) fn collect(program: &Program<'_>, source: &str) -> CollectedExcludes {
    let imports = VitestImports::collect(program);
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
    }
}

fn collect_default_export(
    program: &Program<'_>,
    source: &str,
    imports: &VitestImports<'_>,
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

struct TestLiteralVisitor<'a> {
    imports: &'a VitestImports<'a>,
    source: &'a str,
    arrays: Vec<ExcludeArray>,
}

impl<'a> Visit<'a> for TestLiteralVisitor<'_> {
    fn visit_object_expression(&mut self, object: &ObjectExpression<'a>) {
        if let Some(test) = property(object, "test").and_then(|prop| literal_object(&prop.value)) {
            if let Some(array) = self.imports.exclude_array(test, self.source) {
                self.arrays.push(array);
            }
        }
        walk::walk_object_expression(self, object);
    }
}

fn property<'a>(object: &'a ObjectExpression<'a>, name: &str) -> Option<&'a ObjectProperty<'a>> {
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

#[derive(Default)]
struct VitestImports<'a> {
    wrappers: FxHashSet<&'a str>,
    defaults: FxHashSet<&'a str>,
}

impl<'a> VitestImports<'a> {
    fn collect(program: &'a Program<'a>) -> Self {
        let mut imports = Self::default();
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
                match crate::ast::module_export_name(&binding.imported).as_ref() {
                    "defineConfig" | "defineProject" => {
                        imports.wrappers.insert(binding.local.name.as_str());
                    }
                    "configDefaults" => {
                        imports.defaults.insert(binding.local.name.as_str());
                    }
                    _ => {}
                }
            }
        }
        imports
    }

    fn config_object(&self, expression: &'a Expression<'a>) -> Option<&'a ObjectExpression<'a>> {
        match unwrap_ts_wrappers(expression) {
            Expression::ObjectExpression(object) => Some(object),
            Expression::CallExpression(call) => self.wrapped_object(call),
            _ => None,
        }
    }

    fn wrapped_object(&self, call: &'a CallExpression<'a>) -> Option<&'a ObjectExpression<'a>> {
        let Expression::Identifier(callee) = unwrap_ts_wrappers(&call.callee) else {
            return None;
        };
        let [arg] = call.arguments.as_slice() else {
            return None;
        };
        if call.optional || !self.wrappers.contains(callee.name.as_str()) {
            return None;
        }
        literal_object(arg.as_expression()?)
    }

    fn exclude_array(&self, test: &ObjectExpression<'_>, source: &str) -> Option<ExcludeArray> {
        let exclude = property(test, "exclude")?;
        let Expression::ArrayExpression(array) = unwrap_ts_wrappers(&exclude.value) else {
            return None;
        };
        let mut globs = FxHashSet::default();
        let mut spreads_config_defaults = false;
        for element in &array.elements {
            if let ArrayExpressionElement::SpreadElement(spread) = element {
                if !self.default_spread(&spread.argument) {
                    return None;
                }
                spreads_config_defaults = true;
            } else {
                globs.insert(literal_glob(element.as_expression()?)?.into_owned());
            }
        }
        Some(ExcludeArray {
            line: line_number(source, exclude.key.span().start),
            globs,
            spreads_config_defaults,
        })
    }

    fn default_spread(&self, expression: &Expression<'_>) -> bool {
        let Expression::StaticMemberExpression(member) = unwrap_ts_wrappers(expression) else {
            return false;
        };
        !member.optional
            && member.property.name == "exclude"
            && matches!(unwrap_ts_wrappers(&member.object), Expression::Identifier(binding)
                if self.defaults.contains(binding.name.as_str()))
    }
}

fn literal_glob<'a>(expression: &'a Expression<'a>) -> Option<std::borrow::Cow<'a, str>> {
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
