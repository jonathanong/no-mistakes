//! Which call sites enumerate a directory tree.
use super::config::Enumerator;
use crate::codebase::dependencies::graph::{KnownCallTargetRef, ResolvedCallSite};

pub(super) const FS_MODULES: &[&str] = &["fs", "fs/promises", "fs-extra"];
const FS_ENUMERATORS: &[&str] = &[
    "readdir",
    "readdirSync",
    "opendir",
    "opendirSync",
    "glob",
    "globSync",
];
pub(super) const GLOB_MODULES: &[&str] = &["glob", "fast-glob", "tinyglobby", "globby"];
/// Exports of the glob packages that only inspect or rewrite a pattern.
const GLOB_PURE: &[&str] = &[
    "escapePath",
    "convertPathToPattern",
    "isDynamicPattern",
    "hasMagic",
    "escape",
    "unescape",
    "generateGlobTasks",
    "isGitIgnored",
];
/// Method names that enumerate when the receiver could not be resolved.
const UNRESOLVED_NAMES: &[&str] = &[
    "readdir",
    "readdirSync",
    "opendir",
    "opendirSync",
    "globSync",
];

pub(super) enum Scan {
    /// A call whose target resolves to an enumerating import.
    Resolved,
    /// A call that looks like enumeration but whose receiver is not resolved.
    Unresolved,
}

pub(super) fn scan_of(site: &ResolvedCallSite, extra: &[Enumerator]) -> Option<Scan> {
    // Only the language's import.meta receiver has this unshadowable spelling.
    if matches!(
        site.source_callee.as_str(),
        "import.meta.glob" | "import.meta.globEager"
    ) {
        return Some(Scan::Resolved);
    }
    if site.targets().any(|candidate| match candidate.target {
        KnownCallTargetRef::ModuleExport {
            specifier,
            export_path,
            ..
        } => enumerates(specifier, export_path, extra),
        _ => false,
    }) {
        return Some(Scan::Resolved);
    }
    let terminal = site.source_callee.rsplit('.').next().unwrap_or_default();
    (site.has_unknown_target() && UNRESOLVED_NAMES.contains(&terminal)).then_some(Scan::Unresolved)
}

pub(super) fn enumerates(specifier: &str, export_path: &str, extra: &[Enumerator]) -> bool {
    let module = specifier.strip_prefix("node:").unwrap_or(specifier);
    let path = export_path.strip_prefix("default.").unwrap_or(export_path);
    let terminal = path.rsplit('.').next().unwrap_or(path);
    let builtin = if FS_MODULES.contains(&module) {
        let name = path.strip_prefix("promises.").unwrap_or(path);
        FS_ENUMERATORS.contains(&name)
    } else {
        GLOB_MODULES.contains(&module) && !GLOB_PURE.contains(&terminal)
    };
    builtin
        || extra.iter().any(|extra| {
            extra.module.strip_prefix("node:").unwrap_or(&extra.module) == module
                && extra.export.as_deref().is_none_or(|export| export == path)
        })
}

/// An unresolved CommonJS binding can still expose a scanner through a child.
/// Only exports known to be plain reads or pattern helpers are safe to omit.
pub(super) fn may_enumerate_export(
    specifier: &str,
    export_path: &str,
    extra: &[Enumerator],
) -> bool {
    let module = specifier.strip_prefix("node:").unwrap_or(specifier);
    if extra.iter().any(|configured| {
        configured
            .module
            .strip_prefix("node:")
            .unwrap_or(&configured.module)
            == module
            && configured.export.as_deref().is_none_or(|export| {
                export == export_path
                    || export
                        .strip_prefix(export_path)
                        .is_some_and(|tail| tail.starts_with('.'))
            })
    }) {
        return true;
    }
    if FS_MODULES.contains(&module) {
        return !matches!(export_path, "readFile" | "readFileSync");
    }
    !(GLOB_MODULES.contains(&module) && GLOB_PURE.contains(&export_path))
}
