use super::Projector;
use crate::codebase::ts_source::module_facts::arguments::{aliases::AliasSafety, path_origin};
use crate::codebase::ts_source::module_facts::extract_from_semantic;
use crate::fx::FxHashSet;
use oxc_allocator::Allocator;
use oxc_parser::Parser;
use oxc_semantic::{Semantic, SemanticBuilder};
use oxc_span::SourceType;

pub(super) struct PreparedFixture<'a, 's> {
    semantic: &'s Semantic<'a>,
    temp_factories: path_origin::TempFactories,
    exported: FxHashSet<usize>,
}

impl PreparedFixture<'_, '_> {
    pub(super) fn with_projector(&self, test: impl FnOnce(&mut Projector<'_, '_>)) {
        let mut immutable = AliasSafety::default();
        let mut projector = Projector {
            semantic: self.semantic,
            call_offset: 0,
            remaining: 256,
            exported: &self.exported,
            immutable: &mut immutable,
            temp_factories: &self.temp_factories,
        };
        test(&mut projector);
    }
}

pub(super) fn with_prepared_fixture(fixture: &str, test: impl FnOnce(&PreparedFixture<'_, '_>)) {
    let path =
        crate::test_support::rule_fixture_root("test-no-live-repository-scan/facts").join(fixture);
    let source_type = SourceType::from_path(&path).unwrap();
    let source = std::fs::read_to_string(path).unwrap();
    let allocator = Allocator::default();
    let parsed = Parser::new(&allocator, &source, source_type).parse();
    assert!(parsed.diagnostics.is_empty(), "{:?}", parsed.diagnostics);
    let built = SemanticBuilder::new()
        .with_build_nodes(true)
        .build(&parsed.program);
    assert!(built.diagnostics.is_empty(), "{:?}", built.diagnostics);
    let semantic = built.semantic;
    let facts = extract_from_semantic(&semantic, &[], false);
    let prepared = PreparedFixture {
        semantic: &semantic,
        temp_factories: path_origin::temp_factories(&semantic, &facts),
        exported: FxHashSet::default(),
    };
    test(&prepared);
}
