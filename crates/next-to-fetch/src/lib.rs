mod analyze;
mod cli;
mod pipeline;
mod report;

pub use cli::run_cli;

#[cfg(test)]
mod tests;
