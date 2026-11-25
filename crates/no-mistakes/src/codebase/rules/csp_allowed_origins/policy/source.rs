use super::origin::{
    host_matches, is_valid_csp_host, normalize_default_port, scheme_matches,
    split_host_port_pattern, ParsedOrigin, PortPattern,
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) enum CspSource {
    Any,
    SelfKeyword,
    NoneKeyword,
    Scheme(String),
    Host {
        scheme: Option<String>,
        host: String,
        port: PortPattern,
    },
}

pub(super) fn parse_source_token(raw: &str) -> Option<CspSource> {
    let token = raw.trim();
    if token.is_empty() {
        return None;
    }
    if token == "*" {
        return Some(CspSource::Any);
    }
    let lower = token.to_ascii_lowercase();
    if lower == "'self'" {
        return Some(CspSource::SelfKeyword);
    }
    if lower == "'none'" {
        return Some(CspSource::NoneKeyword);
    }
    if lower.starts_with('\'') && lower.ends_with('\'') {
        return None;
    }
    if let Some(scheme) = lower.strip_suffix(':') {
        return matches!(scheme, "http" | "https" | "ws" | "wss")
            .then(|| CspSource::Scheme(scheme.to_owned()));
    }
    let (scheme, authority_and_path) = match lower.split_once("://") {
        Some((scheme, rest)) if matches!(scheme, "http" | "https" | "ws" | "wss") => {
            (Some(scheme.to_owned()), rest)
        }
        Some(_) => return None,
        None => (None, lower.as_str()),
    };
    let authority = authority_and_path.split(['/', '?', '#']).next()?;
    let (host, port) = split_host_port_pattern(authority)?;
    if !is_valid_csp_host(&host) {
        return None;
    }
    let port = match (&scheme, port) {
        (Some(scheme), PortPattern::Exact(p)) => {
            PortPattern::Exact(normalize_default_port(scheme, p))
        }
        (_, other) => other,
    };
    Some(CspSource::Host { scheme, host, port })
}

pub(super) fn source_matches(
    source: &CspSource,
    origin: &ParsedOrigin,
    self_origins: &[ParsedOrigin],
) -> bool {
    match source {
        CspSource::Any => true,
        CspSource::SelfKeyword => self_origins.contains(origin),
        CspSource::NoneKeyword => false,
        CspSource::Scheme(scheme) => scheme_matches(scheme, &origin.scheme),
        CspSource::Host { scheme, host, port } => {
            let scheme_ok = scheme
                .as_deref()
                .is_none_or(|expected| scheme_matches(expected, &origin.scheme));
            let host_ok = host_matches(host, &origin.host);
            let port_ok = match port {
                PortPattern::Any => true,
                PortPattern::Exact(expected) => {
                    normalize_default_port(&origin.scheme, *expected) == origin.port
                }
            };
            scheme_ok && host_ok && port_ok
        }
    }
}
