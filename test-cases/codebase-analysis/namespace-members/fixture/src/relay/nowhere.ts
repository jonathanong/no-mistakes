// `./missing` is not a file of the repository, so the names this module
// exports are not all known.
export * from "./missing";

export namespace Nowhere {
  export class NowhereClass extends Error {}
}
