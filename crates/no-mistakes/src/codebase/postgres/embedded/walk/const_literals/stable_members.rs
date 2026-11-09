use super::{BindingState, ScopeVisitor};
use crate::codebase::ts_source::{static_property_key_name, unwrap_ts_wrappers};
use oxc_ast::ast::*;
use std::collections::{HashMap, HashSet};

type Fields = HashMap<String, u32>;

#[derive(Default)]
pub(super) struct StableMembers {
    frames: Vec<HashMap<String, Fields>>,
    poisoned: HashSet<u32>,
}

impl StableMembers {
    pub(super) fn push_frame(&mut self) {
        self.frames.push(HashMap::new());
    }
    pub(super) fn pop_frame(&mut self) {
        self.frames.pop();
    }
    fn fields(&self, scopes: &[HashMap<String, BindingState>], name: &str) -> Option<&Fields> {
        self.frames
            .iter()
            .zip(scopes)
            .rev()
            .find_map(|(frame, scope)| scope.contains_key(name).then(|| frame.get(name)))
            .flatten()
    }
    pub(super) fn has(&self, scopes: &[HashMap<String, BindingState>], name: &str) -> bool {
        self.fields(scopes, name).is_some()
    }
    pub(super) fn poison(&mut self, scopes: &[HashMap<String, BindingState>], name: &str) {
        if let Some(fields) = self.fields(scopes, name) {
            let identities: Vec<_> = fields.values().copied().collect();
            self.poisoned.extend(identities);
        }
    }
    pub(super) fn poisoned(&self, identity: u32) -> bool {
        self.poisoned.contains(&identity)
    }
}

impl ScopeVisitor<'_> {
    pub(super) fn note_stable_members(&mut self, init: &Expression<'_>, scope: usize, name: &str) {
        let Expression::ObjectExpression(object) = unwrap_ts_wrappers(init) else {
            return;
        };
        let mut fields = Fields::new();
        for property in &object.properties {
            let ObjectPropertyKind::ObjectProperty(property) = property else {
                return;
            };
            if property.computed || property.method || property.kind != PropertyKind::Init {
                return;
            }
            let Some(key) = static_property_key_name(&property.key) else {
                return;
            };
            if key == "__proto__" {
                return;
            }
            let Expression::Identifier(value) = unwrap_ts_wrappers(&property.value) else {
                return;
            };
            let Some(identity) = self
                .lookup(value.name.as_str())
                .and_then(|value| value.value_identity)
            else {
                return;
            };
            if !self.stable_primitives.contains(&(identity.get() - 1)) {
                return;
            }
            fields.insert(key.to_string(), property.span.start);
        }
        let members = &mut self.literal_objects.stable_members;
        if let Some(frame) = members.frames.get_mut(scope) {
            frame.insert(name.to_string(), fields);
        }
    }

    pub(in crate::codebase::postgres::embedded::walk) fn stable_member_identity(
        &self,
        member: &StaticMemberExpression<'_>,
        call: oxc_span::Span,
        tag: oxc_span::Span,
    ) -> Option<u32> {
        let Expression::Identifier(root) = unwrap_ts_wrappers(&member.object) else {
            return None;
        };
        let root_identity = self
            .lookup(root.name.as_str())
            .and_then(|binding| binding.value_identity)?;
        if let Some(parameter) = self.stable_parameter_fields.get(&(root_identity.get() - 1)) {
            if parameter.calls.iter().all(|effect| {
                *effect == tag || (effect.start <= call.start && effect.end >= call.end)
            }) {
                return parameter.fields.get(member.property.name.as_str()).copied();
            }
            return None;
        }
        let objects = &self.literal_objects;
        let name = root.name.as_str();
        let members = &objects.stable_members;
        let identity = *members
            .fields(&self.scopes, name)?
            .get(member.property.name.as_str())?;
        (!members.poisoned(identity)).then_some(identity)
    }
}
