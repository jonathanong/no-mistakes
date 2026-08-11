use super::{comments::strip_static_comments, static_tokens};

pub(crate) fn shell_body_has_static_failure(script: &str) -> bool {
    let script = strip_static_comments(script);
    script
        .split(['\n', ';'])
        .flat_map(|group| group.split("&&"))
        .filter_map(static_tokens)
        .any(|tokens| match tokens.first().map(String::as_str) {
            Some("false" | "return") => true,
            Some("exit") => exit_status_fails(&tokens),
            _ => false,
        })
}

fn exit_status_fails(tokens: &[String]) -> bool {
    match tokens {
        [command, status] if command == "exit" => status
            .parse::<i64>()
            .map_or(true, |status| status.rem_euclid(256) != 0),
        [command] if command == "exit" => false,
        _ => true,
    }
}

#[cfg(test)]
mod tests;
