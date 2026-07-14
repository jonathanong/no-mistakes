use std::collections::{BTreeSet, HashMap, HashSet};
use std::hash::Hash;

/// Compact equivalence classes for typed nodes sharing one public identity.
#[derive(Debug, Clone)]
pub(crate) struct NodeAliases<Node> {
    groups: Vec<Vec<Node>>,
    group_by_node: HashMap<Node, usize>,
}

impl<Node> NodeAliases<Node>
where
    Node: Clone + Eq + Hash + Ord,
{
    pub(crate) fn from_groups(groups: impl IntoIterator<Item = Vec<Node>>) -> Self {
        let mut aliases = Self {
            groups: Vec::new(),
            group_by_node: HashMap::new(),
        };
        for mut group in groups {
            group.sort();
            group.dedup();
            if group.len() < 2 {
                continue;
            }
            let group_id = aliases.groups.len();
            for node in &group {
                aliases.group_by_node.insert(node.clone(), group_id);
            }
            aliases.groups.push(group);
        }
        aliases
    }

    pub(crate) fn group(&self, node: &Node) -> Option<&[Node]> {
        self.group_by_node
            .get(node)
            .map(|&group_id| self.groups[group_id].as_slice())
    }

    pub(crate) fn expand(&self, nodes: impl IntoIterator<Item = Node>) -> BTreeSet<Node> {
        let mut expanded = BTreeSet::new();
        let mut expanded_groups = HashSet::new();
        for node in nodes {
            if let Some(&group_id) = self.group_by_node.get(&node) {
                if expanded_groups.insert(group_id) {
                    expanded.extend(self.groups[group_id].iter().cloned());
                }
            } else {
                expanded.insert(node);
            }
        }
        expanded
    }
}
