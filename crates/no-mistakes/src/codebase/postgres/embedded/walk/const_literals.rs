use super::{BindingState, ScopeVisitor};
use crate::codebase::postgres::embedded::{
    tags, unpublished_sql_text, EmbeddedSqlCall, EmbeddedSqlKind,
};
use crate::codebase::ts_source::{static_property_key_name, unwrap_ts_wrappers};
use oxc_ast::ast::{
    CallExpression, Expression, IdentifierReference, ObjectPropertyKind, PropertyKind,
    TemplateLiteral,
};
use std::cell::RefCell;
use std::collections::{HashMap, HashSet};

type Members = HashMap<String, String>;

#[derive(Default)]
pub(super) struct LiteralObjects {
    frames: Vec<HashMap<String, Members>>,
    poisoned: HashSet<String>,
    literals: RefCell<Vec<(String, String)>>,
    pub(super) read_spans: Vec<u32>,
}

#[rustfmt::skip]
impl LiteralObjects {
    pub(super) fn push_frame(&mut self) { self.frames.push(HashMap::new()); }
    pub(super) fn pop_frame(&mut self) { self.frames.pop(); }
    fn object<'a>(&'a self, scopes: &'a [HashMap<String, BindingState>], name: &str) -> Option<&'a Members> {
        for (frame, scope) in self.frames.iter().zip(scopes).rev() {
            if scope.contains_key(name) { return frame.get(name); }
        }
        None
    }
    fn poison_id(&mut self, scopes: &[HashMap<String, BindingState>], name: &str) {
        if self.object(scopes, name).is_some() { self.poisoned.insert(name.to_string()); }
    }
    fn rewrite(&self, text: &str) -> Option<String> {
        if self.literals.borrow().is_empty() { return Some(text.to_string()); }
        let literals = self.literals.borrow();
        let mut out = String::new();
        let mut rest = text;
        while let Some(start) = rest.find('\0') {
            out.push_str(&rest[..start]);
            let after = &rest[start + '\0'.len_utf8()..];
            let end = after.find('\0')?;
            let (name, literal) = literals.get(after[..end].parse::<usize>().ok()?)?;
            if self.poisoned.contains(name) { return None; }
            out.push_str(literal);
            rest = &after[end + '\0'.len_utf8()..];
        }
        out.push_str(rest);
        Some(out)
    }
}

#[rustfmt::skip]
fn object_members(expr: &Expression<'_>) -> Option<Members> {
    let Expression::ObjectExpression(object) = unwrap_ts_wrappers(expr) else { return None };
    let mut members = Members::new();
    for property in &object.properties {
        let ObjectPropertyKind::ObjectProperty(property) = property else { return None };
        if property.computed || property.method || property.shorthand || property.kind != PropertyKind::Init {
            return None;
        }
        let value = match unwrap_ts_wrappers(&property.value) {
            Expression::StringLiteral(literal) => literal.value.to_string(),
            Expression::TemplateLiteral(template) if template.expressions.is_empty() => unpublished_sql_text(&property.value)?,
            _ => return None,
        };
        members.insert(static_property_key_name(&property.key)?.to_string(), value);
    }
    Some(members)
}

#[rustfmt::skip]
fn apply_call(objects: &LiteralObjects, call: &mut EmbeddedSqlCall) {
    let Some(text) = call.sql_text.clone().filter(|text| text.contains('\0')) else { return };
    match objects.rewrite(&text) {
        Some(sql) => {
            if sql.len() != text.len() { call.recovered_placeholder_positions.clear(); }
            call.sql_text = Some(sql);
            call.variants.retain(|variant| !variant.sql_text.contains('\0'));
        }
        None => {
            *call = EmbeddedSqlCall {
                line: call.line,
                callee: std::mem::take(&mut call.callee),
                kind: EmbeddedSqlKind::Dynamic,
                ..EmbeddedSqlCall::default()
            };
        }
    }
}

#[rustfmt::skip]
fn apply_fragment(objects: &LiteralObjects, sql: &mut Option<String>) {
    let Some(text) = sql.clone().filter(|text| text.contains('\0')) else { return };
    *sql = objects.rewrite(&text);
}

#[rustfmt::skip]
impl ScopeVisitor<'_> {
    pub(super) fn note_const_object(&mut self, init: &Expression<'_>, scope_index: usize, name: &str) {
        let Some(members) = object_members(init) else { return };
        if let Some(frame) = self.literal_objects.frames.get_mut(scope_index) {
            frame.insert(name.to_string(), members);
        }
    }
    pub(super) fn spliced_template(&self, template: &TemplateLiteral<'_>, depth: u8) -> Option<String> {
        if depth == 0 { return None; }
        let mut out = String::new();
        for (index, quasi) in template.quasis.iter().enumerate() {
            out.push_str(tags::quasi_text(quasi, false));
            if let Some(expr) = template.expressions.get(index) {
                out.push_str(&self.spliced_piece(expr, depth)?);
            }
        }
        Some(out)
    }
    fn spliced_piece(&self, expr: &Expression<'_>, depth: u8) -> Option<String> {
        match unwrap_ts_wrappers(expr) {
            Expression::StringLiteral(literal) => Some(literal.value.to_string()),
            Expression::TemplateLiteral(template) if template.expressions.is_empty() => unpublished_sql_text(expr),
            Expression::TemplateLiteral(template) => self.spliced_template(template, depth - 1),
            Expression::Identifier(ident) => {
                let binding = self.lookup(ident.name.as_str())?;
                (binding.kind == EmbeddedSqlKind::ImmutableLocal).then_some(binding.sql)?
            }
            Expression::StaticMemberExpression(member) => self.spliced_member(member),
            _ => None,
        }
    }
    pub(super) fn spliced_member(&self, member: &oxc_ast::ast::StaticMemberExpression<'_>) -> Option<String> {
        let Expression::Identifier(ident) = unwrap_ts_wrappers(&member.object) else { return None };
        let name = ident.name.to_string();
        let literal = {
            let objects = &self.literal_objects;
            if objects.poisoned.contains(&name) { return None; }
            objects.object(&self.scopes, &name)?.get(member.property.name.as_str()).cloned()
        }?;
        let mut literals = self.literal_objects.literals.borrow_mut();
        let id = literals.len();
        literals.push((name, literal));
        Some(format!("\u{0}{id}\u{0}"))
    }
    pub(super) fn classified_const_template(&self, expr: &Expression<'_>, is_const: bool) -> (Option<String>, EmbeddedSqlKind) {
        if is_const {
            if let Some(text) = super::resolve::compose::static_fragment(expr, self) {
                return tags::kind_for_const(text, true);
            }
        }
        (unpublished_sql_text(expr), EmbeddedSqlKind::Dynamic)
    }
    pub(super) fn exempt_literal_read(&mut self, object: &Expression<'_>) {
        let Expression::Identifier(ident) = unwrap_ts_wrappers(object) else { return };
        if self.literal_objects.object(&self.scopes, ident.name.as_str()).is_some() {
            self.literal_objects.read_spans.push(ident.span.start);
        }
    }
    pub(super) fn poison_literal_reference(&mut self, ident: &IdentifierReference<'_>) {
        if !self.literal_objects.read_spans.contains(&ident.span.start) {
            self.literal_objects.poison_id(&self.scopes, ident.name.as_str());
        }
    }
    pub(super) fn poison_literal_callee(&mut self, call: &CallExpression<'_>) {
        let object = match unwrap_ts_wrappers(&call.callee) {
            Expression::StaticMemberExpression(member) => &member.object,
            Expression::ComputedMemberExpression(member) => &member.object,
            _ => return,
        };
        if let Expression::Identifier(ident) = unwrap_ts_wrappers(object) {
            self.literal_objects.poison_id(&self.scopes, ident.name.as_str());
        }
    }
    pub(super) fn finish_const_splices(&mut self) {
        for call in &mut self.calls { apply_call(&self.literal_objects, call); }
        for pending in &mut self.pending_calls { apply_call(&self.literal_objects, &mut pending.call); }
        for fragment in &mut self.fragments { apply_fragment(&self.literal_objects, &mut fragment.sql_text); }
    }
}
