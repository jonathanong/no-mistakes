use anyhow::{Context, Result};
use clap::{Args, Subcommand};
use no_mistakes::postgres_catalog::{generate, PostgresCatalogOptions};
use std::path::PathBuf;
use std::process::ExitCode;

#[derive(Args)]
pub struct PostgresArgs {
    #[command(subcommand)]
    command: PostgresCommand,
}
#[derive(Subcommand)]
enum PostgresCommand {
    /// Generate an independent ordering catalog from a live PostgreSQL schema.
    Catalog {
        #[arg(long)]
        connection_env: String,
        #[arg(long)]
        schema: String,
        #[arg(long)]
        output: PathBuf,
    },
}
pub fn run(args: PostgresArgs) -> Result<ExitCode> {
    let PostgresCommand::Catalog {
        connection_env,
        schema,
        output,
    } = args.command;
    let catalog = generate(&PostgresCatalogOptions {
        connection_env,
        schema,
    })?;
    let json = format!("{catalog:#}\n");
    std::fs::write(&output, json)
        .with_context(|| format!("failed to write catalog {}", output.display()))?;
    Ok(ExitCode::SUCCESS)
}
