pub(super) fn skip_block_comment(definition: &str, index: usize) -> usize {
    let mut depth = 1i32;
    let mut cursor = index + 2;
    while cursor < definition.len() && depth > 0 {
        if definition[cursor..].starts_with("/*") {
            depth += 1;
            cursor += 2;
            continue;
        }
        if definition[cursor..].starts_with("*/") {
            depth -= 1;
            cursor += 2;
            continue;
        }
        let step = definition[cursor..]
            .chars()
            .next()
            .map_or(1, char::len_utf8);
        cursor += step;
    }
    cursor
}
