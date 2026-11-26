use super::super::{ResolutionKind, ResolvedPackage};
use text::{apply_source_key, apply_source_value, assignment, hash_on_line, token};

mod text;

#[derive(Clone, Copy)]
pub(crate) enum TomlKind {
    Cargo,
    Poetry,
    Uv,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct TomlError;

#[derive(Clone, Copy, PartialEq, Eq)]
enum Section {
    Outside,
    Body,
    Source,
    Skip,
    MetaFiles,
}

pub(super) struct Draft {
    pub(super) name: String,
    pub(super) version: String,
    pub(super) fingerprint: String,
    pub(super) kind: ResolutionKind,
    active: bool,
}

pub(crate) fn parse(kind: TomlKind, content: &str) -> Vec<ResolvedPackage> {
    parse_checked(kind, content).unwrap_or_default()
}

pub(crate) fn parse_checked(
    kind: TomlKind,
    content: &str,
) -> Result<Vec<ResolvedPackage>, TomlError> {
    let mut out = Vec::new();
    let mut draft = Draft::inactive();
    let mut section = Section::Outside;
    let mut saw_header = false;
    let mut meta_name: Option<String> = None;

    for raw in content.lines() {
        let trimmed = raw.trim().trim_end_matches('\r');
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        if trimmed == "[[package]]" {
            finish(&mut draft, &mut out)?;
            draft = Draft::start(kind);
            section = Section::Body;
            continue;
        }
        if let Some(next) = table(trimmed, &mut draft, &mut out, &mut saw_header)? {
            section = next;
            continue;
        }
        match section {
            Section::Outside => note_header(trimmed, &mut saw_header),
            Section::Body | Section::Source | Section::Skip => {
                apply_package_line(section, trimmed, &mut draft);
            }
            Section::MetaFiles => apply_meta_file(trimmed, &mut meta_name, &mut out),
        }
    }
    finish(&mut draft, &mut out)?;
    if out.is_empty() && !saw_header {
        return Err(TomlError);
    }
    Ok(out)
}

fn table(
    trimmed: &str,
    draft: &mut Draft,
    out: &mut Vec<ResolvedPackage>,
    saw_header: &mut bool,
) -> Result<Option<Section>, TomlError> {
    if !trimmed.starts_with('[') {
        return Ok(None);
    }
    if trimmed.starts_with("[package.") || trimmed == "[package]" {
        let section = if trimmed.starts_with("[package.source]") {
            Section::Source
        } else {
            Section::Skip
        };
        return Ok(Some(section));
    }
    finish(draft, out)?;
    *saw_header |= trimmed.starts_with("[metadata");
    let section = if trimmed.starts_with("[metadata.files]") {
        Section::MetaFiles
    } else {
        Section::Outside
    };
    Ok(Some(section))
}

fn note_header(trimmed: &str, saw_header: &mut bool) {
    if let Some(("version", value)) = assignment(trimmed) {
        if token(value).is_some_and(|value| value.chars().all(|c| c.is_ascii_digit())) {
            *saw_header = true;
        }
    }
}

fn apply_meta_file(trimmed: &str, meta_name: &mut Option<String>, out: &mut [ResolvedPackage]) {
    if let Some((key, value)) = assignment(trimmed) {
        if value.starts_with('[') && !key.contains('{') {
            *meta_name = Some(key.to_string());
        }
    }
    if let Some(hash) = hash_on_line(trimmed) {
        if let Some(name) = meta_name.as_deref() {
            if let Some(pkg) = out.iter_mut().find(|pkg| pkg.name == name) {
                if pkg.fingerprint.is_empty() {
                    pkg.fingerprint = hash.to_string();
                }
            }
        }
    }
}

fn apply_package_line(section: Section, trimmed: &str, draft: &mut Draft) {
    if let Some(hash) = hash_on_line(trimmed) {
        if draft.fingerprint.is_empty() {
            draft.fingerprint = hash.to_string();
        }
    }
    let Some((key, value)) = assignment(trimmed) else {
        return;
    };
    if section == Section::Source {
        apply_source_key(key, value, draft);
        return;
    }
    if section != Section::Body {
        return;
    }
    match key {
        "name" => {
            if let Some(text) = token(value) {
                draft.name = text.to_string();
            }
        }
        "version" => {
            if let Some(text) = token(value) {
                draft.version = text.to_string();
            }
        }
        "checksum" => {
            if let Some(text) = token(value) {
                draft.fingerprint = text.to_string();
            }
        }
        "source" => apply_source_value(value, draft),
        _ => {}
    }
}

fn finish(draft: &mut Draft, out: &mut Vec<ResolvedPackage>) -> Result<(), TomlError> {
    if !draft.active {
        return Ok(());
    }
    if draft.name.is_empty() || draft.version.is_empty() {
        return Err(TomlError);
    }
    out.push(ResolvedPackage {
        name: std::mem::take(&mut draft.name),
        version: std::mem::take(&mut draft.version),
        fingerprint: std::mem::take(&mut draft.fingerprint),
        kind: draft.kind,
    });
    draft.active = false;
    Ok(())
}
