use super::*;

fn member_key(identity: u32) -> String {
    format!("\0parameter-member:{identity}")
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
        self.literal_objects
            .closed
            .type_frames
            .last_mut()
            .expect("closed type scope")
            .insert(name.to_string(), Vec::new());
        self.literal_objects
            .closed
            .relation_type_frames
            .last_mut()
            .expect("closed relation type scope")
            .insert(name.to_string(), Vec::new());
    }

    pub(in crate::codebase::postgres::embedded::walk) fn note_parameter_type_alias(
        &mut self,
        alias: &TSTypeAliasDeclaration<'_>,
    ) {
        let inline = alias
            .type_parameters
            .is_none()
            .then(|| super::types::inline_strings(&alias.type_annotation))
            .flatten()
            .unwrap_or_default();
        let relation = alias
            .type_parameters
            .is_none()
            .then(|| super::types::strings(self, &alias.type_annotation, 8, true))
            .flatten()
            .unwrap_or_default();
        self.literal_objects
            .closed
            .type_frames
            .last_mut()
            .expect("closed type scope")
            .entry(alias.id.name.to_string())
            .and_modify(Vec::clear)
            .or_insert(inline);
        self.literal_objects
            .closed
            .relation_type_frames
            .last_mut()
            .expect("closed relation type scope")
            .entry(alias.id.name.to_string())
            .and_modify(Vec::clear)
            .or_insert(relation);
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
            let Some(values) = property.type_annotation.as_ref().and_then(|annotation| {
                super::types::strings(self, &annotation.type_annotation, 8, false)
            }) else {
                continue;
            };
            fields.push((member_key(*identity), arms(values, *identity)));
        }
        self.literal_objects
            .closed
            .frames
            .last_mut()
            .expect("closed value scope")
            .extend(fields);
    }
}

pub(super) fn recover_member(
    visitor: &ScopeVisitor<'_>,
    member: &StaticMemberExpression<'_>,
) -> Option<Vec<Arm>> {
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
    if parameter
        .suspensions
        .iter()
        .any(|span| span.end <= member.span.start)
    {
        return None;
    }
    let identity = parameter.fields.get(member.property.name.as_str())?;
    visitor
        .literal_objects
        .closed
        .frames
        .get(scope)?
        .get(&member_key(*identity))
        .cloned()
}
