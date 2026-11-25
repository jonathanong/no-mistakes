use regex::Regex;
use std::sync::LazyLock;

static EVENTS: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(concat!(
        r"(?P<directive>#(?:if|elseif|else|endif))(?:[\x00\x0B\t\r\n (!;})\].]|$)|",
        r"\bimport[\x00\x0B\t\r\n ]+",
        r"(?:(?:typealias|struct|class|enum|protocol|let|var|func)[\x00\x0B\t\r\n ]+)?",
        r"(?:Darwin|\x60Darwin\x60)(?:[\x00\x0B\t\r\n .;]|$)",
    ))
    .expect("Swift directives and Darwin imports")
});

static POSITIVE_GUARD: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r"^(?:canImport\s*\(\s*Darwin\s*\)|os\s*\(\s*(?:macOS|iOS|tvOS|watchOS|visionOS)\s*\))$",
    )
    .expect("positive Swift platform guards")
});

pub(super) enum Import {
    Unguarded,
    Unproven,
}

pub(super) fn unproven_imports(code: &str) -> impl Iterator<Item = (usize, Import)> + '_ {
    let mut guards = Vec::new();
    let mut previous_offset = 0;
    let mut line = 1;
    EVENTS.captures_iter(code).filter_map(move |event| {
        let matched = event.get(0).expect("event match");
        line += memchr::memchr_iter(b'\n', &code.as_bytes()[previous_offset..matched.start()]).count();
        previous_offset = matched.start();
        if let Some(directive) = event.name("directive") {
            update_guards(&mut guards, directive.as_str(), &code[directive.end()..]);
            return None;
        }
        match guards.last() {
            None => Some((line, Import::Unguarded)),
            Some(false) => Some((line, Import::Unproven)),
            Some(true) => None,
        }
    })
}

fn update_guards(guards: &mut Vec<bool>, directive: &str, tail: &str) {
    if directive != "#if" {
        guards.pop();
    }
    if directive != "#endif" {
        let inherited = guards.last().copied().unwrap_or(false);
        let positive = directive != "#else" && positive_condition(tail);
        guards.push(inherited || positive);
    }
}

fn positive_condition(tail: &str) -> bool {
    let (condition, following) = tail.split_once('\n').unwrap_or((tail, ""));
    let mut condition = condition.trim();
    while let Some(inner) = condition.strip_prefix('(').and_then(|text| text.strip_suffix(')')) {
        condition = inner.trim();
    }
    let following = following.trim_start();
    POSITIVE_GUARD.is_match(condition) && !following.starts_with("&&") && !following.starts_with("||")
}
