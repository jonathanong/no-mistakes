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
