use std::collections::HashSet;

pub(super) fn message(file: &str, domain: &str, replacement: Option<&str>) -> String {
    match replacement {
        Some(replacement) if !replacement.is_empty() => {
            format!("{file}: test email fixtures must not use `{domain}`; use `{replacement}`")
        }
        _ => format!("{file}: test email fixtures must not use `{domain}`"),
    }
}

pub(super) fn email_domain(value: &str) -> String {
    let decoded = value
        .replace("%40", "@")
        .replace("%2e", ".")
        .replace("%2E", ".")
        .to_lowercase();
    let Some(index) = decoded.rfind('@') else {
        return String::new();
    };
    let raw = &decoded[index + 1..];
    let mut end = 0usize;
    for (index, ch) in raw.char_indices() {
        if ch.is_ascii_alphanumeric() || ch == '.' || ch == '-' {
            end = index + ch.len_utf8();
        } else {
            break;
        }
    }
    normalize_domain(&raw[..end])
}

pub(super) fn normalize_domain(domain: &str) -> String {
    domain.trim().trim_end_matches('.').to_lowercase()
}

/// First DNS labels of banned domains. Empty when a domain has no label, so
/// the file-level prefilter must not skip (zero recall).
pub(super) fn banned_domain_needles(domains: &HashSet<String>) -> Vec<Box<[u8]>> {
    let mut needles = Vec::new();
    for domain in domains {
        let Some(label) = domain.split('.').next() else {
            return Vec::new();
        };
        if label.is_empty() {
            return Vec::new();
        }
        needles.push(label.as_bytes().to_vec().into_boxed_slice());
    }
    needles.sort();
    needles.dedup();
    needles
}
