//! Lexical path cleaning and prefix matching for LCOV `SF:` normalization.

use crate::normalize::PathNormalizeError;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum PathKind {
    Relative,
    PosixAbs,
    WindowsDrive(u8),
    Unc,
}

pub(super) struct CleanedPath {
    pub(super) kind: PathKind,
    pub(super) text: String,
}

impl CleanedPath {
    pub(super) fn suffix_after_drive(&self) -> &str {
        match self.kind {
            PathKind::WindowsDrive(_) => &self.text[2..],
            _ => &self.text,
        }
    }
}

fn clean_segments(rest: &str, allow_pop_at_root: bool) -> Result<Vec<&str>, PathNormalizeError> {
    let mut stack = Vec::new();
    for seg in rest.split('/') {
        match seg {
            "" | "." => {}
            ".." => {
                if stack.pop().is_none() && !allow_pop_at_root {
                    return Err(PathNormalizeError::EscapesRoot);
                }
            }
            _ => stack.push(seg),
        }
    }
    Ok(stack)
}

pub(super) fn clean_path(raw: &str) -> Result<CleanedPath, PathNormalizeError> {
    let slashed = raw.replace('\\', "/");
    let bytes = slashed.as_bytes();

    if slashed.starts_with("//") && !slashed.starts_with("///") {
        return clean_unc_path(&slashed[2..]);
    }

    if bytes.len() >= 2
        && bytes[0].is_ascii_alphabetic()
        && bytes[1] == b':'
        && (bytes.len() == 2 || bytes[2] == b'/')
    {
        let drive_char = bytes[0] as char;
        let segs = clean_segments(&slashed[2..], true)?;
        return Ok(CleanedPath {
            kind: PathKind::WindowsDrive(bytes[0].to_ascii_lowercase()),
            text: format!("{drive_char}:/{}", segs.join("/")),
        });
    }

    if slashed.starts_with('/') {
        let segs = clean_segments(&slashed, true)?;
        return Ok(CleanedPath {
            kind: PathKind::PosixAbs,
            text: format!("/{}", segs.join("/")),
        });
    }

    let segs = clean_segments(&slashed, false)?;
    Ok(CleanedPath {
        kind: PathKind::Relative,
        text: segs.join("/"),
    })
}

fn clean_unc_path(after_slashes: &str) -> Result<CleanedPath, PathNormalizeError> {
    let Some((server, rest)) = after_slashes.split_once('/') else {
        return Err(PathNormalizeError::Empty);
    };
    let (share, tail) = rest.split_once('/').unwrap_or((rest, ""));
    if server.is_empty() || share.is_empty() {
        return Err(PathNormalizeError::Empty);
    }
    let segs = clean_segments(tail, true)?;
    let text = if segs.is_empty() {
        format!("//{server}/{share}")
    } else {
        format!("//{server}/{share}/{}", segs.join("/"))
    };
    Ok(CleanedPath {
        kind: PathKind::Unc,
        text,
    })
}

pub(super) fn strip_prefix_match(
    path: &CleanedPath,
    prefix: &CleanedPath,
) -> Option<Result<String, PathNormalizeError>> {
    if path.kind != prefix.kind {
        return None;
    }
    let target = path.suffix_after_drive();
    let base = prefix.suffix_after_drive();
    if target == base {
        return Some(Err(PathNormalizeError::Empty));
    }
    if base == "/" {
        return Some(Ok(target.trim_start_matches('/').to_string()));
    }
    if target.starts_with(base) && target.as_bytes().get(base.len()) == Some(&b'/') {
        return Some(Ok(target[base.len() + 1..]
            .trim_start_matches('/')
            .to_string()));
    }
    None
}

pub(super) fn is_foreign_absolute(path: PathKind, root: PathKind) -> bool {
    matches!(
        (path, root),
        (
            PathKind::PosixAbs,
            PathKind::WindowsDrive(_) | PathKind::Unc
        ) | (
            PathKind::WindowsDrive(_) | PathKind::Unc,
            PathKind::PosixAbs
        )
    )
}
