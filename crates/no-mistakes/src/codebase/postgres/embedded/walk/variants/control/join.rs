use super::super::{alternatives, Recovered};

pub(in crate::codebase::postgres::embedded::walk::variants) fn joined(
    left: Option<Vec<Recovered>>,
    right: Option<Vec<Recovered>>,
    id: u64,
    arms: (u32, u32),
) -> Option<Vec<Recovered>> {
    if left == right {
        return left;
    }
    let (mut left, mut right) = left.zip(right)?;
    for (values, arm) in [(&mut left, arms.0), (&mut right, arms.1)] {
        for value in values {
            if !value.choices.iter().any(|(other, _)| *other == id) {
                value.choices.push((id, arm));
            }
        }
    }
    alternatives(left, right)
}
