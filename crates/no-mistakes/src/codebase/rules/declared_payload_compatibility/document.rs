use serde::de::{Deserialize, Deserializer, Error, MapAccess, SeqAccess, Visitor};
use serde_json::Value;
use std::fmt;
use std::path::{Path, PathBuf};
use std::sync::Arc;

#[derive(Default)]
pub(super) struct Documents {
    values: dashmap::DashMap<PathBuf, Result<Arc<Value>, String>>,
    pub(super) parses: std::sync::atomic::AtomicUsize,
}

impl Documents {
    pub(super) fn get(
        &self,
        sources: &crate::codebase::ts_source::SourceStore,
        path: &Path,
    ) -> Result<Arc<Value>, String> {
        self.values
            .entry(path.to_path_buf())
            .or_insert_with(|| {
                let source = sources
                    .read_path(path)
                    .map_err(|e| format!("cannot read schema: {e}"))?;
                self.parses
                    .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                parse(&source).map(Arc::new)
            })
            .value()
            .clone()
    }
}

/// JSON declarations must not rely on parser-specific duplicate-key handling.
struct Document(Value);

impl<'de> Deserialize<'de> for Document {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        deserializer.deserialize_any(DocumentVisitor)
    }
}

struct DocumentVisitor;

impl<'de> Visitor<'de> for DocumentVisitor {
    type Value = Document;
    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("JSON without duplicate object keys")
    }
    fn visit_unit<E: Error>(self) -> Result<Document, E> {
        Ok(Document(Value::Null))
    }
    fn visit_bool<E: Error>(self, value: bool) -> Result<Document, E> {
        Ok(Document(Value::Bool(value)))
    }
    fn visit_i64<E: Error>(self, value: i64) -> Result<Document, E> {
        Ok(Document(value.into()))
    }
    fn visit_u64<E: Error>(self, value: u64) -> Result<Document, E> {
        Ok(Document(value.into()))
    }
    fn visit_f64<E: Error>(self, value: f64) -> Result<Document, E> {
        Ok(Document(value.into()))
    }
    fn visit_str<E: Error>(self, value: &str) -> Result<Document, E> {
        Ok(Document(value.into()))
    }
    fn visit_seq<A: SeqAccess<'de>>(self, mut access: A) -> Result<Document, A::Error> {
        let mut array = Vec::new();
        while let Some(Document(value)) = access.next_element()? {
            array.push(value);
        }
        Ok(Document(Value::Array(array)))
    }
    fn visit_map<A: MapAccess<'de>>(self, mut access: A) -> Result<Document, A::Error> {
        let mut object = serde_json::Map::new();
        while let Some((name, Document(value))) = access.next_entry::<String, Document>()? {
            if object.insert(name.clone(), value).is_some() {
                return Err(A::Error::custom(format!("duplicate object key `{name}`")));
            }
        }
        Ok(Document(Value::Object(object)))
    }
}

pub(super) fn parse(source: &str) -> Result<Value, String> {
    let Document(value) = serde_json::from_str(source).map_err(|e| format!("invalid JSON: {e}"))?;
    super::numeric::validate(source)?;
    Ok(value)
}
