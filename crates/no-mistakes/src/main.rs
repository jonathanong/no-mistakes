mod check;
mod check_discovery;
mod check_parallel;
mod check_runner;
mod check_tasks;
mod data_pw;
mod effects;
mod fetches;
mod infra;
mod lockfile;
mod queues;
mod react;
mod registry_extension;
mod rsc_callers;
mod server;
mod swift;

use anyhow::Result;
use clap::{Parser, Subcommand};
use no_mistakes::cli::{init_rayon_threads, JobsArg};
use no_mistakes::codebase::dependencies::{self, Direction, TraverseArgs};
use no_mistakes::codebase::symbols::{self, SymbolsArgs};
use no_mistakes::playwright;
use no_mistakes::{tests_run, TestsArgs};
use std::process::ExitCode;

#[derive(Parser)]
#[command(author, version, about)]
struct Cli {
    #[command(flatten)]
    jobs: JobsArg,
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Find files that the given files depend on.
    Dependencies(TraverseArgs),
    /// Find files that depend on the given files.
    Dependents(TraverseArgs),
    /// Find files that depend on the given files (alias for `dependents`).
    Related(TraverseArgs),
    /// Dump named exports and imports of TS/JS files.
    Symbols(SymbolsArgs),
    /// Map Next.js App Router routes to static fetch API calls.
    Fetches(fetches::FetchesArgs),
    /// Analyze Playwright route, selector, and fetch coverage.
    Playwright(playwright::PlaywrightArgs),
    /// Analyze React component traits.
    React(react::ReactArgs),
    /// Analyze queue producer/worker relationships (BullMQ, glide-mq).
    Queues(queues::QueuesArgs),
    /// Analyze server route graphs (Express, Hono, Koa).
    Server(server::ServerArgs),
    /// Analyze Terraform/OpenTofu resource, module, and output relationships.
    Infra(infra::InfraArgs),
    /// Analyze Swift package importers and covering test targets.
    Swift(swift::SwiftArgs),
    /// Run configured project checks.
    Check(check::CheckArgs),
    /// Plan, explain, and visualize test impacts based on changed files.
    #[command(alias = "test")]
    Tests(TestsArgs),
    /// Analyze lockfile changes (diff packages).
    Lockfile(lockfile::LockfileArgs),
    /// Find all selector-attribute (e.g. data-pw) usages of a value.
    DataPw(data_pw::DataPwArgs),
    /// Report transitive effect call sites reachable from an entry file.
    Effects(effects::EffectsArgs),
    /// Find server components/pages that transitively import a component (RSC).
    RscCallers(rsc_callers::RscCallersArgs),
    /// Summarize how existing entries register in a registry file.
    RegistryExtension(registry_extension::RegistryExtensionArgs),
}

fn main() -> ExitCode {
    match run() {
        Ok(code) => code,
        Err(error) => {
            eprintln!("error: {error:#}");
            ExitCode::from(2)
        }
    }
}

fn run() -> Result<ExitCode> {
    let cli = Cli::parse();
    init_rayon_threads(cli.jobs);
    match cli.command {
        Command::Dependencies(args) => {
            dependencies::run(args, Direction::Deps)?;
            Ok(ExitCode::SUCCESS)
        }
        Command::Dependents(args) | Command::Related(args) => {
            dependencies::run(args, Direction::Dependents)?;
            Ok(ExitCode::SUCCESS)
        }
        Command::Symbols(args) => {
            symbols::run(args)?;
            Ok(ExitCode::SUCCESS)
        }
        Command::Fetches(args) => fetches::run(args),
        Command::Playwright(args) => playwright::run(args),
        Command::React(args) => react::run(args),
        Command::Queues(args) => queues::run(args),
        Command::Server(args) => server::run(args),
        Command::Infra(args) => infra::run(args),
        Command::Swift(args) => swift::run(args),
        Command::Check(args) => check::run(args),
        Command::Tests(args) => tests_run(args),
        Command::Lockfile(args) => lockfile::run(args),
        Command::DataPw(args) => data_pw::run(args),
        Command::Effects(args) => effects::run(args),
        Command::RscCallers(args) => rsc_callers::run(args),
        Command::RegistryExtension(args) => registry_extension::run(args),
    }
}
