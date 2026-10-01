// `namespace-class-shadow.ts` hides these names with declarations in a namespace
// body, so it builds none of the `Dead` classes and extends neither base. It
// builds `MergedBuilt` and `SplitBuilt`, whose names are private to another
// block of their namespace; another block exports its own `SharedShadow`.
export namespace ClassShadow {
  export class ClassShadowDead extends Error {}

  export class ClassShadowBase extends Error {}
}

export namespace FunctionShadow {
  export class FunctionShadowDead extends Error {}
}

export namespace EnumShadow {
  export class EnumShadowDead extends Error {}
}

export namespace LaterShadow {
  export class LaterShadowDead extends Error {}
}

export namespace NestedShadow {
  export class NestedShadowDead extends Error {}
}

export namespace MergedShadow {
  export class MergedBuilt extends Error {}
}

export class BareShadowed extends Error {}

export namespace AliasShadow {
  export class AliasShadowDead extends Error {}
}

export namespace SharedShadow {
  export class SharedShadowDead extends Error {}
}

export namespace SplitShadow {
  export class SplitBuilt extends Error {}
}
