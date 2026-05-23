mod analyze;
mod cli;
mod pipeline;
mod report;

pub(crate) use cli::{run, FetchesArgs};

#[cfg(test)]
mod tests;
