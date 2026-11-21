//! Scan-root classification for `cwd`, a shell `cd`, and a package-manager directory.
use super::super::provenance::{classify, Verdict};
use super::command::OptionsSlot;
use crate::codebase::ts_source::facts::TsFileFacts;
use crate::codebase::ts_source::module_facts::arguments::PathOrigin;

#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum Directory {
    Allowed,
    Live,
    Unresolved,
}

pub(super) fn owned_shift(
    base: Directory,
    wrapped: Option<&str>,
    shell: Option<&str>,
) -> Option<String> {
    if base != Directory::Allowed {
        return None;
    }
    match (wrapped, shell) {
        (Some(dir), _) => Some(format!("in `{dir}`")),
        (None, Some(dir)) => Some(format!("after `cd {dir}`")),
        (None, None) => None,
    }
}

pub(super) fn live_message(callee: &str, label: &str, shift: Option<String>) -> String {
    match shift {
        Some(dir) => format!(
            "live repository scan: `{callee}` runs `{label}` {dir}, and a test root reaches it. `analyzers` names that command. Fix: run it in static CI, or remove that directory change so the owned `cwd` stays the scan root."
        ),
        None => format!(
            "live repository scan: `{callee}` runs `{label}` against the working directory, and a test root reaches it. `analyzers` names that command, and this call passes no owned `cwd`. Fix: run it in static CI, or set `cwd` to a fixtures path or an mkdtemp directory the test owns."
        ),
    }
}

pub(super) fn shell_place(dir: &str, fixture_dirs: &[&str], base: Directory) -> Directory {
    if dir.is_empty() || dir.contains(['$', '`']) {
        return Directory::Unresolved;
    }
    let absolute = dir.starts_with(['/', '\\']);
    let segments: Vec<&str> = dir
        .split(['/', '\\'])
        .filter(|segment| !segment.is_empty() && *segment != ".")
        .collect();
    if segments.is_empty() {
        return if absolute { Directory::Live } else { base };
    }
    if segments.contains(&"..") {
        return if segments.iter().all(|segment| *segment == "..") && base != Directory::Allowed {
            Directory::Live
        } else {
            Directory::Unresolved
        };
    }
    if segments
        .iter()
        .any(|segment| fixture_dirs.contains(segment))
    {
        Directory::Allowed
    } else {
        Directory::Live
    }
}

pub(super) fn directory(
    facts: &TsFileFacts,
    offset: u32,
    options: OptionsSlot,
    fixture_dirs: &[&str],
) -> Directory {
    let OptionsSlot::Object(index) = options else {
        return match options {
            OptionsSlot::Unknown => Directory::Unresolved,
            _ => Directory::Live,
        };
    };
    match cwd_origin(facts, offset, index) {
        Some(origin) => match classify(Some(origin), fixture_dirs) {
            Verdict::Allowed => Directory::Allowed,
            Verdict::Live(_) => Directory::Live,
            Verdict::Unresolved(_) => Directory::Unresolved,
        },
        None if object_closed(facts, offset, index) => Directory::Live,
        None => Directory::Unresolved,
    }
}

fn object_closed(facts: &TsFileFacts, offset: u32, index: usize) -> bool {
    facts
        .module_bindings
        .as_deref()
        .and_then(|bindings| bindings.path_provenance.as_deref())
        .and_then(|provenance| provenance.arg_closed_objects.get(&offset))
        .is_some_and(|indexes| indexes.contains(&index))
}

fn cwd_origin(facts: &TsFileFacts, offset: u32, index: usize) -> Option<&PathOrigin> {
    facts
        .module_bindings
        .as_deref()?
        .path_provenance
        .as_deref()?
        .arg_member_paths
        .get(&offset)?
        .iter()
        .find_map(|(member_index, key, origin)| {
            (*member_index == index && key == "cwd").then_some(origin)
        })
}
