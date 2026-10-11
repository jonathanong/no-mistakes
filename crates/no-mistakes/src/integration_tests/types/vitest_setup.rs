use std::collections::BTreeSet;
use std::path::PathBuf;

/// The Vitest config field that declared a setup module.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) enum VitestSetupField {
    SetupFiles,
    GlobalSetup,
}

impl VitestSetupField {
    pub(crate) const fn as_str(self) -> &'static str {
        match self {
            Self::SetupFiles => "setupFiles",
            Self::GlobalSetup => "globalSetup",
        }
    }
}

/// A single statically declared (or conservatively dynamic) Vitest setup
/// dependency. `specifier` is absent only for a dynamic expression; a literal
/// with no `resolved_path` is an unresolved candidate and must remain visible
/// to impact planning.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) struct VitestSetupDependency {
    pub(crate) field: VitestSetupField,
    pub(crate) specifier: Option<String>,
    /// An imported setup helper could not resolve through the preliminary
    /// catalog, but its bare specifier may resolve through a package catalog
    /// discovered from runner project roots.
    pub(crate) needs_final_catalog_reparse: bool,
    /// A static config `extends` target could not be read, so this synthetic
    /// dependency keeps a project-bounded fallback trigger without inventing
    /// a setup module edge.
    pub(crate) unresolved_config_extends: Option<String>,
    /// A readable static config `extends` target. This preserves its resolved
    /// path and candidates for owner-scoped config-change planning without
    /// creating a synthetic setup edge.
    pub(crate) config_extends_provenance: bool,
    pub(crate) resolved_path: Option<PathBuf>,
    /// Effective Vitest project root used to resolve a relative setup
    /// specifier. This deliberately differs from `declaration_path` for
    /// imported config objects and projects with an explicit `root`.
    pub(crate) resolution_base: PathBuf,
    pub(crate) declaration_path: PathBuf,
    pub(crate) declaration_line: u32,
    /// Config, helper, and literal-resolution candidate modules that
    /// contributed this declaration. These paths are conservative impact
    /// triggers, including targets deleted before planning.
    pub(crate) trigger_paths: BTreeSet<PathBuf>,
    /// Candidates derived from the import resolver. Keep these separate from
    /// parser provenance so a later scoped re-resolution can replace only its
    /// own facts without dropping config and helper triggers.
    pub(crate) resolver_candidate_paths: BTreeSet<PathBuf>,
    /// Literal setup specifiers retained after bounded static enumeration.
    /// They are resolved only after the effective project root is known.
    pub(crate) conservative_specifiers: BTreeSet<String>,
    /// Runtime candidates reached through a resolved setup module. These stay
    /// separate so a deletion can recover the missing graph edge without
    /// treating unrelated alternate candidates as unsafe fallbacks.
    pub(crate) transitive_trigger_paths: BTreeSet<PathBuf>,
}
