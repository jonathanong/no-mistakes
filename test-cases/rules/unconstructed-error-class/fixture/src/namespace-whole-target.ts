export namespace Whole {
  // Not flagged: `namespace-escape-use.ts` uses the module namespace as a
  // value, so any exported namespace may be reached through it.
  export class WholeDead extends Error {}
}
