// `Unseen` here is a local namespace that this file never exports, so its
// class is never reported. The sourced clause exports the `Unseen` of
// `namespace-sourced-target.ts` as `Public`.
namespace Unseen {
  export class LocalUnseen extends Error {}
}

export { Unseen as Public } from "./namespace-sourced-target";
