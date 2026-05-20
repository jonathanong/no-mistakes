mod analysis;
mod ast;
mod cli;
mod config;
mod fsutil;
mod matcher;
pub mod playwright_config;
mod playwright_tests;
pub mod playwright_urls;
mod routes;
pub mod selectors;
#[cfg(test)]
mod test_support;
#[cfg(test)]
mod tests;
mod url;

pub use analysis::pipeline::run;
pub use cli::{Cli, Command};
