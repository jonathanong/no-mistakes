pub(super) fn invalid_message(text: &str) -> Option<String> {
    let colon = text.contains(':');
    let brace = text.contains('{') || text.contains('}');
    let control = text.chars().any(char::is_control);
    if !colon && !brace && !control {
        return None;
    }
    let mut reasons = Vec::new();
    if colon {
        reasons.push("':'");
    }
    if brace {
        reasons.push("'{' or '}'");
    }
    if control {
        reasons.push("a control character");
    }
    Some(format!(
        "custom jobId contains {}; use '-' or '_' as the separator",
        join_reasons(&reasons)
    ))
}

fn join_reasons(reasons: &[&str]) -> String {
    match reasons {
        [one] => (*one).to_string(),
        [first, second] => format!("{first} and {second}"),
        _ => format!("{}, {}, and {}", reasons[0], reasons[1], reasons[2]),
    }
}
