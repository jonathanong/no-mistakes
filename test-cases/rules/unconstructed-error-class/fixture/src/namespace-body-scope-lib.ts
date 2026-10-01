// `namespace-body-scope.ts` builds the `Built` classes and never the `Dead` ones.
export namespace BodyScope {
  export class ScopeBuilt extends Error {}

  export class ScopeDead extends Error {}
}

export namespace BodyVar {
  export class VarBuilt extends Error {}

  export class VarDead extends Error {}
}
