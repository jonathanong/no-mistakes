use super::*;

pub(super) fn strings(
    visitor: &ScopeVisitor<'_>,
    ty: &TSType<'_>,
    depth: u8,
) -> Option<Vec<String>> {
    let depth = depth.checked_sub(1)?;
    match ty {
        TSType::TSParenthesizedType(inner) => strings(visitor, &inner.type_annotation, depth),
        TSType::TSLiteralType(literal) => match &literal.literal {
            TSLiteral::StringLiteral(value) => {
                Some(vec![crate::ast::js_literal(value).into_owned()])
            }
            _ => None,
        },
        TSType::TSUnionType(union) => {
            let mut values = Vec::new();
            for ty in &union.types {
                for value in strings(visitor, ty, depth)? {
                    if !values.contains(&value) {
                        values.push(value);
                    }
                    if values.len() > MAX_EMBEDDED_SQL_VARIANTS {
                        return None;
                    }
                }
            }
            Some(values)
        }
        TSType::TSTypeReference(reference) if reference.type_arguments.is_none() => {
            let TSTypeName::IdentifierReference(name) = &reference.type_name else {
                return None;
            };
            visitor
                .literal_objects
                .closed
                .type_frames
                .iter()
                .rev()
                .find_map(|frame| frame.get(name.name.as_str()))
                .filter(|values| !values.is_empty())
                .cloned()
        }
        TSType::TSIndexedAccessType(indexed)
            if matches!(indexed.index_type, TSType::TSNumberKeyword(_)) =>
        {
            indexed_array_strings(visitor, &indexed.object_type)
        }
        _ => None,
    }
}

fn indexed_array_strings(visitor: &ScopeVisitor<'_>, ty: &TSType<'_>) -> Option<Vec<String>> {
    let ty = match ty {
        TSType::TSParenthesizedType(inner) => &inner.type_annotation,
        _ => ty,
    };
    let TSType::TSTypeQuery(query) = ty else {
        return None;
    };
    let TSTypeQueryExprName::IdentifierReference(name) = &query.expr_name else {
        return None;
    };
    let arms = lookup(visitor, name.name.as_str())?;
    let mut values = Vec::new();
    for arm in arms {
        let Node::Array(items) = arm.node else {
            return None;
        };
        for item in items {
            let Node::Text(text) = item else {
                return None;
            };
            if !values.contains(&text) {
                values.push(text);
            }
            if values.len() > MAX_EMBEDDED_SQL_VARIANTS {
                return None;
            }
        }
    }
    (!values.is_empty()).then_some(values)
}

fn inline_strings(ty: &TSType<'_>) -> Option<Vec<String>> {
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
                values.extend(inline_strings(ty)?);
            }
            (values.len() <= MAX_EMBEDDED_SQL_VARIANTS).then_some(values)
        }
        _ => None,
    }
}

pub(in crate::codebase::postgres::embedded::walk) fn recover_relation(
    visitor: &ScopeVisitor<'_>,
    expr: &Expression<'_>,
) -> Option<Vec<Recovered>> {
    if let Some(values) = super::recover(visitor, expr) {
        return Some(values);
    }
    let Expression::Identifier(id) = unwrap_ts_wrappers(expr) else {
        return None;
    };
    let i = visitor
        .scopes
        .iter()
        .rposition(|scope| scope.contains_key(id.name.as_str()))?;
    let arms = visitor
        .literal_objects
        .closed
        .relation_frames
        .get(i)?
        .get(id.name.as_str())?
        .clone();
    super::materialize_arms(visitor, expr, arms)
}

impl ClosedValues {
    pub(in crate::codebase::postgres::embedded::walk) fn has_string_type_alias(
        &self,
        name: &str,
    ) -> bool {
        self.type_frames
            .iter()
            .rev()
            .find_map(|frame| frame.get(name))
            .is_some_and(|values| !values.is_empty())
    }
}

impl ScopeVisitor<'_> {
    pub(in crate::codebase::postgres::embedded::walk) fn note_closed_parameter(
        &mut self,
        param: &FormalParameter<'_>,
    ) {
        self.note_closed_parameter_members(param);
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
        let Some(annotation) = param.type_annotation.as_ref() else {
            return;
        };
        let (values, inline) = if let Some(values) = inline_strings(&annotation.type_annotation) {
            (values, true)
        } else if let Some(values) = strings(self, &annotation.type_annotation, 8) {
            (values, false)
        } else {
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
        let target = if inline {
            self.literal_objects.closed.frames.last_mut()
        } else {
            self.literal_objects.closed.relation_frames.last_mut()
        };
        if let Some(frame) = target {
            frame.insert(id.name.to_string(), arms);
        }
    }
}
