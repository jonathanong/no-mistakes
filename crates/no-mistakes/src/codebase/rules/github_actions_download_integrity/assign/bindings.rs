use super::super::{env, url, words};
use super::{classify, probe, uppercase_name, Kind};
use crate::fx::{fx_map, FxHashMap};

#[derive(Default)]
pub(in super::super) struct Bindings {
    values: FxHashMap<String, Option<Binding>>,
}

struct Binding {
    value: String,
    line: usize,
    partial_url: bool,
}

impl Bindings {
    pub(in super::super) fn collect(script: &str, parsed: &words::Parse) -> Self {
        probe::note_binding_scan();
        if !script.contains('$') {
            return Self::default();
        }
        let lines: Vec<_> = script.lines().collect();
        let mut values = fx_map::<String, Option<Binding>>();
        for command in &parsed.commands {
            let mut candidate = assignment(command, &lines);
            if env::observe_writes(&command.words, |name| {
                if uppercase_name(name) {
                    let binding = candidate
                        .take_if(|(key, _)| *key == name)
                        .map(|(_, binding)| binding);
                    values
                        .entry(name.to_owned())
                        .and_modify(|prior| *prior = None)
                        .or_insert(binding);
                }
            })
            .is_none()
            {
                return Self::default();
            }
        }
        Self { values }
    }

    pub(in super::super) fn has_literals(&self) -> bool {
        self.values
            .values()
            .flatten()
            .any(|binding| !binding.partial_url)
    }

    pub(in super::super) fn has_partial_urls(&self) -> bool {
        self.values
            .values()
            .flatten()
            .any(|binding| binding.partial_url)
    }

    pub(in super::super) fn literal(&self, name: &str, line: usize) -> Option<&str> {
        let binding = self.before(name, line)?;
        (!binding.partial_url).then_some(&binding.value)
    }

    pub(in super::super) fn resolve_partial_url(
        &self,
        word: &words::Word,
        operand: &str,
    ) -> Option<String> {
        let offset = word.text.len() - operand.len();
        let (_, len) = [word.quoted_expansion, word.bare_expansion]
            .into_iter()
            .flatten()
            .find(|(start, _)| *start == offset)?;
        let (reference, suffix) = operand.split_at_checked(len)?;
        let name = super::reference_name(reference.strip_prefix('$')?);
        let binding = self.before(name, word.line)?;
        binding
            .partial_url
            .then(|| format!("{}{suffix}", binding.value))
    }

    fn before(&self, name: &str, line: usize) -> Option<&Binding> {
        let binding = self.values.get(name)?.as_ref()?;
        (binding.line < line).then_some(binding)
    }
}

fn assignment<'a>(command: &words::Command, lines: &[&'a str]) -> Option<(&'a str, Binding)> {
    let [word] = command.words.as_slice() else {
        return None;
    };
    let line = lines.get(word.line - 1)?.trim();
    let (name, raw) = line.split_once('=')?;
    let (value, partial_url) = match classify(line) {
        Kind::Literal(_, value) => (value, false),
        Kind::Other if uppercase_name(name) => (partial_url(raw)?, true),
        _ => return None,
    };
    if word.text.strip_prefix(name)?.strip_prefix('=')? != value {
        return None;
    }
    Some((
        name,
        Binding {
            value,
            line: word.line,
            partial_url,
        },
    ))
}

fn partial_url(raw: &str) -> Option<String> {
    let value = super::quoted(raw).or_else(|| {
        (!raw.is_empty() && !raw.contains(char::is_whitespace)).then(|| raw.to_owned())
    })?;
    url::has_static_url_prefix(&value).then_some(value)
}
