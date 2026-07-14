# Integration runner parse sharing

The Vitest and Playwright runner configs are graph-only TypeScript source files
and both import the same discoverable TypeScript helper. Integration analysis
must parse every actual file once, reuse the helper's union facts for the
general source, graph, symbol, and Playwright passes, and never parse synthetic
function-body sources.
