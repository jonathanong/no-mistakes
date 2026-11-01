use super::super::{span, Selection, Span, Syntax};
use super::functions::{self, Params};
use super::{executor_spans, is_choice, mark_uses};
use crate::codebase::ts_source::{module_facts::TypeScriptModuleFacts, unwrap_ts_wrappers};
use crate::fx::{FxHashMap, FxHashSet};
use oxc_ast::ast::{BindingPattern, Expression, ObjectPropertyKind, Program, VariableDeclarator};
use oxc_ast_visit::{walk, Visit};

struct Candidate {
    binding: usize,
    sources: Vec<usize>,
    uses: Vec<Span>,
}

pub(in crate::codebase::postgres::sql_identifier_syntax) fn prepare(
    program: &Program<'_>,
    module: &TypeScriptModuleFacts,
    selections: &mut [Selection],
    syntax: &mut Syntax<'_>,
) {
    let mut ids = FxHashMap::default();
    for binding in &module.bindings {
        ids.insert((binding.span.start, binding.span.end), binding.id);
        for reference in &binding.references {
            ids.insert((reference.span.start, reference.span.end), binding.id);
        }
    }
    let params = functions::classify(program, module, &ids, selections);
    let mut visitor = Visitor {
        ids: &ids,
        selections,
        syntax,
        params: &params,
        candidates: Vec::new(),
        called: FxHashSet::default(),
        passed: Vec::new(),
        bindings: module
            .bindings
            .iter()
            .map(|binding| (binding.id, binding))
            .collect(),
    };
    visitor.visit_program(program);
    visitor.finish();
}

struct Visitor<'a, 'source> {
    ids: &'a FxHashMap<Span, usize>,
    selections: &'a mut [Selection],
    syntax: &'a mut Syntax<'source>,
    params: &'a Params,
    candidates: Vec<Candidate>,
    called: FxHashSet<usize>,
    passed: Vec<(usize, Vec<Option<usize>>)>,
    bindings: FxHashMap<usize, &'a crate::codebase::ts_source::module_facts::ModuleBinding>,
}

impl Visitor<'_, '_> {
    fn consider_const(&mut self, declarator: &VariableDeclarator<'_>) {
        let Some(init) = &declarator.init else {
            return;
        };
        if !is_choice(init, self.syntax) {
            return;
        }
        let BindingPattern::BindingIdentifier(binding) = &declarator.id else {
            return;
        };
        let Some(id) = self.ids.get(&span(binding.span)).copied() else {
            return;
        };
        let mut uses = Vec::new();
        executor_spans(init, self.syntax, &mut uses);
        let sources = uses
            .iter()
            .filter_map(|at| self.ids.get(at).copied())
            .collect::<Vec<_>>();
        if sources.is_empty() {
            return;
        }
        self.candidates.push(Candidate {
            binding: id,
            sources,
            uses,
        });
    }
}

include!("adopt.rs");

impl<'a> Visit<'a> for Visitor<'_, '_> {
    fn visit_variable_declaration(&mut self, value: &oxc_ast::ast::VariableDeclaration<'a>) {
        if value.kind.is_const() {
            for declarator in &value.declarations {
                self.consider_const(declarator);
            }
        }
        walk::walk_variable_declaration(self, value);
    }

    fn visit_call_expression(&mut self, call: &oxc_ast::ast::CallExpression<'a>) {
        if is_choice(&call.callee, self.syntax) {
            self.suppress_carried(call);
        }
        if let Some((root, parts)) = super::super::chain(&call.callee) {
            if parts.len() == 1 {
                if let Some(callee) = self.ids.get(&root).copied() {
                    self.called.insert(callee);
                    if self.direct_executor(callee) {
                        self.suppress_carried(call);
                    }
                    if let Some(indexes) = self.params.quiet.get(&callee) {
                        for index in indexes {
                            self.suppress_query(call, *index);
                        }
                    }
                    if let Some(slots) = self.params.typed.get(&callee) {
                        for (index, param) in slots {
                            self.suppress_choice(call, *index, *param);
                        }
                    }
                    let arguments = call
                        .arguments
                        .iter()
                        .map(|argument| {
                            let expression = argument.as_expression()?;
                            let Expression::Identifier(id) = unwrap_ts_wrappers(expression) else {
                                return None;
                            };
                            self.ids.get(&span(id.span)).copied()
                        })
                        .collect::<Vec<_>>();
                    self.passed.push((callee, arguments));
                }
            }
        }
        walk::walk_call_expression(self, call);
    }
}
