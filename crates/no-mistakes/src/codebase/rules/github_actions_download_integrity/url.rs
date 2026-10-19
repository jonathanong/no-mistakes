pub(super) struct UrlHost {
    pub(super) host: String,
    pub(super) checksum: bool,
}

pub(super) fn http_url(word: &str) -> Option<UrlHost> {
    let rest = if word.len() >= 8 && word[..8].eq_ignore_ascii_case("https://") {
        &word[8..]
    } else if word.len() >= 7 && word[..7].eq_ignore_ascii_case("http://") {
        &word[7..]
    } else {
        return None;
    };
    let (host, path) = split_host(rest);
    // A dynamic path is not a checksum file. A dynamic authority cannot be judged.
    if host.is_empty() || host.contains(['$', '`', '{']) {
        return None;
    }
    Some(UrlHost {
        host,
        checksum: checksum_path(path),
    })
}

pub(super) fn configured_host(raw: &str) -> Option<String> {
    let raw = raw.trim();
    if raw.is_empty() || raw.contains(['/', '?', '#', ' ', '@']) {
        return None;
    }
    let (host, path) = split_host(raw);
    if host.is_empty() || !path.is_empty() {
        return None;
    }
    Some(host)
}

fn split_host(rest: &str) -> (String, &str) {
    if let Some(body) = rest.strip_prefix('[') {
        let Some((host, after)) = body.split_once(']') else {
            return (String::new(), "");
        };
        let path = after
            .find('/')
            .map(|index| &after[index + 1..])
            .unwrap_or("");
        return (host.to_ascii_lowercase(), path);
    }
    let end = rest.find(['/', '?', '#']).unwrap_or(rest.len());
    let authority = rest[..end]
        .rsplit_once('@')
        .map(|(_, host)| host)
        .unwrap_or(&rest[..end]);
    let host = match authority.rsplit_once(':') {
        Some((host, port)) if !host.is_empty() && port.chars().all(|ch| ch.is_ascii_digit()) => {
            host
        }
        _ => authority,
    };
    let path = rest.get(end + 1..).unwrap_or("");
    (host.to_ascii_lowercase(), path)
}

fn checksum_path(path: &str) -> bool {
    let path = path.split(['?', '#']).next().unwrap_or(path);
    if path.contains(['$', '`', '{']) {
        return false;
    }
    let name = path.rsplit('/').next().unwrap_or(path);
    let lower = name.to_ascii_lowercase();
    lower.ends_with(".sha256")
        || lower.ends_with(".sha256sum")
        || lower.ends_with(".sha512")
        || lower.ends_with(".sha512sum")
        || matches!(
            lower.as_str(),
            "sha256sums" | "sha256sums.txt" | "sha512sums" | "sha512sums.txt"
        )
}

pub(super) fn trusted(host: &str, hosts: &[String]) -> bool {
    hosts.iter().any(|item| item == host)
}

pub(super) fn has_skip(text: &str) -> bool {
    text.lines().any(skip_line)
}

fn skip_line(line: &str) -> bool {
    let Some(comment) = unquoted_hash(line) else {
        return false;
    };
    let rest = comment.trim_start();
    let Some(rest) = rest.strip_prefix("integrity-check:") else {
        return false;
    };
    let Some(rest) = rest.trim_start().strip_prefix("skip") else {
        return false;
    };
    if !rest.starts_with(char::is_whitespace) {
        return false;
    }
    let Some(rest) = rest.trim_start().strip_prefix("reason=") else {
        return false;
    };
    let slug = rest.trim_end();
    let mut chars = slug.chars();
    chars.next().is_some_and(|ch| ch.is_ascii_alphanumeric())
        && chars.all(|ch| ch.is_ascii_alphanumeric() || ch == '_' || ch == '-')
        && !slug.contains(char::is_whitespace)
}

fn unquoted_hash(line: &str) -> Option<&str> {
    let mut quote = None;
    for (index, ch) in line.char_indices() {
        if let Some(active) = quote {
            if ch == active {
                quote = None;
            }
            continue;
        }
        if ch == '"' || ch == '\'' {
            quote = Some(ch);
            continue;
        }
        if ch == '#' {
            return Some(&line[index + 1..]);
        }
    }
    None
}
