//! Per-package `package.json` dependency-field lookup, used to classify a
//! production import as allowed, declared only under a non-production field
//! (`dev-only`), or absent entirely (`undeclared`).

use crate::codebase::package_deps;
use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

/// How a package name is declared (or not) in a manifest's dependency fields.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum Classification {
    /// Declared under at least one of the rule's configured `allowedFields`.
    Allowed,
    /// Declared, but only under fields outside `allowedFields` (e.g. only
    /// `devDependencies`).
    DevOnly,
    /// Not declared under any dependency field.
    Undeclared,
}

/// The dependency-field membership of one package's manifest, keyed by
/// dependency name to the set of fields it appears under.
pub(super) struct PackageManifest {
    fields_by_name: BTreeMap<String, BTreeSet<String>>,
}

impl PackageManifest {
    pub(super) fn load(
        manifest_path: &Path,
        sources: &crate::codebase::ts_source::SourceStore,
    ) -> Self {
        let mut fields_by_name: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
        for entry in package_deps::dependency_entries_from_source_store(
            manifest_path,
            package_deps::ALL_DEPENDENCY_FIELDS,
            sources,
        ) {
            fields_by_name
                .entry(entry.name)
                .or_default()
                .insert(entry.field);
        }
        Self { fields_by_name }
    }

    pub(super) fn classify(&self, name: &str, allowed_fields: &BTreeSet<String>) -> Classification {
        let Some(fields) = self.fields_by_name.get(name) else {
            return Classification::Undeclared;
        };
        if fields.iter().any(|field| allowed_fields.contains(field)) {
            Classification::Allowed
        } else {
            Classification::DevOnly
        }
    }
}

#[cfg(test)]
mod tests;
