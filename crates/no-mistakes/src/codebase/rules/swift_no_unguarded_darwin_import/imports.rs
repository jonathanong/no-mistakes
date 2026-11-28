use super::conditions::parse_condition;
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

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) enum Proof<'a> {
    None,
    All,
    Modules(Vec<&'a str>),
}

impl<'a> Proof<'a> {
    pub(super) fn covers(&self, module: &str) -> bool {
        match self {
            Self::None => false,
            Self::All => true,
            Self::Modules(modules) => modules.contains(&module),
        }
    }

    pub(super) fn and(self, other: Self) -> Self {
        match (self, other) {
            (Self::All, _) | (_, Self::All) => Self::All,
            (Self::None, rhs) => rhs,
            (lhs, Self::None) => lhs,
            (Self::Modules(mut lhs), Self::Modules(rhs)) => {
                for m in rhs {
                    if !lhs.contains(&m) {
                        lhs.push(m);
                    }
                }
                Self::Modules(lhs)
            }
        }
    }

    pub(super) fn or(self, other: Self) -> Self {
        match (self, other) {
            (Self::None, _) | (_, Self::None) => Self::None,
            (Self::All, rhs) => rhs,
            (lhs, Self::All) => lhs,
            (Self::Modules(lhs), Self::Modules(rhs)) => {
                let shared: Vec<_> = lhs.into_iter().filter(|m| rhs.contains(m)).collect();
                if shared.is_empty() {
                    Self::None
                } else {
                    Self::Modules(shared)
                }
            }
        }
    }
}

pub(super) fn valid_module_name(name: &str) -> bool {
    MODULE_NAME.is_match(name)
}

pub(super) enum Import {
    Unguarded,
    Unproven,
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
        let module = event
            .name("module")
            .expect("import module")
            .as_str()
            .trim_matches('\x60');
        if !modules.contains(module) || guards.iter().any(|proof| proof.covers(module)) {
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

fn update_guards<'a>(guards: &mut Vec<Proof<'a>>, directive: &str, tail: &'a str) {
    if directive != "#if" {
        guards.pop();
    }
    if directive != "#endif" {
        let proof = if directive == "#else" {
            Proof::None
        } else {
            parse_condition(tail)
        };
        guards.push(proof);
    }
}
