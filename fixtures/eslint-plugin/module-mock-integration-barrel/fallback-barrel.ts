// "./fallback-leaf.js" has no real .ts/.tsx source — only the decoy
// fallback-leaf.js.ts, which must NOT be picked up via generic fallback probing.
export * from "./fallback-leaf.js";
