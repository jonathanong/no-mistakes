use super::*;
use crate::codebase::postgres::executor_detachment::{
    bind_method, executor_bindings, executor_receiver,
};
use oxc_ast::ast::BindingIdentifier;
mod assignments;
mod patterns;
mod references;
#[derive(Clone, Copy, PartialEq)]
pub(super) enum Kind {
    Receiver,
    Executor,
}
#[derive(Clone, Copy)]
pub(super) struct Value {
    pub kind: Kind,
    pub origin: usize,
    pub exact: bool,
}
#[derive(Clone, Copy)]
pub(super) enum Form {
    Copy,
    Query,
    Bind(Option<usize>, bool),
}
pub(super) struct Node {
    pub default_span: Option<Span>,
    pub profile: usize,
    pub id: usize,
    pub source: usize,
    pub form: Form,
    pub exact: bool,
    pub uses: Vec<Span>,
}
pub(super) struct Member {
    pub span: Span,
    pub root: Span,
    pub name: String,
}
pub(super) struct Invocation {
    pub span: Span,
    pub root: Span,
    pub parts: Vec<String>,
    pub exact: bool,
}
pub(super) struct Collector<'p> {
    pub direct_callees: FxHashSet<Span>,
    parameter: bool,
    pub profiles: &'p [EmbeddedSqlOptions],
    pub members: Vec<Member>,
    pub owners: FxHashMap<Span, usize>,
    bindings: FxHashMap<Span, usize>,
    pub nodes: Vec<Node>,
    pub node_ids: FxHashSet<usize>,
    pub calls: Vec<Invocation>,
}
impl<'p> Collector<'p> {
    pub fn new(module: &TypeScriptModuleFacts, profiles: &'p [EmbeddedSqlOptions]) -> Self {
        let mut owners = FxHashMap::default();
        let mut bindings = FxHashMap::default();
        for binding in &module.bindings {
            bindings.insert((binding.span.start, binding.span.end), binding.id);
            for usage in &binding.references {
                if usage.runtime {
                    owners.insert((usage.span.start, usage.span.end), binding.id);
                }
            }
        }
        Self {
            parameter: false,
            direct_callees: FxHashSet::default(),
            profiles,
            members: Vec::new(),
            owners,
            bindings,
            nodes: Vec::new(),
            node_ids: FxHashSet::default(),
            calls: Vec::new(),
        }
    }
    fn add(
        &mut self,
        alias: &BindingIdentifier<'_>,
        profile: usize,
        source: usize,
        form: Form,
        exact: bool,
        uses: Vec<Span>,
    ) {
        if let Some(id) = self.bindings.get(&span(alias.span)).copied() {
            self.nodes.push(Node {
                default_span: self.parameter.then_some(span(alias.span)),
                profile,
                id,
                source,
                form,
                exact,
                uses,
            });
            self.node_ids.insert(id);
        }
    }
}
impl<'a> Visit<'a> for Collector<'_> {
    fn visit_assignment_expression(&mut self, value: &oxc_ast::ast::AssignmentExpression<'a>) {
        assignments::collect(self, value);
        walk::walk_assignment_expression(self, value);
    }
    fn visit_static_member_expression(&mut self, value: &oxc_ast::ast::StaticMemberExpression<'a>) {
        references::member(
            self,
            span(value.span),
            &value.object,
            value.property.name.to_string(),
        );
        walk::walk_static_member_expression(self, value);
    }
    fn visit_computed_member_expression(
        &mut self,
        value: &oxc_ast::ast::ComputedMemberExpression<'a>,
    ) {
        if let Some(name) = crate::ast::computed_property_name(value) {
            references::member(self, span(value.span), &value.object, name.into_owned());
        }
        walk::walk_computed_member_expression(self, value);
    }
    fn visit_formal_parameter(&mut self, value: &oxc_ast::ast::FormalParameter<'a>) {
        if let Some(init) = &value.initializer {
            self.parameter = true;
            self.pattern(&value.pattern, init, false);
            self.parameter = false;
        }
        walk::walk_formal_parameter(self, value);
    }
    fn visit_call_expression(&mut self, call: &CallExpression<'a>) {
        self.direct_callees
            .insert(span(oxc_span::GetSpan::span(unwrap_ts_wrappers(
                &call.callee,
            ))));
        if let Some((root, parts)) = chain(&call.callee) {
            self.calls.push(Invocation {
                span: span(call.span),
                root,
                parts,
                exact: !call.optional
                    && call.arguments.iter().all(|argument| {
                        !matches!(argument, oxc_ast::ast::Argument::SpreadElement(_))
                    }),
            });
        }
        walk::walk_call_expression(self, call);
    }
    fn visit_variable_declaration(&mut self, declaration: &VariableDeclaration<'a>) {
        for item in &declaration.declarations {
            if let Some(init) = &item.init {
                self.pattern(&item.id, init, declaration.kind.is_const());
            }
        }
        walk::walk_variable_declaration(self, declaration);
    }
}
