use super::*;
use clap::{error::ErrorKind, CommandFactory};

#[test]
fn check_sql_verdict_flag_is_explicit_opt_in() {
    let default = Cli::try_parse_from(["no-mistakes", "check"]).unwrap();
    let requested =
        Cli::try_parse_from(["no-mistakes", "check", "--include-sql-verdicts"]).unwrap();
    assert!(matches!(default.command, Command::Check(args) if !args.include_sql_verdicts));
    assert!(matches!(requested.command, Command::Check(args) if args.include_sql_verdicts));
}

fn leaf_paths(command: &clap::Command) -> Vec<Vec<String>> {
    fn visit(command: &clap::Command, prefix: &mut Vec<String>, out: &mut Vec<Vec<String>>) {
        let subcommands = command.get_subcommands().collect::<Vec<_>>();
        if subcommands.is_empty() {
            out.push(prefix.clone());
            return;
        }
        for subcommand in subcommands {
            prefix.push(subcommand.get_name().to_string());
            visit(subcommand, prefix, out);
            prefix.pop();
        }
    }

    let mut out = Vec::new();
    visit(command, &mut Vec::new(), &mut out);
    out
}

#[test]
fn every_cli_leaf_accepts_root_diagnostics_flags_at_every_command_boundary() {
    let leaves = leaf_paths(&Cli::command());
    assert_eq!(leaves.len(), 52, "update the documented CLI leaf matrix");

    for leaf in leaves {
        for flag in ["--timings", "--verbose-timings", "--progress"] {
            for insertion in 0..=leaf.len() {
                let mut argv = vec!["no-mistakes".to_string()];
                argv.extend(leaf.iter().cloned());
                argv.insert(insertion + 1, flag.to_string());
                argv.push("--help".to_string());
                let Err(error) = Cli::try_parse_from(&argv) else {
                    panic!("--help must stop parsing for {argv:?}");
                };
                assert_eq!(
                    error.kind(),
                    ErrorKind::DisplayHelp,
                    "failed to parse {argv:?}: {error}"
                );
            }
        }
    }
}

#[test]
fn test_alias_accepts_root_diagnostics_flags_at_every_command_boundary() {
    let aliases = leaf_paths(&Cli::command())
        .into_iter()
        .filter(|leaf| leaf.first().is_some_and(|command| command == "tests"))
        .map(|mut leaf| {
            leaf[0] = "test".to_string();
            leaf
        })
        .collect::<Vec<_>>();
    assert_eq!(aliases.len(), 7, "update the tests/test alias leaf matrix");

    for leaf in aliases {
        for flag in ["--timings", "--verbose-timings", "--progress"] {
            for insertion in 0..=leaf.len() {
                let mut argv = vec!["no-mistakes".to_string()];
                argv.extend(leaf.iter().cloned());
                argv.insert(insertion + 1, flag.to_string());
                argv.push("--help".to_string());
                let Err(error) = Cli::try_parse_from(&argv) else {
                    panic!("--help must stop parsing for {argv:?}");
                };
                assert_eq!(error.kind(), ErrorKind::DisplayHelp, "{argv:?}: {error}");
            }
        }
    }
}

#[test]
fn diagnostics_flags_combine_at_every_cli_leaf() {
    for leaf in leaf_paths(&Cli::command()) {
        let mut argv = vec!["no-mistakes".to_string(), "--timings".to_string()];
        argv.extend(leaf);
        argv.push("--verbose-timings".to_string());
        argv.push("--help".to_string());
        let Err(error) = Cli::try_parse_from(&argv) else {
            panic!("--help must stop parsing for {argv:?}");
        };
        assert_eq!(error.kind(), ErrorKind::DisplayHelp, "{argv:?}: {error}");
    }
}

#[test]
fn progress_interval_is_positive_and_requires_progress() {
    for args in [
        vec!["no-mistakes", "check", "--progress-interval", "0"],
        vec!["no-mistakes", "check", "--progress-interval", "10"],
    ] {
        assert!(Cli::try_parse_from(args).is_err());
    }
    let cli = Cli::try_parse_from([
        "no-mistakes",
        "check",
        "--progress",
        "--progress-interval",
        "10",
    ])
    .unwrap();
    assert_eq!(
        cli.diagnostics.progress_interval(),
        Some(std::time::Duration::from_millis(10))
    );
    let cli = Cli::try_parse_from(["no-mistakes", "check", "--timings"]).unwrap();
    assert!(cli.diagnostics.progress_interval().is_none());
}

#[test]
fn queue_and_server_graph_flags_keep_global_placement_and_format_conflicts() {
    for (command, leaf) in [("queues", "check"), ("server", "contracts")] {
        for args in [
            vec!["no-mistakes", command, "--format", "json", leaf],
            vec!["no-mistakes", command, leaf, "--format", "json"],
            vec!["no-mistakes", command, leaf, "--json"],
            vec!["no-mistakes", command, "--max-depth", "2", leaf],
        ] {
            assert!(Cli::try_parse_from(&args).is_ok(), "{args:?}");
        }
        assert!(Cli::try_parse_from(
            ["no-mistakes", command, leaf, "--json", "--format", "human",]
        )
        .is_err());
        assert!(Cli::try_parse_from(["no-mistakes", "--format", "json", command, leaf]).is_err());
        assert!(Cli::try_parse_from(["no-mistakes", "--json", command, leaf]).is_err());
    }
}

#[test]
fn every_cli_leaf_accepts_the_millisecond_invocation_budget() {
    for leaf in leaf_paths(&Cli::command()) {
        for insertion in 0..=leaf.len() {
            let mut argv = vec!["no-mistakes".to_string()];
            argv.extend(leaf.iter().cloned());
            argv.splice(
                insertion + 1..insertion + 1,
                ["--invocation-timeout-ms".to_string(), "3500".to_string()],
            );
            argv.push("--help".to_string());
            let Err(error) = Cli::try_parse_from(&argv) else {
                panic!("help must stop parsing {argv:?}")
            };
            assert_eq!(error.kind(), ErrorKind::DisplayHelp, "{argv:?}: {error}");
        }
    }
}

#[test]
fn check_effort_is_optional_and_accepts_only_implemented_levels() {
    use no_mistakes::config::v2::Effort;
    let default = Cli::try_parse_from(["no-mistakes", "check"]).unwrap();
    assert!(matches!(default.command, Command::Check(args) if args.effort.is_none()));
    for (value, expected) in [("low", Effort::Low), ("max", Effort::Max)] {
        let parsed = Cli::try_parse_from(["no-mistakes", "check", "--effort", value]).unwrap();
        assert!(matches!(parsed.command, Command::Check(args) if args.effort == Some(expected)));
    }
    assert!(Cli::try_parse_from(["no-mistakes", "check", "--effort", "medium"]).is_err());
}
