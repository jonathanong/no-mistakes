use anyhow::{Context, Result};
use oxc_allocator::Allocator;
use oxc_ast::ast::{
    Argument, CallExpression, Expression, ImportExpression, Program, TemplateLiteral,
};
use oxc_ast_visit::{walk, Visit};
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
    let parsed = crate::ast::parse(path, &allocator, source, source_type);
    Ok(extract_program(source, &parsed.program))
}

pub(crate) fn extract_program(source: &str, program: &Program<'_>) -> TestFacts {
    let mut visitor = Collector::new(source);
    visitor.visit_program(program);
    visitor.facts
}

pub(crate) struct Collector<'s> {
    source: &'s str,
    facts: TestFacts,
    /// Byte offsets (`Span::start`) of `ImportExpression`s used as the first argument of a
    /// mock call, e.g. `vi.mock(import("./dep"), factory)`. These are type carriers for the
    /// mocked module, not runtime dynamic imports, so `visit_import_expression` skips
    /// recording them into `TestFacts.dynamic_imports`. See issue #506.
    mock_import_starts: HashSet<u32>,
}

impl<'s> Collector<'s> {
    pub(crate) fn new(source: &'s str) -> Self {
        Self {
            source,
            facts: TestFacts::default(),
            mock_import_starts: HashSet::new(),
        }
    }

    pub(crate) fn into_facts(self) -> TestFacts {
        self.facts
    }

    pub(crate) fn record_import_expression(&mut self, import: &ImportExpression<'_>) {
        if self.mock_import_starts.contains(&import.span.start) {
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
    }

    pub(crate) fn record_call_expression(&mut self, call: &CallExpression<'_>) {
        if !is_mock_call(call) {
            return;
        }
        let Some(first) = call.arguments.first() else {
            return;
        };
        if let Some(specifier) = string_arg(first) {
            self.facts.mock_specifiers.push(specifier);
        } else if accepts_typed_import_specifier(call) {
            if let Argument::ImportExpression(import) = first {
                if let Some(specifier) = string_expr(&import.source) {
                    self.facts.mock_specifiers.push(specifier);
                    self.mock_import_starts.insert(import.span.start);
                }
            }
        }
    }
}

impl<'a> Visit<'a> for Collector<'_> {
    fn visit_import_expression(&mut self, import: &ImportExpression<'a>) {
        self.record_import_expression(import);
        walk::walk_import_expression(self, import);
    }

    fn visit_call_expression(&mut self, call: &CallExpression<'a>) {
        self.record_call_expression(call);
        walk::walk_call_expression(self, call);
    }
}

/// Returns the `vi`/`jest` mock method name for `call` (e.g. `"mock"`, `"doMock"`), or
/// `None` if the callee is not a `vi.<method>()` / `jest.<method>()` static member call.
/// Shared by `is_mock_call` and `accepts_typed_import_specifier` so both callee shapes are
/// matched in one place.
fn mock_method_name<'e>(call: &'e CallExpression<'_>) -> Option<&'e str> {
    let Expression::StaticMemberExpression(member) = &call.callee else {
        return None;
    };
    let Expression::Identifier(object) = &member.object else {
        return None;
    };
    if !matches!(object.name.as_str(), "vi" | "jest") {
        return None;
    }
    Some(member.property.name.as_str())
}

fn is_mock_call(call: &CallExpression<'_>) -> bool {
    matches!(
        mock_method_name(call),
        Some("mock" | "doMock" | "unstable_mockModule" | "setMock")
    )
}

/// Whether `call`'s first argument may be a typed `import(...)` mock specifier. Only
/// `vi.mock` / `vi.doMock` / `jest.mock` / `jest.doMock` support the module-promise
/// overload; `jest.setMock` and `jest.unstable_mockModule` accept a plain string module
/// name only, so a literal `import(...)` passed there is a genuine runtime dynamic import
/// (its result is not used as a specifier) and must still be tracked as one.
fn accepts_typed_import_specifier(call: &CallExpression<'_>) -> bool {
    matches!(mock_method_name(call), Some("mock" | "doMock"))
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
