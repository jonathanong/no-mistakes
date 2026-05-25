use oxc_ast::ast::{Expression, ImportExpression, Program, TemplateLiteral};
use oxc_ast_visit::{walk, Visit};
use std::path::{Path, PathBuf};

pub(super) fn collect(
    abs_path: &Path,
    program: &Program<'_>,
    resolver: &crate::codebase::ts_resolver::ImportResolver<'_>,
) -> Vec<PathBuf> {
    let mut collector = DynamicImportCollector {
        abs_path,
        resolver,
        imports: Vec::new(),
    };
    collector.visit_program(program);
    collector.imports
}

struct DynamicImportCollector<'a, 'r> {
    abs_path: &'a Path,
    resolver: &'r crate::codebase::ts_resolver::ImportResolver<'r>,
    imports: Vec<PathBuf>,
}

impl<'a> Visit<'a> for DynamicImportCollector<'_, '_> {
    fn visit_import_expression(&mut self, import: &ImportExpression<'a>) {
        if let Some(specifier) = static_string_expr(&import.source) {
            if let Some(resolved) = self.resolver.resolve(&specifier, self.abs_path) {
                self.imports.push(resolved);
            }
        }
        walk::walk_import_expression(self, import);
    }
}

fn static_string_expr(expr: &Expression<'_>) -> Option<String> {
    match crate::codebase::ts_source::unwrap_ts_wrappers(expr) {
        Expression::StringLiteral(literal) => Some(literal.value.as_str().to_string()),
        Expression::TemplateLiteral(template) => static_template_string(template),
        _ => None,
    }
}

fn static_template_string(template: &TemplateLiteral<'_>) -> Option<String> {
    if !template.expressions.is_empty() {
        return None;
    }
    let mut value = String::new();
    for quasi in &template.quasis {
        value.push_str(quasi.value.cooked.as_ref().unwrap_or(&quasi.value.raw));
    }
    Some(value)
}
