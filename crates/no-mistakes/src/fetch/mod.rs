pub mod cache;
pub mod cache_opts;
pub mod file_analysis;
pub(crate) mod file_facts;
pub mod import_routes;
pub mod import_shape;
pub mod imports;
pub mod resolve;
pub mod route_analysis;
pub mod types;
pub mod url_extract;
pub mod visit_helpers;
pub mod visitor;
pub mod visitor_state;

#[doc(hidden)]
pub use file_facts::ParsedFileCache;
#[doc(hidden)]
pub use import_routes::route_reaches_target_from_visible_with_facts;
#[doc(hidden)]
pub use route_analysis::collect_route_fetches_from_visible_with_facts;
