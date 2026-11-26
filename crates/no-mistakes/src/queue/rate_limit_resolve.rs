use crate::codebase::ts_resolver::ImportResolverFacade;
use crate::codebase::ts_source::FrozenPathRemapper;
use crate::queue::extract::FileFacts;
use crate::queue::extract_model::RateLimitSiteKind;
use std::collections::HashMap;
use std::path::PathBuf;

#[path = "rate_limit_resolve_follow.rs"]
mod follow;

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub(super) enum Expect<'a> {
    Error,
    Worker,
    WorkerOrNamespace,
    NamespaceWithMember(&'a str),
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum Outcome {
    Queue,
    NonQueue,
    Unresolved { saw_rate_limit_name: bool },
}

pub(super) struct ResolveCtx<'a, R> {
    pub(super) facts: &'a HashMap<PathBuf, FileFacts>,
    pub(super) resolver: &'a R,
    pub(super) remapper: &'a FrozenPathRemapper,
}

pub(in crate::queue) fn resolve<R: ImportResolverFacade>(
    facts: &mut HashMap<PathBuf, FileFacts>,
    resolver: &R,
    remapper: &FrozenPathRemapper,
) {
    let ctx = ResolveCtx {
        facts,
        resolver,
        remapper,
    };
    let mut updates: Vec<(PathBuf, usize, Option<RateLimitSiteKind>)> = Vec::new();
    for (path, file) in facts.iter() {
        for (index, site) in file.rate_limits.iter().enumerate() {
            let outcome = match &site.kind {
                RateLimitSiteKind::Confirmed | RateLimitSiteKind::Unresolved => continue,
                RateLimitSiteKind::ImportedError {
                    source,
                    imported,
                    is_rate_limit_name,
                } => ctx.follow_root(path, source, imported, Expect::Error, *is_rate_limit_name),
                RateLimitSiteKind::ImportedMemberObject { source, imported } => {
                    ctx.follow_root(path, source, imported, Expect::WorkerOrNamespace, true)
                }
                RateLimitSiteKind::NamespaceError { source } => {
                    ctx.follow_root(path, source, "RateLimitError", Expect::Error, true)
                }
                RateLimitSiteKind::NamespaceWorkerError { source, member } => {
                    ctx.follow_root(path, source, member, Expect::Worker, true)
                }
                RateLimitSiteKind::ImportedNamespaceWorkerError {
                    source,
                    imported,
                    member,
                } => ctx.follow_root(
                    path,
                    source,
                    imported,
                    Expect::NamespaceWithMember(member),
                    true,
                ),
            };
            updates.push((path.clone(), index, resolve_outcome(outcome)));
        }
    }
    for (path, index, resolved) in updates.into_iter().rev() {
        let file = facts.get_mut(&path).expect("path came from facts");
        match resolved {
            Some(kind) => file.rate_limits[index].kind = kind,
            None => {
                file.rate_limits.remove(index);
            }
        }
    }
}

fn resolve_outcome(outcome: Outcome) -> Option<RateLimitSiteKind> {
    match outcome {
        Outcome::Queue => Some(RateLimitSiteKind::Confirmed),
        Outcome::NonQueue => None,
        Outcome::Unresolved {
            saw_rate_limit_name: true,
        } => Some(RateLimitSiteKind::Unresolved),
        Outcome::Unresolved {
            saw_rate_limit_name: false,
        } => None,
    }
}
