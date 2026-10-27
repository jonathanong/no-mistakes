use crate::queue::extract_visitor::QueueVisitor;
use oxc_ast::ast::{BindingPattern, Expression};

pub(super) fn note_destructured_ids(
    visitor: &mut QueueVisitor<'_>,
    pattern: &BindingPattern<'_>,
    init: &Expression<'_>,
) {
    if !visitor.tracks_job_object(init) {
        return;
    }
    let BindingPattern::ObjectPattern(object) = pattern else {
        return;
    };
    // `tracks_job_object` only returns true when a frame exists.
    let index = visitor.job_frames.len() - 1;
    let frame = &mut visitor.job_frames[index];
    for property in &object.properties {
        // A computed key can still have a static name (`['id']`). Ignore it.
        let Some(key) = crate::ast::property_key_name(&property.key) else {
            continue;
        };
        if property.computed {
            continue;
        }
        if !matches!(key.as_ref(), "id" | "deduplicationId") {
            continue;
        }
        let Some(name) = bound_id_name(&property.value) else {
            continue;
        };
        frame.id_idents.push(name.to_string());
    }
}

fn bound_id_name<'a>(pattern: &'a BindingPattern<'a>) -> Option<&'a str> {
    // A default (`id = fallback`) is the fallback when the property is missing,
    // so only a bare name is the job id.
    let BindingPattern::BindingIdentifier(id) = pattern else {
        return None;
    };
    Some(id.name.as_str())
}
