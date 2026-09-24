use super::PnpmValidationError;
use serde::Deserialize;

/// Non-null YAML documents from one `pnpm-lock.yaml`, in file order.
///
/// pnpm 12 writes an optional env document first and the project document
/// last. A later document that fails to parse fails the whole file: returning
/// the env document alone would describe a project with no dependencies.
#[derive(Debug)]
pub(crate) struct PnpmDocuments {
    documents: Vec<serde_yaml::Value>,
}

impl PnpmDocuments {
    pub(crate) fn project(&self) -> Option<&serde_yaml::Value> {
        self.documents.last()
    }

    /// Installation fields such as `settings` live on the project document.
    /// Comparing document 0 across a one-document and a two-document file
    /// would treat that project block as removed.
    pub(super) fn project_mapping(&self) -> Option<&serde_yaml::Mapping> {
        self.project().and_then(serde_yaml::Value::as_mapping)
    }
}

pub(crate) fn load_documents(content: &str) -> Result<PnpmDocuments, PnpmValidationError> {
    let mut documents = Vec::new();
    let mut saw_document = false;
    for document in serde_yaml::Deserializer::from_str(content) {
        saw_document = true;
        let value =
            serde_yaml::Value::deserialize(document).map_err(|_| PnpmValidationError::Malformed)?;
        // A leading `---` can surface as a null document. It is not a lockfile.
        if !value.is_null() {
            documents.push(value);
        }
    }
    if !saw_document {
        return Err(PnpmValidationError::Malformed);
    }
    Ok(PnpmDocuments { documents })
}

pub(super) fn validate_supported(docs: &PnpmDocuments) -> Result<(), PnpmValidationError> {
    if docs.documents.is_empty() {
        return Err(PnpmValidationError::UnsupportedSchema);
    }
    for document in &docs.documents {
        validate_document(document)?;
    }
    Ok(())
}

pub(crate) fn package_maps(
    docs: &PnpmDocuments,
) -> impl Iterator<Item = &serde_yaml::Mapping> + '_ {
    docs.documents.iter().filter_map(|document| {
        document
            .get("packages")
            .and_then(serde_yaml::Value::as_mapping)
    })
}

pub(crate) fn package_key_strings(docs: &PnpmDocuments) -> Option<Vec<String>> {
    let mut keys = Vec::new();
    let mut saw_packages = false;
    for packages in package_maps(docs) {
        saw_packages = true;
        for key in packages.keys() {
            if let Some(key) = key.as_str() {
                keys.push(key.to_string());
            }
        }
    }
    saw_packages.then_some(keys)
}

fn validate_document(document: &serde_yaml::Value) -> Result<(), PnpmValidationError> {
    if document.as_mapping().is_none() {
        return Err(PnpmValidationError::UnsupportedSchema);
    }
    let version = document
        .get("lockfileVersion")
        .ok_or(PnpmValidationError::UnsupportedSchema)?;
    let major = match version {
        serde_yaml::Value::String(value) => value
            .split('.')
            .next()
            .and_then(|major| major.parse::<u8>().ok()),
        serde_yaml::Value::Number(value) => value.as_f64().map(|value| value as u8),
        _ => None,
    };
    // pnpm 12 keeps schema 9.0. A newer major is still unsupported.
    if !matches!(major, Some(5..=9)) {
        return Err(PnpmValidationError::UnsupportedSchema);
    }
    let mut has_supported_section = false;
    for section in ["packages", "importers", "snapshots"] {
        if let Some(value) = document.get(section) {
            if value.as_mapping().is_none() {
                return Err(PnpmValidationError::UnsupportedSchema);
            }
            has_supported_section = true;
        }
    }
    if !has_supported_section {
        return Err(PnpmValidationError::UnsupportedSchema);
    }
    Ok(())
}
