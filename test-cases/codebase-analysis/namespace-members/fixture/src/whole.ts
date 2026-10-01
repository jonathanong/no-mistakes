// `reach.ts` reads this module as a value, so any namespace it exports may be
// reached through that value.
export namespace Whole {
  export class WholeClass extends Error {}
}
