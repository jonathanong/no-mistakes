use super::Token;

pub(super) fn program_name(token: &str) -> &str {
    strip_suffix(basename(token))
}

pub(super) fn basename(token: &str) -> &str {
    token.rsplit(['/', '\\']).next().unwrap_or(token)
}

pub(super) fn strip_suffix(name: &str) -> &str {
    [".cmd", ".exe", ".js", ".mjs", ".cjs"]
        .into_iter()
        .find_map(|suffix| name.strip_suffix(suffix))
        .unwrap_or(name)
}

const BOOLEAN_FLAGS: &[&str] = &["-r", "--recursive"];
const DIR_FLAGS: &[&str] = &["-C", "--dir"];
const VALUE_FLAGS: &[&str] = &["--filter", "-F"];
const NPX_FLAGS: &[&str] = &["-y", "--yes", "--no-install"];
const NPX_VALUES: &[&str] = &["-p", "--package"];

pub(super) fn unwrap_line(
    executable: &str,
    argv: &[Token],
) -> Option<(Vec<Token>, Option<String>)> {
    let (line, dir) = match program_name(executable) {
        "pnpm" => {
            let (rest, dir) = skip_flags(argv)?;
            (verb(rest, &["exec", "dlx"])?, dir)
        }
        "npm" => {
            let (rest, dir) = skip_flags(argv)?;
            (verb(rest, &["exec"])?, dir)
        }
        "npx" | "bunx" => (skip_separator(skip_npx(argv)?).to_vec(), None),
        "yarn" => yarn(argv),
        _ => return None,
    };
    Some((line, dir))
}

/// Last pnpm `-C`/`--dir` or yarn `--cwd` in this wrapper chain. It is the scan root.
pub(super) fn scan_dir(executable: &Token, argv: &[Token]) -> Option<String> {
    let Token::Known(program) = executable else {
        return None;
    };
    let mut line = vec![Token::Known(program.clone())];
    line.extend(argv.iter().cloned());
    let mut root = None;
    for _ in 0..4 {
        let program = match line.first() {
            Some(Token::Known(program)) => program.clone(),
            _ => break,
        };
        let Some((inner, dir)) = unwrap_line(&program, &line[1..]) else {
            break;
        };
        root = dir.or(root);
        if inner.is_empty() || inner == line {
            break;
        }
        line = inner;
    }
    root
}

fn verb(argv: &[Token], verbs: &[&str]) -> Option<Vec<Token>> {
    let (name, rest) = argv.split_first()?;
    let Token::Known(name) = name else {
        return None;
    };
    verbs
        .contains(&name.as_str())
        .then(|| skip_separator(rest).to_vec())
}

fn skip_flags(argv: &[Token]) -> Option<(&[Token], Option<String>)> {
    let mut rest = argv;
    let mut dir = None;
    while let Some(Token::Known(token)) = rest.first() {
        if !token.starts_with('-') {
            break;
        }
        if BOOLEAN_FLAGS.contains(&token.as_str()) {
            rest = &rest[1..];
        } else if let Some(value) = flag_dir(token) {
            dir = Some(value);
            rest = &rest[1..];
        } else if DIR_FLAGS.contains(&token.as_str()) {
            let Some(Token::Known(value)) = rest.get(1) else {
                return None;
            };
            dir = Some(value.clone());
            rest = &rest[2..];
        } else if VALUE_FLAGS.contains(&token.as_str()) {
            if !matches!(rest.get(1), Some(Token::Known(_))) {
                return None;
            }
            rest = &rest[2..];
        } else if included_value(token) {
            rest = &rest[1..];
        } else if long_value(token, rest) {
            rest = &rest[2..];
        } else {
            return None;
        }
    }
    Some((rest, dir))
}

fn flag_dir(token: &str) -> Option<String> {
    if let Some(value) = token.strip_prefix("--dir=") {
        return (!value.is_empty()).then(|| value.to_string());
    }
    token
        .strip_prefix("-C")
        .filter(|value| !value.is_empty())
        .map(str::to_string)
}

fn included_value(token: &str) -> bool {
    token.contains('=') || (token.starts_with("-F") && token.len() > 2)
}

fn long_value(token: &str, argv: &[Token]) -> bool {
    token.starts_with("--")
        && matches!(argv.get(1), Some(Token::Known(value)) if !value.starts_with('-'))
}

fn skip_npx(argv: &[Token]) -> Option<&[Token]> {
    let mut rest = argv;
    while let Some(Token::Known(token)) = rest.first() {
        if NPX_FLAGS.contains(&token.as_str()) || token.starts_with("--package=") {
            rest = &rest[1..];
        } else if NPX_VALUES.contains(&token.as_str()) {
            if !matches!(rest.get(1), Some(Token::Known(_))) {
                return None;
            }
            rest = &rest[2..];
        } else {
            break;
        }
    }
    Some(rest)
}

fn yarn(argv: &[Token]) -> (Vec<Token>, Option<String>) {
    let (rest, dir) = skip_cwd(argv);
    let line = match rest.first() {
        Some(Token::Known(token)) if token == "dlx" || token == "run" => {
            skip_separator(&rest[1..]).to_vec()
        }
        _ => rest.to_vec(),
    };
    (line, dir)
}

fn skip_cwd(argv: &[Token]) -> (&[Token], Option<String>) {
    let mut rest = argv;
    let mut dir = None;
    loop {
        let Some(Token::Known(token)) = rest.first() else {
            return (rest, dir);
        };
        if token == "--cwd" {
            let Some(Token::Known(value)) = rest.get(1) else {
                return (rest, dir);
            };
            dir = Some(value.clone());
            rest = &rest[2..];
            continue;
        }
        if let Some(value) = token.strip_prefix("--cwd=") {
            if !value.is_empty() {
                dir = Some(value.to_string());
                rest = &rest[1..];
                continue;
            }
        }
        return (rest, dir);
    }
}

fn skip_separator(argv: &[Token]) -> &[Token] {
    match argv.first() {
        Some(Token::Known(token)) if token == "--" => &argv[1..],
        _ => argv,
    }
}
