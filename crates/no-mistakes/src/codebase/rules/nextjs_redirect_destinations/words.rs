use crate::codebase::ts_source::byte_offset_to_line;

pub(super) fn contains_word(source: &str, word: &str) -> bool {
    source
        .match_indices(word)
        .any(|(index, _)| is_word_at(source, index, word.len()))
}

pub(super) fn word_line(source: &str, word: &str) -> usize {
    source
        .match_indices(word)
        .find(|(index, _)| is_word_at(source, *index, word.len()))
        .map(|(index, _)| byte_offset_to_line(source, index) as usize)
        .unwrap_or(1)
}

fn is_word_at(source: &str, index: usize, len: usize) -> bool {
    let before = source[..index].chars().next_back();
    let after = source[index + len..].chars().next();
    !before.is_some_and(is_word_char) && !after.is_some_and(is_word_char)
}

fn is_word_char(ch: char) -> bool {
    ch.is_ascii_alphanumeric() || ch == '_'
}
