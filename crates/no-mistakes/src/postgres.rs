mod lock_inventory;
use anyhow::Result;
use clap::{Args, Subcommand};
use no_mistakes::postgres_catalog::{generate_to, PostgresCatalogCoverage, PostgresCatalogOptions};
use std::path::PathBuf;
use std::process::ExitCode;

#[derive(Args)]
pub struct PostgresArgs {
    #[command(subcommand)]
    command: PostgresCommand,
}
#[derive(Subcommand)]
enum PostgresCommand {
    /// Inventory PostgreSQL locks and maintain a shrinking baseline.
    LockInventory(lock_inventory::LockInventoryArgs),
    /// Generate a schema catalog from a live PostgreSQL schema.
    Catalog {
        #[arg(long)]
        connection_env: String,
        #[arg(long)]
        schema: String,
        /// `complete` carries every fact; `ordering` only what conflict and lock ordering need.
        #[arg(long, value_enum, default_value_t)]
        coverage: PostgresCatalogCoverage,
        /// Include explicit schema/relations evidence for these search_path schemas.
        #[arg(long = "search-path-schema")]
        search_path_schemas: Vec<String>,
        /// Record this name as `currentDatabase` instead of the connected database's name.
        #[arg(long, value_parser = clap::builder::NonEmptyStringValueParser::new())]
        current_database: Option<String>,
        #[arg(long)]
        output: PathBuf,
    },
}
pub fn run(args: PostgresArgs) -> Result<ExitCode> {
    let (connection_env, schema, coverage, search_path_schemas, current_database, output) =
        match args.command {
            PostgresCommand::LockInventory(args) => return lock_inventory::run(args),
            PostgresCommand::Catalog {
                connection_env,
                schema,
                coverage,
                search_path_schemas,
                current_database,
                output,
            } => (
                connection_env,
                schema,
                coverage,
                search_path_schemas,
                current_database,
                output,
            ),
        };
    generate_to(
        &PostgresCatalogOptions {
            connection_env,
            schema,
            coverage,
            search_path_schemas,
            current_database,
        },
        &output,
    )?;
    Ok(ExitCode::SUCCESS)
}
