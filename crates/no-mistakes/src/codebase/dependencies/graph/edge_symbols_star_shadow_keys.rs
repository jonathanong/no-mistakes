#[derive(Clone, Debug, Eq, Hash, PartialEq)]
struct StarExportKey {
    name: String,
    namespace: StarExportNamespace,
}

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
enum StarExportNamespace {
    Type,
    Value,
}

fn explicit_export_keys(
    symbols: &crate::codebase::ts_symbols::FileSymbols,
) -> HashSet<StarExportKey> {
    symbols
        .exports
        .iter()
        .filter(|export| export.name != "*")
        .map(|export| star_export_key(export, false))
        .collect()
}

fn star_export_key(
    export: &crate::codebase::ts_symbols::Export,
    force_type: bool,
) -> StarExportKey {
    StarExportKey {
        name: export_symbol_name(export),
        namespace: if force_type || export.is_type_only {
            StarExportNamespace::Type
        } else {
            StarExportNamespace::Value
        },
    }
}
