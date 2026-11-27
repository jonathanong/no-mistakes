pub fn closest_rule_id(
    id: &str,
    candidates: impl Iterator<Item = &'static str>,
) -> Option<&'static str> {
    candidates.min_by_key(|candidate| (edit_distance(id, candidate), *candidate))
}

fn edit_distance(input: &str, candidate: &str) -> usize {
    let mut row: Vec<_> = (0..=candidate.len()).collect();
    for (i, input_char) in input.chars().enumerate() {
        let mut diagonal = row[0];
        row[0] = i + 1;
        for (j, candidate_byte) in candidate.bytes().enumerate() {
            let previous = row[j + 1];
            row[j + 1] = (row[j] + 1)
                .min(previous + 1)
                .min(diagonal + usize::from(input_char != char::from(candidate_byte)));
            diagonal = previous;
        }
    }
    row[candidate.len()]
}

pub const UNUSED_ALLOW_ENTRY: &str = "unused-allow-entry";
