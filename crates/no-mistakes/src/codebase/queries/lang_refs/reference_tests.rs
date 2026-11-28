use crate::codebase::dependencies::graph::{
    proven_dotnet_reference_files, proven_lang_reference_files, proven_swift_reference_files,
    EdgeKind,
};
use crate::codebase::dotnet::{DotnetFactMap, DotnetFileFacts};
use crate::codebase::lang_frontends::{LangFactMap, LangFileFacts};
use crate::codebase::swift::{SwiftFactMap, SwiftFileFacts};
use std::collections::BTreeSet;
use std::path::PathBuf;

#[test]
fn reference_queries_stop_when_the_declaration_is_not_linked() {
    let declaration = PathBuf::from("decl.py");
    let mut lang = LangFactMap::default();
    assert!(proven_lang_reference_files(
        &lang,
        &declaration,
        "Greeter",
        EdgeKind::PythonImport,
        EdgeKind::PythonReference,
    )
    .is_empty());
    lang.files.insert(
        declaration.clone(),
        LangFileFacts {
            path: declaration.clone(),
            ..LangFileFacts::default()
        },
    );
    assert!(proven_lang_reference_files(
        &lang,
        &declaration,
        "Greeter",
        EdgeKind::PythonImport,
        EdgeKind::PythonReference,
    )
    .is_empty());
    lang.declarations
        .insert("Greeter".to_string(), BTreeSet::from([declaration.clone()]));
    assert!(proven_lang_reference_files(
        &lang,
        &declaration,
        "Greeter",
        EdgeKind::PythonImport,
        EdgeKind::PythonReference,
    )
    .is_empty());

    let mut swift = SwiftFactMap::default();
    assert!(proven_swift_reference_files(&swift, &declaration, "Greeter").is_empty());
    swift
        .declarations
        .insert("Greeter".to_string(), BTreeSet::from([declaration.clone()]));
    swift.files.insert(
        declaration.clone(),
        SwiftFileFacts {
            path: declaration.clone(),
            references: vec!["Greeter".to_string()],
            ..SwiftFileFacts::default()
        },
    );
    assert!(proven_swift_reference_files(&swift, &declaration, "Greeter").is_empty());

    let mut dotnet = DotnetFactMap::default();
    assert!(proven_dotnet_reference_files(&dotnet, &declaration, "Greeter").is_empty());
    dotnet
        .declarations
        .insert("Greeter".to_string(), BTreeSet::from([declaration.clone()]));
    dotnet.files.insert(
        declaration.clone(),
        DotnetFileFacts {
            path: declaration.clone(),
            references: vec!["Greeter".to_string()],
            ..DotnetFileFacts::default()
        },
    );
    assert!(proven_dotnet_reference_files(&dotnet, &declaration, "Greeter").is_empty());
}
