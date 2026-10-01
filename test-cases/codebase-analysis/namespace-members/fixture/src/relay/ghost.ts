// `./missing` is not a file of the repository, so what `Ghost` is stays unknown
// and the namespaces of this module may be reached through it.
export { Ghost } from "./missing";

export namespace Haunted {
  export class HauntedClass extends Error {}
}
