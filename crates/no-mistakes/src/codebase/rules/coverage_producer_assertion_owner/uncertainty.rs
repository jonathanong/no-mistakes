use super::*;
use crate::codebase::rules::FindingCause;
use std::path::PathBuf;

/// Causes named in the message; `causes` carries up to `STRUCTURED_CAUSES`.
const MESSAGE_CAUSES: usize = 5;
const STRUCTURED_CAUSES: usize = 25;

#[derive(PartialEq, Eq, PartialOrd, Ord)]
pub(super) enum Cause {
    Dispatch {
        file: PathBuf,
        line: u32,
        offset: u32,
        callee: String,
        detail: String,
    },
    Limit {
        name: &'static str,
        value: usize,
        node: NodeId,
    },
    Generator(NodeId),
}

impl traversal::Outcome {
    pub(super) fn uncertain(&self) -> bool {
        !self.assertion && !self.causes.is_empty()
    }

    pub(super) fn dispatch(&mut self, site: &ResolvedCallSite, detail: impl Into<String>) {
        self.causes.insert(Cause::Dispatch {
            file: site.file.clone(),
            line: site.line,
            offset: site.offset,
            callee: site.source_callee.clone(),
            detail: detail.into(),
        });
    }

    pub(super) fn limit(&mut self, name: &'static str, value: usize, node: &NodeId) {
        self.causes.insert(Cause::Limit {
            name,
            value,
            node: node.clone(),
        });
    }

    pub(super) fn reason(&self, root: &Path) -> String {
        let shown = self.causes.iter().take(MESSAGE_CAUSES);
        let mut causes = shown
            .map(|cause| match cause {
                Cause::Dispatch {
                    file,
                    line,
                    callee,
                    detail,
                    ..
                } => format!(
                    "unresolved dispatch at {}:{line} callee `{callee}` ({detail})",
                    relative_slash_path(root, file)
                ),
                Cause::Limit { name, value, node } => format!(
                    "traversal limit {name}={value}; last module {}",
                    node.display_name(root).replace('\\', "/")
                ),
                Cause::Generator(node) => format!(
                    "generator body execution is not modeled in {}",
                    node.display_name(root).replace('\\', "/")
                ),
            })
            .collect::<Vec<_>>()
            .join("; ");
        if self.causes.len() > MESSAGE_CAUSES {
            causes.push_str(&format!("; +{} more", self.causes.len() - MESSAGE_CAUSES));
        }
        format!("unanalyzable coverage producer: {causes}; model the dispatch ownership, rewrite it statically, or adjust the named traversal limit")
    }

    pub(super) fn finding_causes(&self, root: &Path) -> Vec<FindingCause> {
        let rel = |file: &Path| relative_slash_path(root, file);
        self.causes
            .iter()
            .take(STRUCTURED_CAUSES)
            .map(|cause| match cause {
                Cause::Dispatch {
                    file,
                    line,
                    callee,
                    detail,
                    ..
                } => FindingCause {
                    kind: "unresolved-dispatch",
                    file: rel(file),
                    line: Some(*line as usize),
                    callee: Some(callee.clone()),
                    detail: detail.clone(),
                },
                Cause::Limit { name, value, node } => FindingCause {
                    kind: "traversal-limit",
                    file: rel(node.as_file().expect("callable file")),
                    line: None,
                    callee: None,
                    detail: format!(
                        "{name}={value}; last module {}",
                        node.display_name(root).replace('\\', "/")
                    ),
                },
                Cause::Generator(node) => FindingCause {
                    kind: "unmodeled-generator",
                    file: rel(node.as_file().expect("callable file")),
                    line: None,
                    callee: None,
                    detail: node.display_name(root).replace('\\', "/"),
                },
            })
            .collect()
    }
}
