use super::Recovered;
use crate::codebase::postgres::embedded::MAX_EMBEDDED_SQL_VARIANTS;

pub(super) fn bounded(values: Vec<Recovered>) -> Option<Vec<Recovered>> {
    let mut unique = Vec::new();
    for mut value in values {
        value.choices.sort_unstable();
        value.choices.dedup();
        // Path extension and composition reject incompatible choices before
        // values arrive here; this boundary only normalizes and caps them.
        unique.extend((!unique.contains(&value)).then_some(value));
        if unique.len() > MAX_EMBEDDED_SQL_VARIANTS {
            return None;
        }
    }
    Some(unique)
}
