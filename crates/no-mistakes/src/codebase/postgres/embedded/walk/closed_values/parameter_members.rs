use super::*;

fn member_key(identity: u32) -> String {
    format!("\0parameter-member:{identity}")
}

fn alias_key(name: &str) -> String {
    format!("\0parameter-type:{name}")
}

fn strings(visitor: &ScopeVisitor<'_>, ty: &TSType<'_>, depth: u8) -> Option<Vec<String>> {
    let depth = depth.checked_sub(1)?;
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
                for value in strings(visitor, ty, depth)? {
                    if !values.contains(&value) {
                        values.push(value);
                    }
                    if values.len() > MAX_EMBEDDED_SQL_VARIANTS {
                        return None;
                    }
                }
            }
            (!values.is_empty()).then_some(values)
        }
        TSType::TSTypeReference(reference) if reference.type_arguments.is_none() => {
            let TSTypeName::IdentifierReference(name) = &reference.type_name else {
                return None;
            };
            let key = alias_key(name.name.as_str());
            let arms = visitor
                .literal_objects
                .closed
                .frames
                .iter()
                .rev()
                .find_map(|frame| frame.get(&key))?;
            if arms.is_empty() {
                return None;
            }
            arms.iter()
                .map(|arm| match &arm.node {
                    Node::Text(text) => Some(text.clone()),
                    _ => None,
                })
                .collect()
        }
        _ => None,
    }
}

fn arms(values: Vec<String>, identity: u32) -> Vec<Arm> {
    values
        .into_iter()
        .enumerate()
        .map(|(index, text)| Arm {
            node: Node::Text(text),
            choices: vec![(u64::from(identity) | (1 << 62), index as u32)],
            root: None,
            provenance: None,
            type_annotation: true,
        })
        .collect()
}

impl ScopeVisitor<'_> {
    pub(in crate::codebase::postgres::embedded::walk) fn shadow_parameter_type(
        &mut self,
        name: &str,
    ) {
        if let Some(frame) = self.literal_objects.closed.frames.last_mut() {
            frame.insert(alias_key(name), Vec::new());
        }
    }

    pub(in crate::codebase::postgres::embedded::walk) fn note_parameter_type_alias(
        &mut self,
        alias: &TSTypeAliasDeclaration<'_>,
    ) {
        let values = alias
            .type_parameters
            .is_none()
            .then(|| super::types::strings(&alias.type_annotation))
            .flatten();
        let values = values
            .map(|values| arms(values, alias.id.span.start))
            .unwrap_or_default();
        if let Some(frame) = self.literal_objects.closed.frames.last_mut() {
            frame
                .entry(alias_key(alias.id.name.as_str()))
                .and_modify(Vec::clear)
                .or_insert(values);
        }
    }

    pub(super) fn note_closed_parameter_members(&mut self, param: &FormalParameter<'_>) {
        let BindingPattern::BindingIdentifier(id) = &param.pattern else {
            return;
        };
        if param.optional || param.initializer.is_some() || !param.decorators.is_empty() {
            return;
        }
        let Some(stable) = self.stable_parameter_fields.get(&id.span.start) else {
            return;
        };
        let Some(TSType::TSTypeLiteral(literal)) = param
            .type_annotation
            .as_ref()
            .map(|annotation| &annotation.type_annotation)
        else {
            return;
        };
        let mut fields = Vec::new();
        for signature in &literal.members {
            let TSSignature::TSPropertySignature(property) = signature else {
                continue;
            };
            if property.computed || property.optional {
                continue;
            }
            let Some(key) = crate::codebase::ts_source::static_property_key_name(&property.key)
            else {
                continue;
            };
            let Some(identity) = stable.fields.get(key.as_ref()) else {
                continue;
            };
            let Some(values) = property
                .type_annotation
                .as_ref()
                .and_then(|annotation| strings(self, &annotation.type_annotation, 8))
            else {
                continue;
            };
            fields.push((member_key(*identity), arms(values, *identity)));
        }
        if let Some(frame) = self.literal_objects.closed.frames.last_mut() {
            frame.extend(fields);
        }
    }
}

pub(super) fn recover_member(
    visitor: &ScopeVisitor<'_>,
    member: &StaticMemberExpression<'_>,
    depth: u8,
) -> Option<Vec<Arm>> {
    if depth == 0 || member.optional {
        return None;
    }
    let Expression::Identifier(root) = unwrap_ts_wrappers(&member.object) else {
        return None;
    };
    let scope = visitor
        .scopes
        .iter()
        .rposition(|scope| scope.contains_key(root.name.as_str()))?;
    let binding = visitor.scopes.get(scope)?.get(root.name.as_str())?;
    let parameter = visitor
        .stable_parameter_fields
        .get(&(binding.value_identity?.get() - 1))?;
    let identity = parameter.fields.get(member.property.name.as_str())?;
    visitor
        .literal_objects
        .closed
        .frames
        .get(scope)?
        .get(&member_key(*identity))
        .cloned()
}
