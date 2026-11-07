use super::*;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub(super) struct Frame {
    pub(super) file: PathBuf,
    pub(super) owner: CallableId,
    pub(super) arguments: Vec<bool>,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub(super) enum Key {
    Binding(PathBuf, usize, String, u32, Option<Frame>),
    Call(PathBuf, u32, u32, bool, Option<Frame>),
}

pub(super) struct Prepared<'a> {
    pub(super) calls: FxHashMap<(u32, u32), &'a ValueCall>,
    pub(super) returns: FxHashMap<CallableId, Arc<[&'a ValueExpression]>>,
    pub(super) generated: FxHashMap<CallableId, Arc<[&'a ValueExpression]>>,
}

impl<'a, 'b> Proof<'a, 'b> {
    pub(super) fn prepared(&mut self, file: &Path) -> Option<&Prepared<'a>> {
        if !self.files.contains_key(file) {
            let ts = self.facts.get_ts_facts(file)?;
            let flow = &ts.value_flow;
            if ts.parse_error.is_some()
                || ts.operational_error.is_some()
                || flow.incomplete_reason.is_some()
            {
                return None;
            }
            let mut returns: FxHashMap<_, Vec<_>> = FxHashMap::default();
            for (owner, value) in &flow.returns {
                returns.entry(*owner).or_default().push(value);
            }
            let mut generated: FxHashMap<_, Vec<_>> = FxHashMap::default();
            for returned in &flow.generated_identifier_returns {
                generated
                    .entry(returned.owner)
                    .or_default()
                    .push(&returned.prefix);
            }
            self.files.insert(
                file.to_path_buf(),
                Prepared {
                    calls: flow
                        .calls
                        .iter()
                        .map(|call| ((call.offset, call.end), call))
                        .collect(),
                    returns: returns
                        .into_iter()
                        .map(|(id, values)| (id, values.into()))
                        .collect(),
                    generated: generated
                        .into_iter()
                        .map(|(id, values)| (id, values.into()))
                        .collect(),
                },
            );
        }
        self.files.get(file)
    }
}
