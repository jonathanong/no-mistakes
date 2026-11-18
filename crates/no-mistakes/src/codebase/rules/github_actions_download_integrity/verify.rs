use super::words::{self, Command};

pub(super) fn command_verifies(command: &Command) -> bool {
    if command.dynamic {
        return false;
    }
    let Some((name, offset)) = words::command_at(&command.words) else {
        return false;
    };
    let args = &command.words[offset + 1..];
    match name.as_str() {
        "sha256sum" => sha256sum_verifies(args),
        "shasum" => shasum_256_check(args),
        _ => false,
    }
}

fn sha256sum_verifies(args: &[words::Word]) -> bool {
    for word in args {
        if sha256_check(&word.text) {
            return true;
        }
        if ends_known_options(word) {
            break;
        }
    }
    false
}

fn sha256_check(word: &str) -> bool {
    word == "-c"
        || word == "--check"
        || word.starts_with("--check=")
        || (word.starts_with('-')
            && !word.starts_with("--")
            && word.contains('c')
            && word
                .chars()
                .skip(1)
                .all(|ch| matches!(ch, 'b' | 'c' | 't' | 'w' | 'z')))
}

fn shasum_256_check(args: &[words::Word]) -> bool {
    let mut args = args.iter();
    let mut algorithm = false;
    let mut check = false;
    while let Some(word) = args.next() {
        match word.text.as_str() {
            "-a" => {
                let Some(value) = args.next() else {
                    return false;
                };
                if ends_known_options(value) {
                    return false;
                }
                algorithm = value.text == "256";
            }
            "-a256" => algorithm = true,
            "-c" | "--check" => check = true,
            value if value.starts_with("--check=") => check = true,
            _ if ends_known_options(word) => break,
            _ => {}
        }
        if algorithm && check {
            return true;
        }
    }
    false
}

fn ends_known_options(word: &words::Word) -> bool {
    word.text == "--" || word.expansion_may_split || word.quoted_expansion.is_some()
}

pub(super) fn takes_url_slot(command: &str, token: &str) -> bool {
    // `--` and `--url=` are peeled off before this is called.
    if token.starts_with("--") && token.contains('=') {
        return false;
    }
    let curl = [
        "-o",
        "--output",
        "-H",
        "--header",
        "-d",
        "--data",
        "--data-raw",
        "--data-binary",
        "-e",
        "--referer",
        "-A",
        "--user-agent",
        "-u",
        "--user",
        "-x",
        "--proxy",
        "-w",
        "--write-out",
        "-b",
        "--cookie",
        "--output-dir",
    ];
    let wget = [
        "-O",
        "--output-document",
        "-o",
        "--output-file",
        "--header",
        "--post-data",
        "--user",
        "--password",
        "-P",
        "--directory-prefix",
    ];
    match command {
        "curl" => curl.contains(&token),
        "wget" => wget.contains(&token),
        _ => false,
    }
}
