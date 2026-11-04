use clap::{Args, Parser, Subcommand, ValueEnum};
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
    #[arg(long, value_enum, default_value = "resolved")]
    view: ConfigView,
}

#[derive(Clone, Copy, ValueEnum)]
enum ConfigView {
    Resolved,
    Declared,
}

pub fn run(args: ConfigArgs) -> anyhow::Result<ExitCode> {
    match args.command {
        ConfigCommand::Resolve(resolve) => {
            let output = match resolve.view {
                ConfigView::Resolved => serde_json::to_string(&resolve_config(
                    &resolve.root,
                    resolve.config.as_deref(),
                )?)?,
                ConfigView::Declared => {
                    serde_json::to_string(&no_mistakes::config::v2::declared_config(
                        &resolve.root,
                        resolve.config.as_deref(),
                    )?)?
                }
            };
            no_mistakes::invocation::commit_timeout()?;
            println!("{output}");
            Ok(ExitCode::SUCCESS)
        }
    }
}
