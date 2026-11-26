use std::collections::BTreeMap;

mod origin;
mod source;
mod text;

pub(crate) use origin::{CspDirective, ParsedOrigin};
use source::{parse_source_token, source_matches, CspSource};

#[derive(Default)]
pub(crate) struct CspPolicy {
    directives: BTreeMap<CspDirective, Vec<CspSource>>,
}

#[derive(Default)]
pub(crate) struct CspPolicySet {
    policies: Vec<CspPolicy>,
}

impl CspPolicySet {
    pub(crate) fn is_empty(&self) -> bool {
        self.policies.is_empty()
    }

    pub(crate) fn extend(&mut self, other: Self) {
        self.policies.extend(other.policies);
    }

    pub(crate) fn parse_from_strings(strings: &[String]) -> Self {
        let mut policies = Vec::new();
        for text in strings {
            policies.extend(CspPolicy::parse_list(text));
        }
        Self { policies }
    }

    pub(crate) fn parse_from_text(text: &str) -> Self {
        Self {
            policies: text::parse_policies_from_text(text),
        }
    }

    pub(crate) fn allows(
        &self,
        directive: CspDirective,
        origin: &ParsedOrigin,
        self_origins: &[ParsedOrigin],
    ) -> bool {
        !self.policies.is_empty()
            && self
                .policies
                .iter()
                .all(|policy| policy.allows(directive, origin, self_origins))
    }
}

impl CspPolicy {
    pub(super) fn is_empty(&self) -> bool {
        self.directives.is_empty()
    }

    pub(super) fn parse_list(text: &str) -> Vec<Self> {
        let mut policies = Vec::new();
        for serialized in text.split(',') {
            let mut policy = Self::default();
            policy.parse_single_policy(serialized);
            if !policy.is_empty() {
                policies.push(policy);
            }
        }
        policies
    }

    pub(super) fn parse_single_policy(&mut self, text: &str) {
        for part in text.split(';') {
            let mut tokens = part.split_whitespace();
            let Some(name) = tokens.next() else {
                continue;
            };
            let Some(directive) = CspDirective::parse(name) else {
                continue;
            };
            let sources: Vec<CspSource> = tokens.filter_map(parse_source_token).collect();
            self.directives.entry(directive).or_insert(sources);
        }
    }

    fn allows(
        &self,
        directive: CspDirective,
        origin: &ParsedOrigin,
        self_origins: &[ParsedOrigin],
    ) -> bool {
        let Some(sources) = self.effective_sources(directive) else {
            return true;
        };
        sources
            .iter()
            .any(|source| source_matches(source, origin, self_origins))
    }

    fn effective_sources(&self, directive: CspDirective) -> Option<&[CspSource]> {
        match directive {
            CspDirective::Script => self
                .directives
                .get(&CspDirective::ScriptElem)
                .or_else(|| self.directives.get(&CspDirective::Script)),
            CspDirective::Style => self
                .directives
                .get(&CspDirective::StyleElem)
                .or_else(|| self.directives.get(&CspDirective::Style)),
            CspDirective::Frame => self
                .directives
                .get(&CspDirective::Frame)
                .or_else(|| self.directives.get(&CspDirective::Child)),
            other => self.directives.get(&other),
        }
        .or_else(|| self.directives.get(&CspDirective::Default))
        .map(Vec::as_slice)
    }
}
