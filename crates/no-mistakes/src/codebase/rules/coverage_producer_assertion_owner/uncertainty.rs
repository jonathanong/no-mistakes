use super::*;
use std::path::PathBuf;

#[derive(PartialEq, Eq, PartialOrd, Ord)]
pub(super) enum Cause {
    Dispatch {
        file: PathBuf,
        line: u32,
        offset: u32,
        callee: String,
        detail: &'static str,
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

    pub(super) fn dispatch(&mut self, site: &ResolvedCallSite, detail: &'static str) {
        self.causes.insert(Cause::Dispatch {
            file: site.file.clone(),
            line: site.line,
            offset: site.offset,
            callee: site.source_callee.clone(),
            detail,
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
        let causes = self
            .causes
            .iter()
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
        format!("unanalyzable coverage producer: {causes}; model the dispatch ownership, rewrite it statically, or adjust the named traversal limit")
    }
}
