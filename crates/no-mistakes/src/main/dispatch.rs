//! Command dispatch from the parsed CLI to each command module.
use super::*;

enum CommandDispatch {
    Handled(ExitCode),
    Unhandled(Box<Command>),
}

pub(super) fn run_command(command: Command) -> Result<ExitCode> {
    match run_codebase_command(command)? {
        CommandDispatch::Handled(code) => Ok(code),
        CommandDispatch::Unhandled(command) => match run_domain_command(*command)? {
            CommandDispatch::Handled(code) => Ok(code),
            CommandDispatch::Unhandled(_) => unreachable!("all command variants are dispatched"),
        },
    }
}

fn run_success(result: Result<()>) -> Result<CommandDispatch> {
    result?;
    Ok(CommandDispatch::Handled(ExitCode::SUCCESS))
}

fn run_codebase_command(command: Command) -> Result<CommandDispatch> {
    match command {
        Command::Dependencies(args) => run_success(dependencies::run(args, Direction::Deps)),
        Command::Dependents(args) | Command::Related(args) => {
            run_success(dependencies::run(args, Direction::Dependents))
        }
        Command::Symbols(args) => run_success(symbols::run(args)),
        Command::ImportUsages(args) => run_success(import_usages::run(args)),
        Command::Importers(args) => Ok(CommandDispatch::Handled(queries::importers::run(args)?)),
        Command::ExportsOf(args) => Ok(CommandDispatch::Handled(queries::exports_of::run(args)?)),
        Command::DeadExports(args) => {
            Ok(CommandDispatch::Handled(queries::dead_exports::run(args)?))
        }
        Command::CallSites(args) => Ok(CommandDispatch::Handled(queries::call_sites::run(args)?)),
        Command::ResolveCheck(args) => {
            Ok(CommandDispatch::Handled(queries::resolve_check::run(args)?))
        }
        command => Ok(CommandDispatch::Unhandled(Box::new(command))),
    }
}

fn run_domain_command(command: Command) -> Result<CommandDispatch> {
    let code = match command {
        Command::Fetches(args) => fetches::run(args),
        Command::Flow(args) => flow::run(args),
        Command::Playwright(args) => playwright::run(args),
        Command::React(args) => react::run(args),
        Command::Queues(args) => queues::run(args),
        Command::Server(args) => server::run(args),
        Command::Infra(args) => infra::run(args),
        Command::Swift(args) => swift::run(args),
        Command::Postgres(args) => postgres::run(args),
        Command::Check(args) => check::run(args),
        Command::Config(args) => config_cmd::run(args),
        Command::Tests(args) => tests_run(args),
        Command::Lock(args) => lock_cmd::run(args),
        Command::Lockfile(args) => lockfile::run(args),
        Command::Ci(args) => ci_run(args),
        Command::ImpactedChecks(args) => impacted_checks_run(args),
        Command::DataPw(args) => data_pw::run(args),
        Command::Effects(args) => effects::run(args),
        Command::RscCallers(args) => rsc_callers::run(args),
        Command::RegistryExtension(args) => registry_extension::run(args),
        command => return Ok(CommandDispatch::Unhandled(Box::new(command))),
    }?;
    Ok(CommandDispatch::Handled(code))
}
