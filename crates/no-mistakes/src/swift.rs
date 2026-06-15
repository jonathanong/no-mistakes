use anyhow::{Context, Result};
use clap::{Args, Subcommand};
use no_mistakes::cli::{resolve_root, Format};
use no_mistakes::swift_api::{analyze_project, ImporterRow, TestTargetRow};
use std::path::PathBuf;
use std::process::ExitCode;

#[derive(Args)]
pub(crate) struct SwiftArgs {
    /// Project root directory.
    #[arg(long, default_value = ".", global = true)]
    root: PathBuf,
    /// Path to a no-mistakes config file (defaults to discovery from root).
    #[arg(long, global = true)]
    config: Option<PathBuf>,
    /// Output format: json, yml, md, paths, human.
    #[arg(
        long,
        value_enum,
        default_value = "human",
        global = true,
        conflicts_with = "json"
    )]
    format: Format,
    /// Alias for --format json; cannot be combined with --format.
    #[arg(long, global = true, conflicts_with = "format")]
    json: bool,
    #[command(subcommand)]
    command: SwiftCommand,
}

#[derive(Subcommand)]
enum SwiftCommand {
    /// Swift files that import or reference the given Swift file.
    Importers {
        /// The Swift source file, relative to the root.
        file: String,
    },
    /// SwiftPM test targets that transitively cover the given Swift file.
    TestTargets {
        /// The Swift source file, relative to the root.
        file: String,
    },
}

pub(crate) fn run(args: SwiftArgs) -> Result<ExitCode> {
    let base = std::env::current_dir().context("cwd must be accessible")?;
    let root = resolve_root(&args.root, &base);
    let format = if args.json { Format::Json } else { args.format };
    let report = analyze_project(&root, args.config.as_deref())?;

    match &args.command {
        SwiftCommand::Importers { file } => {
            print_importers(&report.importers(file), format)?;
        }
        SwiftCommand::TestTargets { file } => {
            print_test_targets(&report.test_targets(file), format)?;
        }
    }
    Ok(ExitCode::SUCCESS)
}

fn print_importers(rows: &[ImporterRow], format: Format) -> Result<()> {
    match format {
        Format::Json => println!("{}", serde_json::to_string_pretty(rows)?),
        Format::Yml => println!("{}", serde_yaml::to_string(rows)?),
        Format::Md => {
            println!("# Importers");
            for row in rows {
                println!("- `{}` (depth {})", row.file, row.depth);
            }
        }
        Format::Paths => {
            for row in rows {
                println!("{}", row.file);
            }
        }
        Format::Human => {
            for row in rows {
                println!("{} (depth {})", row.file, row.depth);
            }
        }
    }
    Ok(())
}

fn print_test_targets(rows: &[TestTargetRow], format: Format) -> Result<()> {
    match format {
        Format::Json => println!("{}", serde_json::to_string_pretty(rows)?),
        Format::Yml => println!("{}", serde_yaml::to_string(rows)?),
        Format::Md => {
            println!("# Covering test targets");
            for row in rows {
                println!("- `{}` (`{}`)", row.target, row.package);
            }
        }
        Format::Paths => {
            for row in rows {
                println!("{}", row.package);
            }
        }
        Format::Human => {
            for row in rows {
                println!("{} ({}): {}", row.target, row.package, row.command);
            }
        }
    }
    Ok(())
}
