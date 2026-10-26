use super::*;

impl Tracker {
    pub(super) fn record_reference(
        &mut self,
        id: &oxc_ast::ast::IdentifierReference<'_>,
        ids: &FxHashMap<Span, usize>,
    ) -> Node {
        let at = span(id.span);

        if id.name == "Object" && !ids.contains_key(&at) {
            self.globals.insert(at);
        }
        if id.name == "globalThis" && !ids.contains_key(&at) {
            self.global_escape = true;
        }
        ids.get(&at).map_or(Node::Scalar, |id| Node::Ref(*id, at))
    }
    pub(super) fn record_chain(
        &mut self,
        element: &oxc_ast::ast::ChainElement<'_>,
        ids: &FxHashMap<Span, usize>,
    ) -> Node {
        use oxc_ast::ast::ChainElement;
        // A chain element shares the wrapper span and has no expression callback.
        match element {
            ChainElement::StaticMemberExpression(value) => Node::Read(
                span(value.object.span()),
                Some(value.property.name.to_string()),
            ),
            ChainElement::ComputedMemberExpression(value) => {
                Node::Read(span(value.object.span()), static_key(&value.expression))
            }
            ChainElement::TSNonNullExpression(value) => Node::Copy(span(value.expression.span())),
            ChainElement::CallExpression(call) => self.record_call(call, ids),
            _ => Node::Scalar,
        }
    }
    pub(super) fn record_call(
        &mut self,
        call: &CallExpression<'_>,
        ids: &FxHashMap<Span, usize>,
    ) -> Node {
        self.calls.push((
            span(call.callee.span()),
            call.arguments
                .iter()
                .filter_map(|arg| arg.as_expression().map(|arg| span(arg.span())))
                .collect(),
        ));
        if let Some(object) = freeze_object(call, ids) {
            self.candidates.insert(object);
            Node::Freeze(span(call.arguments[0].as_expression().unwrap().span()))
        } else {
            match unwrap_ts_wrappers(&call.callee) {
                Expression::Identifier(id) => ids
                    .get(&span(id.span))
                    .copied()
                    .map_or(Node::Scalar, Node::Invoke),
                _ => Node::InvokeValue(span(call.callee.span())),
            }
        }
    }
}
