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
use no_mistakes::codebase::import_usages::{self, ImportUsagesArgs};
use no_mistakes::codebase::queries;
use no_mistakes::codebase::symbols::{self, SymbolsArgs};
use no_mistakes::playwright;
use no_mistakes::{ci_run, impacted_checks_run, tests_run, CiArgs, ImpactedChecksArgs, TestsArgs};
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
    /// Report direct import usages in TS/JS files.
    ImportUsages(ImportUsagesArgs),
    /// List the files that directly import a file, plus a dependents count.
    Importers(queries::ImportersArgs),
    /// List a file's named exports and who imports each one.
    ExportsOf(queries::ExportsOfArgs),
    /// Check whether any files still import the given exports (non-zero if dead).
    DeadExports(queries::DeadExportsArgs),
    /// List call sites of an exported function with their argument shapes.
    CallSites(queries::CallSitesArgs),
    /// Check whether all imports in a file resolve (non-zero if any do not).
    ResolveCheck(queries::ResolveCheckArgs),
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
    /// Map changed files to triggered GitHub Actions workflows and env usage.
    Ci(CiArgs),
    /// List the minimal local validation commands for changed files.
    ImpactedChecks(ImpactedChecksArgs),
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
        Command::ImportUsages(args) => {
            import_usages::run(args)?;
            Ok(ExitCode::SUCCESS)
        }
        Command::Importers(args) => queries::importers::run(args),
        Command::ExportsOf(args) => queries::exports_of::run(args),
        Command::DeadExports(args) => queries::dead_exports::run(args),
        Command::CallSites(args) => queries::call_sites::run(args),
        Command::ResolveCheck(args) => queries::resolve_check::run(args),
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
        Command::Ci(args) => ci_run(args),
        Command::ImpactedChecks(args) => impacted_checks_run(args),
        Command::DataPw(args) => data_pw::run(args),
        Command::Effects(args) => effects::run(args),
        Command::RscCallers(args) => rsc_callers::run(args),
        Command::RegistryExtension(args) => registry_extension::run(args),
    }
}
