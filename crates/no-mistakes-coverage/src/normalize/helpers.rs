//! URL decoding and CI runner workspace prefix stripping helpers.

use crate::normalize::PathNormalizeError;
use std::borrow::Cow;

pub(super) fn strip_file_url(raw: &str) -> Result<Cow<'_, str>, PathNormalizeError> {
    let Some(prefix) = raw.get(..7) else {
        return Ok(Cow::Borrowed(raw));
    };
    if !prefix.eq_ignore_ascii_case("file://") {
        return Ok(Cow::Borrowed(raw));
    }

    let mut rest = &raw[7..];
    if let Some(after_host) = rest
        .get(..10)
        .filter(|h| h.eq_ignore_ascii_case("localhost/"))
        .map(|_| &rest[9..])
    {
        rest = after_host;
    }

    let decoded = percent_decode_lossy(rest);
    if decoded.contains('\0') {
        return Err(PathNormalizeError::ContainsNul);
    }

    let bytes = decoded.as_bytes();
    if bytes.len() >= 4
        && bytes[0] == b'/'
        && bytes[1].is_ascii_alphabetic()
        && bytes[2] == b':'
        && (bytes[3] == b'/' || bytes[3] == b'\\')
    {
        return Ok(Cow::Owned(decoded[1..].to_string()));
    }

    Ok(Cow::Owned(decoded))
}

fn percent_decode_lossy(input: &str) -> String {
    let bytes = input.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut idx = 0;
    while idx < bytes.len() {
        if bytes[idx] == b'%' && idx + 2 < bytes.len() {
            if let (Some(hi), Some(lo)) = (hex_val(bytes[idx + 1]), hex_val(bytes[idx + 2])) {
                out.push((hi << 4) | lo);
                idx += 3;
                continue;
            }
        }
        out.push(bytes[idx]);
        idx += 1;
    }
    String::from_utf8_lossy(&out).into_owned()
}

fn hex_val(byte: u8) -> Option<u8> {
    match byte {
        b'0'..=b'9' => Some(byte - b'0'),
        b'a'..=b'f' => Some(byte - b'a' + 10),
        b'A'..=b'F' => Some(byte - b'A' + 10),
        _ => None,
    }
}

pub(super) fn strip_ci_work_segments(rest: &str) -> Option<String> {
    let trimmed = rest.strip_prefix('/').unwrap_or(rest);
    let segments: Vec<&str> = trimmed.split('/').filter(|s| !s.is_empty()).collect();
    for idx in 0..segments.len() {
        if (segments[idx] == "work" || segments[idx] == "_work")
            && idx + 3 < segments.len()
            && segments[idx + 1] == segments[idx + 2]
        {
            return Some(segments[idx + 3..].join("/"));
        }
    }
    None
}
