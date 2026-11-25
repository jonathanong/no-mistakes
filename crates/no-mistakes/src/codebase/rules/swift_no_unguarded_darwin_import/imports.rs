use regex::Regex;
use rustc_hash::FxHashSet;
use std::sync::LazyLock;

const IDENTIFIER: &str = r"[_\p{XID_Start}][_\p{XID_Continue}]*";

static MODULE_NAME: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(&format!(r"\A{IDENTIFIER}\z")).expect("Swift module identifier"));

static EVENTS: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(&format!(
        concat!(
            r"(?P<directive>#(?:if|elseif|else|endif))(?:[\x00\x0B\t\r\n (!;}})\].]|$)|",
            r"\bimport[\x00\x0B\t\r\n ]+",
            r"(?:(?:typealias|struct|class|enum|protocol|let|var|func)[\x00\x0B\t\r\n ]+)?",
            r"(?P<module>{0}|\x60{0}\x60)(?:[\x00\x0B\t\r\n .;]|$)"
        ),
        IDENTIFIER,
    ))
    .expect("Swift directives and imports")
});

static POSITIVE_GUARD: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(&format!(
        concat!(
            r"^(?:canImport\s*\(\s*(?P<module>{0}|\x60{0}\x60)\s*\)|",
            r"os\s*\(\s*(?:macOS|OSX|iOS|tvOS|watchOS|visionOS|xrOS|Windows|Android",
            r"|FreeBSD|OpenBSD|PS4|Cygwin|Haiku|WASI|Emscripten|Firmware|none)\s*\))$"
        ),
        IDENTIFIER,
    ))
    .expect("positive Swift platform guards")
});

pub(super) fn valid_module_name(name: &str) -> bool {
    MODULE_NAME.is_match(name)
}

pub(super) enum Import {
    Unguarded,
    Unproven,
}

enum Guard<'a> {
    Module(&'a str),
    OutsideLinux,
}

pub(super) fn unproven_imports<'a>(
    code: &'a str,
    modules: &'a FxHashSet<String>,
) -> impl Iterator<Item = (usize, &'a str, Import)> + 'a {
    let mut guards = Vec::new();
    let mut previous_offset = 0;
    let mut line = 1;
    EVENTS.captures_iter(code).filter_map(move |event| {
        let matched = event.get(0).expect("event match");
        line +=
            memchr::memchr_iter(b'\n', &code.as_bytes()[previous_offset..matched.start()]).count();
        previous_offset = matched.start();
        if let Some(directive) = event.name("directive") {
            update_guards(&mut guards, directive.as_str(), &code[directive.end()..]);
            return None;
        }
        let module = event.name("module")?.as_str().trim_matches('\x60');
        if !modules.contains(module)
            || guards.iter().flatten().any(|guard| match guard {
                Guard::Module(guarded) => *guarded == module,
                Guard::OutsideLinux => true,
            })
        {
            return None;
        }
        let kind = if guards.is_empty() {
            Import::Unguarded
        } else {
            Import::Unproven
        };
        Some((line, module, kind))
    })
}

fn update_guards<'a>(guards: &mut Vec<Option<Guard<'a>>>, directive: &str, tail: &'a str) {
    if directive != "#if" {
        guards.pop();
    }
    if directive != "#endif" {
        let guard = if directive == "#else" {
            None
        } else {
            positive_condition(tail)
        };
        guards.push(guard);
    }
}

fn positive_condition(tail: &str) -> Option<Guard<'_>> {
    let (condition, following) = tail.split_once('\n').unwrap_or((tail, ""));
    let mut condition = condition.trim();
    while let Some(inner) = condition
        .strip_prefix('(')
        .and_then(|text| text.strip_suffix(')'))
    {
        condition = inner.trim();
    }
    let following = following.trim_start();
    if following.starts_with("&&") || following.starts_with("||") {
        return None;
    }
    let guard = POSITIVE_GUARD.captures(condition)?;
    Some(match guard.name("module") {
        Some(module) => Guard::Module(module.as_str().trim_matches('\x60')),
        None => Guard::OutsideLinux,
    })
}
