use super::super::{family_scope::applies, CheckFactPlan};
use crate::codebase::rules::nextjs_no_caching::{
    finish_visitor, prepare_visitor, NextjsCachingFinding, NextjsCachingVisitor,
};
use crate::codebase::rules::test_no_unmocked_dynamic_imports::ast::{Collector, TestFacts};
use crate::codebase::storybook::StorybookFileFacts;
use oxc_ast::ast::Program;
use oxc_ast_visit::Visit;
use std::collections::HashSet;
use std::path::Path;

mod scopes;
mod visit;

#[cfg(not(test))]
macro_rules! record_fused_work {
    ($kind:ident) => {};
}

#[cfg(test)]
#[macro_use]
mod tests;

pub(super) struct FusedCheckFacts {
    pub dynamic_imports: Option<TestFacts>,
    pub nextjs_caching: Option<Vec<NextjsCachingFinding>>,
    pub nextjs_caching_low: Option<Vec<NextjsCachingFinding>>,
    pub storybook: Option<StorybookFileFacts>,
}

pub(super) fn collect_fused_check_program(
    path: &Path,
    source: &str,
    program: &Program<'_>,
    plan: &CheckFactPlan,
) -> FusedCheckFacts {
    let storybook = plan.storybook && applies(&plan.storybook_paths, path);
    let nextjs_caching = plan.nextjs_caching && applies(&plan.nextjs_caching_paths, path);
    let nextjs_caching_low = plan.nextjs_caching_low && applies(&plan.nextjs_caching_paths, path);
    if !plan.dynamic_imports && !nextjs_caching && !nextjs_caching_low && !storybook {
        return FusedCheckFacts {
            dynamic_imports: None,
            nextjs_caching: None,
            nextjs_caching_low: None,
            storybook: None,
        };
    }
    let mut visitor = CheckProgramVisitor {
        dynamic: plan
            .dynamic_imports
            .then(|| Collector::new(source, program)),
        nextjs: nextjs_caching.then(|| {
            record_fused_work!(nextjs);
            prepare_visitor(path, source, program)
        }),
        nextjs_low: nextjs_caching_low.then(|| {
            record_fused_work!(nextjs);
            crate::codebase::rules::nextjs_no_caching::low::Collector::new(path, source, program)
        }),
        storybook_ids: storybook.then(HashSet::new),
    };
    record_fused_work!(walk);
    visitor.visit_program(program);
    FusedCheckFacts {
        dynamic_imports: visitor.dynamic.map(Collector::into_facts),
        nextjs_caching: visitor.nextjs.map(finish_visitor),
        nextjs_caching_low: visitor.nextjs_low.map(|collector| collector.finish()),
        storybook: visitor.storybook_ids.map(|ids| {
            crate::codebase::storybook::extract_program_with_references(source, program, &ids)
        }),
    }
}

struct CheckProgramVisitor<'a> {
    dynamic: Option<Collector<'a>>,
    nextjs: Option<NextjsCachingVisitor<'a>>,
    nextjs_low: Option<crate::codebase::rules::nextjs_no_caching::low::Collector<'a>>,
    storybook_ids: Option<HashSet<&'a str>>,
}
