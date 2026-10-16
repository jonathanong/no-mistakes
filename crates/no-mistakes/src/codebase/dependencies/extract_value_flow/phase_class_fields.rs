use super::*;
use oxc_ast_visit::Visit;

pub(super) fn property<'a>(
    walk: &mut phase_walk::PhaseWalk<'_>,
    property: &oxc_ast::ast::PropertyDefinition<'a>,
) {
    if property.r#static || walk.collect_reference_effects {
        oxc_ast_visit::walk::walk_property_definition(walk, property);
    } else {
        oxc_ast_visit::walk::walk_decorators(walk, &property.decorators);
        walk.visit_property_key(&property.key);
        if let Some(annotation) = &property.type_annotation {
            walk.visit_ts_type_annotation(annotation);
        }
    }
}
pub(super) fn accessor<'a>(
    walk: &mut phase_walk::PhaseWalk<'_>,
    property: &oxc_ast::ast::AccessorProperty<'a>,
) {
    if property.r#static || walk.collect_reference_effects {
        oxc_ast_visit::walk::walk_accessor_property(walk, property);
    } else {
        oxc_ast_visit::walk::walk_decorators(walk, &property.decorators);
        walk.visit_property_key(&property.key);
        if let Some(annotation) = &property.type_annotation {
            walk.visit_ts_type_annotation(annotation);
        }
    }
}

pub(super) fn allow_mutable(walk: &phase_walk::PhaseWalk<'_>, expression: &Expression<'_>) -> bool {
    let Expression::StaticMemberExpression(member) = expression else {
        return true;
    };
    let Expression::Identifier(root) =
        crate::codebase::ts_source::unwrap_ts_wrappers(&member.object)
    else {
        return true;
    };
    !walk.changed_roots.as_ref().is_some_and(|names| {
        let changed = names.contains(root.name.as_str());
        #[cfg(all(test, feature = "test-instrumentation"))]
        {
            phase_changed_roots_tests::record(3);
            if changed {
                phase_changed_roots_tests::record(8);
            }
        }
        changed
    }) && walk.collector.is_none_or(|collector| {
        collector.mutable_root_unchanged(root.name.as_str(), member.span.end)
    })
}
