pub(super) struct UrlHost {
    pub(super) host: String,
    pub(super) checksum: bool,
}

fn after_scheme(word: &str) -> Option<&str> {
    if word
        .get(..8)
        .is_some_and(|prefix| prefix.eq_ignore_ascii_case("https://"))
    {
        Some(&word[8..])
    } else if word
        .get(..7)
        .is_some_and(|prefix| prefix.eq_ignore_ascii_case("http://"))
    {
        Some(&word[7..])
    } else {
        None
    }
}

pub(super) fn has_static_url_prefix(word: &str) -> bool {
    let Some((prefix, _)) = word.split_once('$') else {
        return false;
    };
    after_scheme(prefix).is_some_and(|rest| {
        authority_end(rest) < rest.len()
            && rest[authority_end(rest)..].starts_with('/')
            && !prefix.contains(['\\', '`', '{', '}'])
            && !prefix.contains(char::is_whitespace)
            && http_url(prefix).is_some()
    })
}

pub(super) fn http_url(word: &str) -> Option<UrlHost> {
    let rest = after_scheme(word)?;
    if dynamic(&rest[..authority_end(rest)]) {
        return None;
    }
    let (host, path) = split_host(rest);
    // A dynamic path is not a checksum file. A dynamic authority cannot be judged.
    if host.is_empty() {
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
    let (host, suffix) = if let Some(body) = raw.strip_prefix('[') {
        let (host, after) = body.split_once(']')?;
        host.parse::<std::net::Ipv6Addr>().ok()?;
        (host, after)
    } else {
        let end = raw.find(':').unwrap_or(raw.len());
        (&raw[..end], &raw[end..])
    };
    if host.is_empty() || host.contains(char::is_whitespace) {
        return None;
    }
    if !suffix.is_empty() {
        let port = suffix.strip_prefix(':')?;
        if !port.chars().all(|ch| ch.is_ascii_digit()) || port.parse::<u16>().is_err() {
            return None;
        }
    }
    Some(host.to_ascii_lowercase())
}

fn split_host(rest: &str) -> (String, &str) {
    if let Some(body) = rest.strip_prefix('[') {
        let Some((host, after)) = body.split_once(']') else {
            return (String::new(), "");
        };
        let path = path_after_authority(after, after.find(['/', '?', '#']).unwrap_or(after.len()));
        return (host.to_ascii_lowercase(), path);
    }
    let end = authority_end(rest);
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
    let path = path_after_authority(rest, end);
    (host.to_ascii_lowercase(), path)
}

fn authority_end(url: &str) -> usize {
    url.find(['/', '?', '#']).unwrap_or(url.len())
}

fn path_after_authority(url: &str, end: usize) -> &str {
    url[end..].strip_prefix('/').unwrap_or("")
}

pub(super) fn dynamic(value: &str) -> bool {
    value.contains(['$', '`', '{'])
}

fn checksum_path(path: &str) -> bool {
    let path = path.split(['?', '#']).next().unwrap_or(path);
    if dynamic(path) {
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
    let mut quote = None;
    let mut starts_word = true;
    for line in text.lines() {
        let mut chars = line.char_indices();
        let mut continued = false;
        while let Some((index, ch)) = chars.next() {
            if ch == '\\' && quote != Some('\'') {
                if chars.next().is_none() {
                    continued = true;
                    break;
                }
                starts_word = false;
            } else if let Some(active) = quote {
                if ch == active {
                    quote = None;
                }
            } else if matches!(ch, '"' | '\'') {
                quote = Some(ch);
                starts_word = false;
            } else if ch == '#' && starts_word {
                if skip_comment(&line[index + 1..]) {
                    return true;
                }
                break;
            } else {
                starts_word = ch.is_whitespace() || ";&|()<>".contains(ch);
            }
        }
        if !continued && quote.is_none() {
            starts_word = true;
        }
    }
    false
}

pub(super) fn skip_comment(comment: &str) -> bool {
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
