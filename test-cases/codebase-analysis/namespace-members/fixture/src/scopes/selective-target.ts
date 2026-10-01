// `selective-barrel.ts` passes on `value` and `Exposed`, never `Kept`.
export namespace Kept {
  export class KeptClass extends Error {}
}

export namespace Exposed {
  export class ExposedClass extends Error {}
}

export const value = 1;
