use std::net::{Ipv4Addr, Ipv6Addr};

pub(super) fn valid_static_reference(value: &str) -> bool {
    let (name, digest) = match value.split_once('@') {
        Some((name, digest)) => (name, Some(digest)),
        None => (value, None),
    };
    !name.is_empty() && !name.contains('@') && digest.is_none_or(valid_digest) && valid_name(name)
}

fn valid_digest(digest: &str) -> bool {
    let Some((algorithm, encoded)) = digest.split_once(':') else {
        return false;
    };
    !algorithm.is_empty()
        && algorithm.split(['+', '.', '_', '-']).all(|component| {
            component
                .bytes()
                .next()
                .is_some_and(|byte| byte.is_ascii_alphabetic())
                && component.bytes().all(|byte| byte.is_ascii_alphanumeric())
        })
        && valid_encoded_digest_length(algorithm, encoded)
        && encoded.bytes().all(|byte| byte.is_ascii_hexdigit())
}

fn valid_encoded_digest_length(algorithm: &str, encoded: &str) -> bool {
    let expected_length = [
        ("md5", 32),
        ("sha1", 40),
        ("sha224", 56),
        ("sha256", 64),
        ("sha384", 96),
        ("sha512", 128),
    ]
    .into_iter()
    .find_map(|(recognized, length)| algorithm.eq_ignore_ascii_case(recognized).then_some(length));
    expected_length.map_or(encoded.len() >= 32, |length| encoded.len() == length)
}

fn valid_name(value: &str) -> bool {
    let (path, tag) = match value.rfind(':') {
        Some(index) if value.rfind('/').is_none_or(|slash| index > slash) => {
            (&value[..index], Some(&value[index + 1..]))
        }
        _ => (value, None),
    };
    value.len() <= 255 && tag.is_none_or(valid_tag) && valid_path(path)
}

fn valid_tag(tag: &str) -> bool {
    !tag.is_empty()
        && tag.len() <= 128
        && tag
            .bytes()
            .next()
            .is_some_and(|byte| byte.is_ascii_alphanumeric() || byte == b'_')
        && tag
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'.' | b'-'))
}

fn valid_path(value: &str) -> bool {
    let segments = value.split('/').collect::<Vec<_>>();
    if segments.is_empty() || segments.iter().any(|segment| segment.is_empty()) {
        return false;
    }
    let registry = segments.len() > 1
        && (segments[0] == "localhost"
            || segments[0].contains(['.', ':'])
            || segments[0].starts_with('['));
    (!registry || valid_registry(segments[0]))
        && segments[usize::from(registry)..]
            .iter()
            .all(|segment| valid_path_component(segment))
}

fn valid_registry(value: &str) -> bool {
    if let Some(rest) = value.strip_prefix('[') {
        let Some((address, port)) = rest.split_once(']') else {
            return false;
        };
        return address.parse::<Ipv6Addr>().is_ok()
            && (port.is_empty() || valid_port(port.strip_prefix(':')));
    }
    let (host, port) = value
        .rsplit_once(':')
        .map_or((value, None), |(host, port)| (host, Some(port)));
    valid_host(host) && port.is_none_or(|port| valid_port(Some(port)))
}

fn valid_port(port: Option<&str>) -> bool {
    port.is_some_and(|port| port.parse::<u16>().is_ok_and(|port| port > 0))
}

fn valid_host(value: &str) -> bool {
    value.parse::<Ipv4Addr>().is_ok()
        || (!value.is_empty()
            && value.len() <= 253
            && value.split('.').all(|label| {
                !label.is_empty()
                    && label.len() <= 63
                    && !label.starts_with('-')
                    && !label.ends_with('-')
                    && label
                        .bytes()
                        .all(|byte| byte.is_ascii_alphanumeric() || byte == b'-')
            }))
}

fn valid_path_component(value: &str) -> bool {
    let bytes = value.as_bytes();
    if !bytes
        .first()
        .is_some_and(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit())
    {
        return false;
    }
    let mut index = 1;
    while index < bytes.len() {
        if bytes[index].is_ascii_lowercase() || bytes[index].is_ascii_digit() {
            index += 1;
            continue;
        }
        let separator = bytes[index];
        if !matches!(separator, b'.' | b'_' | b'-') {
            return false;
        }
        let start = index;
        while bytes.get(index) == Some(&separator) {
            index += 1;
        }
        let count = index - start;
        if (separator != b'-' && count != 1 && !(separator == b'_' && count == 2))
            || !bytes
                .get(index)
                .is_some_and(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit())
        {
            return false;
        }
    }
    true
}

#[cfg(test)]
mod tests;
