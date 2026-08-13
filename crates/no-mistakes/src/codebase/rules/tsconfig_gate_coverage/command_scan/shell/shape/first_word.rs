use super::effective_command::effective_first_word;

pub(super) fn has_dynamic_first_word(command: &str) -> bool {
    let Some(word) = effective_first_word(command) else {
        return false;
    };
    if word
        .strip_prefix('\'')
        .and_then(|word| word.strip_suffix('\''))
        .is_some()
    {
        return false;
    }
    if let Some(double_quoted) = word
        .strip_prefix('"')
        .and_then(|word| word.strip_suffix('"'))
    {
        return double_quoted.contains(['$', '`']);
    }
    word.contains(['\'', '"', '$', '`'])
}

pub(super) fn has_dangling_escape(command: &str) -> bool {
    command
        .chars()
        .rev()
        .take_while(|character| *character == '\\')
        .count()
        % 2
        == 1
}
