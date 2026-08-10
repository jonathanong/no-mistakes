use serde_yaml::Value;

pub(super) fn schedule_config_valid(config: &Value) -> bool {
    config.as_sequence().is_some_and(|entries| {
        !entries.is_empty()
            && entries.iter().all(|entry| {
                entry.as_mapping().is_some_and(|entry| {
                    entry.len() == 1
                        && entry
                            .get("cron")
                            .and_then(Value::as_str)
                            .is_some_and(cron_expression_valid)
                })
            })
    })
}

/// GitHub schedules use five-field POSIX cron with lists, ranges, and steps.
fn cron_expression_valid(cron: &str) -> bool {
    let fields = cron.split_ascii_whitespace().collect::<Vec<_>>();
    fields.len() == 5
        && fields
            .iter()
            .enumerate()
            .all(|(position, field)| cron_field_valid(field, position))
}

fn cron_field_valid(field: &str, position: usize) -> bool {
    field
        .split(',')
        .all(|part| cron_field_part_valid(part, position))
}

fn cron_field_part_valid(part: &str, position: usize) -> bool {
    let mut step_parts = part.split('/');
    let base = step_parts.next().expect("split always returns one part");
    let Some(step) = step_parts.next() else {
        return cron_atom_or_range_valid(base, position);
    };
    if step_parts.next().is_some() || base.is_empty() || step.is_empty() {
        return false;
    }
    cron_atom_or_range_valid(base, position) && cron_number_in_range(step, position, false)
}

fn cron_atom_or_range_valid(base: &str, position: usize) -> bool {
    if base == "*" {
        return true;
    }
    let mut range_parts = base.split('-');
    let start = range_parts.next().expect("split always returns one part");
    let Some(end) = range_parts.next() else {
        return cron_atom_valid(start, position);
    };
    range_parts.next().is_none()
        && cron_atom_valid(start, position)
        && cron_atom_valid(end, position)
        && cron_atom_position(start, position) <= cron_atom_position(end, position)
}

fn cron_atom_valid(atom: &str, position: usize) -> bool {
    cron_atom_position(atom, position).is_some()
}

fn cron_atom_position(atom: &str, position: usize) -> Option<u8> {
    let names = match position {
        3 => Some(
            &[
                "JAN", "FEB", "MAR", "APR", "MAY", "JUN", "JUL", "AUG", "SEP", "OCT", "NOV", "DEC",
            ][..],
        ),
        4 => Some(&["SUN", "MON", "TUE", "WED", "THU", "FRI", "SAT"][..]),
        _ => None,
    };
    if let Some(names) = names {
        if let Some(index) = names
            .iter()
            .position(|name| atom.eq_ignore_ascii_case(name))
        {
            return u8::try_from(index).ok();
        }
    }
    let number = atom.parse::<u8>().ok()?;
    cron_number_in_range(atom, position, true).then_some(number)
}

fn cron_number_in_range(number: &str, position: usize, atom: bool) -> bool {
    let Ok(number) = number.parse::<u8>() else {
        return false;
    };
    let maximum = [59, 23, 31, 12, 7][position];
    let minimum = if atom { [0, 0, 1, 1, 0][position] } else { 1 };
    (minimum..=maximum).contains(&number)
}

#[cfg(test)]
mod tests;
