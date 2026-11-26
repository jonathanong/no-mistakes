use std::net::Ipv6Addr;

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum CspDirective {
    Default,
    Script,
    ScriptElem,
    Style,
    StyleElem,
    Img,
    Connect,
    Frame,
    Child,
    Font,
    Media,
}

impl CspDirective {
    pub(crate) fn as_str(self) -> &'static str {
        match self {
            Self::Default => "default-src",
            Self::Script => "script-src",
            Self::ScriptElem => "script-src-elem",
            Self::Style => "style-src",
            Self::StyleElem => "style-src-elem",
            Self::Img => "img-src",
            Self::Connect => "connect-src",
            Self::Frame => "frame-src",
            Self::Child => "child-src",
            Self::Font => "font-src",
            Self::Media => "media-src",
        }
    }

    pub(crate) fn parse(token: &str) -> Option<Self> {
        match token.to_ascii_lowercase().as_str() {
            "default-src" => Some(Self::Default),
            "script-src" => Some(Self::Script),
            "script-src-elem" => Some(Self::ScriptElem),
            "style-src" => Some(Self::Style),
            "style-src-elem" => Some(Self::StyleElem),
            "img-src" => Some(Self::Img),
            "connect-src" => Some(Self::Connect),
            "frame-src" => Some(Self::Frame),
            "child-src" => Some(Self::Child),
            "font-src" => Some(Self::Font),
            "media-src" => Some(Self::Media),
            _ => None,
        }
    }
}

#[derive(PartialEq, Eq)]
pub(crate) struct ParsedOrigin {
    pub(crate) scheme: String,
    pub(crate) host: String,
    pub(crate) port: Option<u16>,
}

impl ParsedOrigin {
    pub(crate) fn parse(raw: &str) -> Option<Self> {
        let trimmed = raw.trim();
        let (scheme, rest) = if let Some(rest) = trimmed.strip_prefix("//") {
            ("https".to_owned(), rest)
        } else {
            let (scheme, rest) = trimmed.split_once("://")?;
            let scheme = scheme.to_ascii_lowercase();
            if !matches!(scheme.as_str(), "http" | "https" | "ws" | "wss") {
                return None;
            }
            (scheme, rest)
        };
        let authority = authority_prefix(rest);
        if authority.is_empty() || authority.contains('@') {
            return None;
        }
        let (host, port) = split_host_port(authority)?;
        if host.is_empty() || host.contains(char::is_whitespace) {
            return None;
        }
        let port = normalize_default_port(&scheme, port);
        Some(Self { scheme, host, port })
    }

    pub(crate) fn is_network_like_url(raw: &str) -> bool {
        let trimmed = raw.trim();
        trimmed.starts_with("//")
            || trimmed.split_once("://").is_some_and(|(scheme, _)| {
                matches!(
                    scheme.to_ascii_lowercase().as_str(),
                    "http" | "https" | "ws" | "wss"
                )
            })
    }

    pub(crate) fn display(&self) -> String {
        match self.port {
            Some(port) => format!("{}://{}:{port}", self.scheme, self.host),
            None => format!("{}://{}", self.scheme, self.host),
        }
    }
}

pub(super) enum PortPattern {
    Any,
    Exact(Option<u16>),
}

pub(super) fn authority_prefix(rest: &str) -> &str {
    let end = rest.find(['/', '?', '#']).unwrap_or(rest.len());
    &rest[..end]
}

pub(super) fn scheme_matches(allowed: &str, actual: &str) -> bool {
    allowed == actual
        || (allowed == "http" && actual == "https")
        || (allowed == "ws" && actual == "wss")
}

pub(super) fn host_matches(pattern: &str, actual: &str) -> bool {
    if let Some(suffix) = pattern.strip_prefix("*.") {
        actual.len() > suffix.len()
            && actual.ends_with(suffix)
            && actual.as_bytes()[actual.len() - suffix.len() - 1] == b'.'
    } else {
        pattern == actual
    }
}

pub(super) fn is_valid_csp_host(host: &str) -> bool {
    !host.is_empty()
        && (host.contains('.')
            || host == "localhost"
            || (host.starts_with('[') && host.ends_with(']')))
}

fn split_authority(authority: &str) -> Option<(String, Option<&str>)> {
    if authority.starts_with('[') {
        let end = authority.find(']')?;
        let ip: Ipv6Addr = authority[1..end].parse().ok()?;
        let rest = &authority[end + 1..];
        let port = if rest.is_empty() {
            None
        } else {
            Some(rest.strip_prefix(':')?)
        };
        return Some((format!("[{ip}]"), port));
    }
    if authority.contains(['[', ']']) {
        return None;
    }
    match authority.rsplit_once(':') {
        Some((host, port)) if !host.contains(':') => Some((host.to_ascii_lowercase(), Some(port))),
        Some(_) => None,
        None => Some((authority.to_ascii_lowercase(), None)),
    }
}

pub(super) fn split_host_port(authority: &str) -> Option<(String, Option<u16>)> {
    let (host, port) = split_authority(authority)?;
    let port = port.map(str::parse::<u16>).transpose().ok()?;
    Some((host, port))
}

pub(super) fn split_host_port_pattern(authority: &str) -> Option<(String, PortPattern)> {
    let (host, port) = split_authority(authority)?;
    let port = match port {
        None => PortPattern::Exact(None),
        Some("*") => PortPattern::Any,
        Some(raw) => PortPattern::Exact(Some(raw.parse::<u16>().ok()?)),
    };
    Some((host, port))
}

pub(super) fn normalize_default_port(scheme: &str, port: Option<u16>) -> Option<u16> {
    match (scheme, port) {
        ("http" | "ws", Some(80)) | ("https" | "wss", Some(443)) => None,
        (_, other) => other,
    }
}
