use super::super::ScopeVisitor;
use super::{combine, Recovered};
use crate::codebase::ts_source::unwrap_ts_wrappers;
use oxc_ast::ast::{
    AssignmentExpression, AssignmentOperator, AssignmentTarget, CallExpression, Expression,
    VariableDeclarator,
};

impl ScopeVisitor<'_> {
    pub(in crate::codebase::postgres::embedded::walk) fn set_variants(
        &mut self,
        name: &str,
        variants: Option<Vec<Recovered>>,
    ) {
        for scope in self.scopes.iter_mut().rev() {
            if let Some(binding) = scope.get_mut(name) {
                binding.variants = variants;
                return;
            }
        }
    }
    pub(in crate::codebase::postgres::embedded::walk) fn initialize_variants(
        &mut self,
        declaration: &VariableDeclarator<'_>,
        variants: Option<Vec<Recovered>>,
    ) {
        // Stored typed-helper results alias a mutable caller builder. Its
        // current SQL cannot become a snapshot for a later execution.
        let variants = if declaration
            .init
            .as_ref()
            .is_some_and(|init| super::helper_alias::contains(self, init))
        {
            None
        } else {
            variants
        };
        if let oxc_ast::ast::BindingPattern::BindingIdentifier(id) = &declaration.id {
            self.set_variants(id.name.as_str(), variants);
        }
    }
    pub(in crate::codebase::postgres::embedded::walk) fn assigned_variants(
        &self,
        assignment: &AssignmentExpression<'_>,
    ) -> Option<Vec<Recovered>> {
        let AssignmentTarget::AssignmentTargetIdentifier(id) = &assignment.left else {
            return None;
        };
        if self.append_crosses_function(id.name.as_str())
            || super::helper_alias::contains(self, &assignment.right)
        {
            return None;
        }
        let right = self.recover_variants(&assignment.right)?;
        match assignment.operator {
            AssignmentOperator::Assign => Some(right),
            AssignmentOperator::Addition => {
                combine(self.lookup(id.name.as_str())?.variants?, right)
            }
            _ => None,
        }
    }
    pub(in crate::codebase::postgres::embedded::walk) fn appended_variants(
        &self,
        call: &CallExpression<'_>,
    ) -> Option<(String, Option<Vec<Recovered>>)> {
        let Expression::StaticMemberExpression(member) = unwrap_ts_wrappers(&call.callee) else {
            return None;
        };
        if member.property.name != "append" {
            return None;
        }
        let Expression::Identifier(id) = unwrap_ts_wrappers(&member.object) else {
            return None;
        };
        let variants = (|| {
            if self.append_crosses_function(id.name.as_str()) {
                return None;
            }
            let prefix = self.lookup(id.name.as_str())?.variants?;
            if !prefix.iter().all(|value| value.fragment) {
                return None;
            }
            let suffix = self.recover_variants(super::super::super::first_call_argument(call)?)?;
            combine(prefix, suffix)
        })();
        Some((id.name.to_string(), variants))
    }
}
