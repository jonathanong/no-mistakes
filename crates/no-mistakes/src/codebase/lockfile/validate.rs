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
    match pnpm::validate_for_planning(content) {
        Ok(()) if pnpm::parse(content).is_empty() => Err(LockfileValidationError {
            kind: "lockfile-pnpm-malformed",
            description: "unparsed or empty pnpm lockfile content",
        }),
        Ok(()) => Ok(()),
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
    let malformed = || LockfileValidationError {
        kind: "lockfile-npm-malformed",
        description: "malformed npm JSON",
    };
    let root: serde_json::Value = serde_json::from_str(content).map_err(|_| malformed())?;
    if !root.is_object() {
        return Err(malformed());
    }
    if npm::parse(content).is_empty() {
        return Err(LockfileValidationError {
            kind: "lockfile-npm-malformed",
            description: "unparsed or empty npm lockfile content",
        });
    }
    Ok(())
}

fn validate_yarn(content: &str) -> Result<(), LockfileValidationError> {
    if content.contains("__metadata:") {
        let root: serde_yaml::Value =
            serde_yaml::from_str(content).map_err(|_| LockfileValidationError {
                kind: "lockfile-yarn-malformed",
                description: "malformed yarn YAML",
            })?;
        if !root.is_mapping() || yarn::parse(content).is_empty() {
            return Err(LockfileValidationError {
                kind: "lockfile-yarn-malformed",
                description: "unparsed or empty yarn lockfile content",
            });
        }
        return Ok(());
    }
    let packages = yarn::parse(content);
    let has_valid_entry = packages
        .iter()
        .any(|pkg| !pkg.version.is_empty() || !pkg.fingerprint.is_empty());
    if !has_valid_entry {
        return Err(LockfileValidationError {
            kind: "lockfile-yarn-malformed",
            description: "malformed or unparsed yarn lockfile content",
        });
    }
    Ok(())
}

fn validate_bun(content: &str) -> Result<(), LockfileValidationError> {
    let stripped = bun::strip_jsonc(content);
    let root: serde_json::Value =
        serde_json::from_str(&stripped).map_err(|_| LockfileValidationError {
            kind: "lockfile-bun-malformed",
            description: "malformed bun JSONC",
        })?;
    if !root.is_object() || bun::parse(content).is_empty() {
        return Err(LockfileValidationError {
            kind: "lockfile-bun-malformed",
            description: "unparsed or empty bun lockfile content",
        });
    }
    Ok(())
}

#[cfg(all(test, any(nm_shard_all, nm_shard_0)))]
#[path = "validate_tests.rs"]
mod tests;
