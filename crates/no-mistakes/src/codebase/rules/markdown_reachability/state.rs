use super::BaselineEntry;

pub(super) struct RuleState {
    pub(super) finding_file: String,
    pub(super) depth: Option<usize>,
    pub(super) allowed: bool,
    pub(super) invalid_intermediary: bool,
}

pub(super) fn expected_state(depth: Option<usize>, allowed: bool) -> Option<BaselineEntry> {
    (!allowed).then(|| match depth {
        Some(depth) => BaselineEntry::depth(depth),
        None => BaselineEntry::unreachable(),
    })
}
