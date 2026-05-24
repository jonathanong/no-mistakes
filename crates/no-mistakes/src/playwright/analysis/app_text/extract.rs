use super::AppTextVisitor;
use crate::playwright::analysis::text_types::AppTextTarget;
use crate::playwright::ast;
use crate::playwright::config::Settings;
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
        let mut visitor = AppTextVisitor {
            root,
            path,
            settings,
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
