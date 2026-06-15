use anyhow::{Context, Result};
use clap::Args;
use no_mistakes::cli::{resolve_root, Format};
use no_mistakes::registry_extension_query::{self, RegistryExtensionReport};
use std::path::PathBuf;
use std::process::ExitCode;

#[derive(Args, Debug)]
pub(crate) struct RegistryExtensionArgs {
    /// Registry file to summarize the entry pattern of.
    pub(crate) registry_file: PathBuf,
    #[arg(long, default_value = ".", global = true)]
    pub(crate) root: PathBuf,
    /// Output format: json, yml, md, paths, human.
    #[arg(
        long,
        value_enum,
        default_value = "human",
        global = true,
        conflicts_with = "json"
    )]
    pub(crate) format: Format,
    /// Shorthand for --format json.
    #[arg(long, global = true, conflicts_with = "format")]
    pub(crate) json: bool,
}

pub(crate) fn run(args: RegistryExtensionArgs) -> Result<ExitCode> {
    let RegistryExtensionArgs {
        registry_file,
        root,
        format,
        json,
    } = args;
    let effective_format = if json { Format::Json } else { format };
    let cwd = std::env::current_dir().context("cwd must be accessible")?;
    let root = resolve_root(&root, &cwd);
    let report = registry_extension_query::run(&root, &registry_file)?;
    match effective_format {
        Format::Json => {
            println!(
                "{}",
                serde_json::to_string_pretty(&report)
                    .expect("serialization of Rust structs never fails")
            );
        }
        Format::Yml => {
            println!(
                "{}",
                serde_yaml::to_string(&report).expect("serialization of Rust structs never fails")
            );
        }
        Format::Md => print_md(&report),
        Format::Paths => {
            // The registry file plus any distinct imported module specifiers.
            println!("{}", report.registry_file);
            let mut specifiers: Vec<&str> = report
                .entries
                .iter()
                .filter_map(|entry| entry.entry_import.as_ref().map(|i| i.specifier.as_str()))
                .collect();
            specifiers.sort();
            specifiers.dedup();
            for specifier in specifiers {
                println!("{specifier}");
            }
        }
        Format::Human => print_human(&report),
    }
    Ok(ExitCode::SUCCESS)
}

fn print_human(report: &RegistryExtensionReport) {
    println!(
        "{}: {} ({} confidence, {} entries)",
        report.registry_file,
        report.pattern_kind,
        report.confidence,
        report.entries.len()
    );
    if let Some(template) = &report.template {
        println!("  template: {template}");
    }
    for entry in &report.entries {
        println!("  {}: {}", entry.line, entry.call_shape);
    }
    for note in &report.notes {
        println!("  note: {note}");
    }
}

fn print_md(report: &RegistryExtensionReport) {
    println!("# registry-extension `{}`", report.registry_file);
    println!(
        "\n- pattern: `{}` ({} confidence)",
        report.pattern_kind, report.confidence
    );
    if let Some(template) = &report.template {
        println!("- template: `{template}`");
    }
    println!("\n## Entries");
    for entry in &report.entries {
        println!(
            "- `{}:{}` `{}`",
            report.registry_file, entry.line, entry.call_shape
        );
    }
}
