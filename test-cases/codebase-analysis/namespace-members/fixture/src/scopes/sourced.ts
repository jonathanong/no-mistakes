// `Hidden` here is a local namespace that this file never exports. The sourced
// clause exports the `Hidden` of `sourced-target.ts` as `Public`.
namespace Hidden {
  export class LocalHidden extends Error {}
}

export { Hidden as Public } from "./sourced-target";
