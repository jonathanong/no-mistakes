use super::properties::merge_property;
use super::{
    deadlines, expression_object_options, merge_options, shared, Ctx, DeadlineUnknownReason,
    Extends, Options,
};
use anyhow::Result;
use oxc_ast::ast::ObjectProperty;
use oxc_span::GetSpan;

pub(super) fn apply_property(
    options: &mut Options,
    property: &ObjectProperty<'_>,
    nested_test: bool,
    ctx: &mut Ctx<'_, '_>,
) -> Result<()> {
    let name = shared::property_key_name(&property.key);
    if property.computed && name.is_none() {
        deadlines::obscure(
            &mut options.deadlines,
            ctx.path,
            property.span(),
            DeadlineUnknownReason::ComputedProperty,
            true,
        );
        options.extends = Some(Extends::Unknown {
            path: ctx.path.to_path_buf(),
            span: Some((property.span.start, property.span.end)),
        });
        if !ctx.is_test_object {
            options.deadline_extends = options.extends.clone();
        }
        return Ok(());
    }
    if property.method || property.kind != oxc_ast::ast::PropertyKind::Init {
        apply_accessor(options, property, name.as_deref(), nested_test, ctx);
        return Ok(());
    }
    if name.as_deref() == Some("test") {
        replace_test_object(options, property, ctx)?;
        return Ok(());
    }
    let nested_test_scope = nested_test || options.nested_test_scope;
    merge_property(options, name, &property.value, nested_test_scope, ctx)?;
    Ok(())
}

fn apply_accessor(
    options: &mut Options,
    property: &ObjectProperty<'_>,
    name: Option<&str>,
    nested_test: bool,
    ctx: &Ctx<'_, '_>,
) {
    match name {
        Some("testTimeout") if !nested_test => {
            options.deadlines.case = Some(deadlines::unknown(
                DeadlineUnknownReason::Accessor,
                ctx.path,
                Some(property.span()),
            ))
        }
        Some("hookTimeout") if !nested_test => {
            options.deadlines.hook = Some(deadlines::unknown(
                DeadlineUnknownReason::Accessor,
                ctx.path,
                Some(property.span()),
            ))
        }
        Some("test") => {
            options.nested_test_scope = true;
            deadlines::obscure(
                &mut options.deadlines,
                ctx.path,
                property.span(),
                DeadlineUnknownReason::Accessor,
                true,
            );
        }
        Some("extends") => {
            options.extends = Some(Extends::Unknown {
                path: ctx.path.to_path_buf(),
                span: Some((property.span.start, property.span.end)),
            })
        }
        _ => {}
    }
    if name == Some("extends") && !ctx.is_test_object {
        options.deadline_extends = options.extends.clone();
    }
}

fn replace_test_object(
    options: &mut Options,
    property: &ObjectProperty<'_>,
    ctx: &mut Ctx<'_, '_>,
) -> Result<()> {
    let was_test_object = ctx.is_test_object;
    ctx.is_test_object = true;
    let parsed = expression_object_options(&property.value, ctx);
    ctx.is_test_object = was_test_object;
    if let Some(mut test_options) = parsed? {
        options.name = None;
        options.include = None;
        options.exclude = None;
        options.setup_files = None;
        options.global_setup = None;
        options.setup_files_cleared = false;
        options.global_setup_cleared = false;
        options.deadlines = Default::default();
        // SDK extends is an outer project field, never test.extends.
        test_options.deadline_extends = None;
        test_options.nested_test_scope = true;
        merge_options(options, test_options);
    } else {
        options.nested_test_scope = true;
        deadlines::obscure(
            &mut options.deadlines,
            ctx.path,
            property.value.span(),
            DeadlineUnknownReason::OpaqueTestObject,
            true,
        );
    }
    Ok(())
}
