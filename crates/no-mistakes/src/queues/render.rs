use anyhow::Result;
use no_mistakes::cli::{print_json, Format};
use no_mistakes::queue::{
    format_diagnostic, is_worker_rate_limit_unresolved_diagnostic, CheckFinding, CheckReport,
    Diagnostic, Edge,
};
use std::collections::BTreeSet;

pub(super) fn print_edges(edges: &[Edge], format: Format) -> Result<()> {
    match format {
        Format::Json => print_json(edges),
        Format::Yml => println!("{}", serde_yaml::to_string(edges)?),
        Format::Md => {
            println!("# Queue edges");
            for edge in edges {
                println!("- `{}` -> `{}` ({})", edge.from, edge.to, edge.kind);
            }
        }
        Format::Paths => print_edge_paths(edges),
        Format::Human => {
            for edge in edges {
                println!("{} -> {}", edge.from, edge.to);
            }
        }
    }
    Ok(())
}

pub(super) fn print_related(roots: &[String], edges: &[Edge], format: Format) -> Result<()> {
    match format {
        Format::Json => print_json(edges),
        Format::Yml => println!("{}", serde_yaml::to_string(edges)?),
        Format::Md => {
            println!("# Related queue files");
            for edge in edges {
                println!("- `{}` -> `{}`", edge.from, edge.to);
            }
        }
        Format::Paths => print_edge_paths(edges),
        Format::Human => {
            println!("{}", roots.join(", "));
            for edge in edges {
                println!("  {} -> {}", edge.from, edge.to);
            }
        }
    }
    Ok(())
}

fn print_edge_paths(edges: &[Edge]) {
    let paths: BTreeSet<&str> = edges
        .iter()
        .flat_map(|e| [e.from.as_str(), e.to.as_str()])
        .collect();
    for p in paths {
        println!("{p}");
    }
}

pub(super) fn print_diagnostics_stderr(diagnostics: &[Diagnostic], include_all: bool) {
    for diagnostic in diagnostics {
        if !include_all && !is_worker_rate_limit_unresolved_diagnostic(diagnostic) {
            continue;
        }
        no_mistakes::diagnostics::print_stderr(format_args!("{}", format_diagnostic(diagnostic)));
    }
}

pub(super) fn print_check(
    findings: &[CheckFinding],
    diagnostics: &[Diagnostic],
    format: Format,
    include_diagnostics: bool,
) -> Result<()> {
    match format {
        Format::Json if include_diagnostics => print_json(&CheckReport {
            findings,
            diagnostics,
        }),
        Format::Json => print_json(findings),
        Format::Yml if include_diagnostics => println!(
            "{}",
            serde_yaml::to_string(&CheckReport {
                findings,
                diagnostics,
            })?
        ),
        Format::Yml => println!("{}", serde_yaml::to_string(findings)?),
        Format::Md => {
            println!("# Queue check");
            for f in findings {
                println!("- `{}`:{} {}", f.file, f.line, f.message);
            }
            if include_diagnostics {
                println!("## diagnostics");
                for d in diagnostics {
                    println!("- `{}`:{} ({}) {}", d.file, d.line, d.severity, d.message);
                }
            }
        }
        Format::Paths => {
            for f in findings {
                println!("{}:{}", f.file, f.line);
            }
        }
        Format::Human => {
            for f in findings {
                println!(
                    "{}[{}] {}:{} {}",
                    f.kind,
                    f.job.as_deref().unwrap_or("*"),
                    f.file,
                    f.line,
                    f.message
                );
            }
        }
    }
    Ok(())
}
