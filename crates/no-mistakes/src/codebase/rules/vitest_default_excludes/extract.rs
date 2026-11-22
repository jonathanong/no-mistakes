use super::{ExcludeArray, DEFAULT_GLOBS};
use crate::codebase::ts_source::{line_number, unwrap_ts_wrappers};
use oxc_ast::ast::{
    ArrayExpressionElement, CallExpression, Expression, ImportDeclarationSpecifier,
    ObjectExpression, ObjectProperty, Program, Statement,
};
use oxc_span::GetSpan;
use rustc_hash::FxHashSet;

pub(crate) fn collect(program: &Program<'_>, source: &str) -> Vec<ExcludeArray> {
    let imports = VitestImports::collect(program);
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
        for element in &array.elements {
            if let ArrayExpressionElement::SpreadElement(spread) = element {
                if !self.default_spread(&spread.argument) {
                    return None;
                }
                globs.extend(DEFAULT_GLOBS.into_iter().map(str::to_owned));
            } else {
                let Expression::StringLiteral(value) = unwrap_ts_wrappers(element.as_expression()?)
                else {
                    return None;
                };
                globs.insert(crate::ast::js_literal(value).into_owned());
            }
        }
        Some(ExcludeArray {
            line: line_number(source, exclude.key.span().start),
            globs,
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
