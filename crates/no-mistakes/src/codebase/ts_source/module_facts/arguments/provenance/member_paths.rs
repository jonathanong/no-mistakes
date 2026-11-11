use super::super::{ArgumentText, PathOrigin, PathProvenance, Projector};
use super::argument_source::ArgumentSource;
use super::call_args::MAX_ARGUMENTS;
use oxc_ast::ast::*;
use oxc_span::GetSpan;

pub(super) fn collect<'a>(
    projector: &mut Projector<'a, '_>,
    call: &CallExpression<'a>,
    source: &ArgumentSource<'_>,
    out: &mut PathProvenance,
) -> (Vec<(usize, String, PathOrigin)>, Vec<ArgumentText>) {
    let mut members = Vec::new();
    let mut texts = Vec::new();
    for index in 0..MAX_ARGUMENTS {
        let Some(expression) = projector.argument_expression(call, index) else { break; };
        let (paths, unknown) = projector.object_shape(expression, 0);
        if !unknown { out.arg_closed_objects.entry(call.span.start).or_default().push(index); }
        for (key, origin) in paths {
            if let Expression::ObjectExpression(object) = expression.get_inner_expression() {
                if let Some(value) = object.properties.iter().rev().find_map(|property| {
                    let ObjectPropertyKind::ObjectProperty(property) = property else { return None; };
                    (!property.computed && crate::ast::property_key_name(&property.key).as_deref() == Some(&key)).then_some(&property.value)
                }) {
                    texts.push(super::call_args::argument_text(source, index, Some(key.clone()), value.span()));
                }
            }
            members.push((index, key, origin));
        }
    }
    members.sort_by(|a, b| (&a.0, &a.1).cmp(&(&b.0, &b.1)));
    texts.sort_by(|a, b| (&a.index, &a.property).cmp(&(&b.index, &b.property)));
    (members, texts)
}
