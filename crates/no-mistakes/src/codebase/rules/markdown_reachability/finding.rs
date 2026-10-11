use super::{BaselineEntry, RuleFinding, RULE_ID};

pub(super) fn finding(
    file: &str,
    state: &BaselineEntry,
    max_depth: usize,
    invalid_intermediary: bool,
) -> RuleFinding {
    let message = if invalid_intermediary {
        format!(
            "reachable at depth {}, but an intermediary must be a configured index Markdown file",
            state.depth.unwrap_or_default()
        )
    } else if state.state == "unreachable" {
        format!("not reachable from a configured root Markdown file within {max_depth} hops")
    } else {
        format!(
            "reachable only at depth {}; maximum is {max_depth}",
            state.depth.unwrap_or_default()
        )
    };
    RuleFinding {
        rule: RULE_ID.to_string(),
        file: file.to_string(),
        line: 1,
        message,
        import: None,
        target: None,
    }
}

pub(super) fn stale(file: &str, message: &str) -> RuleFinding {
    RuleFinding {
        rule: RULE_ID.to_string(),
        file: file.to_string(),
        line: 1,
        message: format!("stale baseline entry: {message}"),
        import: None,
        target: None,
    }
}
