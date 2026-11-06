use super::*;
use mutable_origins::WorkBudget;
type Pending = mutation_projection::PendingMutation<nested::RootRead>;

pub(super) fn expand(
    origins: &parameter_call_origins::ParameterCallOrigins,
    source: Pending,
    budget: &mut WorkBudget,
) -> Option<(Vec<Pending>, Vec<Pending>)> {
    let mut pending = vec![(source, false)];
    let (mut definite, mut possible, mut seen) = (Vec::new(), Vec::new(), fx_set());
    while let Some((source, uncertain)) = pending.pop() {
        budget.charge(1 + source.0 .0 .1.len() + source.0 .1.len())?;
        let key = (source.0 .0 .0 .0, source.0 .0 .0 .1.clone());
        let read = source.0 .2;
        let selected = origins.values_at(&key, read);
        let uncertain = uncertain || selected.uncertain;
        if !seen.insert((key, read, uncertain))
            || (selected.values.is_none() && selected.possible.iter().all(Option::is_none))
        {
            if uncertain {
                possible.push(source);
            } else {
                definite.push(source);
            }
            continue;
        }
        let ((((_, _), path), reads, _), exiting, _, propagated) = source;
        for (values, newly_uncertain) in [
            (selected.values, false),
            (selected.possible[0], true),
            (selected.possible[1], true),
            (selected.possible[2], true),
        ] {
            for value in values.into_iter().flatten() {
                let mut references = budget.output();
                nested::write_receiver_references(value, &path, &reads, &mut references);
                for reference in references.finish()? {
                    budget.charge(1 + reference.0 .1.len() + reference.1.len())?;
                    let item = (reference, exiting, true, propagated);
                    let uncertain = uncertain || newly_uncertain;
                    if origins.has_values(&item.0 .0 .0) {
                        pending.push((item, uncertain));
                    } else if uncertain {
                        possible.push(item);
                    } else {
                        definite.push(item);
                    }
                }
            }
        }
    }
    Some((definite, possible))
}
