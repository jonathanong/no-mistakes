use crate::fx::{FxHashMap, FxHashSet};
use oxc_ast::{ast::*, AstKind};
use oxc_semantic::Semantic;
use oxc_span::{GetSpan, Span};
use oxc_syntax::node::NodeId;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InstancePropertyValue {
    Positive,
    Invalid,
    AbsentOrNonFinite,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct InstancePropertyEvidence {
    pub properties: FxHashMap<String, InstancePropertyValue>,
    pub opaque_mutation: bool,
    pub throw_count: u32,
    pub positive_through_throws: FxHashSet<String>,
}

impl InstancePropertyEvidence {
    pub(super) fn opaque() -> Self {
        Self {
            opaque_mutation: true,
            ..Self::default()
        }
    }

    pub(super) fn is_empty(&self) -> bool {
        self.properties.is_empty() && !self.opaque_mutation && self.throw_count == 0
    }

    pub(super) fn note_throw(&mut self) {
        let positive = self
            .properties
            .iter()
            .filter(|(_, value)| **value == InstancePropertyValue::Positive)
            .map(|(key, _)| key.clone())
            .collect::<FxHashSet<_>>();
        if self.throw_count == 0 {
            self.positive_through_throws = positive;
        } else {
            self.positive_through_throws
                .retain(|key| positive.contains(key));
        }
        self.throw_count += 1;
    }

    pub(super) fn invalidate_key(&mut self, key: Option<&str>) {
        self.opaque_mutation = true;
        match key {
            Some(key) => {
                self.properties.remove(key);
            }
            None => self.properties.clear(),
        }
    }

    pub(super) fn escape(&mut self) {
        self.opaque_mutation = true;
        self.properties.clear();
    }
}

pub(super) fn non_transparent_parent(semantic: &Semantic<'_>, start: NodeId) -> (Span, NodeId) {
    let mut child_span = semantic.nodes().kind(start).span();
    let parent_id = semantic
        .nodes()
        .ancestors(start)
        .find(|ancestor| {
            let transparent = matches!(
                ancestor.kind(),
                AstKind::ParenthesizedExpression(_)
                    | AstKind::TSAsExpression(_)
                    | AstKind::TSSatisfiesExpression(_)
                    | AstKind::TSNonNullExpression(_)
            );
            if transparent {
                child_span = ancestor.kind().span();
            }
            !transparent
        })
        .map_or(start, |a| a.id());
    (child_span, parent_id)
}

pub(super) fn is_unshadowed_object_assign(
    semantic: &Semantic<'_>,
    callee: &Expression<'_>,
) -> bool {
    let Expression::StaticMemberExpression(member) = callee.get_inner_expression() else {
        return false;
    };
    if member.property.name != "assign" {
        return false;
    }
    let Expression::Identifier(id) = member.object.get_inner_expression() else {
        return false;
    };
    id.name == "Object"
        && id
            .reference_id
            .get()
            .is_some_and(|r| semantic.scoping().get_reference(r).symbol_id().is_none())
}

pub(super) fn enclosing_block(semantic: &Semantic<'_>, node_id: NodeId) -> Option<NodeId> {
    semantic.nodes().ancestors(node_id).find_map(|a| {
        matches!(
            a.kind(),
            AstKind::BlockStatement(_) | AstKind::FunctionBody(_) | AstKind::Program(_)
        )
        .then_some(a.id())
    })
}

pub(super) fn same_unconditional_block(
    semantic: &Semantic<'_>,
    use_id: NodeId,
    decl_block: Option<NodeId>,
) -> bool {
    decl_block.is_some_and(|target| {
        semantic
            .nodes()
            .ancestors(use_id)
            .find(|a| {
                a.id() == target
                    || !matches!(
                        a.kind(),
                        AstKind::ExpressionStatement(_)
                            | AstKind::AssignmentExpression(_)
                            | AstKind::ParenthesizedExpression(_)
                            | AstKind::TSAsExpression(_)
                            | AstKind::TSSatisfiesExpression(_)
                            | AstKind::TSNonNullExpression(_)
                    )
            })
            .is_some_and(|a| a.id() == target)
    })
}

pub(super) fn static_string_expr<'a>(expr: &'a Expression<'a>) -> Option<&'a str> {
    match expr.get_inner_expression() {
        Expression::StringLiteral(lit) => lit.value.as_str(),
        Expression::TemplateLiteral(tpl) if tpl.expressions.is_empty() => tpl
            .quasis
            .first()
            .and_then(|q| q.value.cooked.as_ref().and_then(|s| s.as_str())),
        _ => None,
    }
}

pub(crate) fn is_call_derived_construct(semantic: &Semantic<'_>, callee: &Expression<'_>) -> bool {
    match callee.get_inner_expression() {
        Expression::CallExpression(_) => true,
        Expression::Identifier(id) => id
            .reference_id
            .get()
            .and_then(|r| semantic.scoping().get_reference(r).symbol_id())
            .is_some_and(|symbol| {
                let AstKind::VariableDeclarator(decl) = semantic
                    .nodes()
                    .kind(semantic.scoping().symbol_declaration(symbol))
                else {
                    return false;
                };
                decl.init.as_ref().is_some_and(|init| {
                    matches!(init.get_inner_expression(), Expression::CallExpression(_))
                })
            }),
        _ => false,
    }
}

pub(super) fn sorted_symbol_refs(
    semantic: &Semantic<'_>,
    symbol: oxc_syntax::symbol::SymbolId,
) -> Vec<(NodeId, bool)> {
    let mut refs: Vec<(u32, NodeId, bool)> = semantic
        .scoping()
        .get_resolved_references(symbol)
        .map(|r| {
            let id = r.node_id();
            (
                semantic.nodes().kind(id).span().start,
                id,
                r.flags().is_write(),
            )
        })
        .collect();
    refs.sort_by_key(|(start, _, _)| *start);
    refs.into_iter().map(|(_, id, w)| (id, w)).collect()
}
