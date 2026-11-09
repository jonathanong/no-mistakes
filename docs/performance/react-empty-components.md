# Empty component proof for default graph demand

Default forbidden-dependency relationships still request React render edges.
React file analysis first uses the existing component extractor. When that
extractor returns no components, the complete file result is empty: environment,
import resolution, identifier-reference, trait, and fetch walks cannot add a
component. The analyzer returns that exact empty result before those walks.

This proof applies to the existing exported component definition, including
uppercase functions and arrows, React classes, anonymous default exports, and
local re-exports. JSX alone does not imply an exported component. Files with any
component retain the full analysis, and other graph relationships keep their
original demand. No filename convention, new source cache, or private graph is
introduced.

Fixture tests compare the complete serialized file analysis with the original
entry path and count actual detailed-analysis passes. A default-relationship
check fixture preserves its forbidden dependency and a direct React-render edge
query preserves the parent-to-child edge. The separate performance corpus lives
in `fixtures/performance/react-empty-components`; matched measurements remain a
required adoption gate.
