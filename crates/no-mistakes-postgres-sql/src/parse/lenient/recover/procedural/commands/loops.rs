use super::*;

pub(in super::super) fn for_header(code: &[&Token]) -> bool {
    let Some(into) = code.iter().position(|token| word(token, "IN")) else {
        return false;
    };
    if into < 2
        || !assignment::comma_parts(&code[1..into])
            .is_some_and(|parts| parts.into_iter().all(assignment::target))
    {
        return false;
    }
    let mut source = &code[into + 1..];
    let reverse = source.first().is_some_and(|token| word(token, "REVERSE"));
    if reverse {
        source = &source[1..];
    }
    if source.first().is_some_and(|token| word(token, "EXECUTE"))
        && (static_execute(source) || super::super::sql::matches(source))
    {
        return !reverse;
    }
    if row_query(source) {
        return !reverse;
    }
    let step = source.iter().position(|token| word(token, "BY"));
    if let Some(at) = step {
        if !assignment::expression(&source[at + 1..]) {
            return false;
        }
        source = &source[..at];
    }
    let Some(range) = source
        .iter()
        .position(|token| super::super::ranges::marker(token))
    else {
        return false;
    };
    assignment::expression(&source[..range]) && assignment::expression(&source[range + 1..])
}

pub(in super::super) fn foreach_header(code: &[&Token]) -> bool {
    if code.len() < 5 || !matches!(code[1], Token::Word(_)) {
        return false;
    }
    let Some(into) = code.iter().position(|token| word(token, "IN")) else {
        return false;
    };
    let slice = &code[2..into];
    (slice.is_empty()
        || slice.first().is_some_and(|token| word(token, "SLICE"))
            && assignment::expression(&slice[1..]))
        && code.get(into + 1).is_some_and(|token| word(token, "ARRAY"))
        && assignment::expression(&code[into + 2..])
}
