use super::{BindingState, ScopeVisitor};
use crate::codebase::postgres::embedded::{tags, unpublished_sql_text, EmbeddedSqlKind};
use crate::codebase::ts_source::{static_property_key_name, unwrap_ts_wrappers};
use oxc_ast::ast::{
    CallExpression, Expression, IdentifierReference, ObjectPropertyKind, PropertyKind,
    TemplateLiteral,
};
use std::cell::RefCell;
use std::collections::{HashMap, HashSet};

mod apply;

type Members = HashMap<String, String>;

pub(super) struct Splice {
    name: String,
    literal: String,
    fallback: String,
}

#[derive(Default)]
pub(super) struct LiteralObjects {
    frames: Vec<HashMap<String, Members>>,
    text_consts: Vec<HashSet<String>>,
    pub(in crate::codebase::postgres::embedded::walk) fragment_consts: Vec<HashSet<String>>,
    poisoned: HashSet<String>,
    literals: RefCell<Vec<Splice>>,
    pub(super) read_spans: Vec<u32>,
}

#[rustfmt::skip]
impl LiteralObjects {
    pub(super) fn push_frame(&mut self) {
        self.frames.push(HashMap::new());
        self.text_consts.push(HashSet::new());
        self.fragment_consts.push(HashSet::new());
    }
    pub(super) fn pop_frame(&mut self) {
        self.frames.pop();
        self.text_consts.pop();
        self.fragment_consts.pop();
    }
    fn object<'a>(&'a self, scopes: &'a [HashMap<String, BindingState>], name: &str) -> Option<&'a Members> {
        for (frame, scope) in self.frames.iter().zip(scopes).rev() {
            if scope.contains_key(name) { return frame.get(name); }
        }
        None
    }
    pub(super) fn is_text_const(&self, scopes: &[HashMap<String, BindingState>], name: &str) -> bool {
        for (names, scope) in self.text_consts.iter().zip(scopes).rev() {
            if scope.contains_key(name) { return names.contains(name); }
        }
        false
    }
    fn poison_id(&mut self, scopes: &[HashMap<String, BindingState>], name: &str) {
        if self.object(scopes, name).is_some() { self.poisoned.insert(name.to_string()); }
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
            Expression::StringLiteral(literal) => crate::ast::js_literal(&literal).into_owned(),
            Expression::TemplateLiteral(template) if template.expressions.is_empty() => unpublished_sql_text(&property.value)?,
            _ => return None,
        };
        members.insert(static_property_key_name(&property.key)?.to_string(), value);
    }
    Some(members)
}

#[rustfmt::skip]
impl ScopeVisitor<'_> {
    pub(super) fn note_const_object(&mut self, init: &Expression<'_>, scope_index: usize, name: &str) {
        let Some(members) = object_members(init) else { return };
        if let Some(frame) = self.literal_objects.frames.get_mut(scope_index) {
            frame.insert(name.to_string(), members);
        }
    }
    pub(super) fn note_text_const(&mut self, init: &Expression<'_>, scope_index: usize, name: &str) {
        let text = matches!(
            unwrap_ts_wrappers(init),
            Expression::StringLiteral(_) | Expression::TemplateLiteral(_)
        );
        if text {
            if let Some(names) = self.literal_objects.text_consts.get_mut(scope_index) {
                names.insert(name.to_string());
            }
        }
    }
    /// `export const` objects can be written by importers. Scalar consts stay.
    pub(super) fn poison_exported_const_objects(&mut self, decl: &oxc_ast::ast::ExportDeclaration<'_>) {
        let oxc_ast::ast::Declaration::VariableDeclaration(var) = &decl.declaration else { return };
        for declarator in &var.declarations {
            let oxc_ast::ast::BindingPattern::BindingIdentifier(ident) = &declarator.id else { continue };
            self.literal_objects.poison_id(&self.scopes, ident.name.as_str());
        }
    }
    pub(super) fn spliced_template(&self, template: &TemplateLiteral<'_>, depth: u8) -> Option<String> {
        if depth == 0 { return None; }
        let mut out = String::new();
        for (index, quasi) in template.quasis.iter().enumerate() {
            out.push_str(&tags::quasi_text(quasi, false));
            if let Some(expr) = template.expressions.get(index) {
                out.push_str(&self.spliced_piece(expr, depth, index as u32 + 1)?);
            }
        }
        Some(out)
    }
    fn spliced_piece(&self, expr: &Expression<'_>, depth: u8, placeholder: u32) -> Option<String> {
        match unwrap_ts_wrappers(expr) {
            Expression::StringLiteral(literal) => Some(crate::ast::js_literal(&literal).into_owned()),
            Expression::TemplateLiteral(template) if template.expressions.is_empty() => unpublished_sql_text(expr),
            Expression::TemplateLiteral(template) => self.spliced_template(template, depth - 1),
            Expression::Identifier(ident) => super::imported::constants::identifier_piece(self, ident),
            Expression::StaticMemberExpression(member) => {
                super::imported::constants::namespace::piece(self, member, false)
                    .or_else(|| self.spliced_member(member, placeholder))
            },
            _ => None,
        }
    }
    pub(super) fn spliced_member(&self, member: &oxc_ast::ast::StaticMemberExpression<'_>, placeholder: u32) -> Option<String> {
        let Expression::Identifier(ident) = unwrap_ts_wrappers(&member.object) else { return None };
        let name = ident.name.to_string();
        let literal = {
            let objects = &self.literal_objects;
            if objects.poisoned.contains(&name) { return None; }
            objects.object(&self.scopes, &name)?.get(member.property.name.as_str()).cloned()
        }?;
        let mut literals = self.literal_objects.literals.borrow_mut();
        let id = literals.len();
        literals.push(Splice { name, literal, fallback: format!("sql_placeholder_{placeholder}") });
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
        for call in &mut self.calls { apply::apply_call(&self.literal_objects, call); }
        for pending in &mut self.pending_calls { apply::apply_call(&self.literal_objects, &mut pending.call); }
        for fragment in &mut self.fragments {
            let mut unused_positions = Vec::new();
            let _ = apply::apply_optional(&self.literal_objects, &mut fragment.sql_text, &mut unused_positions, &mut fragment.recovered_placeholder_positions);
        }
        for group in &mut self.fragment_variants {
            for variant in group { apply::apply_variant(&self.literal_objects, variant); }
        }
    }
}
