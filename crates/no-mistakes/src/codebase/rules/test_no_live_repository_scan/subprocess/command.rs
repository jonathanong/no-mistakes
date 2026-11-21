//! Executable plus leading arguments, after package-manager wrappers.
use super::super::config::AnalyzerCommand;
use crate::codebase::dependencies::graph::{KnownCallTargetRef, ResolvedCallSite};
use crate::codebase::ts_source::module_facts::arguments::ArgumentValue;
use serde_json::Value;

#[path = "shell_split.rs"]
mod shell;
#[path = "wrappers.rs"]
mod wrappers;

const LAUNCHERS: &[&str] = &[
    "exec",
    "execFile",
    "execFileSync",
    "execSync",
    "fork",
    "spawn",
    "spawnSync",
];

#[derive(Clone, PartialEq, Eq, Debug)]
pub(super) enum Token {
    Known(String),
    Unknown,
}

#[derive(Clone, Copy)]
pub(super) enum OptionsSlot {
    Absent,
    Unknown,
    Object(usize),
}

pub(super) struct Invocation {
    pub(super) executable: Token,
    pub(super) argv: Vec<Token>,
    pub(super) options: OptionsSlot,
    /// Directory from `cd <dir>` before `&&` or `;`. It replaces the `cwd` option.
    pub(super) shell_dir: Option<String>,
}

pub(super) fn launcher_name(site: &ResolvedCallSite) -> Option<&str> {
    site.targets().find_map(|candidate| match candidate.target {
        KnownCallTargetRef::ModuleExport {
            specifier,
            export_path,
            ..
        } => {
            let module = specifier.strip_prefix("node:").unwrap_or(specifier);
            let path = export_path.strip_prefix("default.").unwrap_or(export_path);
            (module == "child_process" && LAUNCHERS.contains(&path)).then_some(path)
        }
        _ => None,
    })
}

/// `exec` and `execSync` always take a shell string. `spawn` and `spawnSync` do
/// only when `shell` is boolean `true` and the command is not an argument array.
pub(super) fn uses_shell(name: &str, values: &[ArgumentValue]) -> bool {
    if matches!(values.get(1), Some(ArgumentValue::Array(_))) {
        return false;
    }
    matches!(name, "exec" | "execSync")
        || (matches!(name, "spawn" | "spawnSync") && shell::flag_true(values))
}

pub(super) fn launches(values: &[ArgumentValue], shell: bool) -> Vec<Invocation> {
    if shell {
        return shell::splits(values);
    }
    let Some(first) = values.first() else {
        return Vec::new();
    };
    let (argv, options) = match values.get(1) {
        None => (Vec::new(), OptionsSlot::Absent),
        Some(ArgumentValue::Array(items)) => {
            (items.iter().map(token).collect(), options_at(values, 2))
        }
        Some(ArgumentValue::Object(_)) => (Vec::new(), OptionsSlot::Object(1)),
        Some(_) => (vec![Token::Unknown], OptionsSlot::Unknown),
    };
    vec![Invocation {
        executable: token(first),
        argv,
        options,
        shell_dir: None,
    }]
}

fn options_at(values: &[ArgumentValue], index: usize) -> OptionsSlot {
    match values.get(index) {
        None => OptionsSlot::Absent,
        Some(ArgumentValue::Object(_)) => OptionsSlot::Object(index),
        Some(_) => OptionsSlot::Unknown,
    }
}

fn token(value: &ArgumentValue) -> Token {
    match value {
        ArgumentValue::Scalar(Value::String(text)) => Token::Known(text.clone()),
        _ => Token::Unknown,
    }
}

/// The configured command this launch runs, when the executable and leading arguments match.
pub(super) fn matched(invocation: &Invocation, analyzers: &[AnalyzerCommand]) -> Option<String> {
    let Token::Known(executable) = &invocation.executable else {
        return None;
    };
    let mut lines = vec![command_line(executable, &invocation.argv)];
    for _ in 0..4 {
        let current = lines.last().expect("wrapper walk keeps a command line");
        let Some(Token::Known(program)) = current.first() else {
            break;
        };
        let Some((inner, _)) = wrappers::unwrap_line(program, &current[1..]) else {
            break;
        };
        if inner.is_empty() || lines.iter().any(|line| line == &inner) {
            break;
        }
        lines.push(inner);
    }
    analyzers.iter().find_map(|analyzer| {
        lines
            .iter()
            .any(|line| prefix(line, &analyzer.command, &analyzer.args))
            .then(|| display(analyzer))
    })
}

pub(super) fn wrapper_dir(invocation: &Invocation) -> Option<String> {
    wrappers::scan_dir(&invocation.executable, &invocation.argv)
}

fn command_line(executable: &str, argv: &[Token]) -> Vec<Token> {
    let mut line = vec![Token::Known(executable.to_string())];
    line.extend(argv.iter().cloned());
    line
}

fn display(analyzer: &AnalyzerCommand) -> String {
    let mut text = analyzer.command.clone();
    for arg in &analyzer.args {
        text.push(' ');
        text.push_str(arg);
    }
    text
}

fn prefix(line: &[Token], command: &str, args: &[String]) -> bool {
    let mut tokens = line.iter();
    let Some(Token::Known(executable)) = tokens.next() else {
        return false;
    };
    if !same_program(executable, command) {
        return false;
    }
    args.iter()
        .all(|expected| matches!(tokens.next(), Some(Token::Known(token)) if token == expected))
}

fn same_program(token: &str, command: &str) -> bool {
    let base = wrappers::basename(token);
    base == command || wrappers::strip_suffix(base) == command
}

#[cfg(test)]
#[path = "command_tests.rs"]
mod tests;

#[cfg(test)]
#[path = "tests/command_shell_tests.rs"]
mod shell_tests;

#[cfg(test)]
#[path = "command_wrapper_tests.rs"]
mod wrapper_tests;

#[cfg(test)]
#[path = "command_reporter_tests.rs"]
mod reporter_tests;

#[cfg(test)]
#[path = "command_script_tests.rs"]
mod script_tests;
