/// Output format shared across no-mistakes commands.
/// Each command chooses its own default.
#[derive(Debug, Clone, Copy, PartialEq, clap::ValueEnum)]
pub enum Format {
    /// JSON object. Default is per command; `resolve_format` uses JSON when stdout is not a terminal.
    Json,
    /// Markdown nested bullet list.
    Md,
    /// YAML document with the same structure as JSON.
    Yml,
    /// One relative path per line (for shell `$()` substitution).
    Paths,
    /// Indented tree. Default is per command; `resolve_format` uses this on a terminal.
    Human,
}

/// Resolve the effective output format from the `--json` shorthand, an explicit
/// `--format`, or the TTY default (Human on a terminal, Json otherwise).
pub fn resolve_format(json: bool, format: Option<Format>, stdout_is_terminal: bool) -> Format {
    if json {
        Format::Json
    } else if let Some(format) = format {
        format
    } else if stdout_is_terminal {
        Format::Human
    } else {
        Format::Json
    }
}

#[cfg(all(test, any(nm_shard_all, nm_shard_0)))]
mod tests;
