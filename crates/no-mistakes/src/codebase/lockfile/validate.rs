use super::{bun, npm, pnpm, yarn, PackageManager};

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
    }
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
