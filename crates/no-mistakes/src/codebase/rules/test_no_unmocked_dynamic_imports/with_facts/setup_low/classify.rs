use oxc_ast::ast::{
    Argument, CallExpression, Declaration, Expression, ImportExpression, Program, Statement,
    TemplateLiteral,
};
use oxc_ast_visit::{walk, Visit};
use std::collections::HashSet;

/// Shapes that may register setup coverage or load another setup helper.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct Classified {
    pub(super) accepted_mocks: Vec<String>,
    pub(super) accepted_requires: Vec<String>,
    pub(super) rejected: Vec<Rejected>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct Rejected {
    pub(super) line: usize,
    pub(super) target: &'static str,
}

pub(super) fn classify(path: &std::path::Path, source: &str) -> anyhow::Result<Classified> {
    crate::ast::with_program(path, source, |program, source| {
        classify_program(source, program)
    })
}

fn classify_program(source: &str, program: &Program<'_>) -> Classified {
    let mut accepted_require_spans = HashSet::new();
    for statement in &program.body {
        note_statement(&mut accepted_require_spans, statement);
    }
    let mut scan = Scan {
        source,
        accepted_require_spans: &accepted_require_spans,
        mock_import_starts: HashSet::new(),
        accepted_mocks: Vec::new(),
        accepted_requires: Vec::new(),
        rejected: Vec::new(),
    };
    scan.visit_program(program);
    Classified {
        accepted_mocks: scan.accepted_mocks,
        accepted_requires: scan.accepted_requires,
        rejected: scan.rejected,
    }
}

/// Only a direct top-level `require("literal")` is accepted. A condition,
/// sequence, `void`, or nested call is a different shape and stays rejected.
fn note_statement(spans: &mut HashSet<u32>, statement: &Statement<'_>) {
    match statement {
        Statement::ExpressionStatement(statement) => note_expr(spans, &statement.expression),
        Statement::VariableDeclaration(declaration) => note_variables(spans, declaration),
        Statement::ExportDefaultDeclaration(declaration) => {
            if let Some(expression) = declaration.declaration.as_expression() {
                note_expr(spans, expression);
            }
        }
        Statement::ExportDeclaration(declaration) => {
            if let Declaration::VariableDeclaration(declaration) = &declaration.declaration {
                note_variables(spans, declaration);
            }
        }
        _ => {}
    }
}

fn note_variables(spans: &mut HashSet<u32>, declaration: &oxc_ast::ast::VariableDeclaration<'_>) {
    for declarator in &declaration.declarations {
        if let Some(init) = &declarator.init {
            note_expr(spans, init);
        }
    }
}

fn note_expr(spans: &mut HashSet<u32>, expression: &Expression<'_>) {
    let Expression::CallExpression(call) =
        crate::codebase::ts_source::unwrap_ts_wrappers(expression)
    else {
        return;
    };
    if !call.optional && is_require(&call.callee) {
        spans.insert(call.span.start);
    }
}

struct Scan<'a> {
    source: &'a str,
    accepted_require_spans: &'a HashSet<u32>,
    mock_import_starts: HashSet<u32>,
    accepted_mocks: Vec<String>,
    accepted_requires: Vec<String>,
    rejected: Vec<Rejected>,
}

impl Scan<'_> {
    fn reject(&mut self, start: u32, target: &'static str) {
        let line = crate::codebase::ts_source::byte_offset_to_line(self.source, start as usize);
        self.rejected.push(Rejected {
            line: line as usize,
            target,
        });
    }

    fn record_call(&mut self, call: &CallExpression<'_>) {
        if let Some(method) = mock_method(call) {
            if let Some(Argument::ImportExpression(import)) = call.arguments.first() {
                self.mock_import_starts.insert(import.span.start);
            }
            if method == "mock" {
                if let Some(specifier) = mock_specifier(call) {
                    self.accepted_mocks.push(specifier);
                }
                return;
            }
            if let Some(target) = rejected_mock(method) {
                self.reject(call.span.start, target);
            }
            return;
        }
        if !is_require(&call.callee) {
            return;
        }
        let literal = call.arguments.first().and_then(literal_arg);
        if !call.optional && self.accepted_require_spans.contains(&call.span.start) {
            if let Some(specifier) = literal {
                self.accepted_requires.push(specifier);
                return;
            }
        }
        self.reject(call.span.start, "require");
    }
}

impl<'a> Visit<'a> for Scan<'_> {
    fn visit_import_expression(&mut self, import: &ImportExpression<'a>) {
        if !self.mock_import_starts.contains(&import.span.start) {
            self.reject(import.span.start, "import");
        }
        walk::walk_import_expression(self, import);
    }

    fn visit_call_expression(&mut self, call: &CallExpression<'a>) {
        self.record_call(call);
        walk::walk_call_expression(self, call);
    }
}

fn mock_method<'a>(call: &'a CallExpression<'_>) -> Option<&'a str> {
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

fn rejected_mock(method: &str) -> Option<&'static str> {
    Some(match method {
        "doMock" => "doMock",
        "setMock" => "setMock",
        "unstable_mockModule" => "unstable_mockModule",
        _ => return None,
    })
}

fn is_require(callee: &Expression<'_>) -> bool {
    matches!(callee, Expression::Identifier(identifier) if identifier.name == "require")
}

fn mock_specifier(call: &CallExpression<'_>) -> Option<String> {
    let first = call.arguments.first()?;
    if let Some(specifier) = literal_arg(first) {
        return Some(specifier);
    }
    let Argument::ImportExpression(import) = first else {
        return None;
    };
    literal_expr(crate::codebase::ts_source::unwrap_ts_wrappers(
        &import.source,
    ))
}

fn literal_arg(argument: &Argument<'_>) -> Option<String> {
    argument.as_expression().and_then(|expression| {
        literal_expr(crate::codebase::ts_source::unwrap_ts_wrappers(expression))
    })
}

fn literal_expr(expression: &Expression<'_>) -> Option<String> {
    match expression {
        Expression::StringLiteral(literal) => Some(literal.value.to_string()),
        Expression::TemplateLiteral(template) => static_template(template),
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
