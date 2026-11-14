use super::*;

impl ScopeVisitor<'_> {
    pub(in crate::codebase::postgres::embedded::walk) fn use_closed_static_member(
        &mut self,
        member: &StaticMemberExpression<'_>,
    ) {
        let key = Arm {
            node: Node::Text(member.property.name.to_string()),
            choices: Vec::new(),
            root: None,
            provenance: None,
        };
        self.use_closed_member(member.span, &member.object, Some(vec![key]));
    }
    pub(in crate::codebase::postgres::embedded::walk) fn use_closed_computed_member(
        &mut self,
        member: &ComputedMemberExpression<'_>,
    ) {
        let keys = expression(self, &member.expression, 8);
        self.use_closed_member(member.span, &member.object, keys);
    }
    fn use_closed_member(
        &mut self,
        span: oxc_span::Span,
        object: &Expression<'_>,
        keys: Option<Vec<Arm>>,
    ) {
        if self.head_calls_only {
            return;
        }
        let Some(source) = expression(self, object, 8) else {
            return;
        };
        let roots = source
            .iter()
            .filter(|arm| match &arm.node {
                Node::Text(_) => false,
                Node::Array(items) => items.iter().any(|item| !matches!(item, Node::Text(_))),
                Node::Object(_) => true,
            })
            .filter_map(|arm| arm.root.clone())
            .collect();
        let scalar = keys
            .and_then(|keys| super::expression::select(source, keys))
            .is_some_and(|arms| arms.iter().all(|arm| matches!(arm.node, Node::Text(_))));
        if scalar || self.literal_objects.member_uses.contains(&span) {
            self.literal_objects
                .member_uses
                .push(unwrap_ts_wrappers(object).span());
        } else {
            self.poison_closed_roots(roots);
        }
    }
}
