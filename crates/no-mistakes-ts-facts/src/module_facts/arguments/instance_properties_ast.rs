use crate::fx::FxHashMap;
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
}

impl InstancePropertyEvidence {
    pub(super) fn opaque() -> Self {
        Self {
            properties: FxHashMap::default(),
            opaque_mutation: true,
        }
    }

    pub(super) fn is_empty(&self) -> bool {
        self.properties.is_empty() && !self.opaque_mutation
    }
}

pub(super) fn non_transparent_parent(
    semantic: &Semantic<'_>,
    start: NodeId,
) -> Option<(Span, NodeId)> {
    let mut child_span = semantic.nodes().kind(start).span();
    for ancestor in semantic.nodes().ancestors(start) {
        if matches!(
            ancestor.kind(),
            AstKind::ParenthesizedExpression(_)
                | AstKind::TSAsExpression(_)
                | AstKind::TSSatisfiesExpression(_)
                | AstKind::TSNonNullExpression(_)
        ) {
            child_span = ancestor.kind().span();
            continue;
        }
        return Some((child_span, ancestor.id()));
    }
    None
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
    let Some(target) = decl_block else {
        return false;
    };
    for ancestor in semantic.nodes().ancestors(use_id) {
        if ancestor.id() == target {
            return true;
        }
        if !matches!(
            ancestor.kind(),
            AstKind::ExpressionStatement(_)
                | AstKind::AssignmentExpression(_)
                | AstKind::ParenthesizedExpression(_)
                | AstKind::TSAsExpression(_)
                | AstKind::TSSatisfiesExpression(_)
                | AstKind::TSNonNullExpression(_)
        ) {
            return false;
        }
    }
    false
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
