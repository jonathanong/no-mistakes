use crate::playwright::selectors::scoped_defaults::ScopedStaticIdentifierDefault;

pub(super) fn element_role(
    opening: &oxc_ast::ast::JSXOpeningElement<'_>,
    tag: Option<&str>,
    source: &str,
    scoped_static_identifier_defaults: &[ScopedStaticIdentifierDefault],
) -> Option<String> {
    if let Some(role) =
        super::jsx::string_attr(opening, "role", source, scoped_static_identifier_defaults)
            .and_then(|value| first_role_token(&value))
    {
        return Some(role);
    }
    implicit_role(opening, tag, source, scoped_static_identifier_defaults).map(str::to_string)
}

fn first_role_token(value: &str) -> Option<String> {
    value
        .split_whitespace()
        .find(|role| valid_concrete_role(role))
        .map(str::to_string)
}

fn valid_concrete_role(role: &str) -> bool {
    matches!(
        role,
        "alert"
            | "alertdialog"
            | "application"
            | "article"
            | "banner"
            | "blockquote"
            | "button"
            | "caption"
            | "cell"
            | "checkbox"
            | "code"
            | "columnheader"
            | "combobox"
            | "complementary"
            | "contentinfo"
            | "definition"
            | "dialog"
            | "directory"
            | "document"
            | "emphasis"
            | "feed"
            | "figure"
            | "form"
            | "generic"
            | "grid"
            | "gridcell"
            | "group"
            | "heading"
            | "img"
            | "link"
            | "list"
            | "listbox"
            | "listitem"
            | "log"
            | "main"
            | "marquee"
            | "math"
            | "meter"
            | "menu"
            | "menubar"
            | "menuitem"
            | "menuitemcheckbox"
            | "menuitemradio"
            | "navigation"
            | "none"
            | "note"
            | "option"
            | "paragraph"
            | "presentation"
            | "progressbar"
            | "radio"
            | "radiogroup"
            | "region"
            | "row"
            | "rowgroup"
            | "rowheader"
            | "scrollbar"
            | "search"
            | "searchbox"
            | "separator"
            | "slider"
            | "spinbutton"
            | "status"
            | "strong"
            | "subscript"
            | "superscript"
            | "switch"
            | "tab"
            | "table"
            | "tablist"
            | "tabpanel"
            | "term"
            | "textbox"
            | "time"
            | "timer"
            | "toolbar"
            | "tooltip"
            | "tree"
            | "treegrid"
            | "treeitem"
    )
}

fn implicit_role(
    opening: &oxc_ast::ast::JSXOpeningElement<'_>,
    tag: Option<&str>,
    source: &str,
    scoped_static_identifier_defaults: &[ScopedStaticIdentifierDefault],
) -> Option<&'static str> {
    match tag? {
        "a" | "area" if super::jsx::attr_exists_at_runtime(opening, "href") => Some("link"),
        "button" => Some("button"),
        "h1" | "h2" | "h3" | "h4" | "h5" | "h6" => Some("heading"),
        "img"
            if super::jsx::string_attr(
                opening,
                "alt",
                source,
                scoped_static_identifier_defaults,
            )
            .is_some() =>
        {
            Some("img")
        }
        "input" => input_role(opening, source, scoped_static_identifier_defaults),
        "select" => select_role(opening, source, scoped_static_identifier_defaults),
        "textarea" => Some("textbox"),
        _ => None,
    }
}

fn input_role(
    opening: &oxc_ast::ast::JSXOpeningElement<'_>,
    source: &str,
    scoped_static_identifier_defaults: &[ScopedStaticIdentifierDefault],
) -> Option<&'static str> {
    match super::jsx::string_attr(opening, "type", source, scoped_static_identifier_defaults)
        .as_deref()
        .unwrap_or("text")
    {
        "button" | "image" | "reset" | "submit" => Some("button"),
        "checkbox" => Some("checkbox"),
        "number" => Some("spinbutton"),
        "radio" => Some("radio"),
        "range" => Some("slider"),
        "search" => Some("searchbox"),
        "email" | "tel" | "text" | "url" => Some("textbox"),
        _ => None,
    }
}

fn select_role(
    opening: &oxc_ast::ast::JSXOpeningElement<'_>,
    source: &str,
    scoped_static_identifier_defaults: &[ScopedStaticIdentifierDefault],
) -> Option<&'static str> {
    if super::jsx::bool_attr(opening, "multiple").unwrap_or(false) {
        return Some("listbox");
    }
    match super::jsx::numeric_attr(opening, "size", source).or_else(|| {
        super::jsx::string_attr(opening, "size", source, scoped_static_identifier_defaults)
            .and_then(|value| value.parse::<u32>().ok())
    }) {
        Some(size) if size > 1 => Some("listbox"),
        _ => Some("combobox"),
    }
}
