use super::super::MemberEffect;
use super::Projector;
use oxc_ast::{ast::*, AstKind};
use oxc_span::GetSpan;

impl<'a> Projector<'a, '_> {
    pub(in super::super) fn forwarded_member_effects(
        &mut self,
        expression: &Expression<'a>,
        depth: usize,
    ) -> Vec<MemberEffect> {
        let Expression::Identifier(id) = expression.get_inner_expression() else {
            return vec![MemberEffect::unknown()];
        };
        self.identifier_effects(id, depth, true)
    }

    pub(super) fn identifier_effects(
        &mut self,
        id: &IdentifierReference<'a>,
        depth: usize,
        follow_init: bool,
    ) -> Vec<MemberEffect> {
        if depth >= 16 || !self.take() {
            return vec![MemberEffect::unknown()];
        }
        let Some(symbol) = self.binding_symbol(id) else {
            return vec![MemberEffect::unknown()];
        };
        let semantic = self.semantic;
        let mut effects = Vec::new();
        for reference in semantic.scoping().get_resolved_references(symbol) {
            if !self.take() || reference.flags().is_write() {
                return vec![MemberEffect::unknown()];
            }
            if semantic.nodes().kind(reference.node_id()).span() == id.span {
                continue;
            }
            if let Some(effect) = self.member_effect(reference.node_id()) {
                effects.push(effect);
            }
        }
        if follow_init {
            if let Some(init) = self.const_init(id) {
                if !matches!(init.get_inner_expression(), Expression::ObjectExpression(_)) {
                    effects.extend(self.forwarded_member_effects(init, depth + 1));
                }
            }
        }
        effects.sort();
        effects.dedup();
        effects
    }

    fn member_effect(&mut self, mut node: oxc_semantic::NodeId) -> Option<MemberEffect> {
        let semantic = self.semantic;
        let mut path = Vec::new();
        loop {
            if !self.take() {
                return Some(MemberEffect::unknown());
            }
            let span = semantic.nodes().kind(node).span();
            let parent = semantic.nodes().parent_node(node);
            let key = match parent.kind() {
                AstKind::StaticMemberExpression(member) if member.object.span() == span => {
                    Some(member.property.name.to_string())
                }
                AstKind::ComputedMemberExpression(member) if member.object.span() == span => {
                    let Expression::StringLiteral(value) = member.expression.get_inner_expression()
                    else {
                        return Some(MemberEffect {
                            path,
                            escaped: false,
                        });
                    };
                    Some(crate::ast::js_literal(value).into_owned())
                }
                _ => None,
            };
            let Some(key) = key else {
                break;
            };
            path.push(key);
            node = parent.id();
        }
        let span = semantic.nodes().kind(node).span();
        let parent = semantic.nodes().parent_kind(node);
        let invoked = semantic
            .nodes()
            .ancestors(node)
            .find_map(|ancestor| match ancestor.kind() {
                AstKind::CallExpression(call) => Some(call.callee.span().contains_inclusive(span)),
                AstKind::NewExpression(call) => Some(call.callee.span().contains_inclusive(span)),
                AstKind::ParenthesizedExpression(_)
                | AstKind::TSAsExpression(_)
                | AstKind::TSTypeAssertion(_)
                | AstKind::TSNonNullExpression(_)
                | AstKind::TSSatisfiesExpression(_) => None,
                _ => Some(false),
            })
            .unwrap_or(false);
        if invoked {
            path.pop();
            return Some(MemberEffect {
                path,
                escaped: true,
            });
        }
        let pure = match parent {
            AstKind::UnaryExpression(unary) => matches!(
                unary.operator,
                UnaryOperator::LogicalNot | UnaryOperator::Typeof | UnaryOperator::Void
            ),
            AstKind::BinaryExpression(binary) => matches!(
                binary.operator,
                BinaryOperator::StrictEquality | BinaryOperator::StrictInequality
            ),
            AstKind::IfStatement(statement) => statement.test.span() == span,
            AstKind::ConditionalExpression(expression) => expression.test.span() == span,
            _ => false,
        };
        if pure {
            return None;
        }
        Some(MemberEffect {
            path,
            escaped: !super::member_mutations::member_assignment_target(semantic.nodes(), node),
        })
    }
}
