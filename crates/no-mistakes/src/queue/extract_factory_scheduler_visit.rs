use super::{Collector, ScopeKind};
use oxc_ast::ast::{BindingPattern, ClassType, Expression, FunctionType};
use oxc_ast::ast_kind::AstKind;
use oxc_span::{GetSpan, Span};
use oxc_syntax::scope::ScopeId;
use std::cell::Cell;

pub(super) fn bind_pattern<'a>(
    collector: &mut Collector<'a>,
    pattern: &BindingPattern<'a>,
    scope_start: u32,
    scope_end: u32,
    visible_from: u32,
    resolve_at: u32,
    value: Option<&'a Expression<'a>>,
) {
    if let BindingPattern::BindingIdentifier(id) = pattern {
        collector.bindings.insert(super::ObjectBinding {
            name: id.name.as_str(),
            scope_start,
            scope_end,
            visible_from,
            resolve_at,
            value,
        });
        return;
    }
    let mut names = Vec::new();
    collect_names(pattern, &mut names);
    for name in names {
        collector.bindings.insert(super::ObjectBinding {
            name,
            scope_start,
            scope_end,
            visible_from,
            resolve_at: visible_from,
            value: None,
        });
    }
}

pub(super) fn collect_names<'a>(pattern: &BindingPattern<'a>, names: &mut Vec<&'a str>) {
    match pattern {
        BindingPattern::BindingIdentifier(id) => names.push(id.name.as_str()),
        BindingPattern::ObjectPattern(object) => {
            for property in &object.properties {
                collect_names(&property.value, names);
            }
            if let Some(rest) = &object.rest {
                collect_names(&rest.argument, names);
            }
        }
        BindingPattern::ArrayPattern(array) => {
            for element in array.elements.iter().flatten() {
                collect_names(element, names);
            }
            if let Some(rest) = &array.rest {
                collect_names(&rest.argument, names);
            }
        }
        BindingPattern::AssignmentPattern(assignment) => collect_names(&assignment.left, names),
    }
}

impl<'a> Collector<'a> {
    pub(in crate::queue) fn enter_node(&mut self, kind: AstKind<'a>) {
        match kind {
            AstKind::Program(program) => {
                #[cfg(all(test, any(nm_shard_all, nm_shard_0)))]
                {
                    self.bindings.program_visits += 1;
                }
                self.push_span(program.span, ScopeKind::Function);
            }
            AstKind::Function(func) => {
                if func.r#type == FunctionType::FunctionDeclaration {
                    if let Some(id) = &func.id {
                        self.mask(id.name.as_str(), self.current().start);
                    }
                }
                self.push_span(func.span, ScopeKind::Function);
                if func.r#type == FunctionType::FunctionExpression {
                    if let Some(id) = &func.id {
                        self.mask(id.name.as_str(), func.span.start);
                    }
                }
                self.mask_params(&func.params, func.span.start);
            }
            AstKind::ArrowFunctionExpression(arrow) => {
                self.push_span(arrow.span, ScopeKind::Arrow);
                self.mask_params(&arrow.params, arrow.span.start);
            }
            AstKind::BlockStatement(block) => self.push_span(block.span, ScopeKind::Block),
            AstKind::CatchClause(clause) => {
                self.push_span(clause.span, ScopeKind::Block);
                if let Some(param) = &clause.param {
                    self.mask_pattern(&param.pattern, clause.span.start);
                }
            }
            AstKind::ForStatement(stmt) => self.push_span(stmt.span, ScopeKind::Block),
            AstKind::ForInStatement(stmt) => self.push_span(stmt.span, ScopeKind::Block),
            AstKind::ForOfStatement(stmt) => self.push_span(stmt.span, ScopeKind::Block),
            AstKind::StaticBlock(block) => self.push_span(block.span, ScopeKind::Function),
            AstKind::SwitchStatement(stmt) => self.switches.push((
                &stmt.scope_id,
                Span::new(stmt.discriminant.span().end, stmt.span.end),
            )),
            AstKind::Class(class) if class.r#type == ClassType::ClassDeclaration => {
                if let Some(id) = &class.id {
                    self.mask(id.name.as_str(), self.current().start);
                }
            }
            AstKind::VariableDeclaration(decl) => {
                for declarator in &decl.declarations {
                    self.record_declarator(decl.kind, declarator);
                }
            }
            _ => {}
        }
    }

    pub(in crate::queue) fn enter_scope(&mut self, scope_id: &Cell<Option<ScopeId>>) {
        // The switch discriminant is visited before its case bindings enter scope.
        if let Some((switch_id, span)) = self.switches.last() {
            if std::ptr::eq(*switch_id, scope_id) {
                self.push_span(*span, ScopeKind::Block);
            }
        }
    }

    pub(in crate::queue) fn leave_node(&mut self, kind: AstKind<'a>) {
        if let AstKind::SwitchStatement(_) = kind {
            self.switches.pop();
        }
        if matches!(
            kind,
            AstKind::Program(_)
                | AstKind::Function(_)
                | AstKind::ArrowFunctionExpression(_)
                | AstKind::BlockStatement(_)
                | AstKind::CatchClause(_)
                | AstKind::ForStatement(_)
                | AstKind::ForInStatement(_)
                | AstKind::ForOfStatement(_)
                | AstKind::StaticBlock(_)
                | AstKind::SwitchStatement(_)
        ) {
            self.scopes.pop();
        }
    }

    fn push_span(&mut self, span: Span, kind: ScopeKind) {
        self.push_scope(span.start, span.end, kind);
    }
}
