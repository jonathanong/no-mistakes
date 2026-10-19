use super::*;

pub(super) fn expand(
    origins: &parameter_call_origins::ParameterCallOrigins,
    source: mutation_projection::PendingMutation<nested::RootRead>,
) -> Vec<mutation_projection::PendingMutation<nested::RootRead>> {
    // Follow parameter origins until the receiver is not itself a parameter.
    // A cycle keeps that parameter instead of looping.
    let mut pending = vec![source];
    let mut output = Vec::new();
    let mut seen = fx_set();
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
            let next = reference.0 .0.clone();
            if origins.values(&next).is_some() {
                pending.push((reference, exiting, true, propagated));
            } else {
                output.push((reference, exiting, true, propagated));
            }
        }
    }
    output
}
