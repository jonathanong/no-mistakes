use super::words::{basename, commands, is_assignment};

pub(super) fn step_verifies(script: &str) -> bool {
    commands(script).into_iter().any(|command| {
        if command.dynamic {
            return false;
        }
        let words: Vec<&str> = command
            .words
            .iter()
            .skip_while(|word| is_assignment(&word.text))
            .map(|word| word.text.as_str())
            .collect();
        let Some(name) = words.first().map(|word| basename(word)) else {
            return false;
        };
        if name == "sha256sum" {
            return words.iter().any(|word| sha256_check(word));
        }
        name == "shasum" && shasum_256_check(&words)
    })
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

fn shasum_256_check(words: &[&str]) -> bool {
    let algorithm = words.windows(2).any(|pair| pair == ["-a", "256"])
        || words.contains(&"-a256");
    let check = words
        .iter()
        .any(|word| *word == "-c" || *word == "--check" || word.starts_with("--check="));
    algorithm && check
}

pub(super) fn takes_url_slot(command: &str, token: &str) -> bool {
    if token == "--" || token.starts_with("--url=") {
        return false;
    }
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
