// `export *` never re-exports a target's default binding (ES module semantics),
// so this barrel must NOT be treated as exposing a tagged `default` export even
// though default-leaf.ts's default export is itself tagged.
export * from "./default-leaf";
