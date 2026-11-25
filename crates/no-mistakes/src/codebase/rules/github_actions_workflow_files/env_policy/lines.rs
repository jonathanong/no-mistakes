use super::super::job_lines::document::Document;

#[derive(Clone)]
pub(in crate::codebase::rules) enum Seg {
    Key(String),
    Idx(usize),
}

pub(in crate::codebase::rules) fn parse(source: &str) -> Option<Document> {
    Document::parse(source)
}

// Returns the deepest line that matches; a missing child falls back to its parent.
pub(in crate::codebase::rules) fn locate(doc: Option<&Document>, path: &[Seg]) -> usize {
    let Some(doc) = doc else {
        return 1;
    };
    let Some(&first) = doc.roots().first() else {
        return 1;
    };
    let mut node = first;
    let mut found = doc.nodes[node].line;
    for seg in path {
        match seg {
            Seg::Key(key) => {
                let Some(pair) = doc.mapping(node).and_then(|pairs| {
                    pairs
                        .as_chunks::<2>()
                        .0
                        .iter()
                        .find(|pair| doc.key(pair[0]) == Some(key.as_str()))
                }) else {
                    break;
                };
                found = doc.nodes[pair[0]].line;
                node = pair[1];
            }
            Seg::Idx(want) => {
                let Some(&child) = doc.sequence(node).and_then(|items| items.get(*want)) else {
                    break;
                };
                found = doc.nodes[child].line;
                node = child;
            }
        }
    }
    found
}
