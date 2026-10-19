use anyhow::Result;
use std::collections::BTreeSet;

use super::super::schema::SwiftConfig;

#[cfg(test)]
mod tests;

pub(super) fn validate(swift: &SwiftConfig) -> Result<()> {
    let declared: BTreeSet<&str> = swift
        .packages
        .iter()
        .map(|package| package.trim_end_matches('/'))
        .collect();
    let mut seen = BTreeSet::new();
    for (package, argv) in &swift.base_commands {
        let key = package.trim_end_matches('/');
        if key.is_empty() || !declared.contains(key) {
            anyhow::bail!(
                "tests.swift.baseCommands.{package} is not a declared tests.swift.packages entry; declare that package or remove the override"
            );
        }
        if !seen.insert(key) {
            anyhow::bail!(
                "tests.swift.baseCommands declares `{key}` more than once after trailing-slash normalization; keep one argv array"
            );
        }
        if argv.is_empty() {
            anyhow::bail!(
                "tests.swift.baseCommands.{package} must be a non-empty argv array such as [swift, test]"
            );
        }
        for (index, arg) in argv.iter().enumerate() {
            if arg.trim().is_empty() {
                anyhow::bail!(
                    "tests.swift.baseCommands.{package}[{index}] is empty; each element is one literal argument, not a shell string"
                );
            }
        }
    }
    Ok(())
}
