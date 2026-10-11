use super::*;

impl LocalFunctions {
    pub(crate) fn get_call(
        &self,
        call: &CallExpression<'_>,
        name: &str,
        depth: u8,
        is_shadowed: &mut impl FnMut(&str) -> bool,
        resolve_binding: &impl Fn(&str) -> Option<String>,
    ) -> Option<String> {
        if let Some(builder) = self.parameter_builders.get(name) {
            let argument = builder.checked_argument(call)?;
            let base = match crate::codebase::ts_source::unwrap_ts_wrappers(argument) {
                Expression::Identifier(ident) => resolve_binding(ident.name.as_str()),
                Expression::TaggedTemplateExpression(_) => {
                    // This input is already a template, so a call-target
                    // lookup cannot run. Keep the chain's depth and opacity
                    // checks while recovering the template directly.
                    depth.checked_sub(1).and_then(|_| {
                        super::super::chain::tagged_sql(
                            argument,
                            is_shadowed,
                            self.imported_sql_tags(),
                        )
                    })
                }
                _ => None,
            }?;
            return Some(format!(
                "{base}{}",
                super::super::super::super::placeholders::renumber_placeholders(
                    &builder.suffix,
                    super::super::super::super::placeholders::count_placeholders(&base)
                )
            ));
        }
        self.get(name)
    }
}
