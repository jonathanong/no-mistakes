use super::*;
use clap::{error::ErrorKind, CommandFactory};

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
    assert_eq!(leaves.len(), 47, "update the documented CLI leaf matrix");

    for leaf in leaves {
        for flag in ["--timings", "--verbose-timings"] {
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
    assert_eq!(aliases.len(), 6, "update the tests/test alias leaf matrix");

    for leaf in aliases {
        for flag in ["--timings", "--verbose-timings"] {
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
