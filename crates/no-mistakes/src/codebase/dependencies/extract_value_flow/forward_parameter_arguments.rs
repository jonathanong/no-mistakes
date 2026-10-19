use super::*;

pub(super) fn expand(
    origins: &parameter_call_origins::ParameterCallOrigins,
    source: mutation_projection::PendingMutation<nested::RootRead>,
) -> Vec<mutation_projection::PendingMutation<nested::RootRead>> {
    let (mut pending, mut output, mut seen) = (vec![source], Vec::new(), fx_set());
    while let Some(source) = pending.pop() {
        let key = (source.0 .0 .0 .0, source.0 .0 .0 .1.clone());
        if !seen.insert(key.clone()) {
            output.push(source);
            continue;
        }
        let Some(values) = origins.values(&key) else {
            output.push(source);
            continue;
        };
        let ((((_, _), path), reads, _), exiting, _, propagated) = source;
        let mut references = Vec::new();
        for value in values {
            nested::write_receiver_references(value, &path, &reads, &mut references);
        }
        for reference in references {
            let item = (reference, exiting, true, propagated);
            if origins.values(&item.0 .0 .0).is_some() {
                pending.push(item);
            } else {
                output.push(item);
            }
        }
    }
    output
}
