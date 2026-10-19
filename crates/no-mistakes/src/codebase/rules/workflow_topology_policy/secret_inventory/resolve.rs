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
        let Some(name) = &reference.name else {
            self.emit_incomplete(
                path,
                reference.line,
                "dynamic selector or whole secrets context",
            );
            return false;
        };
        let key = name.to_ascii_lowercase();
        let environment = match environment {
            Environment::Unknown => {
                return self.unknown_environment(path, reference, &key, aliases, reusable);
            }
            Environment::Named(name) => name.to_ascii_lowercase(),
            Environment::Repository => String::new(),
        };
        if self
            .inventory
            .environments
            .get(&environment)
            .is_some_and(|(_, names)| names.contains_key(&key))
        {
            self.used.insert((environment, key));
        } else if aliases.contains(&key) {
            // A reusable contract alias does not declare or consume a repository name.
        } else if self.inventory.exemptions.contains(&key) {
            if self.inventory.repository.contains_key(&key) {
                self.used.insert((String::new(), key));
            }
        } else if reusable {
            return self.reusable_without_source(path, reference, &key);
        } else if self.inventory.repository.contains_key(&key) {
            self.used.insert((String::new(), key));
        } else {
            self.push_undeclared(path, reference, &key);
        }
        true
    }

    /// A non-literal `environment:` still sees repository secrets. A name that
    /// is absent from the repository and every configured environment is
    /// undeclared. A name that exists only on some environment is not proven.
    fn unknown_environment(
        &mut self,
        path: &str,
        reference: &Reference,
        key: &str,
        aliases: &BTreeSet<String>,
        reusable: bool,
    ) -> bool {
        if aliases.contains(key) {
            return true;
        }
        if self.inventory.exemptions.contains(key) {
            if self.inventory.repository.contains_key(key) {
                self.used.insert((String::new(), key.to_owned()));
            }
            return true;
        }
        if reusable {
            return self.reusable_without_source(path, reference, key);
        }
        if self.inventory.repository.contains_key(key) {
            self.used.insert((String::new(), key.to_owned()));
            return true;
        }
        if self
            .inventory
            .environments
            .values()
            .any(|(_, names)| names.contains_key(key))
        {
            return false;
        }
        self.push_undeclared(path, reference, key);
        true
    }

    /// No contract alias blocks the unused proof. A name absent from the
    /// repository and every environment is still undeclared, because
    /// `secrets: inherit` can only pass a secret the caller has.
    fn reusable_without_source(&mut self, path: &str, reference: &Reference, key: &str) -> bool {
        self.emit_incomplete(
            path,
            reference.line,
            "reusable secret has no declared contract source",
        );
        if !self.declared_anywhere(key) {
            self.push_undeclared(path, reference, key);
        }
        false
    }

    fn declared_anywhere(&self, key: &str) -> bool {
        self.inventory.repository.contains_key(key)
            || self
                .inventory
                .environments
                .values()
                .any(|(_, names)| names.contains_key(key))
    }

    /// Workflow-level `env` versus job `environment` is not verified. Keep the
    /// undeclared finding, but do not report those environment names unused.
    pub(super) fn withhold_workflow_level_environment_unused(&mut self, reference: &Reference) {
        let Some(name) = &reference.name else {
            return;
        };
        let key = name.to_ascii_lowercase();
        for (environment, (_, names)) in &self.inventory.environments {
            if names.contains_key(&key) {
                self.used.insert((environment.clone(), key.clone()));
            }
        }
    }

    fn push_undeclared(&mut self, path: &str, reference: &Reference, key: &str) {
        let name = reference.name.as_deref().unwrap_or(key);
        self.findings.push(finding(path, reference.line, format!("undeclared secret {name}; declare its name in secretInventory.repository or the selected environment inventory")));
    }

    fn emit_incomplete(&mut self, path: &str, line: usize, reason: &str) {
        if self.inventory.report_unused {
            incomplete(self.findings, path, line, reason);
        }
    }
}
