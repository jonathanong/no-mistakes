use super::findings::{finding, incomplete};
use super::CompiledInventory;
use crate::codebase::ci_workflows::secret_facts::{Environment, Reference};
use crate::codebase::rules::RuleFinding;
use std::collections::BTreeSet;

pub(super) struct Resolver<'a> {
    pub(super) inventory: &'a CompiledInventory,
    pub(super) used: &'a mut BTreeSet<(String, String)>,
    pub(super) findings: &'a mut Vec<RuleFinding>,
}

impl Resolver<'_> {
    pub(super) fn reference(
        &mut self,
        path: &str,
        reference: &Reference,
        environment: &Environment,
        aliases: &BTreeSet<String>,
        reusable: bool,
    ) -> bool {
        let Self {
            inventory,
            used,
            findings,
        } = self;
        let Some(name) = &reference.name else {
            incomplete(
                findings,
                path,
                reference.line,
                "dynamic selector or whole secrets context",
            );
            return false;
        };
        let key = name.to_ascii_lowercase();
        let environment = match environment {
            Environment::Unknown => return false,
            Environment::Named(name) => name.to_ascii_lowercase(),
            Environment::Repository => String::new(),
        };
        if inventory
            .environments
            .get(&environment)
            .is_some_and(|(_, names)| names.contains_key(&key))
        {
            used.insert((environment, key));
        } else if aliases.contains(&key) {
            // A reusable contract alias does not declare or consume a repository name.
        } else if inventory.exemptions.contains(&key) {
            if inventory.repository.contains_key(&key) {
                used.insert((String::new(), key));
            }
        } else if reusable {
            incomplete(
                findings,
                path,
                reference.line,
                "reusable secret has no declared contract source",
            );
            return false;
        } else if inventory.repository.contains_key(&key) {
            used.insert((String::new(), key));
        } else {
            findings.push(finding(path, reference.line, format!("undeclared secret {name}; declare its name in secretInventory.repository or the selected environment inventory")));
        }
        true
    }
}
