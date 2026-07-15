// `export *` never re-exports a target's default binding, even when the tag sits
// on a named-export-list alias like `export { x as default }` rather than an
// `export default` declaration.
export * from "./named-default-leaf";
