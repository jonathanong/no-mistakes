#[derive(Clone, Default)]
pub(crate) struct StaticExportValues {
    pub(crate) named: std::collections::HashMap<String, Vec<String>>,
    pub(crate) default: Vec<String>,
}

impl StaticExportValues {
    pub(crate) fn values(&self, exported_name: &str, is_default: bool) -> &[String] {
        if is_default {
            &self.default
        } else {
            self.named
                .get(exported_name)
                .map(Vec::as_slice)
                .unwrap_or_default()
        }
    }
}
