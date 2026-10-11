use super::{Ctx, Extends, Options, Path};

pub(super) fn inherit_named_deadlines(options: &mut Options, inherited: &mut Options, path: &Path) {
    // Importer-selected base provenance is observed, not a proved SDK config root.
    // Even absent slots in that file may exist in the SDK-selected file.
    if matches!(options.deadline_extends, Some(Extends::Config(_))) {
        for slot in [&mut inherited.deadlines.case, &mut inherited.deadlines.hook] {
            if let Some(declaration) = slot {
                declaration.value = crate::integration_tests::types::DeadlineValue::Unknown(
                    crate::integration_tests::types::DeadlineUnknownReason::UnprovedConfigRoot,
                );
            } else {
                *slot = Some(crate::integration_tests::test_config::deadlines::unknown(
                    crate::integration_tests::types::DeadlineUnknownReason::UnprovedConfigRoot,
                    path,
                    None,
                ));
            }
        }
        options.deadlines.inherit_missing(
            &inherited.deadlines,
            crate::integration_tests::types::DeadlineInheritance {
                path: path.to_path_buf(),
                project: options.name.clone(),
            },
        );
    }
}

pub(super) fn inherit_unresolved_deadlines(options: &mut Options, ctx: &Ctx<'_, '_>) {
    if matches!(options.deadline_extends, Some(Extends::Config(_))) {
        let mut unknown = crate::integration_tests::types::DeclaredDeadlines::default();
        crate::integration_tests::test_config::deadlines::obscure(
            &mut unknown,
            ctx.path,
            oxc_span::Span::new(0, 0),
            crate::integration_tests::types::DeadlineUnknownReason::UnresolvedExtends,
            true,
        );
        for declaration in [&mut unknown.case, &mut unknown.hook].into_iter().flatten() {
            declaration.span = None;
        }
        options.deadlines.inherit_missing(
            &unknown,
            crate::integration_tests::types::DeadlineInheritance {
                path: ctx.path.to_path_buf(),
                project: options.name.clone(),
            },
        );
    }
}
