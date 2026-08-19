use crate::react_traits::analyze::import_table::ImportTable;
use crate::react_traits::analyze::jsx_resolve::{element_root_and_suffix, resolve_target};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

#[cfg(test)]
mod tests;

pub(crate) fn jsx_element_child(
    elem: &oxc_ast::ast::JSXElement<'_>,
    import_table: &ImportTable,
    local_components: &HashMap<String, String>,
    file_path: &Path,
) -> Option<(PathBuf, String)> {
    let (root_name, member_suffix) = element_root_and_suffix(&elem.opening_element.name);
    let root = root_name?;
    resolve_target(
        &root,
        member_suffix.as_deref(),
        import_table,
        local_components,
        file_path,
    )
}
