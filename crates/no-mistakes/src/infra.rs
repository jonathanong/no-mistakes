use anyhow::{Context, Result};
use clap::{Args, Subcommand};
use no_mistakes::cli::{resolve_root, Format};
use no_mistakes::terraform_api::{
    analyze_project, ModuleOutputsResult, ResourceRefRow, TestForRow,
};
use std::path::PathBuf;
use std::process::ExitCode;

#[derive(Args)]
pub(crate) struct InfraArgs {
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
    command: InfraCommand,
}

#[derive(Subcommand)]
enum InfraCommand {
    /// Resources/modules/outputs that reference the given `<type>.<name>`.
    ResourceRefs {
        /// The resource or data-source address, e.g. `aws_route53_record.foo`.
        address: String,
    },
    /// Outputs a module exports and the root modules that consume them.
    Outputs {
        /// The module directory, relative to the root, e.g. `infra/modules/network`.
        module_dir: String,
    },
    /// Test files covering resources defined in the given `.tf` file.
    TestFor {
        /// The `.tf` file, relative to the root.
        tf_file: String,
    },
}

pub(crate) fn run(args: InfraArgs) -> Result<ExitCode> {
    let base = std::env::current_dir().context("cwd must be accessible")?;
    let root = resolve_root(&args.root, &base);
    let format = if args.json { Format::Json } else { args.format };
    let report = analyze_project(&root, args.config.as_deref())?;

    match &args.command {
        InfraCommand::ResourceRefs { address } => {
            print_resource_refs(address, &report.resource_refs(address), format)?;
        }
        InfraCommand::Outputs { module_dir } => {
            print_outputs(&report.outputs(module_dir), format)?;
        }
        InfraCommand::TestFor { tf_file } => {
            print_test_for(&report.test_for(tf_file), format)?;
        }
    }
    Ok(ExitCode::SUCCESS)
}

fn print_resource_refs(address: &str, rows: &[ResourceRefRow], format: Format) -> Result<()> {
    match format {
        Format::Json => println!("{}", serde_json::to_string_pretty(rows)?),
        Format::Yml => println!("{}", serde_yaml::to_string(rows)?),
        Format::Md => {
            println!("# References to `{address}`");
            for row in rows {
                println!("- `{}` in `{}`", row.address, row.file);
            }
        }
        Format::Paths => {
            for row in rows {
                println!("{}", row.file);
            }
        }
        Format::Human => {
            for row in rows {
                println!("{} ({})", row.address, row.file);
            }
        }
    }
    Ok(())
}

fn print_outputs(result: &ModuleOutputsResult, format: Format) -> Result<()> {
    match format {
        Format::Json => println!("{}", serde_json::to_string_pretty(result)?),
        Format::Yml => println!("{}", serde_yaml::to_string(result)?),
        Format::Md => {
            println!("# Outputs of `{}`", result.module);
            for export in &result.exports {
                println!("- `{}`", export.name);
            }
            println!("## Consumers");
            for consumer in &result.consumers {
                println!(
                    "- `{}` <- `{}` in `{}`",
                    consumer.output, consumer.from, consumer.file
                );
            }
        }
        Format::Paths => {
            for consumer in &result.consumers {
                println!("{}", consumer.file);
            }
        }
        Format::Human => {
            println!("exports: {}", join_names(result));
            for consumer in &result.consumers {
                println!(
                    "  {} consumed by {} ({})",
                    consumer.output, consumer.from, consumer.file
                );
            }
        }
    }
    Ok(())
}

fn join_names(result: &ModuleOutputsResult) -> String {
    result
        .exports
        .iter()
        .map(|export| export.name.as_str())
        .collect::<Vec<_>>()
        .join(", ")
}

fn print_test_for(rows: &[TestForRow], format: Format) -> Result<()> {
    match format {
        Format::Json => println!("{}", serde_json::to_string_pretty(rows)?),
        Format::Yml => println!("{}", serde_yaml::to_string(rows)?),
        Format::Md => {
            println!("# Covering tests");
            for row in rows {
                println!("- `{}`", row.test_file);
            }
        }
        Format::Paths | Format::Human => {
            for row in rows {
                println!("{}", row.test_file);
            }
        }
    }
    Ok(())
}
