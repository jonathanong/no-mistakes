//! Turn a finite unique-key remainder into an empty pin when it fits the cap.
use super::Keys;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(in crate::codebase::rules::postgres_bounded_statements) enum Credit {
    Accept(u64),
    Over(u64),
}

enum State {
    Value,
    Domain(u64),
    Open,
    Dependent,
}

enum Key {
    Value,
    Accept(u64),
    Over(u64),
    Incomplete,
}

enum Arm {
    Rejected,
    Value,
    Accept(u64),
    Over(u64),
    Incomplete,
}

pub(super) fn apply(
    prepared: &mut Keys,
    domains: &[Vec<Vec<Option<u64>>>],
    rejected: &[usize],
    cap: u64,
) -> Option<Credit> {
    let keys: Vec<Vec<Key>> = prepared
        .iter()
        .enumerate()
        .map(|(alternative, choices)| {
            if rejected.contains(&alternative) {
                return Vec::new();
            }
            choices
                .iter()
                .enumerate()
                .map(|(index, columns)| {
                    key_outcome(
                        columns,
                        domains
                            .get(alternative)
                            .and_then(|row| row.get(index))
                            .map(Vec::as_slice),
                        cap,
                    )
                })
                .collect()
        })
        .collect();
    let arms: Vec<Arm> = keys
        .iter()
        .enumerate()
        .map(|(alternative, keys)| {
            if rejected.contains(&alternative) {
                Arm::Rejected
            } else {
                arm_outcome(keys)
            }
        })
        .collect();
    if arms.iter().any(|arm| matches!(arm, Arm::Incomplete)) {
        return None;
    }
    let mut sum = 0u64;
    let mut used = false;
    for arm in &arms {
        let add = match arm {
            Arm::Rejected => 0,
            Arm::Value => 1,
            Arm::Accept(product) | Arm::Over(product) => {
                used = true;
                *product
            }
            Arm::Incomplete => return None,
        };
        sum = sum.checked_add(add)?;
    }
    if !used {
        return None;
    }
    if sum <= cap
        && arms
            .iter()
            .all(|arm| matches!(arm, Arm::Rejected | Arm::Value | Arm::Accept(_)))
    {
        fill(prepared, &keys);
        return Some(Credit::Accept(sum));
    }
    Some(Credit::Over(sum))
}

fn key_outcome(columns: &[Vec<Vec<usize>>], domains: Option<&[Option<u64>]>, cap: u64) -> Key {
    let states: Vec<State> = columns
        .iter()
        .enumerate()
        .map(|(index, sources)| {
            column_state(
                sources,
                domains.and_then(|row| row.get(index).copied()).flatten(),
            )
        })
        .collect();
    if states.iter().all(|state| matches!(state, State::Value)) {
        return Key::Value;
    }
    if states
        .iter()
        .any(|state| matches!(state, State::Open | State::Dependent))
    {
        return Key::Incomplete;
    }
    let mut product = 1u64;
    let mut used = false;
    for state in &states {
        if let State::Domain(width) = state {
            used = true;
            let Some(next) = product.checked_mul(*width) else {
                return Key::Incomplete;
            };
            product = next;
        }
    }
    if !used {
        return Key::Incomplete;
    }
    if product <= cap {
        Key::Accept(product)
    } else {
        Key::Over(product)
    }
}

fn column_state(sources: &[Vec<usize>], domain: Option<u64>) -> State {
    if sources.iter().any(|source| source.is_empty()) {
        State::Value
    } else if sources.is_empty() {
        match domain {
            Some(width) if width > 0 => State::Domain(width),
            _ => State::Open,
        }
    } else {
        State::Dependent
    }
}

fn arm_outcome(keys: &[Key]) -> Arm {
    if keys.iter().any(|key| matches!(key, Key::Value)) {
        return Arm::Value;
    }
    if let Some(product) = keys
        .iter()
        .filter_map(|key| match key {
            Key::Accept(product) => Some(*product),
            _ => None,
        })
        .min()
    {
        return Arm::Accept(product);
    }
    if let Some(product) = keys
        .iter()
        .filter_map(|key| match key {
            Key::Over(product) => Some(*product),
            _ => None,
        })
        .min()
    {
        return Arm::Over(product);
    }
    Arm::Incomplete
}

fn fill(prepared: &mut Keys, keys: &[Vec<Key>]) {
    for (choices, outcomes) in prepared.iter_mut().zip(keys) {
        for (columns, outcome) in choices.iter_mut().zip(outcomes) {
            if !matches!(outcome, Key::Accept(_)) {
                continue;
            }
            // A free domain column has no pin source. An empty source satisfies it.
            for column in columns.iter_mut() {
                if column.is_empty() {
                    column.push(Vec::new());
                }
            }
        }
    }
}
