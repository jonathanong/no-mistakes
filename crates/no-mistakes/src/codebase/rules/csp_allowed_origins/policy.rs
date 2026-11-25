use std::collections::BTreeMap;

mod origin;
use origin::{
    host_matches, normalize_default_port, scheme_matches, split_host_port_pattern, PortPattern,
};
pub(crate) use origin::{CspDirective, ParsedOrigin};

#[derive(Clone, Debug, PartialEq, Eq)]
enum CspSource {
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

#[derive(Default, Clone, Debug)]
pub(crate) struct CspPolicy {
    directives: BTreeMap<CspDirective, Vec<CspSource>>,
}

impl CspPolicy {
    pub(crate) fn is_empty(&self) -> bool {
        self.directives.is_empty()
    }

    pub(crate) fn merge(&mut self, other: Self) {
        for (directive, sources) in other.directives {
            self.directives.entry(directive).or_default().extend(sources);
        }
    }

    pub(crate) fn parse_from_strings(strings: &[String]) -> Self {
        let mut policy = Self::default();
        for text in strings {
            policy.parse_policy_fragment(text);
        }
        policy
    }

    pub(crate) fn parse_from_text(text: &str) -> Self {
        let mut policy = Self::default();
        for line in text.lines() {
            let trimmed = line.trim();
            if trimmed.starts_with('#') || trimmed.starts_with("//") {
                continue;
            }
            for fragment in extract_text_fragments(trimmed) {
                policy.parse_policy_fragment(fragment);
            }
        }
        policy
    }

    fn parse_policy_fragment(&mut self, text: &str) {
        for part in text.split(';') {
            let mut tokens = part.split_whitespace();
            let Some(name) = tokens.next() else {
                continue;
            };
            let Some(directive) = CspDirective::parse(name) else {
                continue;
            };
            let sources: Vec<CspSource> = tokens.filter_map(parse_source_token).collect();
            self.directives.entry(directive).or_default().extend(sources);
        }
    }

    pub(crate) fn allows(
        &self,
        directive: CspDirective,
        origin: &ParsedOrigin,
        self_origins: &[ParsedOrigin],
    ) -> bool {
        let Some(sources) = self.effective_sources(directive) else {
            return false;
        };
        sources
            .iter()
            .any(|source| source_matches(source, origin, self_origins))
    }

    fn effective_sources(&self, directive: CspDirective) -> Option<&[CspSource]> {
        if let Some(sources) = self.directives.get(&directive) {
            return Some(sources.as_slice());
        }
        if directive == CspDirective::Frame {
            if let Some(sources) = self.directives.get(&CspDirective::Child) {
                return Some(sources.as_slice());
            }
        }
        self.directives
            .get(&CspDirective::Default)
            .map(Vec::as_slice)
    }
}

fn extract_text_fragments(line: &str) -> Vec<&str> {
    let mut fragments = Vec::new();
    let mut rest = line;
    while let Some(idx) = rest.find('"') {
        rest = &rest[idx + 1..];
        let Some(end) = rest.find('"') else {
            break;
        };
        fragments.push(&rest[..end]);
        rest = &rest[end + 1..];
    }
    if fragments.is_empty() {
        if let Some((_, value)) = line.split_once(':') {
            fragments.push(value.trim());
        } else {
            fragments.push(line);
        }
    }
    fragments
}

fn parse_source_token(raw: &str) -> Option<CspSource> {
    let token = raw.trim().trim_end_matches(',');
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
    if host.is_empty() || (!host.contains('.') && host != "localhost") {
        return None;
    }
    let port = match (&scheme, port) {
        (Some(scheme), PortPattern::Exact(p)) => {
            PortPattern::Exact(normalize_default_port(scheme, p))
        }
        (_, other) => other,
    };
    Some(CspSource::Host {
        scheme,
        host: host.to_owned(),
        port,
    })
}

fn source_matches(
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
