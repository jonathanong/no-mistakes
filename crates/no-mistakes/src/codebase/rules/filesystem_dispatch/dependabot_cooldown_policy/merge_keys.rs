use anyhow::Result;
use serde_yaml::{Mapping, Value};

pub(super) fn expand(value: &mut Value) -> Result<()> {
    let mut pending = vec![value];
    while let Some(value) = pending.pop() {
        match value {
            Value::Mapping(mapping) => {
                merge_sources(mapping)?;
                pending.extend(mapping.values_mut());
            }
            Value::Sequence(items) => pending.extend(items),
            Value::Tagged(tagged) => pending.push(&mut tagged.value),
            _ => {}
        }
    }
    Ok(())
}

fn merge_sources(mapping: &mut Mapping) -> Result<()> {
    let Some(merge) = mapping.remove("<<") else {
        return Ok(());
    };
    let mut pending = Vec::new();
    push_merge_sources(merge, &mut pending)?;
    // Each source's explicit keys win over its own merge, which is consumed
    // before later sequence sources. No merge source is copied or revisited.
    while let Some(mut source) = pending.pop() {
        if let Some(merge) = source.remove("<<") {
            push_merge_sources(merge, &mut pending)?;
        }
        for (key, value) in source {
            mapping.entry(key).or_insert(value);
        }
    }
    Ok(())
}

fn push_merge_sources(merge: Value, pending: &mut Vec<Mapping>) -> Result<()> {
    match merge {
        Value::Mapping(source) => pending.push(source),
        Value::Sequence(sources) => {
            let start = pending.len();
            for source in sources {
                let source = match source {
                    Value::Mapping(source) => source,
                    Value::Sequence(_) => {
                        anyhow::bail!("expected a mapping for merging, but found sequence")
                    }
                    Value::Tagged(_) => anyhow::bail!("unexpected tagged value in merge"),
                    _ => anyhow::bail!("expected a mapping for merging, but found scalar"),
                };
                pending.push(source);
            }
            pending[start..].reverse();
        }
        Value::Tagged(_) => anyhow::bail!("unexpected tagged value in merge"),
        _ => anyhow::bail!("expected a mapping or list of mappings for merging, but found scalar"),
    }
    Ok(())
}
