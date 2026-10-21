use oxc_ast::ast::{BindingPattern, ModuleExportName};

pub(super) fn binding_name<'a>(pattern: &BindingPattern<'a>) -> Option<&'a str> {
    match pattern {
        BindingPattern::BindingIdentifier(ident) => Some(ident.name.as_str()),
        _ => None,
    }
}

pub(super) fn local_name<'a>(name: &ModuleExportName<'a>) -> Option<&'a str> {
    match name {
        ModuleExportName::IdentifierReference(ident) => Some(ident.name.as_str()),
        ModuleExportName::IdentifierName(ident) => Some(ident.name.as_str()),
        ModuleExportName::StringLiteral(_) => None,
    }
}
