//! Static runtime filesystem resource call extraction.
//!
//! This intentionally consumes the already-parsed OXC program. It never
//! evaluates JavaScript or loads a glob implementation.

mod bindings;
mod bindings_calls;
mod paths;
mod visitor;
mod visitor_bindings;
mod visitor_calls;
mod visitor_paths;
mod visitor_scopes;
mod visitor_vars;
mod visitor_visit;
mod visitor_walk;

use oxc_ast::ast::Program;
use visitor::ResourceVisitor;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Ord, PartialOrd)]
pub enum ResourceCallKind {
    ReadFile,
    ReadFileSync,
    ReadDirectory,
    ReadDirectorySync,
    Glob,
    GlobSync,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Ord, PartialOrd)]
pub enum ResourcePathBase {
    AnalysisRoot,
    SourceModule,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ResourcePath {
    pub value: String,
    pub base: ResourcePathBase,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ResourceCall {
    pub kind: ResourceCallKind,
    pub path: ResourcePath,
    pub cwd: Option<ResourcePath>,
    pub line: usize,
    pub function_scope: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Ord, PartialOrd)]
pub enum ResourceDiagnosticKind {
    DynamicPath,
    DynamicPattern,
    DynamicCwd,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ResourceDiagnostic {
    pub kind: ResourceDiagnosticKind,
    pub line: usize,
    pub function_scope: Option<String>,
}

#[derive(Debug, Clone, Default)]
pub struct ResourceFacts {
    pub calls: Vec<ResourceCall>,
    pub diagnostics: Vec<ResourceDiagnostic>,
}

/// Extract resource calls from the parser-owned program. Bindings must originate
/// from a supported module; identically named local functions are ignored.
pub fn extract(program: &Program<'_>, source: &str) -> ResourceFacts {
    let mut visitor = ResourceVisitor {
        source,
        ..ResourceVisitor::default()
    };
    for statement in &program.body {
        if let oxc_ast::ast::Statement::ImportDeclaration(import) = statement {
            visitor.register_import(import);
        }
    }
    visitor.predeclare_statement_bindings(&program.body);
    visitor.predeclare_var_bindings_in_statements(&program.body);
    oxc_ast_visit::Visit::visit_program(&mut visitor, program);
    visitor.facts.calls.sort_by(|left, right| {
        (left.line, left.kind, &left.path.value, &left.function_scope).cmp(&(
            right.line,
            right.kind,
            &right.path.value,
            &right.function_scope,
        ))
    });
    visitor.facts.calls.dedup();
    visitor.facts.diagnostics.sort_by(|left, right| {
        (left.line, left.kind, &left.function_scope).cmp(&(
            right.line,
            right.kind,
            &right.function_scope,
        ))
    });
    visitor.facts.diagnostics.dedup();
    visitor.facts
}

#[cfg(test)]
mod tests;
