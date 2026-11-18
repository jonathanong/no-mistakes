use super::super::words::{command_at, Word};
use super::valid_name;

pub(super) fn observe(mut words: &[Word], mut mark: impl FnMut(&str)) -> Option<()> {
    for word in words {
        if let Some(name) = parameter_assignment(&word.text) {
            mark(name);
        }
    }

    loop {
        let head = command_at(words);
        let offset = head.as_ref().map_or(words.len(), |(_, index)| *index);
        for word in &words[..offset] {
            if let Some(name) = assignment_target(&word.text) {
                mark(name);
            }
        }
        let Some((name, offset)) = head else {
            return Some(());
        };
        if let Some(name) = assignment_target(&words[offset].text) {
            mark(name);
            words = &words[offset + 1..];
            continue;
        }
        let operands = &words[offset + 1..];
        return if name == "builtin" {
            let (name, args) = operands.split_first()?;
            observe_builtin(&name.text, args, &mut mark)
        } else {
            observe_builtin(&name, operands, &mut mark)
        };
    }
}

fn observe_builtin(name: &str, args: &[Word], mark: &mut impl FnMut(&str)) -> Option<()> {
    match name {
        "." | "source" | "eval" => None,
        "printf" => printf_target(args, mark),
        "declare" | "local" | "typeset" | "export" | "readonly" | "unset" => {
            declaration_targets(name, args, mark)
        }
        "read" => input_targets(args, "REPLY", "dinNptu", mark),
        "mapfile" | "readarray" => input_targets(args, "MAPFILE", "cdnOsu", mark),
        "for" | "select" => loop_target(args, mark),
        "getopts" => getopts_targets(args, mark),
        _ => Some(()),
    }
}

fn printf_target(args: &[Word], mark: &mut impl FnMut(&str)) -> Option<()> {
    let Some(attached) = args.first().and_then(|word| word.text.strip_prefix("-v")) else {
        return Some(());
    };
    let target = if attached.is_empty() {
        &args.get(1)?.text
    } else {
        attached
    };
    mark(variable_name(target)?);
    Some(())
}

fn loop_target(args: &[Word], mark: &mut impl FnMut(&str)) -> Option<()> {
    mark(variable_name(&args.first()?.text)?);
    Some(())
}

fn getopts_targets(args: &[Word], mark: &mut impl FnMut(&str)) -> Option<()> {
    if let Some(target) = args.get(1) {
        mark(variable_name(&target.text)?);
        mark("OPTARG");
        mark("OPTIND");
    }
    Some(())
}

fn declaration_targets(name: &str, mut args: &[Word], mark: &mut impl FnMut(&str)) -> Option<()> {
    let mut nameref = false;
    while let Some((flag, rest)) = args.split_first() {
        let Some(flags) = flag
            .text
            .strip_prefix('-')
            .or_else(|| flag.text.strip_prefix('+'))
        else {
            break;
        };
        args = rest;
        if flags == "-" {
            break;
        }
        if flags.contains(['f', 'F', 'p']) {
            return Some(());
        }
        nameref |= matches!(name, "declare" | "typeset" | "local")
            && flag.text.starts_with('-')
            && flags.contains('n');
    }
    if nameref && !args.is_empty() {
        return None;
    }
    for arg in args {
        if matches!(name, "export" | "readonly") && !arg.text.contains('=') {
            continue;
        }
        let target = arg
            .text
            .split_once('=')
            .map_or(arg.text.as_str(), |(name, _)| name);
        mark(variable_name(target.trim_end_matches('+'))?);
    }
    Some(())
}

fn input_targets(
    mut args: &[Word],
    default: &str,
    value_flags: &str,
    mark: &mut impl FnMut(&str),
) -> Option<()> {
    let mut explicit_target = false;
    while let Some((flag, rest)) = args.split_first() {
        let Some(flags) = flag.text.strip_prefix('-') else {
            break;
        };
        args = rest;
        if flags == "-" {
            break;
        }
        let read_array = default == "REPLY";
        let option_with_value = flags.char_indices().find(|&(_, flag)| {
            value_flags.contains(flag)
                || (read_array && flag == 'a')
                || (!read_array && flag == 'C')
        });
        if let Some((offset, flag)) = option_with_value {
            if flag == 'C' {
                return None;
            }
            let attached = &flags[offset + flag.len_utf8()..];
            let value = if attached.is_empty() {
                let (word, rest) = args.split_first()?;
                args = rest;
                &word.text
            } else {
                attached
            };
            if flag == 'a' {
                mark(variable_name(value)?);
                explicit_target = true;
            }
        }
    }
    if args.is_empty() && !explicit_target {
        mark(default);
    }
    for target in args {
        mark(variable_name(&target.text)?);
    }
    Some(())
}

fn assignment_target(text: &str) -> Option<&str> {
    let (target, _) = text.trim_start_matches(['(', '{']).split_once('=')?;
    variable_name(target.trim_end_matches('+'))
}

fn variable_name(text: &str) -> Option<&str> {
    let name = text.split_once('[').map_or(text, |(name, _)| name);
    valid_name(name).then_some(name)
}

fn parameter_assignment(text: &str) -> Option<&str> {
    let (target, _) = text.strip_prefix("${")?.split_once('=')?;
    variable_name(target.trim_end_matches(':'))
}
