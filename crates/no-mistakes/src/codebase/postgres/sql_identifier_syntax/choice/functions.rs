use super::super::{span, Selection, Span};
use crate::codebase::ts_source::{
    module_facts::{ModuleBinding, TypeScriptModuleFacts},
    unwrap_ts_wrappers,
};
use crate::fx::{FxHashMap, FxHashSet};
use oxc_ast::ast::{BindingPattern, Expression, Function, Program};
use oxc_ast_visit::{walk, Visit};

pub(super) struct Params {
    pub quiet: FxHashMap<usize, Vec<usize>>,
    pub typed: FxHashMap<usize, Vec<(usize, usize)>>,
}

pub(super) fn classify(
    program: &Program<'_>,
    module: &TypeScriptModuleFacts,
    ids: &FxHashMap<Span, usize>,
    selections: &[Selection],
) -> Params {
    let mut params = Params {
        quiet: FxHashMap::default(),
        typed: FxHashMap::default(),
    };
    let mut scanner = Scanner {
        ids,
        selections,
        bindings: module
            .bindings
            .iter()
            .map(|binding| (binding.id, binding))
            .collect(),
        params: &mut params,
    };
    scanner.visit_program(program);
    params
}

struct Scanner<'a> {
    ids: &'a FxHashMap<Span, usize>,
    selections: &'a [Selection],
    bindings: FxHashMap<usize, &'a ModuleBinding>,
    params: &'a mut Params,
}

#[derive(Clone, Copy)]
enum Body<'a> {
    Function(&'a oxc_ast::ast::FunctionBody<'a>),
    Arrow(&'a oxc_ast::ast::ArrowFunctionExpression<'a>),
}

impl Scanner<'_> {
    fn note(
        &mut self,
        name: oxc_span::Span,
        parameters: &oxc_ast::ast::FormalParameters<'_>,
        start: u32,
        end: u32,
        body: Body<'_>,
    ) {
        let Some(function) = self.ids.get(&span(name)).copied() else {
            return;
        };
        for (index, parameter) in parameters.items.iter().enumerate() {
            let BindingPattern::BindingIdentifier(binding) = &parameter.pattern else {
                continue;
            };
            let Some(param) = self.ids.get(&span(binding.span)).copied() else {
                continue;
            };
            if !self
                .selections
                .iter()
                .any(|selection| selection.typed.contains(&param))
            {
                continue;
            }
            {
                let slots = self.params.typed.entry(function).or_default();
                if !slots.iter().any(|(slot, _)| *slot == index) {
                    slots.push((index, param));
                }
            }
            if self.idle(param, start, end, body) {
                let quiet = self.params.quiet.entry(function).or_default();
                if !quiet.contains(&index) {
                    quiet.push(index);
                }
            }
        }
    }

    fn idle(&self, param: usize, start: u32, end: u32, body: Body<'_>) -> bool {
        let Some(binding) = self.bindings.get(&param) else {
            return false;
        };
        let mut explained = FxHashSet::default();
        let mut explainer = Explainer {
            param,
            ids: self.ids,
            explained: &mut explained,
        };
        match body {
            Body::Function(body) => explainer.visit_function_body(body),
            Body::Arrow(arrow) => explainer.visit_arrow_function_expression(arrow),
        }
        binding
            .references
            .iter()
            .filter(|reference| reference.runtime)
            .all(|reference| {
                let at = (reference.span.start, reference.span.end);
                at.0 < start || at.1 > end || explained.contains(&at)
            })
    }
}

impl<'a> Visit<'a> for Scanner<'_> {
    fn visit_function(&mut self, function: &Function<'a>, flags: oxc_syntax::scope::ScopeFlags) {
        if let (Some(id), Some(body)) = (&function.id, &function.body) {
            let start = body.span.start;
            let end = body.span.end;
            self.note(id.span, &function.params, start, end, Body::Function(body));
        }
        walk::walk_function(self, function, flags);
    }

    fn visit_variable_declarator(&mut self, declarator: &oxc_ast::ast::VariableDeclarator<'a>) {
        if let (BindingPattern::BindingIdentifier(name), Some(init)) =
            (&declarator.id, &declarator.init)
        {
            match unwrap_ts_wrappers(init) {
                Expression::ArrowFunctionExpression(arrow) => {
                    let (start, end) = (arrow.span.start, arrow.span.end);
                    self.note(name.span, &arrow.params, start, end, Body::Arrow(arrow));
                }
                Expression::FunctionExpression(function) => {
                    if let Some(body) = &function.body {
                        let (start, end) = (body.span.start, body.span.end);
                        self.note(
                            name.span,
                            &function.params,
                            start,
                            end,
                            Body::Function(body),
                        );
                    }
                }
                _ => {}
            }
        }
        walk::walk_variable_declarator(self, declarator);
    }
}

struct Explainer<'a> {
    param: usize,
    ids: &'a FxHashMap<Span, usize>,
    explained: &'a mut FxHashSet<Span>,
}

impl<'a> Visit<'a> for Explainer<'a> {
    fn visit_static_member_expression(&mut self, value: &oxc_ast::ast::StaticMemberExpression<'a>) {
        if value.property.name != "query" {
            if let Expression::Identifier(id) = unwrap_ts_wrappers(&value.object) {
                let at = span(id.span);
                if self.ids.get(&at) == Some(&self.param) {
                    self.explained.insert(at);
                }
            }
        }
        walk::walk_static_member_expression(self, value);
    }
}
