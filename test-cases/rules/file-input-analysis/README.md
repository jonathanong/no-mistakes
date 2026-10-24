Required-source analysis fixtures intentionally contain invalid UTF-8 files,
malformed Rust with allow/test markers, and a malformed TypeScript route.
Keep byte-invalid siblings and file suppression directives: they protect recall
without retrying physical reads or analyzing lossy source. Marker-negative Rust
remains outside AST demand; max-lines-only analysis never needs a Rust parse.
Each saved rule config selects one definite violation beside uncertain inputs.
