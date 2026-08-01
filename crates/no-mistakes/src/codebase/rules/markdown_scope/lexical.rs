use std::path::Path;

/// Return a portable lexical relative path when both paths have compatible
/// roots. Windows path syntax is parsed independently of the host OS so a
/// cross-volume finding never becomes a misleading `../../` traversal.
pub(crate) fn lexical_relative_slash_path(root: &Path, path: &Path) -> Option<String> {
    let root = LexicalPath::parse(root);
    let path = LexicalPath::parse(path);
    root.prefix.compatible_with(&path.prefix).then(|| {
        let common = root
            .components
            .iter()
            .zip(&path.components)
            .take_while(|(left, right)| root.prefix.component_eq(left, right))
            .count();
        std::iter::repeat_n("..".to_string(), root.components.len() - common)
            .chain(path.components[common..].iter().cloned())
            .collect::<Vec<_>>()
            .join("/")
    })
}

pub(super) fn lexical_normalized_slash_path(path: &Path) -> String {
    LexicalPath::parse(path).render()
}

#[derive(Debug, PartialEq, Eq)]
enum LexicalPrefix {
    Relative,
    Posix,
    Drive(String),
    Unc(String, String),
}

impl LexicalPrefix {
    fn compatible_with(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Relative, Self::Relative) | (Self::Posix, Self::Posix) => true,
            (Self::Drive(left), Self::Drive(right)) => left.eq_ignore_ascii_case(right),
            (Self::Unc(left_server, left_share), Self::Unc(right_server, right_share)) => {
                left_server.eq_ignore_ascii_case(right_server)
                    && left_share.eq_ignore_ascii_case(right_share)
            }
            _ => false,
        }
    }

    fn component_eq(&self, left: &str, right: &str) -> bool {
        match self {
            Self::Drive(_) | Self::Unc(_, _) => left.eq_ignore_ascii_case(right),
            Self::Relative | Self::Posix => left == right,
        }
    }
}

struct LexicalPath {
    prefix: LexicalPrefix,
    components: Vec<String>,
}

impl LexicalPath {
    fn parse(path: &Path) -> Self {
        let raw = path.to_string_lossy().replace('\\', "/");
        let (prefix, remainder) = if let Some(remainder) = raw.strip_prefix("//") {
            let mut parts = remainder.splitn(3, '/');
            match (parts.next(), parts.next()) {
                (Some(server), Some(share)) if !server.is_empty() && !share.is_empty() => (
                    LexicalPrefix::Unc(server.to_string(), share.to_string()),
                    parts.next().unwrap_or_default(),
                ),
                _ => (LexicalPrefix::Posix, remainder),
            }
        } else if raw.len() >= 3
            && raw.as_bytes()[0].is_ascii_alphabetic()
            && raw.as_bytes()[1] == b':'
            && raw.as_bytes()[2] == b'/'
        {
            (
                LexicalPrefix::Drive((raw.as_bytes()[0] as char).to_string()),
                &raw[3..],
            )
        } else if let Some(remainder) = raw.strip_prefix('/') {
            (LexicalPrefix::Posix, remainder)
        } else {
            (LexicalPrefix::Relative, raw.as_str())
        };
        let mut components = Vec::new();
        for component in remainder.split('/') {
            match component {
                "" | "." => {}
                ".." if components.last().is_some_and(|part| part != "..") => {
                    components.pop();
                }
                ".." if matches!(&prefix, LexicalPrefix::Relative) => {
                    components.push(component.to_string());
                }
                ".." => {}
                component => components.push(component.to_string()),
            }
        }
        Self { prefix, components }
    }

    fn render(&self) -> String {
        let base = match &self.prefix {
            LexicalPrefix::Relative => String::new(),
            LexicalPrefix::Posix => "/".to_string(),
            LexicalPrefix::Drive(drive) => format!("{drive}:/"),
            LexicalPrefix::Unc(server, share) => format!("//{server}/{share}"),
        };
        if self.components.is_empty() {
            return base;
        }
        match &self.prefix {
            LexicalPrefix::Unc(_, _) => format!("{base}/{}", self.components.join("/")),
            LexicalPrefix::Relative => self.components.join("/"),
            LexicalPrefix::Posix | LexicalPrefix::Drive(_) => {
                format!("{base}{}", self.components.join("/"))
            }
        }
    }
}
