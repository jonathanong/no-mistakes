//! Per-family extraction scopes. `All` is today's whole-universe behavior.
//!
//! `Only` is the union of enabled rules' include/projects plus, for families
//! that read other files' facts, those files' relative import closure. A
//! specifier that is not relative and not a queue package falls back to `All`
//! so recall stays complete (issue #2460 item 2).

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};
use std::sync::Arc;

mod closure;
mod configure;
mod nextjs;
mod rules;

pub use configure::configure_family_path_scopes;
pub use nextjs::configure_nextjs_api_route_source_scope;
pub use nextjs::configure_nextjs_caching_scope;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub enum FamilyPathScope {
    #[default]
    All,
    Only(Arc<BTreeSet<PathBuf>>),
}

pub fn applies(scope: &FamilyPathScope, path: &Path) -> bool {
    match scope {
        FamilyPathScope::All => true,
        FamilyPathScope::Only(paths) => {
            paths.contains(path)
                || paths.contains(&crate::codebase::ts_resolver::normalize_path(path))
        }
    }
}

pub fn merge(
    self_on: bool,
    self_scope: &FamilyPathScope,
    other_on: bool,
    other_scope: &FamilyPathScope,
) -> FamilyPathScope {
    match (self_on, other_on) {
        (false, false) => FamilyPathScope::All,
        (true, false) => self_scope.clone(),
        (false, true) => other_scope.clone(),
        (true, true) => match (self_scope, other_scope) {
            (FamilyPathScope::All, _) | (_, FamilyPathScope::All) => FamilyPathScope::All,
            (FamilyPathScope::Only(left), FamilyPathScope::Only(right)) => {
                if Arc::ptr_eq(left, right) {
                    FamilyPathScope::Only(Arc::clone(left))
                } else {
                    let mut paths = left.as_ref().clone();
                    paths.extend(right.iter().cloned());
                    FamilyPathScope::Only(Arc::new(paths))
                }
            }
        },
    }
}

pub fn include_scopes(plan: &mut super::CheckFactPlan, other: &super::CheckFactPlan) {
    plan.raw_source_paths = merge(
        plan.raw_source,
        &plan.raw_source_paths,
        other.raw_source,
        &other.raw_source_paths,
    );
    plan.react_paths = merge(
        plan.react,
        &plan.react_paths,
        other.react,
        &other.react_paths,
    );
    plan.queue_paths = merge(
        plan.queue,
        &plan.queue_paths,
        other.queue,
        &other.queue_paths,
    );
    plan.integration_paths = merge(
        plan.integration,
        &plan.integration_paths,
        other.integration,
        &other.integration_paths,
    );
    plan.integration_test_paths = merge(
        plan.integration,
        &plan.integration_test_paths,
        other.integration,
        &other.integration_test_paths,
    );
    plan.symbols_paths = merge(
        plan.symbols,
        &plan.symbols_paths,
        other.symbols,
        &other.symbols_paths,
    );
    plan.storybook_paths = merge(
        plan.storybook,
        &plan.storybook_paths,
        other.storybook,
        &other.storybook_paths,
    );
    plan.nextjs_caching_paths = merge(
        plan.nextjs_caching || plan.nextjs_caching_low,
        &plan.nextjs_caching_paths,
        other.nextjs_caching || other.nextjs_caching_low,
        &other.nextjs_caching_paths,
    );
    plan.boundary_paths = merge(
        plan.server_route_client_boundary,
        &plan.boundary_paths,
        other.server_route_client_boundary,
        &other.boundary_paths,
    );
    plan.react |= other.react;
    plan.queue |= other.queue;
    plan.integration |= other.integration;
    plan.symbols |= other.symbols;
    plan.storybook |= other.storybook;
    plan.server_route_client_boundary |= other.server_route_client_boundary;
}

#[cfg(test)]
mod identity;
#[cfg(test)]
mod tests;
