// Two modules that re-export `Loop` from each other: no namespace is at the end
// of the chain.
export { Loop } from "./loop-b";

export namespace LoopA {
  export class LoopAClass extends Error {}
}
