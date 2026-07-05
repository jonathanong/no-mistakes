use super::super::RULE_ID;

#[derive(Debug, Clone, Copy)]
pub(super) enum LockfileDependencyType {
    Dependencies,
    DevDependencies,
    PeerDependencies,
    OptionalDependencies,
}

impl LockfileDependencyType {
    pub(super) fn field(self) -> &'static str {
        match self {
            Self::Dependencies => "dependencies",
            Self::DevDependencies => "devDependencies",
            Self::PeerDependencies => "peerDependencies",
            Self::OptionalDependencies => "optionalDependencies",
        }
    }

    pub(super) fn importer_entries(
        self,
        importer: &crate::codebase::lockfile::pnpm::PnpmImporter,
    ) -> Option<(
        &'static str,
        &[crate::codebase::lockfile::pnpm::PnpmImporterDependency],
    )> {
        match self {
            Self::Dependencies => Some((self.field(), &importer.dependencies)),
            Self::DevDependencies => Some((self.field(), &importer.dev_dependencies)),
            Self::PeerDependencies => None,
            Self::OptionalDependencies => Some((self.field(), &importer.optional_dependencies)),
        }
    }
}

pub(super) fn validate(
    dependency_types: &[&str],
) -> std::result::Result<Vec<LockfileDependencyType>, String> {
    let mut validated = Vec::new();
    for field in dependency_types {
        validated.push(match *field {
            "dependencies" => LockfileDependencyType::Dependencies,
            "devDependencies" => LockfileDependencyType::DevDependencies,
            "peerDependencies" => LockfileDependencyType::PeerDependencies,
            "optionalDependencies" => LockfileDependencyType::OptionalDependencies,
            _ => {
                return Err(format!(
                    "{RULE_ID}: lockfile dependencyTypes supports dependencies, devDependencies, peerDependencies, and optionalDependencies only; unsupported dependency type '{field}'"
                ));
            }
        });
    }
    Ok(validated)
}
