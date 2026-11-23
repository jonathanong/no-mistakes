use super::is_meta;

pub(in super::super) fn redirects_stdout(chars: &[(usize, char)], gt: usize) -> bool {
    if has_variable_fd(chars, gt) {
        return false;
    }

    let mut start = gt;
    while start > 0 && chars[start - 1].1.is_ascii_digit() {
        start -= 1;
    }
    if start == gt || (start > 0 && !is_boundary(chars[start - 1].1)) {
        return true;
    }
    chars[gt - 1].1 == '1' && chars[start..gt - 1].iter().all(|(_, ch)| *ch == '0')
}

fn has_variable_fd(chars: &[(usize, char)], gt: usize) -> bool {
    let [head @ .., (_, '}')] = &chars[..gt] else {
        return false;
    };
    let name_start = head
        .iter()
        .rposition(|(_, ch)| !ch.is_ascii_alphanumeric() && *ch != '_')
        .map_or(0, |index| index + 1);
    let [prefix @ .., (_, '{')] = &head[..name_start] else {
        return false;
    };
    head.get(name_start)
        .is_some_and(|(_, first)| first.is_ascii_alphabetic() || *first == '_')
        && prefix.last().is_none_or(|(_, ch)| is_boundary(*ch))
}

fn is_boundary(ch: char) -> bool {
    ch.is_whitespace() || is_meta(ch)
}
