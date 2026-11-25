use super::{CallableId, ImportCollector};
use crate::fx::{FxHashMap, FxHashSet};
use oxc_ast::ast::{
    BindingPattern, CallExpression, Expression, IdentifierReference, Statement,
    VariableDeclarationKind,
};
use oxc_span::GetSpan;

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ScopedHandles {
    pub(super) enabled: bool,
    pub starts: FxHashMap<(u32, u32), ScopedHandle>,
    bindings: FxHashMap<usize, FxHashMap<String, (u32, u32)>>,
    pub(super) receiver: Option<u32>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScopedHandle {
    pub owner: Option<CallableId>,
    pub scope_end: u32,
    pub after_begin: u32,
    pub using: bool,
    pub uncertain: bool,
    pub close: Option<(u32, u32)>,
    pub method_calls: FxHashSet<(u32, u32)>,
}

impl ScopedHandles {
    pub(super) fn leave_group(&mut self, scope: usize) {
        if !self.bindings.is_empty() {
            self.bindings.remove(&scope);
        }
    }
}

impl ImportCollector {
    pub(super) fn register_scoped_handles(&mut self, statements: &[Statement<'_>]) {
        if !self.value_flow.scoped_handles.enabled || self.predeclared_imported_bindings.is_empty()
        {
            return;
        }
        let Some(last) = statements.last() else {
            return;
        };
        let scope = self.current_lexical_scope_id();
        let owner = self.current_function_id();
        for statement in statements {
            let Statement::VariableDeclaration(decl) = statement else {
                continue;
            };
            if !matches!(
                decl.kind,
                VariableDeclarationKind::Const
                    | VariableDeclarationKind::Let
                    | VariableDeclarationKind::AwaitUsing
            ) {
                continue;
            }
            for binding in &decl.declarations {
                let BindingPattern::BindingIdentifier(id) = &binding.id else {
                    continue;
                };
                let Some((call, _)) = binding.init.as_ref().and_then(direct_call) else {
                    continue;
                };
                if decl.declarations.len() != 1
                    || call.optional
                    || !import_root(&call.callee)
                        .is_some_and(|name| self.predeclared_imported_bindings.contains(name))
                {
                    continue;
                }
                let call_span = (call.span.start, call.span.end);
                self.value_flow.scoped_handles.starts.insert(
                    call_span,
                    ScopedHandle {
                        owner,
                        scope_end: last.span().end,
                        after_begin: binding.span.end,
                        using: decl.kind == VariableDeclarationKind::AwaitUsing,
                        uncertain: scope == 0 && self.exported_functions.contains(id.name.as_str()),
                        close: None,
                        method_calls: FxHashSet::default(),
                    },
                );
                self.value_flow
                    .scoped_handles
                    .bindings
                    .entry(scope)
                    .or_default()
                    .insert(id.name.to_string(), call_span);
            }
        }
        let handles = &mut self.value_flow.scoped_handles;
        let Some(bindings) = handles.bindings.get(&scope) else {
            return;
        };
        for statement in statements {
            let Statement::ExpressionStatement(statement) = statement else {
                continue;
            };
            let Some((call, true)) = direct_call(&statement.expression) else {
                continue;
            };
            let Some((id, true)) = static_method(call) else {
                continue;
            };
            let Some(handle) = bindings
                .get(id.name.as_str())
                .and_then(|offset| handles.starts.get_mut(offset))
            else {
                continue;
            };
            if call.span.start >= handle.after_begin && !call.optional && handle.close.is_none() {
                handle.close = Some((call.span.start, call.span.end));
            }
        }
    }

    pub(super) fn scoped_handle_receiver(&mut self, call: &CallExpression<'_>) -> Option<u32> {
        if self.value_flow.scoped_handles.bindings.is_empty() {
            return None;
        }
        let (id, closes) = static_method(call)?;
        let owner = self.current_function_id();
        let handle = self.scoped_handle(id)?;
        let admitted_close = handle
            .close
            .is_some_and(|(start, end)| start == call.span.start || end <= call.span.start);
        if handle.owner != owner || (closes && !admitted_close) {
            handle.uncertain = true;
        }
        handle.method_calls.insert((call.span.start, call.span.end));
        Some(id.span.start)
    }

    pub(super) fn note_scoped_handle_use(&mut self, id: &IdentifierReference<'_>) {
        let facts = &self.value_flow.scoped_handles;
        if facts.bindings.is_empty() || facts.receiver == Some(id.span.start) {
            return;
        }
        let owner = self.current_function_id();
        if let Some(handle) = self.scoped_handle(id) {
            handle.uncertain |=
                handle.owner != owner || handle.close.is_none_or(|(_, end)| id.span.start < end);
        }
    }

    fn scoped_handle(&mut self, id: &IdentifierReference<'_>) -> Option<&mut ScopedHandle> {
        let scope = self.callee_binding_scope(id.name.as_str())?;
        let handles = &mut self.value_flow.scoped_handles;
        let offset = handles.bindings.get(&scope)?.get(id.name.as_str())?;
        handles.starts.get_mut(offset)
    }
}

fn direct_call<'ast, 'src>(
    expr: &'ast Expression<'src>,
) -> Option<(&'ast CallExpression<'src>, bool)> {
    let inner = expr.get_inner_expression();
    match inner {
        Expression::AwaitExpression(awaited) => match awaited.argument.get_inner_expression() {
            Expression::CallExpression(call) => Some((call, true)),
            _ => None,
        },
        Expression::CallExpression(call) => Some((call, false)),
        _ => None,
    }
}

fn static_method<'ast, 'src>(
    call: &'ast CallExpression<'src>,
) -> Option<(&'ast IdentifierReference<'src>, bool)> {
    let Expression::StaticMemberExpression(member) = call.callee.get_inner_expression() else {
        return None;
    };
    let Expression::Identifier(id) = member.object.get_inner_expression() else {
        return None;
    };
    if member.optional || call.optional {
        return None;
    }
    Some((
        id,
        matches!(member.property.name.as_str(), "commit" | "rollback"),
    ))
}

fn import_root<'a>(mut expr: &'a Expression<'_>) -> Option<&'a str> {
    loop {
        match expr.get_inner_expression() {
            Expression::Identifier(id) => return Some(id.name.as_str()),
            Expression::StaticMemberExpression(member) => expr = &member.object,
            Expression::ComputedMemberExpression(member) => expr = &member.object,
            _ => return None,
        }
    }
}
