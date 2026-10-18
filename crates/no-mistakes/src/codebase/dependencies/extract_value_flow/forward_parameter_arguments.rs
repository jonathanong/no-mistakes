use super::*;

pub(super) fn expand(
    origins: &parameter_call_origins::ParameterCallOrigins,
    source: mutation_projection::PendingMutation<nested::RootRead>,
) -> Vec<mutation_projection::PendingMutation<nested::RootRead>> {
    let ((((scope, name), path), reads, read), exiting, remember, propagated) = source;
    let Some(values) = origins.values(&(scope, name.clone())) else {
        return vec![(
            (((scope, name), path), reads, read),
            exiting,
            remember,
            propagated,
        )];
    };
    let mut references = Vec::new();
    for value in values {
        nested::write_receiver_references(value, &path, &reads, &mut references);
    }
    references
        .into_iter()
        // The original parameter was remembered before deferral; argument
        // receivers still need their own mutation history leaf.
        .map(|reference| (reference, exiting, true, propagated))
        .collect()
}
