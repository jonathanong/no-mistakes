use super::*;

#[test]
fn a_located_helper_without_matching_sql_facts_is_not_a_summary() {
    let caller = PathBuf::from("/caller.mts");
    let other = PathBuf::from("/other.mts");
    let mut files = FileIdMap::default();
    files.insert(caller.clone(), CheckFileFacts::default());
    files.insert(other.clone(), CheckFileFacts::default());
    {
        let facts = std::sync::Arc::make_mut(&mut files.get_mut(&caller).unwrap().ts);
        facts.imported_bindings.push(ImportedBinding {
            specifier: "./other.mts".to_string(),
            local: "h".to_string(),
            imported: "h".to_string(),
            kind: ImportedBindingKind::Named,
            is_type_only: false,
        });
    }
    {
        let facts = std::sync::Arc::make_mut(&mut files.get_mut(&other).unwrap().ts);
        facts.exported_bindings.push(ExportedBinding {
            specifier: None,
            local: "h".to_string(),
            exported: "h".to_string(),
        });
    }
    let options = EmbeddedSqlOptions::configured("@example/db", &[]);
    let other_path = other.clone();
    let locator = Locator::new(&files, move |specifier: &str, _from: &std::path::Path| {
        (specifier == "./other.mts").then(|| other_path.clone())
    });
    let mut shared = Shared::new(&files, locator);
    let site = ImportedSite {
        local: "h".to_string(),
        member: None,
        args: Vec::new(),
        value_used: true,
        line: 1,
        constant: None,
    };
    assert!(shared.summary_text(&options, &caller, &site).is_none());
}
