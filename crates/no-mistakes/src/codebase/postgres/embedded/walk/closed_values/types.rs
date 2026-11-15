use super::*;

fn strings(ty: &TSType<'_>) -> Option<Vec<String>> {
    match ty {
        TSType::TSLiteralType(literal) => match &literal.literal {
            TSLiteral::StringLiteral(value) => {
                Some(vec![crate::ast::js_literal(value).into_owned()])
            }
            _ => None,
        },
        TSType::TSUnionType(union) => {
            let mut values = Vec::new();
            for ty in &union.types {
                values.extend(strings(ty)?);
            }
            (values.len() <= MAX_EMBEDDED_SQL_VARIANTS).then_some(values)
        }
        _ => None,
    }
}
impl ScopeVisitor<'_> {
    pub(in crate::codebase::postgres::embedded::walk) fn note_closed_parameter(
        &mut self,
        param: &FormalParameter<'_>,
    ) {
        let BindingPattern::BindingIdentifier(id) = &param.pattern else {
            return;
        };
        if param.optional
            || param.initializer.is_some()
            || !param.decorators.is_empty()
            || !self.stable_primitives.contains(&id.span.start)
        {
            return;
        }
        let Some(values) = param
            .type_annotation
            .as_ref()
            .and_then(|ty| strings(&ty.type_annotation))
        else {
            return;
        };
        let arms = values
            .into_iter()
            .enumerate()
            .map(|(i, value)| Arm {
                node: Node::Text(value),
                choices: vec![(id.span.start as u64 | (1 << 62), i as u32)],
                root: None,
                provenance: None,
                type_annotation: true,
            })
            .collect();
        if let Some(frame) = self.literal_objects.closed.frames.last_mut() {
            frame.insert(id.name.to_string(), arms);
        }
    }
}
