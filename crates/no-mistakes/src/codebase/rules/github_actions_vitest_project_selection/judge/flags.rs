pub(super) const MODES: &[&str] = &["run", "watch", "dev", "related", "bench", "list", "init"];

pub(super) const VALUE_FLAGS: &[&str] = &[
    "--config",
    "-c",
    "--reporter",
    "--dir",
    "--shard",
    "--mode",
    "--environment",
    "--pool",
    "--outputFile",
    "--root",
    "-r",
    "--workspace",
    "--exclude",
    "--coverage.include",
    "--coverage.exclude",
    "--cacheDir",
    "--setupFiles",
    "--changed",
    "--api.port",
    "--browser",
    "--browser.name",
    "--testTimeout",
    "--hookTimeout",
    "--teardownTimeout",
    "--maxConcurrency",
    "--minThreads",
    "--maxThreads",
    "--name",
    "--retry",
    "--bail",
    "--coverage.reporter",
    "--coverage.reportsDirectory",
    "--coverage.provider",
    "--sequence.sequencer",
    "--inspect",
    "--inspect-brk",
    "--slowTestThreshold",
    "--typecheck.tsconfig",
    "--execArgv",
    "--cache.dir",
];

pub(super) fn takes_value(name: &str) -> bool {
    VALUE_FLAGS.contains(&name)
        || name.starts_with("--outputFile.")
        || name.starts_with("--coverage.thresholds.")
}

pub(super) fn literal_value(text: &str) -> bool {
    matches!(text, "true" | "false") || is_number(text)
}

fn is_number(text: &str) -> bool {
    let mut digits = 0;
    let mut dot = false;
    for ch in text.chars() {
        if ch == '.' {
            if dot {
                return false;
            }
            dot = true;
            continue;
        }
        if !ch.is_ascii_digit() {
            return false;
        }
        digits += 1;
    }
    digits > 0
}
