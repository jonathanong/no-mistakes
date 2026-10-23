use super::*;
use crate::codebase::ts_source::relative_slash_path;
use std::process::Command;

pub(crate) fn run_shellcheck(
    root: &Path,
    opts: &Options,
    shell_files: &[PathBuf],
) -> Result<Vec<RuleFinding>> {
    let sev = if opts.shellcheck.severity.is_empty() {
        DEFAULT_SEVERITY
    } else {
        &opts.shellcheck.severity
    };

    if !VALID_SEVERITIES.contains(&sev) {
        return Err(anyhow::anyhow!(
            "invalid shellcheck severity: \"{}\". Expected one of: error, warning, info, style",
            sev
        ));
    }

    let mut command = Command::new("shellcheck");
    command
        .args(["-f", "gcc", "-S", sev, "--"])
        .args(shell_files);
    let result = crate::invocation::command_output(&mut command);
    handle_shellcheck_result(root, shell_files, result)
}

pub(crate) fn handle_shellcheck_result(
    root: &Path,
    shell_files: &[PathBuf],
    result: std::io::Result<std::process::Output>,
) -> Result<Vec<RuleFinding>> {
    match result {
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(incomplete(
            root,
            shell_files,
            "ShellCheck executable was not found; install shellcheck and make it available on PATH",
        )),
        Err(e) => Err(anyhow::anyhow!("failed to run shellcheck: {e}")),
        Ok(output) if output.status.success() => Ok(Vec::new()),
        Ok(output) => {
            let stdout = String::from_utf8_lossy(&output.stdout);
            let mut findings: Vec<RuleFinding> = parse_affected_files(&stdout, shell_files)
                .iter()
                .map(|path| make_finding(root, path))
                .collect();
            // Exit 1 with GCC findings is ShellCheck's normal issues-found result.
            // Other failures do not establish that unreported scripts were checked.
            if findings.is_empty() || output.status.code() != Some(1) {
                let affected: std::collections::HashSet<_> = findings
                    .iter()
                    .map(|finding| finding.file.as_str())
                    .collect();
                let incomplete_files: Vec<_> = shell_files
                    .iter()
                    .filter(|path| !affected.contains(relative_slash_path(root, path).as_str()))
                    .cloned()
                    .collect();
                findings.extend(incomplete(root, &incomplete_files,
                    "ShellCheck did not complete with recognized diagnostics; run shellcheck manually and fix its execution before retrying"));
            }
            findings.sort_by(|a, b| a.file.cmp(&b.file));
            Ok(findings)
        }
    }
}

fn make_finding(root: &Path, path: &Path) -> RuleFinding {
    let rel = relative_slash_path(root, path);
    let msg = format!("{rel}: shellcheck found issues (run shellcheck manually for details)");
    RuleFinding {
        rule: RULE_ID.to_string(),
        file: rel,
        line: 1,
        message: msg,
        import: None,
        target: None,
    }
}

static GCC_RE: std::sync::OnceLock<regex::Regex> = std::sync::OnceLock::new();

pub(crate) fn parse_affected_files(stdout: &str, shell_files: &[PathBuf]) -> Vec<PathBuf> {
    let re = GCC_RE.get_or_init(|| {
        regex::Regex::new(r"^(.+):\d+:\d+: [a-z]+: ").expect("failed to compile GCC_RE regex")
    });
    let shell_set: std::collections::HashSet<&PathBuf> = shell_files.iter().collect();
    let mut v: Vec<PathBuf> = stdout
        .lines()
        .filter_map(|l| re.captures(l)?.get(1).map(|m| PathBuf::from(m.as_str())))
        .filter(|p| shell_set.contains(p))
        .collect();
    v.sort();
    v.dedup();
    v
}

fn incomplete(root: &Path, paths: &[PathBuf], reason: &str) -> Vec<RuleFinding> {
    paths.iter().map(|path| {
        let mut finding = make_finding(root, path);
        finding.message = format!("{}: shellcheck analysis is incomplete: {reason}. This check requires ShellCheck to validate selected shell scripts.", finding.file);
        finding.target = Some("unanalyzable".into());
        finding
    }).collect()
}
