export class RepoError extends Error {}
// A block-local class must never turn the exported opaque value into a class proof.
{ class Factory {} }
export const Factory: any = function () { return new Error(); };
