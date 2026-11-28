use super::boundary::{is_relative, stays_in_owner_package, unanalyzable, SpecifierCheck};
use super::Inputs;
use crate::codebase::dependencies::extract::ImportKind;
use crate::codebase::dependencies::graph::{vitest_import_calls, ImportTarget, VitestReceiver};
use no_mistakes_rule_kernel::file_diagnostics::AnalysisDiagnostics;
use crate::codebase::ts_source::facts::TsFileFacts;

impl Inputs<'_> {
    pub(super) fn record_vitest_calls(
        &self,
        ctx: &SpecifierCheck<'_>,
        facts: &TsFileFacts,
        diagnostics: &mut AnalysisDiagnostics,
    ) {
        for call in vitest_import_calls(facts) {
            if !(call.computed || is_relative(call.specifier)) {
                continue;
            }
            if facts.imports.iter().any(|import| {
                import.kind != ImportKind::RequireResolve
                    && import.line == call.line
                    && import.specifier == call.specifier
            }) {
                continue;
            }
            match call.receiver {
                VitestReceiver::Direct => self.check_specifier(
                    ctx,
                    call.line,
                    call.specifier,
                    call.computed,
                    diagnostics,
                ),
                VitestReceiver::Unproven { imported_from } => {
                    if imported_from.is_some_and(|source| {
                        self.graph.import_target(ctx.file, source) == Some(&ImportTarget::External)
                    }) || (!call.computed
                        && stays_in_owner_package(
                            ctx.file,
                            call.specifier,
                            ctx.owner,
                            self.workspace,
                        ))
                    {
                        continue;
                    }
                    diagnostics.unanalyzable(
                        ctx.policy,
                        unanalyzable(
                            ctx.relative_file,
                            call.line,
                            call.specifier,
                            "the receiver of importActual or importMock is unresolved. Import vi directly from Vitest.",
                        ),
                    );
                }
            }
        }
    }
}
