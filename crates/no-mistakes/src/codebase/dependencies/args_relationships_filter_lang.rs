fn language_relationship_edges(relationship: &RelationshipArg) -> Option<&'static [EdgeKind]> {
    Some(match relationship {
        RelationshipArg::Python => &[EdgeKind::PythonImport, EdgeKind::PythonReference],
        RelationshipArg::Go => &[EdgeKind::GoImport, EdgeKind::GoReference],
        RelationshipArg::Rust => &[EdgeKind::RustUse, EdgeKind::RustMod, EdgeKind::RustPackage],
        RelationshipArg::Ruby => &[EdgeKind::RubyRequire, EdgeKind::RubyReference],
        RelationshipArg::Php => &[EdgeKind::PhpUse, EdgeKind::PhpPackage],
        RelationshipArg::Java => &[EdgeKind::JavaImport, EdgeKind::JavaReference],
        RelationshipArg::Kotlin => &[EdgeKind::KotlinImport, EdgeKind::KotlinReference],
        RelationshipArg::Elixir => &[EdgeKind::ElixirImport, EdgeKind::ElixirReference],
        RelationshipArg::Dart => &[EdgeKind::DartImport, EdgeKind::DartReference],
        _ => return None,
    })
}
