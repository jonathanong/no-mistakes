use super::*;
pub(super) fn lower_pattern(
    tracker: &mut Tracker,
    pattern: &BindingPattern<'_>,
    source: Span,
    path: Vec<Option<String>>,
    ids: &FxHashMap<Span, usize>,
) {
    match pattern {
        BindingPattern::BindingIdentifier(id) => {
            if let Some(id) = ids.get(&span(id.span)) {
                tracker.flows.push((*id, source, path));
            }
        }
        BindingPattern::ObjectPattern(object) => {
            for property in &object.properties {
                let mut path = path.clone();
                path.push(property_key(&property.key));
                lower_pattern(tracker, &property.value, source, path, ids);
            }
            if let Some(rest) = &object.rest {
                lower_pattern(tracker, &rest.argument, source, vec![None], ids);
            }
        }
        BindingPattern::ArrayPattern(array) => {
            for (index, value) in array.elements.iter().enumerate() {
                if let Some(value) = value {
                    let mut path = path.clone();
                    path.push(Some(index.to_string()));
                    lower_pattern(tracker, value, source, path, ids);
                }
            }
            if let Some(rest) = &array.rest {
                lower_pattern(tracker, &rest.argument, source, vec![None], ids);
            }
        }
        BindingPattern::AssignmentPattern(value) => {
            lower_pattern(tracker, &value.left, source, path, ids);
            lower_pattern(
                tracker,
                &value.left,
                span(value.right.span()),
                Vec::new(),
                ids,
            );
        }
    }
}
