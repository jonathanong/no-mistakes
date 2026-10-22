use super::{aliases, syntax};
use oxc_ast::ast::{
    Argument, CallExpression, ImportExpression, Program, TSImportEqualsDeclaration,
    VariableDeclarator,
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

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct SetupReachability {
    pub(crate) accepted_mocks: Vec<String>,
    pub(crate) accepted_requires: Vec<String>,
    pub(crate) rejected: Vec<Rejected>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Rejected {
    pub(crate) line: usize,
    pub(crate) target: &'static str,
}

impl From<SetupReachability> for Classified {
    fn from(reachability: SetupReachability) -> Self {
        Self {
            accepted_mocks: reachability.accepted_mocks,
            accepted_requires: reachability.accepted_requires,
            rejected: reachability.rejected,
        }
    }
}

pub(crate) struct Scan<'a> {
    source: &'a str,
    accepted_require_spans: HashSet<u32>,
    mock_import_starts: HashSet<u32>,
    aliased_mocks: HashSet<String>,
    aliased_requires: HashSet<String>,
    accepted_mocks: Vec<String>,
    accepted_requires: Vec<String>,
    rejected: Vec<Rejected>,
}

impl<'a> Scan<'a> {
    /// Top-level require spans are noted before the shared program walk.
    pub(crate) fn new(source: &'a str, program: &Program<'_>) -> Self {
        let mut accepted_require_spans = HashSet::new();
        for statement in &program.body {
            syntax::note_statement(&mut accepted_require_spans, statement);
        }
        Self {
            source,
            accepted_require_spans,
            mock_import_starts: HashSet::new(),
            aliased_mocks: HashSet::new(),
            aliased_requires: HashSet::new(),
            accepted_mocks: Vec::new(),
            accepted_requires: Vec::new(),
            rejected: Vec::new(),
        }
    }

    pub(crate) fn into_reachability(self) -> SetupReachability {
        SetupReachability {
            accepted_mocks: self.accepted_mocks,
            accepted_requires: self.accepted_requires,
            rejected: self.rejected,
        }
    }
}

impl Scan<'_> {
    fn reject(&mut self, start: u32, target: &'static str) {
        let line = crate::codebase::ts_source::byte_offset_to_line(self.source, start as usize);
        self.rejected.push(Rejected {
            line: line as usize,
            target,
        });
    }

    pub(crate) fn record_import(&mut self, import: &ImportExpression<'_>) {
        if !self.mock_import_starts.contains(&import.span.start) {
            self.reject(import.span.start, "import");
        }
    }

    pub(crate) fn record_declarator(&mut self, declarator: &VariableDeclarator<'_>) {
        aliases::note_declarator(
            declarator,
            &mut self.aliased_mocks,
            &mut self.aliased_requires,
        );
    }

    pub(crate) fn record_import_equals(&mut self, declaration: &TSImportEqualsDeclaration<'_>) {
        if aliases::is_require_equals(declaration) {
            self.reject(declaration.span.start, "require");
        }
    }

    pub(crate) fn record_call(&mut self, call: &CallExpression<'_>) {
        if let Some(method) = syntax::mock_method(call) {
            self.note_mock_carrier(call);
            if method == "mock" {
                if let Some(specifier) = syntax::mock_specifier(call) {
                    self.accepted_mocks.push(specifier);
                } else {
                    self.reject(call.span.start, "mock");
                }
                return;
            }
            if let Some(target) = syntax::rejected_mock(method) {
                self.reject(call.span.start, target);
            }
            return;
        }
        if aliases::is_computed_vi_mock(&call.callee) {
            self.note_mock_carrier(call);
            self.reject(call.span.start, "mock");
            return;
        }
        if aliases::is_module_require(&call.callee) {
            self.reject(call.span.start, "require");
            return;
        }
        if let Some(name) = aliases::callee_name(&call.callee) {
            if self.aliased_mocks.contains(name) {
                self.note_mock_carrier(call);
                self.reject(call.span.start, "mock");
                return;
            }
            if self.aliased_requires.contains(name) {
                self.reject(call.span.start, "require");
                return;
            }
        }
        if !syntax::is_require(&call.callee) {
            return;
        }
        let literal = call.arguments.first().and_then(syntax::literal_arg);
        if !call.optional && self.accepted_require_spans.contains(&call.span.start) {
            if let Some(specifier) = literal {
                self.accepted_requires.push(specifier);
                return;
            }
        }
        self.reject(call.span.start, "require");
    }

    fn note_mock_carrier(&mut self, call: &CallExpression<'_>) {
        if let Some(Argument::ImportExpression(import)) = call.arguments.first() {
            self.mock_import_starts.insert(import.span.start);
        }
    }
}

impl<'a> Visit<'a> for Scan<'_> {
    fn visit_import_expression(&mut self, import: &ImportExpression<'a>) {
        self.record_import(import);
        walk::walk_import_expression(self, import);
    }

    fn visit_call_expression(&mut self, call: &CallExpression<'a>) {
        self.record_call(call);
        walk::walk_call_expression(self, call);
    }

    fn visit_variable_declarator(&mut self, declarator: &VariableDeclarator<'a>) {
        self.record_declarator(declarator);
        walk::walk_variable_declarator(self, declarator);
    }

    fn visit_ts_import_equals_declaration(&mut self, declaration: &TSImportEqualsDeclaration<'a>) {
        self.record_import_equals(declaration);
        walk::walk_ts_import_equals_declaration(self, declaration);
    }
}
