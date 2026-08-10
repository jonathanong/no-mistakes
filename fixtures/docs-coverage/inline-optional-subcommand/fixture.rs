// Syntax-parser fixture: the nested module and imports intentionally need not compile.
use clap::{Args, Subcommand};

mod nested {
    #[derive(Args)]
    struct OptionalArgs {
        #[command(subcommand)]
        command: Option<OptionalCommand>,
    }

    #[derive(Subcommand)]
    enum OptionalCommand {
        Check,
    }
}
