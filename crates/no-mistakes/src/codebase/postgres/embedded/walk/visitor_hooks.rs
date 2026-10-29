use super::{imported, resolve, ScopeVisitor};
use oxc_ast::ast;
use oxc_ast_visit::walk;

impl<'a> ScopeVisitor<'a> {
    pub(super) fn walk_runtime_export(&mut self, declaration: &ast::ExportNamedDeclaration<'a>) {
        if !declaration.export_kind.is_type() {
            for specifier in &declaration.specifiers {
                if !specifier.export_kind.is_type() {
                    walk::walk_export_specifier(self, specifier);
                }
            }
        }
    }

    pub(super) fn walk_executor_identifier(&mut self, id: &ast::IdentifierReference<'a>) {
        self.note_executor_identifier(id);
        self.poison_literal_reference(id);
        self.escape_builder_reference(id);
    }
    pub(super) fn walk_executor_expression(&mut self, expression: &ast::Expression<'a>) {
        self.note_executor_member(expression);
        self.revoke_aggregate_values(expression);
        walk::walk_expression(self, expression);
    }
    pub(super) fn walk_computed_member(&mut self, member: &ast::ComputedMemberExpression<'a>) {
        self.with_builder_receiver(&member.object, matches!(crate::codebase::ts_source::unwrap_ts_wrappers(&member.expression), ast::Expression::StringLiteral(name) if name.value == "text"), |visitor| {
            walk::walk_computed_member_expression(visitor, member)
        });
    }

    pub(super) fn walk_program_scope(&mut self, program: &ast::Program<'a>) {
        self.push_scope();
        resolve::hoist_vars(&program.body, self);
        resolve::record_statements(&program.body, self);
        walk::walk_program(self, program);
        if !self.head_calls_only {
            imported::constants::snapshot_exports(self, program);
        }
        self.pop_scope();
    }
    pub(super) fn walk_literal_member(&mut self, member: &ast::StaticMemberExpression<'a>) {
        let reads = self.literal_objects.read_spans.len();
        self.exempt_literal_read(&member.object);
        self.with_builder_receiver(&member.object, member.property.name == "text", |visitor| {
            walk::walk_static_member_expression(visitor, member)
        });
        self.literal_objects.read_spans.truncate(reads);
    }
    pub(super) fn walk_update_binding(&mut self, expression: &oxc_ast::ast::UpdateExpression<'a>) {
        if let oxc_ast::ast::SimpleAssignmentTarget::AssignmentTargetIdentifier(id) =
            &expression.argument
        {
            self.mark_dynamic(id.name.as_str());
        }
        walk::walk_update_expression(self, expression);
    }
    pub(super) fn walk_for_of_scope(&mut self, statement: &oxc_ast::ast::ForOfStatement<'a>) {
        self.push_scope();
        resolve::bind_for_statement_left(&statement.left, self);
        self.with_append_loop(statement.span.start, |visitor| {
            walk::walk_for_of_statement(visitor, statement)
        });
        self.pop_scope();
    }
}
