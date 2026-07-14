struct AppSelectorVisitor<'a, 'r> {
    path: &'r Path,
    source: &'a str,
    attributes: &'r [String],
    component_attributes: &'r BTreeMap<String, String>,
    html_ids: bool,
    scoped_static_identifier_defaults: &'r [ScopedStaticIdentifierDefault],
    dynamic_identifier_values: &'r [DynamicIdentifierValues],
    program: &'a oxc_ast::ast::Program<'a>,
    visible_files: Option<&'r HashSet<PathBuf>>,
    selectors: Vec<AppSelector>,
}

impl<'a> oxc_ast_visit::Visit<'a> for AppSelectorVisitor<'a, '_> {
    fn visit_jsx_opening_element(&mut self, element: &oxc_ast::ast::JSXOpeningElement<'a>) {
        let component = is_component_jsx_element_name(&element.name);
        for item in &element.attributes {
            let oxc_ast::ast::JSXAttributeItem::Attribute(attribute) = item else {
                continue;
            };
            let Some(name) = jsx_attribute_name(&attribute.name) else {
                continue;
            };
            let Some(mapped_attribute) = self.mapped_attribute(name, component).map(str::to_string)
            else {
                continue;
            };

            let values = match self.visible_files {
                Some(visible) => app_selector_values_from_visible(
                    attribute.value.as_ref(),
                    self.source,
                    self.path,
                    self.scoped_static_identifier_defaults,
                    self.dynamic_identifier_values,
                    self.program,
                    visible,
                ),
                None => app_selector_values(
                    attribute.value.as_ref(),
                    self.source,
                    self.path,
                    self.scoped_static_identifier_defaults,
                    self.dynamic_identifier_values,
                    self.program,
                ),
            };
            for value in values {
                self.selectors.push(AppSelector {
                    file: PathBuf::from(self.path),
                    attribute: mapped_attribute.clone(),
                    value,
                });
            }
        }

        oxc_ast_visit::walk::walk_jsx_opening_element(self, element);
    }
}

impl AppSelectorVisitor<'_, '_> {
    fn mapped_attribute<'a>(&'a self, name: &'a str, component: bool) -> Option<&'a str> {
        if self.attributes.iter().any(|attribute| attribute == name) {
            return Some(name);
        }
        if self.html_ids && !component && name == HTML_ID_ATTRIBUTE {
            return Some(HTML_ID_ATTRIBUTE);
        }
        if component {
            return self.component_attributes.get(name).map(String::as_str);
        }
        None
    }
}

pub(super) fn is_component_jsx_element_name(name: &oxc_ast::ast::JSXElementName<'_>) -> bool {
    match name {
        oxc_ast::ast::JSXElementName::Identifier(identifier) => identifier
            .name
            .chars()
            .next()
            .is_some_and(|ch| !ch.is_ascii_lowercase()),
        oxc_ast::ast::JSXElementName::IdentifierReference(identifier) => identifier
            .name
            .chars()
            .next()
            .is_some_and(|ch| !ch.is_ascii_lowercase()),
        oxc_ast::ast::JSXElementName::MemberExpression(_) => true,
        oxc_ast::ast::JSXElementName::NamespacedName(_)
        | oxc_ast::ast::JSXElementName::ThisExpression(_) => false,
    }
}

pub(super) fn jsx_attribute_name<'a>(
    name: &'a oxc_ast::ast::JSXAttributeName<'a>,
) -> Option<&'a str> {
    match name {
        oxc_ast::ast::JSXAttributeName::Identifier(identifier) => Some(identifier.name.as_str()),
        _ => None,
    }
}
