use super::ScopeVisitor;
use oxc_ast::ast::VariableDeclarator;

pub(in crate::codebase::postgres::embedded::walk) fn bind_query_property(
    declarator: &VariableDeclarator<'_>,
    is_const: bool,
    visitor: &mut ScopeVisitor<'_>,
) {
    let Some(init) = &declarator.init else {
        return;
    };
    if !visitor.detached_query_receiver(init) {
        return;
    }
    for (local, exact) in
        crate::codebase::postgres::executor_detachment::query_bindings(&declarator.id)
    {
        if is_const && exact && visitor.detached_receiver_proven(init) {
            visitor.note_query_capture(local.name.as_str(), init);
        } else {
            visitor.note_query_capture_with_certainty(local.name.as_str(), init, false);
        }
    }
}
