/// Output format shared across all no-mistakes tools.
#[derive(Debug, Clone, Copy, PartialEq, clap::ValueEnum)]
pub enum Format {
    /// JSON object. Default when stdout is not a TTY.
    Json,
    /// Markdown nested bullet list.
    Md,
    /// YAML document with the same structure as JSON.
    Yml,
    /// One relative path per line (for shell `$()` substitution).
    Paths,
    /// Indented tree (default on TTY).
    Human,
}
