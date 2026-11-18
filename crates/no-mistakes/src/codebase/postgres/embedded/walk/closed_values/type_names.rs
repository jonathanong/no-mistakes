use super::*;

impl ScopeVisitor<'_> {
    pub(in crate::codebase::postgres::embedded::walk) fn shadow_parameter_generics(
        &mut self,
        parameters: Option<&TSTypeParameterDeclaration<'_>>,
    ) {
        for parameter in parameters
            .into_iter()
            .flat_map(|parameters| &parameters.params)
        {
            self.shadow_parameter_type(parameter.name.name.as_str());
        }
    }

    pub(in crate::codebase::postgres::embedded::walk) fn record_parameter_types(
        &mut self,
        statement: &Statement<'_>,
    ) {
        match statement {
            Statement::ImportDeclaration(import) => {
                for specifier in import.specifiers.iter().flatten() {
                    let name = match specifier {
                        ImportDeclarationSpecifier::ImportSpecifier(specifier) => &specifier.local,
                        ImportDeclarationSpecifier::ImportDefaultSpecifier(specifier) => {
                            &specifier.local
                        }
                        ImportDeclarationSpecifier::ImportNamespaceSpecifier(specifier) => {
                            &specifier.local
                        }
                    };
                    self.shadow_parameter_type(name.name.as_str());
                }
            }
            Statement::ExportDeclaration(export) => {
                self.record_parameter_type_declaration(&export.declaration);
            }
            Statement::ExportDefaultDeclaration(export) => {
                let name = match &export.declaration {
                    ExportDefaultDeclarationKind::ClassDeclaration(class) => class.id.as_ref(),
                    ExportDefaultDeclarationKind::TSInterfaceDeclaration(interface) => {
                        Some(&interface.id)
                    }
                    _ => None,
                };
                if let Some(name) = name {
                    self.shadow_parameter_type(name.name.as_str());
                }
            }
            _ => {
                if let Some(declaration) = statement.as_declaration() {
                    self.record_parameter_type_declaration(declaration);
                }
            }
        }
    }

    fn record_parameter_type_declaration(&mut self, declaration: &Declaration<'_>) {
        let name = match declaration {
            Declaration::TSTypeAliasDeclaration(alias) => {
                self.note_parameter_type_alias(alias);
                return;
            }
            Declaration::TSInterfaceDeclaration(interface) => Some(&interface.id),
            Declaration::TSEnumDeclaration(enumeration) => Some(&enumeration.id),
            Declaration::ClassDeclaration(class) => class.id.as_ref(),
            Declaration::TSNamespaceDeclaration(namespace) => Some(&namespace.id),
            Declaration::TSImportEqualsDeclaration(import) => Some(&import.id),
            _ => None,
        };
        if let Some(name) = name {
            self.shadow_parameter_type(name.name.as_str());
        }
    }
}
