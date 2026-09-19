// Identifier and concatenation specifiers are unprovable. Treating `moduleName`
// or `<computed>` as an npm package would emit a false undeclared-dependency
// finding; resolve-check reports these as computed/unresolved instead.
export async function helper(moduleName: string, suffix: string) {
  await import(moduleName);
  return import("./" + suffix);
}
