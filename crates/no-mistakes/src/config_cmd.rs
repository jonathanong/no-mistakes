use clap::{Args, Parser, Subcommand};
use no_mistakes::config::resolved::resolve_config;
use std::path::PathBuf;
use std::process::ExitCode;

#[derive(Parser, Clone)]
pub struct ConfigArgs {
    #[command(subcommand)]
    command: ConfigCommand,
}

#[derive(Subcommand, Clone)]
enum ConfigCommand {
    /// Print the effective resolved configuration as JSON.
    Resolve(ResolveArgs),
}

#[derive(Args, Clone)]
struct ResolveArgs {
    #[arg(long, default_value = ".")]
    root: PathBuf,
    #[arg(long)]
    config: Option<PathBuf>,
}

pub fn run(args: ConfigArgs) -> anyhow::Result<ExitCode> {
    match args.command {
        ConfigCommand::Resolve(resolve) => {
            let report = resolve_config(&resolve.root, resolve.config.as_deref())?;
            no_mistakes::invocation::commit_timeout()?;
            println!("{}", serde_json::to_string_pretty(&report)?);
            Ok(ExitCode::SUCCESS)
        }
    }
}
