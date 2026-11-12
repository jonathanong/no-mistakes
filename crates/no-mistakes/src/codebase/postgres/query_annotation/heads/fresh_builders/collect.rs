use super::{Candidates, Context, Initial, Root, Span};
use crate::codebase::postgres::query_annotation::heads::{append, syntax, HeadProof};
use crate::codebase::ts_source::unwrap_ts_wrappers;
use oxc_ast::ast::*;

impl Candidates {
    pub fn declaration(&mut self, value: &VariableDeclaration<'_>, context: Context) {
        if context.function.is_none()
            || !matches!(
                value.kind,
                VariableDeclarationKind::Const | VariableDeclarationKind::Let
            )
        {
            return;
        }
        for declaration in &value.declarations {
            let BindingPattern::BindingIdentifier(id) = &declaration.id else {
                continue;
            };
            let Some(init) = declaration.init.as_ref() else {
                continue;
            };
            if let Some(root) = root(init) {
                self.declarations.insert(
                    (id.span.start, id.span.end),
                    Initial {
                        root,
                        span: (declaration.span.start, declaration.span.end),
                        context,
                    },
                );
            }
        }
    }
    pub fn assignment(&mut self, value: &AssignmentExpression<'_>, discarded: Option<Span>) {
        if value.operator != AssignmentOperator::Assign
            || discarded != Some((value.span.start, value.span.end))
        {
            return;
        }
        let AssignmentTarget::AssignmentTargetIdentifier(id) = &value.left else {
            return;
        };
        if let Some(root) = root(&value.right) {
            self.writes.insert((id.span.start, id.span.end), root);
        }
    }
}
fn root(expression: &Expression<'_>) -> Option<Root> {
    let mut pending = vec![expression];
    let mut tags = Vec::new();
    let mut remaining = 255usize;
    while let Some(mut expression) = pending.pop() {
        loop {
            expression = unwrap_ts_wrappers(expression);
            let Expression::CallExpression(call) = expression else {
                break;
            };
            expression = append::prefix_receiver(call)?;
        }
        match expression {
            Expression::ConditionalExpression(branch) => {
                remaining = remaining.checked_sub(1)?;
                pending.push(&branch.alternate);
                pending.push(&branch.consequent);
            }
            Expression::TaggedTemplateExpression(tagged) => {
                if syntax::head(expression).proof(false) != HeadProof::Literal {
                    return None;
                }
                let Expression::Identifier(tag) = unwrap_ts_wrappers(&tagged.tag) else {
                    return None;
                };
                tags.push((tag.span.start, tag.span.end));
            }
            _ => return None,
        }
    }
    (!tags.is_empty()).then_some(Root { tags })
}
