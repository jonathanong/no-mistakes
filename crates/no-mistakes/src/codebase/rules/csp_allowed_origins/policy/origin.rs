#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub(crate) enum CspDirective {
    Default,
    Script,
    Style,
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
            Self::Style => "style-src",
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
            "script-src" | "script-src-elem" => Some(Self::Script),
            "style-src" | "style-src-elem" => Some(Self::Style),
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

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct ParsedOrigin {
    pub(crate) scheme: String,
    pub(crate) host: String,
    pub(crate) port: Option<u16>,
}

impl ParsedOrigin {
    pub(crate) fn parse(raw: &str) -> Option<Self> {
        let trimmed = raw.trim();
        let (scheme, rest) = trimmed.split_once("://")?;
        let scheme = scheme.to_ascii_lowercase();
        if !matches!(scheme.as_str(), "http" | "https" | "ws" | "wss") {
            return None;
        }
        let authority = rest.split(['/', '?', '#']).next()?;
        if authority.is_empty() || authority.contains('@') {
            return None;
        }
        let (host, port) = split_host_port(authority)?;
        if host.is_empty() || host.contains(char::is_whitespace) {
            return None;
        }
        let port = normalize_default_port(&scheme, port);
        Some(Self {
            scheme,
            host: host.to_ascii_lowercase(),
            port,
        })
    }

    pub(crate) fn display(&self) -> String {
        match self.port {
            Some(port) => format!("{}://{}:{port}", self.scheme, self.host),
            None => format!("{}://{}", self.scheme, self.host),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) enum PortPattern {
    Any,
    Exact(Option<u16>),
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

pub(super) fn split_host_port(authority: &str) -> Option<(&str, Option<u16>)> {
    if let Some((host, port)) = authority.rsplit_once(':') {
        let parsed = port.parse::<u16>().ok()?;
        Some((host, Some(parsed)))
    } else {
        Some((authority, None))
    }
}

pub(super) fn split_host_port_pattern(authority: &str) -> Option<(&str, PortPattern)> {
    if let Some((host, port)) = authority.rsplit_once(':') {
        if port == "*" {
            return Some((host, PortPattern::Any));
        }
        let parsed = port.parse::<u16>().ok()?;
        Some((host, PortPattern::Exact(Some(parsed))))
    } else {
        Some((authority, PortPattern::Exact(None)))
    }
}

pub(super) fn normalize_default_port(scheme: &str, port: Option<u16>) -> Option<u16> {
    match (scheme, port) {
        ("http" | "ws", Some(80)) | ("https" | "wss", Some(443)) => None,
        (_, other) => other,
    }
}
