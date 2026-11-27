//! Supplemental route facts retain runner and helper ownership. They never
//! pretend that an integration test is a browser navigation.
pub(crate) use crate::codebase::playwright_facts::integration::{
    configured, RouteLink, RouteOccurrence,
};

mod edges;
mod extract;
mod links;
mod literals;
mod prepare;
mod receivers;
#[cfg(all(test, any(nm_shard_all, nm_shard_0)))]
mod tests;
pub(crate) use edges::edges;
pub(crate) use links::prepare_links;
pub(crate) use prepare::runner_plan;

pub(crate) use extract::collect;
