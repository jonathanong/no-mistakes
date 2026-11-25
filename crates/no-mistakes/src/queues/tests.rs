use crate::{Cli, Command};
use clap::Parser;

#[test]
fn queues_check_include_diagnostics_flag_is_explicit_opt_in() {
    let default = Cli::try_parse_from(["no-mistakes", "queues", "check"]).unwrap();
    let requested = Cli::try_parse_from([
        "no-mistakes",
        "queues",
        "check",
        "--include-diagnostics",
        "--format",
        "json",
    ])
    .unwrap();
    assert!(matches!(
        default.command,
        Command::Queues(args) if matches!(
            args.command,
            super::QueuesCommand::Check { config: None, include_diagnostics: false }
        )
    ));
    assert!(matches!(
        requested.command,
        Command::Queues(args) if matches!(
            args.command,
            super::QueuesCommand::Check { config: None, include_diagnostics: true }
        )
    ));
}
