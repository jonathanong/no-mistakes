use super::ecosystem::TomlKind;
use super::{bun, ecosystem, npm, pnpm, yarn, PackageManager, ResolvedPackage};

#[derive(Debug)]
pub struct LockfileValidationError {
    pub kind: &'static str,
    pub description: &'static str,
}

pub fn validate_for_planning(
    manager: PackageManager,
    content: &str,
) -> Result<(), LockfileValidationError> {
    if content.trim().is_empty() {
        return Ok(());
    }
    match manager {
        PackageManager::Pnpm => validate_pnpm(content),
        PackageManager::Npm => validate_npm(content),
        PackageManager::Yarn => validate_yarn(content),
        PackageManager::Bun => validate_bun(content),
        _ => packages_for_diff(manager, content).map(drop),
    }
}

/// Preserve npm-family diff behavior while checking ecosystem formats.
pub fn packages_for_diff(
    manager: PackageManager,
    content: &str,
) -> Result<Vec<ResolvedPackage>, LockfileValidationError> {
    if content.trim().is_empty() {
        return Ok(Vec::new());
    }
    match manager {
        PackageManager::Npm | PackageManager::Pnpm | PackageManager::Yarn | PackageManager::Bun => {
            Ok(super::parse_lockfile(manager, content))
        }
        PackageManager::Poetry => toml(
            TomlKind::Poetry,
            content,
            "lockfile-poetry-malformed",
            "malformed poetry.lock content",
        ),
        PackageManager::Uv => toml(
            TomlKind::Uv,
            content,
            "lockfile-uv-malformed",
            "malformed uv.lock content",
        ),
        PackageManager::Pipfile => checked(
            ecosystem::parse_pipfile_checked(content),
            "lockfile-pipfile-malformed",
            "malformed Pipfile.lock content",
        ),
        PackageManager::Go => checked(
            ecosystem::parse_gomod_checked(content),
            "lockfile-gomod-malformed",
            "malformed go.mod content",
        ),
        PackageManager::Cargo => toml(
            TomlKind::Cargo,
            content,
            "lockfile-cargo-malformed",
            "malformed Cargo.lock content",
        ),
        PackageManager::Gemfile => checked(
            ecosystem::parse_gemfile_checked(content),
            "lockfile-gemfile-malformed",
            "malformed Gemfile.lock content",
        ),
        PackageManager::Composer => checked(
            ecosystem::parse_composer_checked(content),
            "lockfile-composer-malformed",
            "malformed composer.lock content",
        ),
    }
}

fn checked<T, E>(
    result: Result<T, E>,
    kind: &'static str,
    description: &'static str,
) -> Result<T, LockfileValidationError> {
    result.map_err(|_| LockfileValidationError { kind, description })
}

fn toml(
    kind: TomlKind,
    content: &str,
    error_kind: &'static str,
    description: &'static str,
) -> Result<Vec<ResolvedPackage>, LockfileValidationError> {
    checked(
        ecosystem::parse_toml_checked(kind, content),
        error_kind,
        description,
    )
}

fn validate_pnpm(content: &str) -> Result<(), LockfileValidationError> {
    match pnpm::parse_checked(content) {
        Ok(_) => Ok(()),
        Err(pnpm::PnpmValidationError::Malformed) => Err(LockfileValidationError {
            kind: "lockfile-pnpm-malformed",
            description: "malformed pnpm YAML",
        }),
        Err(pnpm::PnpmValidationError::UnsupportedSchema) => Err(LockfileValidationError {
            kind: "lockfile-pnpm-unsupported-schema",
            description: "unsupported pnpm lockfile schema",
        }),
        Err(pnpm::PnpmValidationError::NotEnvPrefix) => Err(LockfileValidationError {
            kind: "lockfile-pnpm-non-env-document",
            description: "a non-env document before the project lockfile",
        }),
    }
}

fn validate_npm(content: &str) -> Result<(), LockfileValidationError> {
    match npm::parse_checked(content) {
        Ok(_) => Ok(()),
        Err(npm::NpmValidationError::UnsupportedVersion) => Err(LockfileValidationError {
            kind: "lockfile-npm-unsupported-version",
            description: "unsupported npm lockfile version",
        }),
        Err(npm::NpmValidationError::Malformed(_)) => Err(LockfileValidationError {
            kind: "lockfile-npm-malformed",
            description: "malformed npm JSON",
        }),
    }
}

fn validate_yarn(content: &str) -> Result<(), LockfileValidationError> {
    match yarn::parse_checked(content) {
        Ok(_) => Ok(()),
        Err(yarn::YarnValidationError::Malformed) => Err(LockfileValidationError {
            kind: "lockfile-yarn-malformed",
            description: "malformed yarn lockfile content",
        }),
    }
}

fn validate_bun(content: &str) -> Result<(), LockfileValidationError> {
    match bun::parse_checked(content) {
        Ok(_) => Ok(()),
        Err(bun::BunValidationError::Malformed) => Err(LockfileValidationError {
            kind: "lockfile-bun-malformed",
            description: "malformed bun lockfile content",
        }),
    }
}

#[cfg(all(test, any(nm_shard_all, nm_shard_0)))]
#[path = "validate_tests.rs"]
mod tests;
