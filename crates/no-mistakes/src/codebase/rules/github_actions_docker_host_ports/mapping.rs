pub(super) enum Verdict {
    Literal(String),
    Allowed,
    Silent,
}

pub(super) fn classify(spec: &str) -> Verdict {
    let Some(spec) = strip_proto(spec) else {
        return Verdict::Silent;
    };
    if spec.is_empty() || spec.chars().any(char::is_whitespace) {
        return Verdict::Silent;
    }
    let Some(host) = host_part(spec) else {
        return Verdict::Silent;
    };
    if host.is_empty() {
        return Verdict::Allowed;
    }
    if is_variable(&host) || allocated_zero(&host) {
        return Verdict::Allowed;
    }
    if is_literal_port(&host) {
        return Verdict::Literal(host);
    }
    Verdict::Silent
}

fn strip_proto(spec: &str) -> Option<&str> {
    let Some((port, proto)) = spec.rsplit_once('/') else {
        return Some(spec);
    };
    if matches!(proto, "tcp" | "udp" | "sctp") && !port.contains('/') {
        Some(port)
    } else {
        None
    }
}

fn host_part(spec: &str) -> Option<String> {
    if let Some(rest) = spec.strip_prefix('[') {
        let (ip_rest, after) = rest.split_once(']')?;
        if ip_rest.is_empty() {
            return None;
        }
        if let Some(container) = after.strip_prefix("::") {
            return nonempty_container(container).map(|_| String::new());
        }
        let rest = after.strip_prefix(':')?;
        let (host, container) = split_once_outside(rest)?;
        nonempty_container(&container)?;
        return Some(host);
    }
    let parts = split_outside(spec)?;
    match parts.len() {
        1 => nonempty_container(&parts[0]).map(|_| String::new()),
        2 => {
            nonempty_container(&parts[1])?;
            Some(parts[0].clone())
        }
        3 => {
            nonempty_container(&parts[2])?;
            Some(parts[1].clone())
        }
        _ => None,
    }
}

fn nonempty_container(container: &str) -> Option<()> {
    (!container.is_empty()).then_some(())
}

fn split_once_outside(spec: &str) -> Option<(String, String)> {
    let mut parts = split_outside(spec)?;
    if parts.len() != 2 {
        return None;
    }
    let container = parts.pop()?;
    let host = parts.pop()?;
    Some((host, container))
}

fn split_outside(spec: &str) -> Option<Vec<String>> {
    let chars: Vec<char> = spec.chars().collect();
    let mut parts = Vec::new();
    let mut current = String::new();
    let mut index = 0;
    while index < chars.len() {
        if chars[index] == '$' {
            let end = expansion_end(&chars, index)?;
            current.extend(chars[index..end].iter());
            index = end;
            continue;
        }
        if chars[index] == ':' {
            parts.push(std::mem::take(&mut current));
            index += 1;
            continue;
        }
        current.push(chars[index]);
        index += 1;
    }
    parts.push(current);
    Some(parts)
}

fn expansion_end(chars: &[char], start: usize) -> Option<usize> {
    if chars.get(start + 1) == Some(&'{') {
        if chars.get(start + 2) == Some(&'{') {
            return find_marker(chars, start + 3, &['}', '}']);
        }
        return find_brace(chars, start + 2);
    }
    let mut end = start + 1;
    while end < chars.len() && (chars[end].is_ascii_alphanumeric() || chars[end] == '_') {
        end += 1;
    }
    (end > start + 1).then_some(end)
}

fn find_brace(chars: &[char], start: usize) -> Option<usize> {
    let mut depth = 1usize;
    let mut index = start;
    while index < chars.len() {
        match chars[index] {
            '{' => depth += 1,
            '}' => {
                depth -= 1;
                if depth == 0 {
                    return Some(index + 1);
                }
            }
            _ => {}
        }
        index += 1;
    }
    None
}

fn find_marker(chars: &[char], start: usize, marker: &[char]) -> Option<usize> {
    let mut index = start;
    while index + marker.len() <= chars.len() {
        if chars[index..index + marker.len()] == *marker {
            return Some(index + marker.len());
        }
        index += 1;
    }
    None
}

fn is_variable(host: &str) -> bool {
    let Some(rest) = host.strip_prefix('$') else {
        return false;
    };
    if let Some(rest) = rest.strip_prefix('{') {
        if let Some(expr) = rest.strip_prefix('{') {
            let Some(inner) = expr.strip_suffix("}}") else {
                return false;
            };
            return !inner.contains("${{") && !inner.contains("}}");
        }
        let Some(inner) = rest.strip_suffix('}') else {
            return false;
        };
        return !inner.is_empty() && !inner.contains('{') && !inner.contains('}');
    }
    !rest.is_empty()
        && rest
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || byte == b'_')
}

fn allocated_zero(host: &str) -> bool {
    let mut sides = host.split('-');
    let Some(start) = sides.next() else {
        return false;
    };
    let end = sides.next();
    if sides.next().is_some() {
        return false;
    }
    let zeros = |text: &str| !text.is_empty() && text.bytes().all(|byte| byte == b'0');
    match end {
        None => zeros(start),
        Some(end) => zeros(start) && zeros(end),
    }
}

fn is_literal_port(host: &str) -> bool {
    let mut sides = host.split('-');
    let Some(start) = sides.next() else {
        return false;
    };
    let end = sides.next();
    if sides.next().is_some() {
        return false;
    }
    let numeric = |text: &str| !text.is_empty() && text.bytes().all(|byte| byte.is_ascii_digit());
    match end {
        None => numeric(start),
        Some(end) => numeric(start) && numeric(end),
    }
}
