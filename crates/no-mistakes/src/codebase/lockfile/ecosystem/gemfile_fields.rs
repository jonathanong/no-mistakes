pub(super) fn checksum_fingerprint(extra: &str) -> Option<String> {
    let mut parts = Vec::new();
    for token in extra.split(',') {
        let Some((algo, digest)) = token.trim().split_once('=') else {
            continue;
        };
        let Some(digest) = canonical_digest(algo, digest) else {
            continue;
        };
        parts.push((algo, digest));
    }
    if parts.is_empty() {
        return None;
    }
    if parts.len() == 1 && parts[0].0 == "sha256" {
        return Some(parts.swap_remove(0).1);
    }
    parts.sort_unstable();
    let mut fingerprint = String::new();
    for (algo, digest) in parts {
        if !fingerprint.is_empty() {
            fingerprint.push(',');
        }
        fingerprint.push_str(algo);
        fingerprint.push('=');
        fingerprint.push_str(&digest);
    }
    Some(fingerprint)
}

fn canonical_digest(algo: &str, digest: &str) -> Option<String> {
    if algo.is_empty() || !algo.bytes().all(|byte| byte.is_ascii_alphanumeric()) {
        return None;
    }
    // Bundler accepts a 43-character sha256 token plus up to two padding marks.
    if algo == "sha256" {
        if let Some(hex) = sha256_base64_hex(digest) {
            return Some(hex);
        }
    }
    if !digest.is_empty() && digest.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        return Some(digest.to_string());
    }
    None
}

fn sha256_base64_hex(digest: &str) -> Option<String> {
    let bytes = digest.as_bytes();
    let pad = bytes.iter().rev().take_while(|byte| **byte == b'=').count();
    if pad > 2 || bytes.len() != 43 + pad {
        return None;
    }
    let mut decoded = [0u8; 32];
    let mut acc = 0u32;
    let mut bits = 0u32;
    let mut at = 0usize;
    for &byte in &bytes[..43] {
        let six = b64_digit(byte)?;
        acc = (acc << 6) | u32::from(six);
        bits += 6;
        if bits >= 8 {
            bits -= 8;
            decoded[at] = (acc >> bits) as u8;
            at += 1;
            acc &= (1u32 << bits) - 1;
        }
    }
    Some(to_hex(&decoded))
}

fn b64_digit(byte: u8) -> Option<u8> {
    match byte {
        b'A'..=b'Z' => Some(byte - b'A'),
        b'a'..=b'z' => Some(byte - b'a' + 26),
        b'0'..=b'9' => Some(byte - b'0' + 52),
        b'+' | b'-' => Some(62),
        b'/' | b'_' => Some(63),
        _ => None,
    }
}

fn to_hex(bytes: &[u8; 32]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut hex = String::with_capacity(64);
    for byte in bytes {
        hex.push(HEX[(byte >> 4) as usize] as char);
        hex.push(HEX[(byte & 0x0f) as usize] as char);
    }
    hex
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
