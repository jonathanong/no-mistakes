mod ast_nav;
mod literals;
mod load;
mod merge;
mod parse;
mod types;

#[cfg(test)]
mod tests;

pub(crate) use load::{load_configs_with_sources, load_many_with_sources, select_loaded};
pub use load::{load_many, load_partial_with_sources};
pub use merge::DEFAULT_TEST_ID_ATTRIBUTE;
pub use parse::parse;
pub use types::{PlaywrightConfig, TestProject};

mod partial_identity;
pub(crate) use partial_identity::config_name_with_sources;
