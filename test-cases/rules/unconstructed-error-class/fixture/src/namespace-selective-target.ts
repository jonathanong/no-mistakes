// `namespace-selective-barrel.ts` passes on `Exposed` and `value`, never
// `Kept`.
export namespace Kept {
  export class KeptDead extends Error {}
}

export namespace Exposed {
  export class ExposedDead extends Error {}
}

export const value = 1;
