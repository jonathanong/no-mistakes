use super::{dynamic, Check, Word};

pub(super) fn cargo(mut args: &[Word], command_dynamic: bool) -> Option<Check> {
    if let Some((first, rest)) = args
        .split_first()
        .filter(|(first, _)| first.text.starts_with('+'))
    {
        if dynamic(first, command_dynamic) {
            return Some(Check::Unknown("selects a dynamic Cargo toolchain"));
        }
        args = rest;
    }
    let args = skip_options(args, command_dynamic, CargoMode::Cargo)?;
    let (first, rest) = args.split_first()?;
    if dynamic(first, command_dynamic) {
        return Some(Check::Unknown("passes a dynamic subcommand to Cargo"));
    }
    (first.text == "insta")
        .then(|| insta(rest, command_dynamic))
        .flatten()
}

pub(super) fn insta(args: &[Word], command_dynamic: bool) -> Option<Check> {
    if args.iter().any(|word| word.text.contains("${{")) {
        return Some(Check::Unknown("uses a GitHub expression in `run`"));
    }
    let unknown = Check::Unknown("passes dynamic arguments to cargo-insta");
    let args = skip_options(args, command_dynamic, CargoMode::Insta)?;
    let (subcommand, rest) = args.split_first()?;
    if dynamic(subcommand, command_dynamic) {
        return Some(unknown);
    }
    let rest = rest
        .split(|word| word.text == "--")
        .next()
        .unwrap_or_default();
    if rest
        .iter()
        .any(|word| matches!(word.text.as_str(), "--help" | "-h"))
    {
        return None;
    }
    match subcommand.text.as_str() {
        "accept" | "approve" => Some(Check::Accept("accept")),
        "test" => {
            for word in rest {
                if dynamic(word, command_dynamic) {
                    return Some(unknown);
                }
                let flag = match word.text.as_str() {
                    "--accept" => "--accept",
                    "--accept-unseen" => "--accept-unseen",
                    "--force-update-snapshots" => "--force-update-snapshots",
                    text if matches!(
                        text.trim_end_matches([')', '}']),
                        "--accept" | "--accept-unseen" | "--force-update-snapshots"
                    ) =>
                    {
                        return Some(Check::Unknown(
                            "uses an ambiguous grouped cargo-insta argument",
                        ))
                    }
                    _ => continue,
                };
                return Some(Check::Accept(flag));
            }
            None
        }
        text if matches!(text.trim_end_matches([')', '}']), "accept" | "approve") => Some(
            Check::Unknown("uses an ambiguous grouped cargo-insta subcommand"),
        ),
        _ => None,
    }
}

enum CargoMode {
    Cargo,
    Insta,
}

fn skip_options(mut args: &[Word], command_dynamic: bool, mode: CargoMode) -> Option<&[Word]> {
    while let [word, rest @ ..] = args {
        let text = word.text.as_str();
        match text {
            "-h" | "--help" | "-V" | "--version" | "--" => return None,
            "--color" => args = rest.get(1..)?,
            "-C" | "--config" | "-Z" if matches!(mode, CargoMode::Cargo) => args = rest.get(1..)?,
            option if option.starts_with('-') && !dynamic(word, command_dynamic) => args = rest,
            _ => break,
        }
    }
    Some(args)
}
