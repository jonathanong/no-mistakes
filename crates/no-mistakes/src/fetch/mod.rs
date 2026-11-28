pub use no_mistakes_ts_analysis::fetch::{
    cache, cache_opts, import_shape, types, url_extract, visit_helpers, visitor, visitor_state,
};
pub mod imports {
    pub use no_mistakes_ts_analysis::fetch::imports::*;

    #[cfg(all(test, any(nm_shard_all, nm_shard_2)))]
    mod tests;
}
pub mod resolve {
    pub use no_mistakes_ts_analysis::fetch::resolve::*;

    #[cfg(all(test, any(nm_shard_all, nm_shard_0)))]
    mod tests;
}
pub mod file_analysis;
pub(crate) mod file_facts;
pub mod import_routes;
pub mod route_analysis;

#[doc(hidden)]
pub use file_facts::ParsedFileCache;
#[doc(hidden)]
pub use import_routes::{
    route_reaches_target_from_visible_with_facts,
    route_reaches_target_from_visible_with_facts_and_session, RouteTargetFacts,
};
#[doc(hidden)]
pub use route_analysis::{
    collect_route_fetches_from_visible_with_facts,
    collect_route_fetches_from_visible_with_facts_and_session,
};
