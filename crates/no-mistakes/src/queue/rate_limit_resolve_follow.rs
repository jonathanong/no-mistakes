use super::{Expect, Outcome, ResolveCtx};
use crate::codebase::ts_resolver::ImportResolverFacade;
use crate::queue::extract_helpers::is_queue_package;
use crate::queue::extract_model::RateLimitExport;
use std::collections::HashSet;
use std::path::{Path, PathBuf};

impl<R: ImportResolverFacade> ResolveCtx<'_, R> {
    pub(super) fn follow_root(
        &self,
        path: &Path,
        specifier: &str,
        exported: &str,
        expect: Expect<'_>,
        saw_rate_limit_name: bool,
    ) -> Outcome {
        let mut visited = HashSet::new();
        self.follow_source(
            path,
            specifier,
            exported,
            expect,
            saw_rate_limit_name,
            &mut visited,
        )
    }

    fn follow_source(
        &self,
        path: &Path,
        specifier: &str,
        exported: &str,
        expect: Expect<'_>,
        saw_rate_limit_name: bool,
        visited: &mut HashSet<(PathBuf, String, u8)>,
    ) -> Outcome {
        let saw = saw_rate_limit_name || exported == "RateLimitError";
        if is_queue_package(specifier) {
            return match (expect, exported) {
                (Expect::Error, "RateLimitError")
                | (Expect::Worker | Expect::WorkerOrNamespace, "Worker" | "TestWorker") => {
                    Outcome::Queue
                }
                _ => Outcome::NonQueue,
            };
        }
        let Some(target) = self
            .resolver
            .resolve(specifier, path)
            .and_then(|resolved| self.remapper.remap(&resolved))
        else {
            return Outcome::Unresolved {
                saw_rate_limit_name: saw,
            };
        };
        if !visited.insert((target.clone(), exported.to_string(), expect_tag(expect))) {
            return Outcome::Unresolved {
                saw_rate_limit_name: saw,
            };
        }
        self.follow_file(&target, exported, expect, saw, visited)
    }

    fn follow_file(
        &self,
        path: &Path,
        exported: &str,
        expect: Expect<'_>,
        saw: bool,
        visited: &mut HashSet<(PathBuf, String, u8)>,
    ) -> Outcome {
        let Some(file) = self.facts.get(path) else {
            return Outcome::Unresolved {
                saw_rate_limit_name: saw,
            };
        };
        if let Some(export) = file.rate_limit_exports.get(exported) {
            return self.follow_export(path, export, expect, saw, visited);
        }
        if let Some((spec, imp)) = file.export_from.get(exported) {
            return self.follow_source(path, spec, imp, expect, saw, visited);
        }
        let mut saw_non_queue = false;
        let mut unresolved = None;
        for specifier in &file.star_exports {
            match self.follow_source(path, specifier, exported, expect, saw, visited) {
                Outcome::Queue => return Outcome::Queue,
                Outcome::NonQueue => saw_non_queue = true,
                Outcome::Unresolved {
                    saw_rate_limit_name,
                } => {
                    unresolved = Some(unresolved.unwrap_or(false) || saw_rate_limit_name);
                }
            }
        }
        match unresolved {
            Some(saw_name) => Outcome::Unresolved {
                saw_rate_limit_name: saw_name,
            },
            None if saw_non_queue => Outcome::NonQueue,
            None => Outcome::Unresolved {
                saw_rate_limit_name: saw,
            },
        }
    }

    fn follow_export(
        &self,
        path: &Path,
        export: &RateLimitExport,
        expect: Expect<'_>,
        saw: bool,
        visited: &mut HashSet<(PathBuf, String, u8)>,
    ) -> Outcome {
        match export {
            RateLimitExport::FromNamed { source, imported } => {
                self.follow_source(path, source, imported, expect, saw, visited)
            }
            RateLimitExport::FromNamespace { source } => match expect {
                Expect::WorkerOrNamespace => {
                    self.follow_source(path, source, "RateLimitError", Expect::Error, true, visited)
                }
                Expect::NamespaceWithMember(member) => {
                    self.follow_source(path, source, member, Expect::Worker, true, visited)
                }
                Expect::Error | Expect::Worker => Outcome::NonQueue,
            },
            RateLimitExport::FromMemberRateLimit { source, imported } => {
                if expect != Expect::Error {
                    return Outcome::NonQueue;
                }
                match imported {
                    Some(imp) => self.follow_source(
                        path,
                        source,
                        imp,
                        Expect::WorkerOrNamespace,
                        true,
                        visited,
                    ),
                    None => self.follow_source(
                        path,
                        source,
                        "RateLimitError",
                        Expect::Error,
                        true,
                        visited,
                    ),
                }
            }
            RateLimitExport::NonQueue => Outcome::NonQueue,
            RateLimitExport::Opaque => Outcome::Unresolved {
                saw_rate_limit_name: saw,
            },
        }
    }
}

fn expect_tag(expect: Expect<'_>) -> u8 {
    match expect {
        Expect::Error => 0,
        Expect::Worker => 1,
        Expect::WorkerOrNamespace => 2,
        Expect::NamespaceWithMember("TestWorker") => 4,
        Expect::NamespaceWithMember(_) => 3,
    }
}
