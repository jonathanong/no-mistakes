fn interpreter_script(words: &[String]) -> Option<&str> {
    let interpreter = words.first()?.as_str();
    if !matches!(
        interpreter,
        "node"
            | "bun"
            | "deno"
            | "tsx"
            | "ts-node"
            | "bash"
            | "sh"
            | "zsh"
            | "python"
            | "python3"
            | "ruby"
    ) {
        return None;
    }
    let mut operands = &words[1..];
    if interpreter == "deno" {
        if operands.first().map(String::as_str) != Some("run") {
            return None;
        }
        operands = &operands[1..];
    }
    let candidate = operands.first()?.as_str();
    if candidate == "--" {
        return operands.get(1).map(String::as_str);
    }
    if matches!(
        candidate,
        "-c" | "-e" | "-m" | "-r" | "--require" | "--loader" | "--import" | "--config"
    ) {
        return None;
    }
    operands
        .iter()
        .find(|candidate| !candidate.starts_with('-'))
        .map(String::as_str)
}

fn package_script_command(words: &[String]) -> Option<&str> {
    match words.first()?.as_str() {
        "npm"
            if matches!(
                words.get(1).map(String::as_str),
                Some("run" | "run-script")
            ) =>
        {
            words.get(2).map(String::as_str)
        }
        "npm" if words.get(1).map(String::as_str) == Some("test") => Some("test"),
        "pnpm" | "bun" if words.get(1).map(String::as_str) == Some("run") => {
            words.get(2).map(String::as_str)
        }
        "pnpm" => words.get(1).map(String::as_str),
        "yarn" if words.get(1).map(String::as_str) == Some("run") => {
            words.get(2).map(String::as_str)
        }
        "yarn" => words.get(1).map(String::as_str),
        _ => None,
    }
}
