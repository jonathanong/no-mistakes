use super::{opaque_interpolated_expression_form, DYNAMIC_VOLUME_EXPRESSION};

pub(super) fn valid(value: &str) -> bool {
    let Some(value) = opaque_interpolated_expression_form(value, DYNAMIC_VOLUME_EXPRESSION) else {
        return false;
    };
    if value.contains(DYNAMIC_VOLUME_EXPRESSION) {
        return true;
    }
    static_reference_valid(&value)
}

fn static_reference_valid(value: &str) -> bool {
    if value.is_empty() || value.bytes().any(|byte| byte.is_ascii_whitespace()) {
        return false;
    }
    let mut at_parts = value.split('@');
    let Some(name_and_tag) = at_parts.next() else {
        return false;
    };
    let digest = at_parts.next();
    if at_parts.next().is_some() || digest.is_some_and(|digest| !digest_valid(digest)) {
        return false;
    }
    let Some(name) = name_without_tag(name_and_tag) else {
        return false;
    };
    repository_name_valid(name)
}

fn name_without_tag(value: &str) -> Option<&str> {
    let slash = value.rfind('/');
    let colon = value.rfind(':');
    if colon.is_some_and(|colon| slash.is_none_or(|slash| colon > slash)) {
        let colon = colon.expect("checked as present");
        if !tag_valid(&value[colon + 1..]) {
            return None;
        }
        Some(&value[..colon])
    } else {
        Some(value)
    }
}

fn repository_name_valid(value: &str) -> bool {
    let components = value.split('/').collect::<Vec<_>>();
    if components.is_empty() || components.iter().any(|component| component.is_empty()) {
        return false;
    }
    let path_start = usize::from(
        components.len() > 1
            && (components[0].contains('.')
                || components[0].contains(':')
                || components[0] == "localhost"),
    );
    (path_start == 0 || registry_valid(components[0]))
        && components[path_start..]
            .iter()
            .all(|component| name_component_valid(component))
}

fn registry_valid(value: &str) -> bool {
    let (host, port) = value
        .rsplit_once(':')
        .map_or((value, None), |(host, port)| (host, Some(port)));
    !host.is_empty()
        && port
            .is_none_or(|port| !port.is_empty() && port.bytes().all(|byte| byte.is_ascii_digit()))
        && host.split('.').all(|label| {
            !label.is_empty()
                && !label.starts_with('-')
                && !label.ends_with('-')
                && label
                    .bytes()
                    .all(|byte| byte.is_ascii_alphanumeric() || byte == b'-')
        })
}

fn name_component_valid(value: &str) -> bool {
    let bytes = value.as_bytes();
    let mut index = consume_lower_alphanumeric(bytes, 0);
    if index == 0 {
        return false;
    }
    while index < bytes.len() {
        index = match bytes[index] {
            b'.' => index + 1,
            b'_' if bytes.get(index + 1) == Some(&b'_') => index + 2,
            b'_' => index + 1,
            b'-' => {
                let mut next = index + 1;
                while bytes.get(next) == Some(&b'-') {
                    next += 1;
                }
                next
            }
            _ => return false,
        };
        let next = consume_lower_alphanumeric(bytes, index);
        if next == index {
            return false;
        }
        index = next;
    }
    true
}

fn consume_lower_alphanumeric(bytes: &[u8], mut index: usize) -> usize {
    while bytes
        .get(index)
        .is_some_and(|byte| byte.is_ascii_digit() || byte.is_ascii_lowercase())
    {
        index += 1;
    }
    index
}

fn tag_valid(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 128
        && value
            .bytes()
            .next()
            .is_some_and(|byte| byte.is_ascii_alphanumeric() || byte == b'_')
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'.' | b'-'))
}

fn digest_valid(value: &str) -> bool {
    value.split_once(':').is_some_and(|(algorithm, encoded)| {
        !algorithm.is_empty()
            && !encoded.is_empty()
            && algorithm.bytes().all(|byte| {
                byte.is_ascii_alphanumeric() || matches!(byte, b'+' | b'.' | b'_' | b'-')
            })
            && encoded
                .bytes()
                .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'=' | b'_' | b'-'))
    })
}

#[cfg(test)]
mod tests;
