macro_rules! visit_import_collector_tail {
    () => {
        fn visit_identifier_reference(&mut self, identifier: &IdentifierReference<'a>) {
            visit_identifier_reference_with_namespaces(self, identifier);
        }

        fn visit_static_member_expression(&mut self, member: &StaticMemberExpression<'a>) {
            visit_static_member_expression_with_getters(self, member);
        }

        fn visit_computed_member_expression(&mut self, member: &ComputedMemberExpression<'a>) {
            visit_computed_member_expression_with_getters(self, member);
        }

        fn visit_ts_type_reference(&mut self, reference: &TSTypeReference<'a>) {
            visit_ts_type_reference_without_name_walk(self, reference);
        }

        fn visit_ts_type_query(&mut self, query: &oxc_ast::ast::TSTypeQuery<'a>) {
            self.walk_as_type_names(|collector| walk::walk_ts_type_query(collector, query));
        }

        fn visit_ts_class_implements(&mut self, implements: &oxc_ast::ast::TSClassImplements<'a>) {
            self.walk_as_type_names(|collector| {
                walk::walk_ts_class_implements(collector, implements)
            });
        }

        fn visit_ts_interface_heritage(
            &mut self,
            heritage: &oxc_ast::ast::TSInterfaceHeritage<'a>,
        ) {
            self.walk_as_type_names(|collector| {
                walk::walk_ts_interface_heritage(collector, heritage)
            });
        }

        fn visit_ts_type_parameter(&mut self, parameter: &TSTypeParameter<'a>) {
            visit_ts_type_parameter_without_name_walk(self, parameter);
        }

        fn visit_jsx_opening_element(&mut self, opening: &JSXOpeningElement<'a>) {
            visit_jsx_opening_element_with_value_reference(self, opening);
        }
    };
}
