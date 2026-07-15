// Same stem, different extension, deliberately WITHOUT taggedAmbiguousProviderCall:
// a `.js` re-export specifier must resolve to ambiguous-leaf.ts (which .js is
// actually emitted from), not this .mts sibling (which emits .mjs).
export function unrelatedAmbiguousExport() {
  return "real";
}
