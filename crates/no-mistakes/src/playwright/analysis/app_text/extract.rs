use super::AppTextVisitor;
use crate::playwright::analysis::text_types::AppTextTarget;
use crate::playwright::ast;
use crate::playwright::config::Settings;
use crate::playwright::selectors::scoped_defaults::collect_scoped_static_identifier_defaults;
use anyhow::Result;
use oxc_ast_visit::Visit;
use std::collections::HashMap;
use std::path::Path;

pub(super) fn extract_app_text_targets(
    root: &Path,
    path: &Path,
    source: &str,
    settings: &Settings,
) -> Result<Vec<AppTextTarget>> {
    ast::with_program(path, source, |program, _| {
        let scoped_static_identifier_defaults = collect_scoped_static_identifier_defaults(program);
        let mut visitor = AppTextVisitor {
            root,
            path,
            source,
            settings,
            scoped_static_identifier_defaults: &scoped_static_identifier_defaults,
            targets: Vec::new(),
            controls_by_id: HashMap::new(),
            pending_labels: Vec::new(),
            texts_by_id: HashMap::new(),
        };
        visitor.visit_program(program);
        visitor.finish();
        visitor.targets
    })
}
