use super::KeyOptions;
use anyhow::{ensure, Context, Result};
use serde_json::Value;
use std::collections::BTreeMap;

pub(super) fn json_entries(source: &str, options: &KeyOptions) -> Result<BTreeMap<String, Value>> {
    let mut root: Value = serde_json::from_str(source).context("invalid JSON")?;
    let (Some(path), Some(id_field)) = (&options.array, &options.id_field) else {
        let Value::Object(entries) = root else {
            anyhow::bail!("expected a JSON object");
        };
        return Ok(entries.into_iter().collect());
    };
    let array = if path.starts_with('/') {
        root.pointer_mut(path)
    } else {
        path.split('.')
            .try_fold(&mut root, |value, field| value.get_mut(field))
    };
    let entries = array
        .and_then(Value::as_array_mut)
        .with_context(|| format!("expected a JSON array at {path:?}"))?;
    let mut records = BTreeMap::new();
    for entry in std::mem::take(entries) {
        let id = entry
            .get(id_field)
            .and_then(Value::as_str)
            .filter(|id| !id.is_empty())
            .with_context(|| format!("JSON array entry needs a nonempty string {id_field:?}"))?
            .to_string();
        insert_unique(&mut records, id, entry)?;
    }
    Ok(records)
}

pub(super) fn resx_entries(source: &str) -> Result<BTreeMap<String, String>> {
    let document = roxmltree::Document::parse(source).context("invalid resx XML")?;
    let root = document.root_element();
    ensure!(root.has_tag_name("root"), "expected a resx root element");
    let mut entries = BTreeMap::new();
    for node in root.children().filter(|node| node.has_tag_name("data")) {
        let name = node
            .attribute("name")
            .context("resx data needs a name attribute")?;
        let mut values = node.children().filter(|child| child.has_tag_name("value"));
        let value = values.next().context("resx data needs a value element")?;
        ensure!(
            values.next().is_none(),
            "resx data has multiple value elements"
        );
        ensure!(
            !value.children().any(|child| child.is_element()),
            "resx values must contain text"
        );
        let text = value
            .children()
            .filter(|child| child.is_text())
            .filter_map(|child| child.text())
            .collect();
        insert_unique(&mut entries, name.to_string(), text)?;
    }
    Ok(entries)
}

pub(super) fn insert_unique<V>(
    entries: &mut BTreeMap<String, V>,
    key: String,
    value: V,
) -> Result<()> {
    use std::collections::btree_map::Entry;
    match entries.entry(key) {
        Entry::Occupied(existing) => anyhow::bail!("duplicate key {:?}", existing.key()),
        Entry::Vacant(slot) => {
            slot.insert(value);
            Ok(())
        }
    }
}
