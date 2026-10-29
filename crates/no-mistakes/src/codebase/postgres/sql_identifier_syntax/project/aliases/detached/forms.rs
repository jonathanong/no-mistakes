use super::*;
use crate::codebase::postgres::executor_detachment::{bind_method, query_bindings, query_receiver};
use oxc_ast::ast::BindingIdentifier;
mod assignments;
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
    pub id: usize,
    pub source: usize,
    pub form: Form,
    pub exact: bool,
    pub uses: Vec<Span>,
}
pub(super) struct Invocation {
    pub span: Span,
    pub root: Span,
    pub parts: Vec<String>,
    pub exact: bool,
}
pub(super) struct Collector {
    pub owners: FxHashMap<Span, usize>,
    bindings: FxHashMap<Span, usize>,
    pub nodes: Vec<Node>,
    pub node_ids: FxHashSet<usize>,
    pub calls: Vec<Invocation>,
}
impl Collector {
    pub fn new(module: &TypeScriptModuleFacts) -> Self {
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
            owners,
            bindings,
            nodes: Vec::new(),
            node_ids: FxHashSet::default(),
            calls: Vec::new(),
        }
    }
    fn form(&self, expression: &Expression<'_>) -> Option<(usize, Form, bool, Vec<Span>)> {
        let receiver = query_receiver(expression);
        if let Some((root, parts)) = chain(receiver.unwrap_or(expression)) {
            if parts.len() != 1 {
                return None;
            }
            let form = if receiver.is_some() {
                Form::Query
            } else {
                Form::Copy
            };
            return Some((*self.owners.get(&root)?, form, true, vec![root]));
        }
        let Expression::CallExpression(call) = unwrap_ts_wrappers(expression) else {
            return None;
        };
        let method = bind_method(call)?;
        let receiver = query_receiver(method);
        let (root, parts) = chain(receiver.unwrap_or(method))?;
        if parts.len() != 1 {
            return None;
        }
        let query = receiver.is_some();
        let mut uses = vec![root];
        let receiver = call
            .arguments
            .first()
            .and_then(|argument| argument.as_expression())
            .and_then(chain)
            .filter(|(_, parts)| parts.len() == 1)
            .and_then(|(at, _)| {
                uses.push(at);
                self.owners.get(&at).copied()
            });
        Some((
            *self.owners.get(&root)?,
            Form::Bind(receiver, query),
            call.arguments.len() == 1 && !call.optional,
            uses,
        ))
    }
    fn add(
        &mut self,
        alias: &BindingIdentifier<'_>,
        source: usize,
        form: Form,
        exact: bool,
        uses: Vec<Span>,
    ) {
        if let Some(id) = self.bindings.get(&span(alias.span)).copied() {
            self.nodes.push(Node {
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
impl<'a> Visit<'a> for Collector {
    fn visit_assignment_expression(&mut self, value: &oxc_ast::ast::AssignmentExpression<'a>) {
        assignments::collect(self, value);
        walk::walk_assignment_expression(self, value);
    }
    fn visit_call_expression(&mut self, call: &CallExpression<'a>) {
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
            let Some((source, form, exact, uses)) =
                item.init.as_ref().and_then(|init| self.form(init))
            else {
                continue;
            };
            match &item.id {
                BindingPattern::BindingIdentifier(alias) => self.add(
                    alias,
                    source,
                    form,
                    exact && declaration.kind.is_const(),
                    uses,
                ),
                BindingPattern::ObjectPattern(_) if matches!(form, Form::Copy) => {
                    for (alias, plain) in query_bindings(&item.id) {
                        self.add(
                            alias,
                            source,
                            Form::Query,
                            exact && declaration.kind.is_const() && plain,
                            uses.clone(),
                        );
                    }
                }
                _ => {}
            }
        }
        walk::walk_variable_declaration(self, declaration);
    }
}
