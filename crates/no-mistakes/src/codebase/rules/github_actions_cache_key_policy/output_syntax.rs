pub(super) fn brace_open(line: &str) -> bool {
    let line = line.trim();
    line == "{" || line.starts_with("{ ") || line.starts_with("{>>") || line.starts_with("{ >>")
}

pub(super) fn brace_close(line: &str) -> bool {
    line.trim().starts_with('}')
}

pub(super) fn command_assign(line: &str) -> Option<(String, String)> {
    let line = line.trim();
    if let Some(rest) = line.strip_prefix("echo") {
        if rest.starts_with(char::is_whitespace) {
            return payload_assign(&split_arg(rest.trim_start()).0);
        }
    }
    let rest = line.strip_prefix("printf")?;
    if !rest.starts_with(char::is_whitespace) {
        return None;
    }
    printf_assign(rest.trim_start())
}

fn printf_assign(rest: &str) -> Option<(String, String)> {
    let (fmt, after) = split_arg(rest);
    let fmt = fmt.replace("\\n", "");
    let (name, pattern) = fmt.split_once('=')?;
    if !is_output_name(name) {
        return None;
    }
    if pattern.contains('%') {
        let (arg, _) = split_arg(after.trim_start());
        return (!arg.is_empty()).then(|| (name.to_string(), arg));
    }
    Some((name.to_string(), pattern.to_string()))
}

fn payload_assign(payload: &str) -> Option<(String, String)> {
    let (name, value) = payload.split_once('=')?;
    is_output_name(name).then(|| (name.to_string(), value.trim().to_string()))
}

pub(super) fn inline_assign(line: &str) -> Option<(String, String)> {
    let line = line.trim();
    if line.starts_with("echo") || line.starts_with("printf") {
        return command_assign(line);
    }
    payload_assign(line)
}

pub(super) fn shell_assignment(line: &str) -> Option<(String, String)> {
    let line = line.trim();
    if line.starts_with("echo")
        || line.starts_with("printf")
        || line.starts_with('{')
        || line.starts_with('}')
    {
        return None;
    }
    let (name, rhs) = line.split_once('=')?;
    let name = name.trim();
    is_ident(name).then(|| (name.to_string(), rhs.trim().to_string()))
}

fn split_arg(input: &str) -> (String, &str) {
    let input = input.trim_start();
    if input.is_empty() {
        return (String::new(), input);
    }
    let quote = input.chars().next().unwrap();
    if quote == '"' || quote == '\'' {
        let body = &input[quote.len_utf8()..];
        let end = body.find(quote).unwrap_or(body.len());
        let rest_at = end + usize::from(end < body.len());
        return (body[..end].to_string(), &body[rest_at..]);
    }
    let end = input.find(char::is_whitespace).unwrap_or(input.len());
    let token = input[..end].split(">>").next().unwrap_or(&input[..end]);
    (token.to_string(), &input[end..])
}

pub(super) fn strip_subst(value: &str) -> Option<&str> {
    let value = value.trim().trim_end_matches(';').trim();
    value
        .strip_prefix("$(")
        .and_then(|body| body.strip_suffix(')'))
        .or_else(|| {
            value
                .strip_prefix('`')
                .and_then(|body| body.strip_suffix('`'))
        })
}

pub(super) fn var_ref(value: &str) -> Option<&str> {
    if let Some(inner) = value
        .strip_prefix("${")
        .and_then(|body| body.strip_suffix('}'))
    {
        return is_ident(inner).then_some(inner);
    }
    let name = value.strip_prefix('$')?;
    is_ident(name).then_some(name)
}

pub(super) fn unquote(value: &str) -> &str {
    let value = value.trim();
    for quote in ['"', '\''] {
        if value.len() >= 2 && value.starts_with(quote) && value.ends_with(quote) {
            return &value[quote.len_utf8()..value.len() - quote.len_utf8()];
        }
    }
    value
}

fn is_ident(name: &str) -> bool {
    let mut chars = name.chars();
    match chars.next() {
        Some(c) if c.is_ascii_alphabetic() || c == '_' => {
            chars.all(|c| c.is_ascii_alphanumeric() || c == '_')
        }
        _ => false,
    }
}

fn is_output_name(name: &str) -> bool {
    let mut chars = name.chars();
    match chars.next() {
        Some(c) if c.is_ascii_alphabetic() || c == '_' => {
            chars.all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-')
        }
        _ => false,
    }
}
