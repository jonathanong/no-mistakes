use super::EdgeKind;

pub(super) fn as_str(kind: &EdgeKind) -> &'static str {
    language_frontend_str(kind).unwrap_or_else(|| other_domain_str(kind))
}

fn language_frontend_str(kind: &EdgeKind) -> Option<&'static str> {
    Some(match kind {
        EdgeKind::SwiftImport => "swift-import",
        EdgeKind::SwiftReference => "swift-ref",
        EdgeKind::SwiftPackageDependency => "swift-package",
        EdgeKind::DotnetUsing => "dotnet-using",
        EdgeKind::DotnetReference => "dotnet-ref",
        EdgeKind::DotnetProjectDependency => "dotnet-project",
        EdgeKind::PythonImport => "python-import",
        EdgeKind::PythonReference => "python-ref",
        EdgeKind::GoImport => "go-import",
        EdgeKind::GoReference => "go-ref",
        EdgeKind::RustUse => "rust-use",
        EdgeKind::RustMod => "rust-mod",
        EdgeKind::RustPackage => "rust-package",
        EdgeKind::RubyRequire => "ruby-require",
        EdgeKind::RubyReference => "ruby-ref",
        EdgeKind::PhpUse => "php-use",
        EdgeKind::PhpPackage => "php-package",
        EdgeKind::JavaImport => "java-import",
        EdgeKind::JavaReference => "java-ref",
        EdgeKind::KotlinImport => "kotlin-import",
        EdgeKind::KotlinReference => "kotlin-ref",
        EdgeKind::ElixirImport => "elixir-import",
        EdgeKind::ElixirReference => "elixir-ref",
        EdgeKind::DartImport => "dart-import",
        EdgeKind::DartReference => "dart-ref",
        _ => return None,
    })
}

fn other_domain_str(kind: &EdgeKind) -> &'static str {
    match kind {
        EdgeKind::HttpCall => "http",
        EdgeKind::ProcessSpawn => "process",
        EdgeKind::AssetImport => "asset",
        EdgeKind::Resource => "resource",
        EdgeKind::ReactRender => "react-render",
        EdgeKind::Selector => "selector",
        EdgeKind::TerraformReference => "terraform-ref",
        EdgeKind::TerraformModuleRef => "terraform-module",
        EdgeKind::TerraformOutputRef => "terraform-output",
        _ => unreachable!("core edge kinds are handled before domain rendering"),
    }
}
