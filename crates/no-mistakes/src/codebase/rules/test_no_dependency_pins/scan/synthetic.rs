//! Values that are fake by construction, so a test can use them without
//! pinning anything real.
//!
//! The exemption is deliberately structural rather than lexical: it never
//! keys off words such as `test` or `fake`, and `1.2.3` stays a pin.

/// Top-level domains reserved for documentation and testing (RFC 2606, 6761).
const RESERVED_TLDS: [&str; 4] = ["test", "example", "invalid", "localhost"];
const RESERVED_DOMAINS: [&str; 3] = ["example.com", "example.net", "example.org"];

/// `versions` is the part of `pin` the all-zero rule reads, normally the whole
/// pin. An action ref is reported whole but its zero versions are read in the
/// last two path components and the ref, so a `v0.0.0` directory higher up the
/// path cannot hide a real ref.
pub(super) fn is_synthetic(pin: &str, versions: &str) -> bool {
    only_zero_versions(versions) || reserved_registry(pin) || repeated_digest_only(pin)
}

/// True when every dotted version in the pin is `0.0.0`-shaped, as in
/// `lychee-v0.0.0-test-x86_64-unknown-linux-gnu.tar.gz`. Pins without a dotted
/// version are never exempt.
pub(super) fn only_zero_versions(pin: &str) -> bool {
    let mut found = false;
    for group in pin.split(|c: char| !c.is_ascii_digit() && c != '.') {
        let parts: Vec<&str> = group.split('.').collect();
        if parts.len() < 2 || parts.contains(&"") {
            continue;
        }
        found = true;
        if parts.iter().any(|part| part.bytes().any(|b| b != b'0')) {
            return false;
        }
    }
    found
}

/// True for `registry.test/app:1.2.3`, `localhost:5000/app:1.2.3`, and
/// `example.com/app:1.2.3`: hosts that can never serve a real dependency. A
/// hostname is case-insensitive and may end in the DNS root dot, so
/// `EXAMPLE.COM.` is as reserved as `example.com`.
fn reserved_registry(pin: &str) -> bool {
    let Some((authority, _)) = pin.split_once('/') else {
        return false;
    };
    let host = authority
        .split(':')
        .next()
        .unwrap_or_default()
        .to_ascii_lowercase();
    let host = host.strip_suffix('.').unwrap_or(&host);
    host == "localhost"
        || host
            .rsplit_once('.')
            .is_some_and(|(_, tld)| RESERVED_TLDS.contains(&tld))
        || RESERVED_DOMAINS
            .iter()
            .any(|domain| host == *domain || host.ends_with(&format!(".{domain}")))
}

/// True for an untagged `repo@sha256:<64 hex>` whose digest is one short block
/// repeated (`aaaa...`, `0000...`, `0123456789abcdef` x4). A real SHA-256 has
/// no period of 32 characters or fewer. A tagged image is judged by its tag.
fn repeated_digest_only(pin: &str) -> bool {
    let Some((name, digest)) = pin.split_once("@sha256:") else {
        return false;
    };
    let tagged = name.rsplit('/').next().unwrap_or_default().contains(':');
    let digest = digest.as_bytes();
    !tagged && digest.len() == 64 && (1..=32).any(|p| digest[p..] == digest[..digest.len() - p])
}

#[cfg(test)]
mod tests;
