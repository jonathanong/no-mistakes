use super::Cli;
use clap::CommandFactory;

fn help_text(args: &[&str]) -> String {
    let command = Cli::command()
        .color(clap::ColorChoice::Never)
        .term_width(200);
    let argv = std::iter::once("no-mistakes").chain(args.iter().copied());
    match command.try_get_matches_from(argv) {
        Err(error) => error.to_string(),
        Ok(_) => panic!("expected --help to render for {args:?}"),
    }
}

#[test]
fn lockfile_diff_head_help_defaults_to_the_working_tree() {
    let help = help_text(&["lockfile", "diff", "--help"]);
    assert!(help.contains("defaults to the working-tree file"), "{help}");
    assert!(
        !help.contains("defaults to HEAD"),
        "omitted --head reads the working tree, not HEAD:\n{help}"
    );
}

#[test]
fn format_help_states_a_per_command_default() {
    let help = help_text(&["check", "--help"]);
    assert!(
        help.contains("Default is per command"),
        "Format values must not claim one universal default:\n{help}"
    );
    assert!(
        help.contains("`resolve_format` uses JSON when stdout is not a terminal"),
        "{help}"
    );
    assert!(
        help.contains("`resolve_format` uses this on a terminal"),
        "{help}"
    );
    assert!(!help.contains("Default when stdout is not a TTY"), "{help}");
    assert!(!help.contains("default on TTY"), "{help}");
    assert!(
        help.contains("default: human"),
        "check keeps its explicit human default when piped:\n{help}"
    );
}

#[test]
fn graph_format_help_keeps_the_terminal_default() {
    let help = help_text(&["dependencies", "--help"]);
    assert!(
        help.contains("Defaults to human on TTY, json on non-TTY"),
        "{help}"
    );
}

#[test]
fn omitted_tsconfig_help_uses_the_owning_config() {
    for command in ["dependencies", "dependents", "related"] {
        let help = help_text(&[command, "--help"]);
        assert!(
            help.contains(
                "each importing file uses the config that owns it among the discovered configs; otherwise the root tsconfig"
            ),
            "{command}:\n{help}"
        );
        assert!(!help.contains("searches upward"), "{command}:\n{help}");
    }

    let symbols = help_text(&["symbols", "--help"]);
    assert!(
        symbols.contains(
            "each file uses the config that owns it among the discovered configs; otherwise the root tsconfig"
        ),
        "{symbols}"
    );
    assert!(!symbols.contains("searches upward"), "{symbols}");

    let resolve_check = help_text(&["resolve-check", "--help"]);
    assert!(
        resolve_check.contains("selects the nearest tsconfig above the file"),
        "{resolve_check}"
    );
    assert!(
        !resolve_check.contains("searches upward"),
        "{resolve_check}"
    );
}
