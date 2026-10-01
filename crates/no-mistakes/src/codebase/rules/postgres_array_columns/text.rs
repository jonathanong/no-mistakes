pub(super) fn ordinary_text(data_type: &str) -> String {
    format!(
        "column is an array ({data_type}); store the values as child rows or an enum, or add an allow entry with a reason"
    )
}

pub(super) fn never_text(element: &str) -> String {
    format!(
        "column is a {element}[] array; model each reference as a child row with a foreign key ({element} arrays cannot be allowlisted)"
    )
}

pub(super) fn excuse_text(object: &str, element: &str) -> String {
    format!("allow entry {object} cannot excuse a {element}[] column")
}
