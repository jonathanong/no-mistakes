use anyhow::{Context, Result};
use clap::{Args, Subcommand};
use no_mistakes::postgres_catalog::{generate, PostgresCatalogOptions};
use std::io::Write;
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
    publish_catalog(&output, &mut |file| file.write_all(json.as_bytes()))
        .with_context(|| format!("failed to write catalog {}", output.display()))?;
    Ok(ExitCode::SUCCESS)
}

fn publish_catalog(
    output: &std::path::Path,
    write: &mut dyn FnMut(&mut std::fs::File) -> std::io::Result<()>,
) -> Result<()> {
    let parent = output
        .parent()
        .filter(|path| !path.as_os_str().is_empty())
        .unwrap_or_else(|| std::path::Path::new("."));
    let mut temporary = tempfile::NamedTempFile::new_in(parent)?;
    write(temporary.as_file_mut())?;
    temporary.persist(output)?;
    Ok(())
}

#[cfg(test)]
mod tests;
