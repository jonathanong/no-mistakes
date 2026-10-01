// `body-scope.ts` builds `Built` and `VarBuilt`, and never `BodyDead` or
// `VarDead`.
export namespace BodyErrors {
  export class Built extends Error {}

  export class BodyDead extends Error {}
}

export namespace VarErrors {
  export class VarBuilt extends Error {}

  export class VarDead extends Error {}
}
