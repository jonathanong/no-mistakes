mod cargo;
mod poetry;
mod uv;

use super::super::ResolvedPackage;

#[derive(Clone, Copy)]
pub(crate) enum TomlKind {
    Cargo,
    Poetry,
    Uv,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct TomlError;

pub(crate) fn parse(kind: TomlKind, content: &str) -> Vec<ResolvedPackage> {
    parse_checked(kind, content).unwrap_or_default()
}

pub(crate) fn parse_checked(
    kind: TomlKind,
    content: &str,
) -> Result<Vec<ResolvedPackage>, TomlError> {
    match kind {
        TomlKind::Cargo => cargo::parse(content),
        TomlKind::Poetry => poetry::parse(content),
        TomlKind::Uv => uv::parse(content),
    }
}

fn git_fingerprint(pin: String, subdirectory: Option<&str>) -> String {
    match subdirectory.filter(|value| !value.is_empty()) {
        Some(path) => format!("{pin}#subdirectory={path}"),
        None => pin,
    }
}
