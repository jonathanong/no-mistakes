use crate::codebase::ts_source::unwrap_ts_wrappers;
use crate::fx::FxHashSet;
use oxc_ast::ast::{CallExpression, Expression, IdentifierReference};
use oxc_ast_visit::{walk, Visit};

pub(super) struct ImportUses<'a> {
    names: &'a FxHashSet<String>,
    allowed: FxHashSet<u32>,
    pub(super) invalid: FxHashSet<String>,
}

impl<'a> ImportUses<'a> {
    pub(super) fn new(names: &'a FxHashSet<String>) -> Self {
        Self {
            names,
            allowed: FxHashSet::default(),
            invalid: FxHashSet::default(),
        }
    }
}

impl<'a> Visit<'a> for ImportUses<'_> {
    fn visit_call_expression(&mut self, call: &CallExpression<'a>) {
        if !call.optional {
            match unwrap_ts_wrappers(&call.callee) {
                Expression::Identifier(id) if self.names.contains(id.name.as_str()) => {
                    self.allowed.insert(id.span.start);
                }
                Expression::StaticMemberExpression(member)
                    if !member.optional && member.property.name == "each" =>
                {
                    if let Expression::Identifier(id) = unwrap_ts_wrappers(&member.object) {
                        if self.names.contains(id.name.as_str()) {
                            self.allowed.insert(id.span.start);
                        }
                    }
                }
                _ => {}
            }
        }
        walk::walk_call_expression(self, call);
    }

    fn visit_identifier_reference(&mut self, id: &IdentifierReference<'a>) {
        if id.name == "eval" {
            self.invalid.extend(self.names.iter().cloned());
        }
        if self.names.contains(id.name.as_str()) && !self.allowed.contains(&id.span.start) {
            self.invalid.insert(id.name.to_string());
        }
    }
}
