// Syntax-parser fixture: this source shape intentionally need not compile.
use clap::{Args, Subcommand};

#[derive(Args)]
struct FixtureArgs {
    #[command(
        subcommand
    )]
    command: FixtureCommand,
}

#[derive(Subcommand)]
enum FixtureCommand {
    Check,
}
