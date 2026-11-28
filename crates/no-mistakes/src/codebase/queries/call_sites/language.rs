use super::{CallSite, CallSitesArgs, CallSitesReport};
use anyhow::Result;

pub(super) fn from_args(args: &CallSitesArgs) -> Result<Option<CallSitesReport>> {
    let Some(rows) = crate::codebase::queries::lang_refs::language_call_rows(
        &args.file,
        &args.export_name,
        args.root.as_deref(),
        None,
    )?
    else {
        return Ok(None);
    };
    let call_sites = rows
        .sites
        .into_iter()
        .map(|site| CallSite {
            file: site.file,
            line: site.line,
            caller: None,
            arg_count: 0,
            has_spread: false,
            args: Vec::new(),
            reference_only: true,
        })
        .collect();
    Ok(Some(CallSitesReport {
        file: rows.file,
        export: args.export_name.clone(),
        call_sites,
        diagnostics: std::sync::Arc::new(rows.diagnostics),
    }))
}
