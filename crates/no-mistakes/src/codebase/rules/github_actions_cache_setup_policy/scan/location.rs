use super::{Document, RuleFinding, RULE_ID};

pub(super) fn field(doc: Option<&Document>, node: Option<usize>, key: &str) -> Option<usize> {
    doc?.field(node?, key)
}

pub(super) fn line(doc: Option<&Document>, node: Option<usize>) -> usize {
    doc.zip(node).map_or(1, |(doc, node)| doc.nodes[node].line)
}

pub(super) struct StepLocation<'a> {
    pub(super) rel: &'a str,
    pub(super) document: Option<&'a Document>,
    pub(super) node: Option<usize>,
    pub(super) workflow: bool,
}

impl StepLocation<'_> {
    pub(super) fn finding(&self, fields: &[&str], target: &str, detail: &str) -> RuleFinding {
        let mut node = self.node;
        let mut found = line(self.document, node);
        for key in fields {
            let Some(next) = field(self.document, node, key) else {
                break;
            };
            found = self
                .document
                .zip(node)
                .and_then(|(doc, node)| doc.field_line(node, key))
                .unwrap_or(found);
            node = Some(next);
        }
        finding(self.rel, found, target, detail)
    }
}

pub(super) fn finding(rel: &str, line: usize, target: &str, detail: &str) -> RuleFinding {
    RuleFinding {
        rule: RULE_ID.to_owned(),
        file: rel.to_owned(),
        line,
        message: format!("{rel}:{line}: {detail}"),
        target: Some(target.to_owned()),
        import: None,
        causes: Vec::new(),
    }
}
