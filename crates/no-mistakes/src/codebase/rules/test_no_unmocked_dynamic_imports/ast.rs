use anyhow::{Context, Result};
use oxc_allocator::Allocator;
use oxc_ast::ast::{
    Argument, CallExpression, Expression, ImportExpression, Program, TemplateLiteral,
};
use oxc_ast_visit::{walk, Visit};
use oxc_parser::Parser;
use oxc_span::SourceType;
use std::collections::HashSet;
use std::path::Path;

#[derive(Clone)]
pub struct DynamicImport {
    pub specifier: Option<String>,
    pub line: usize,
}

#[derive(Clone, Default)]
pub struct TestFacts {
    pub dynamic_imports: Vec<DynamicImport>,
    pub mock_specifiers: Vec<String>,
}

pub fn extract(path: &Path, source: &str) -> Result<TestFacts> {
    let allocator = Allocator::default();
    let source_type = SourceType::from_path(path).context(format!(
        "unsupported JavaScript/TypeScript file: {}",
        path.display()
    ))?;
    let parsed = Parser::new(&allocator, source, source_type).parse();
    Ok(extract_program(source, &parsed.program))
}

pub(crate) fn extract_program(source: &str, program: &Program<'_>) -> TestFacts {
    let mut visitor = Collector {
        source,
        facts: TestFacts::default(),
        mock_import_starts: HashSet::new(),
    };
    visitor.visit_program(program);
    visitor.facts
}

struct Collector<'s> {
    source: &'s str,
    facts: TestFacts,
    /// Byte offsets (`Span::start`) of `ImportExpression`s used as the first argument of a
    /// mock call, e.g. `vi.mock(import("./dep"), factory)`. These are type carriers for the
    /// mocked module, not runtime dynamic imports, so `visit_import_expression` skips
    /// recording them into `TestFacts.dynamic_imports`. See issue #506.
    mock_import_starts: HashSet<u32>,
}

impl<'a> Visit<'a> for Collector<'_> {
    fn visit_import_expression(&mut self, import: &ImportExpression<'a>) {
        if self.mock_import_starts.contains(&import.span.start) {
            // Type-carrier import for a typed mock specifier; already recorded (if static)
            // into `mock_specifiers` by `visit_call_expression`. Still walk its children so
            // any nested dynamic import is not missed.
            walk::walk_import_expression(self, import);
            return;
        }
        let line = crate::codebase::ts_source::byte_offset_to_line(
            self.source,
            import.span.start as usize,
        ) as usize;
        self.facts.dynamic_imports.push(DynamicImport {
            specifier: string_expr(&import.source),
            line,
        });
        walk::walk_import_expression(self, import);
    }

    fn visit_call_expression(&mut self, call: &CallExpression<'a>) {
        if is_mock_call(call) {
            if let Some(first) = call.arguments.first() {
                if let Some(specifier) = string_arg(first) {
                    self.facts.mock_specifiers.push(specifier);
                } else if let Argument::ImportExpression(import) = first {
                    // Typed Vitest/Jest mock specifier, e.g.
                    // `vi.mock(import("./dep"), factory)` — bare `import(...)` form only
                    // (a TS-wrapped carrier like `import("./dep") as unknown` is not matched).
                    // The import exists only so TypeScript can infer the mocked module's
                    // shape; it is not a runtime dynamic import. Record its specifier as a
                    // mock specifier when statically known, and mark its span so
                    // `visit_import_expression` does not also record it as a dynamic import.
                    if let Some(specifier) = string_expr(&import.source) {
                        self.facts.mock_specifiers.push(specifier);
                    }
                    self.mock_import_starts.insert(import.span.start);
                }
            }
        }
        walk::walk_call_expression(self, call);
    }
}

fn is_mock_call(call: &CallExpression<'_>) -> bool {
    let Expression::StaticMemberExpression(member) = &call.callee else {
        return false;
    };
    let Expression::Identifier(object) = &member.object else {
        return false;
    };
    if !matches!(object.name.as_str(), "vi" | "jest") {
        return false;
    }
    matches!(
        member.property.name.as_str(),
        "mock" | "doMock" | "unstable_mockModule" | "setMock"
    )
}

fn string_arg(arg: &Argument<'_>) -> Option<String> {
    match arg {
        Argument::StringLiteral(s) => Some(s.value.as_str().to_string()),
        Argument::TemplateLiteral(t) => static_template(t),
        _ => None,
    }
}

fn string_expr(expr: &Expression<'_>) -> Option<String> {
    match crate::codebase::ts_source::unwrap_ts_wrappers(expr) {
        Expression::StringLiteral(s) => Some(s.value.as_str().to_string()),
        Expression::TemplateLiteral(t) => static_template(t),
        _ => None,
    }
}

fn static_template(template: &TemplateLiteral<'_>) -> Option<String> {
    if !template.expressions.is_empty() {
        return None;
    }
    let mut value = String::new();
    for quasi in &template.quasis {
        value.push_str(quasi.value.cooked.as_ref().unwrap_or(&quasi.value.raw));
    }
    Some(value)
}

#[cfg(test)]
mod tests;
