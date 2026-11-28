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
            super::relation::indexed_array_strings(visitor, &indexed.object_type)
        }
        _ => None,
    }
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

impl ClosedValues {
    pub(in crate::codebase::postgres::embedded::walk) fn push(&mut self) {
        self.frames.push(FxHashMap::default());
        self.relation_frames.push(FxHashMap::default());
        self.const_literal_arrays.push(FxHashMap::default());
        self.type_frames.push(FxHashMap::default());
    }
    pub(in crate::codebase::postgres::embedded::walk) fn pop(&mut self) {
        self.frames.pop();
        self.relation_frames.pop();
        self.const_literal_arrays.pop();
        self.type_frames.pop();
    }
    pub(super) fn remove_at(&mut self, index: usize, name: &str) {
        if let Some(frame) = self.frames.get_mut(index) {
            frame.remove(name);
        }
        if let Some(frame) = self.relation_frames.get_mut(index) {
            frame.remove(name);
        }
        if let Some(frame) = self.const_literal_arrays.get_mut(index) {
            frame.remove(name);
        }
    }
    pub(in crate::codebase::postgres::embedded::walk) fn roots(
        &self,
        scopes: &[HashMap<String, super::super::BindingState>],
        name: &str,
    ) -> Vec<String> {
        scopes
            .iter()
            .rposition(|scope| scope.contains_key(name))
            .and_then(|i| self.frames.get(i))
            .and_then(|frame| frame.get(name))
            .into_iter()
            .flatten()
            .filter(|arm| !matches!(arm.node, Node::Text(_)))
            .filter_map(|arm| arm.root.clone())
            .collect()
    }
    pub(in crate::codebase::postgres::embedded::walk) fn has(
        &self,
        scopes: &[HashMap<String, super::super::BindingState>],
        name: &str,
    ) -> bool {
        scopes
            .iter()
            .enumerate()
            .rev()
            .find(|(_, scope)| scope.contains_key(name))
            .is_some_and(|(i, _)| {
                self.frames
                    .get(i)
                    .is_some_and(|frame| frame.contains_key(name))
            })
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
