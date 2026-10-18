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
            self.emit_incomplete(
                path,
                reference.line,
                "reusable secret has no declared contract source",
            );
            return false;
        } else if self.inventory.repository.contains_key(&key) {
            self.used.insert((String::new(), key));
        } else {
            self.findings.push(finding(path, reference.line, format!("undeclared secret {name}; declare its name in secretInventory.repository or the selected environment inventory")));
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
            self.emit_incomplete(
                path,
                reference.line,
                "reusable secret has no declared contract source",
            );
            return false;
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
        let name = reference.name.as_deref().unwrap_or(key);
        self.findings.push(finding(path, reference.line, format!("undeclared secret {name}; declare its name in secretInventory.repository or the selected environment inventory")));
        true
    }

    fn emit_incomplete(&mut self, path: &str, line: usize, reason: &str) {
        if self.inventory.report_unused {
            incomplete(self.findings, path, line, reason);
        }
    }
}
