//! Executable plus leading arguments, after package-manager wrappers.
use super::super::config::AnalyzerCommand;
use crate::codebase::dependencies::graph::{KnownCallTargetRef, ResolvedCallSite};
use crate::codebase::ts_source::module_facts::arguments::ArgumentValue;
use serde_json::Value;

const LAUNCHERS: &[&str] = &["execFile", "execFileSync", "spawn", "spawnSync"];

#[derive(Clone, PartialEq, Eq, Debug)]
pub(super) enum Token {
    Known(String),
    Unknown,
}

pub(super) enum OptionsSlot {
    Absent,
    Unknown,
    Object(usize),
}

pub(super) struct Invocation {
    pub(super) executable: Token,
    pub(super) argv: Vec<Token>,
    pub(super) options: OptionsSlot,
}

pub(super) fn is_launcher(site: &ResolvedCallSite) -> bool {
    site.targets().any(|candidate| match candidate.target {
        KnownCallTargetRef::ModuleExport {
            specifier,
            export_path,
            ..
        } => {
            let module = specifier.strip_prefix("node:").unwrap_or(specifier);
            let path = export_path.strip_prefix("default.").unwrap_or(export_path);
            module == "child_process" && LAUNCHERS.contains(&path)
        }
        _ => false,
    })
}

pub(super) fn invocation(values: &[ArgumentValue]) -> Option<Invocation> {
    let executable = token(values.first()?);
    let (argv, options) = match values.get(1) {
        None => (Vec::new(), OptionsSlot::Absent),
        Some(ArgumentValue::Array(items)) => {
            (items.iter().map(token).collect(), options_at(values, 2))
        }
        Some(ArgumentValue::Object(_)) => (Vec::new(), OptionsSlot::Object(1)),
        Some(_) => (vec![Token::Unknown], OptionsSlot::Unknown),
    };
    Some(Invocation {
        executable,
        argv,
        options,
    })
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
    if let Some(inner) = unwrap_wrapper(executable, &invocation.argv) {
        lines.push(inner);
    }
    analyzers.iter().find_map(|analyzer| {
        lines
            .iter()
            .any(|line| prefix(line, &analyzer.command, &analyzer.args))
            .then(|| display(analyzer))
    })
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

fn unwrap_wrapper(executable: &str, argv: &[Token]) -> Option<Vec<Token>> {
    match program_name(executable) {
        "pnpm" => {
            let (verb, rest) = argv.split_first()?;
            let Token::Known(verb) = verb else {
                return None;
            };
            if verb != "exec" && verb != "dlx" {
                return None;
            }
            Some(skip_separator(rest).to_vec())
        }
        "npx" => Some(skip_yes(argv).to_vec()),
        _ => None,
    }
}

fn skip_separator(argv: &[Token]) -> &[Token] {
    match argv.first() {
        Some(Token::Known(token)) if token == "--" => &argv[1..],
        _ => argv,
    }
}

fn skip_yes(argv: &[Token]) -> &[Token] {
    let mut rest = argv;
    while let Some(Token::Known(token)) = rest.first() {
        if token == "-y" || token == "--yes" {
            rest = &rest[1..];
        } else {
            break;
        }
    }
    rest
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
    token == command || program_name(token) == command
}

fn program_name(token: &str) -> &str {
    let name = token.rsplit(['/', '\\']).next().unwrap_or(token);
    name.strip_suffix(".cmd")
        .or_else(|| name.strip_suffix(".exe"))
        .unwrap_or(name)
}

#[cfg(test)]
#[path = "command_tests.rs"]
mod tests;
