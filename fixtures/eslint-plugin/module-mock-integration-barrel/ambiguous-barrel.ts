// Both ambiguous-leaf.ts and ambiguous-leaf.mts exist. A ".js" specifier must
// resolve to the .ts sibling (what .js is actually emitted from under NodeNext),
// not probe .mts (which emits .mjs) first just because it's config-list order.
export * from "./ambiguous-leaf.js";
