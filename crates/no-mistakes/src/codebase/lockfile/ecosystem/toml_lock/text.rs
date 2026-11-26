use super::super::super::ResolutionKind;
use super::{Draft, TomlKind};

pub(super) fn apply_source_key(key: &str, value: &str, draft: &mut Draft) {
    let Some(text) = token(value) else {
        return;
    };
    match key {
        "type" => match text {
            "git" => draft.kind = ResolutionKind::Git,
            "directory" => draft.kind = ResolutionKind::Directory,
            "url" => draft.kind = ResolutionKind::Tarball,
            _ => {}
        },
        "reference" if draft.fingerprint.is_empty() => {
            draft.fingerprint = text.to_string();
        }
        _ => {}
    }
}

pub(super) fn apply_source_value(value: &str, draft: &mut Draft) {
    let trimmed = value.trim();
    if trimmed.starts_with('{') {
        apply_inline(trimmed, draft);
        return;
    }
    let Some(source) = quoted(trimmed) else {
        return;
    };
    if source.starts_with("registry+") || source.starts_with("sparse+") {
        draft.kind = ResolutionKind::Registry;
    } else if source.starts_with("git+") {
        draft.kind = ResolutionKind::Git;
        if draft.fingerprint.is_empty() {
            draft.fingerprint = source.to_string();
        }
    } else if source.starts_with("path+") {
        draft.kind = ResolutionKind::Directory;
    } else {
        draft.kind = ResolutionKind::Other;
        if draft.fingerprint.is_empty() {
            draft.fingerprint = source.to_string();
        }
    }
}

fn apply_inline(value: &str, draft: &mut Draft) {
    let inner = value
        .trim()
        .trim_start_matches('{')
        .trim_end_matches(['}', ',', ' ']);
    for part in inner.split(',') {
        let Some((key, raw)) = part.split_once('=') else {
            continue;
        };
        let Some(text) = quoted(raw) else {
            continue;
        };
        match key.trim() {
            "git" => draft.kind = ResolutionKind::Git,
            "rev" if draft.fingerprint.is_empty() => {
                draft.fingerprint = text.to_string();
            }
            "editable" => draft.kind = ResolutionKind::Directory,
            "registry" => draft.kind = ResolutionKind::Registry,
            _ => {}
        }
    }
}

pub(super) fn assignment(line: &str) -> Option<(&str, &str)> {
    let (key, value) = line.split_once('=')?;
    Some((key.trim(), value.trim()))
}

pub(super) fn token(value: &str) -> Option<&str> {
    let value = value.trim().trim_end_matches(',').trim();
    if let Some(inner) = quoted(value) {
        return Some(inner);
    }
    value.split_whitespace().next()
}

fn quoted(value: &str) -> Option<&str> {
    let value = value.trim().trim_end_matches(',').trim();
    if !value.starts_with('"') {
        return None;
    }
    let end = value.rfind('"')?;
    if end == 0 {
        return None;
    }
    Some(&value[1..end])
}

impl Draft {
    pub(super) fn inactive() -> Self {
        Self {
            name: String::new(),
            version: String::new(),
            fingerprint: String::new(),
            kind: ResolutionKind::Registry,
            active: false,
        }
    }

    pub(super) fn start(kind: TomlKind) -> Self {
        Self {
            name: String::new(),
            version: String::new(),
            fingerprint: String::new(),
            kind: match kind {
                TomlKind::Cargo => ResolutionKind::Workspace,
                TomlKind::Poetry | TomlKind::Uv => ResolutionKind::Registry,
            },
            active: true,
        }
    }
}

pub(super) fn hash_on_line(line: &str) -> Option<&str> {
    let mut rest = line;
    while let Some(idx) = rest.find("hash") {
        let after = rest[idx + 4..].trim_start();
        if let Some(value) = after.strip_prefix('=') {
            return quoted(value);
        }
        rest = &rest[idx + 4..];
    }
    None
}
