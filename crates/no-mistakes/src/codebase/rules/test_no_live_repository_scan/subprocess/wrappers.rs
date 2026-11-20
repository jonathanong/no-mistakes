use super::{program_name, Token};

const VALUE_FLAGS: &[&str] = &["-C", "--dir", "--filter", "-F"];
const NPX_FLAGS: &[&str] = &["-y", "--yes", "--no-install"];
const NPX_VALUES: &[&str] = &["-p", "--package"];

pub(super) fn unwrap_line(executable: &str, argv: &[Token]) -> Option<Vec<Token>> {
    let line = match program_name(executable) {
        "pnpm" => verb(skip_flags(argv, VALUE_FLAGS)?, &["exec", "dlx"])?,
        "npm" => verb(skip_flags(argv, VALUE_FLAGS)?, &["exec"])?,
        "npx" | "bunx" => skip_separator(skip_npx(argv)?).to_vec(),
        "yarn" => yarn(argv).to_vec(),
        _ => return None,
    };
    Some(line)
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

fn skip_flags<'a>(argv: &'a [Token], values: &[&str]) -> Option<&'a [Token]> {
    let mut rest = argv;
    while let Some(Token::Known(token)) = rest.first() {
        if !token.starts_with('-') {
            break;
        }
        if values.contains(&token.as_str()) {
            if !matches!(rest.get(1), Some(Token::Known(_))) {
                return None;
            }
            rest = &rest[2..];
        } else {
            rest = &rest[1..];
        }
    }
    Some(rest)
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

fn yarn(argv: &[Token]) -> &[Token] {
    match argv.first() {
        Some(Token::Known(token)) if token == "dlx" => skip_separator(&argv[1..]),
        _ => argv,
    }
}

fn skip_separator(argv: &[Token]) -> &[Token] {
    match argv.first() {
        Some(Token::Known(token)) if token == "--" => &argv[1..],
        _ => argv,
    }
}
