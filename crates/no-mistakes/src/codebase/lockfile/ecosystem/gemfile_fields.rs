pub(super) fn checksum_fingerprint(extra: &str) -> Option<String> {
    let mut parts = Vec::new();
    for token in extra.split(',') {
        let Some((algo, digest)) = token.trim().split_once('=') else {
            continue;
        };
        if algo.is_empty()
            || digest.is_empty()
            || !algo.chars().all(|ch| ch.is_ascii_alphanumeric())
            || !digest.chars().all(|ch| ch.is_ascii_hexdigit())
        {
            continue;
        }
        parts.push((algo, digest));
    }
    if parts.is_empty() {
        return None;
    }
    if parts.len() == 1 && parts[0].0 == "sha256" {
        return Some(parts[0].1.to_string());
    }
    parts.sort_unstable();
    let mut fingerprint = String::new();
    for (algo, digest) in parts {
        if !fingerprint.is_empty() {
            fingerprint.push(',');
        }
        fingerprint.push_str(algo);
        fingerprint.push('=');
        fingerprint.push_str(digest);
    }
    Some(fingerprint)
}

pub(super) fn is_gem_version(version: &str) -> bool {
    let (release, suffix) = match version.split_once('-') {
        Some((release, suffix)) => (release, Some(suffix)),
        None => (version, None),
    };
    let mut segments = release.split('.');
    let Some(first) = segments.next() else {
        return false;
    };
    if first.is_empty() || !first.bytes().all(|byte| byte.is_ascii_digit()) {
        return false;
    }
    if !segments.all(|segment| {
        !segment.is_empty() && segment.bytes().all(|byte| byte.is_ascii_alphanumeric())
    }) {
        return false;
    }
    suffix.is_none_or(|suffix| {
        !suffix.is_empty()
            && !suffix.starts_with('-')
            && !suffix.starts_with('.')
            && !suffix.ends_with('-')
            && !suffix.ends_with('.')
            && suffix.bytes().all(|byte| {
                byte.is_ascii_alphanumeric() || byte == b'-' || byte == b'.' || byte == b'_'
            })
    })
}
