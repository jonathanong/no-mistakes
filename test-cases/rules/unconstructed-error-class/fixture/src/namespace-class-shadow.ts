// A class, function, or enum declared in a namespace body hides an import of
// its name inside that body, and so does a nested namespace, an `import`
// alias, and a `const` declared after the code that reads it. The constructions
// and the base below name those values, never the imported ones.
import {
  AliasShadow,
  BareShadowed,
  ClassShadow,
  EnumShadow,
  FunctionShadow,
  LaterShadow,
  MergedShadow,
  NestedShadow,
  SharedShadow,
  SplitShadow,
} from "./namespace-class-shadow-lib";

declare function make(): any;

export namespace Shadowing {
  class ClassShadow {
    static ClassShadowDead = Error;
    static ClassShadowBase = Error;
  }
  function FunctionShadow() {}
  function BareShadowed() {}
  enum EnumShadow {
    EnumShadowDead,
  }
  export namespace NestedShadow {
    export class NestedLocal extends Error {}
  }

  export const byClass = new ClassShadow.ClassShadowDead();
  export const byFunction = new FunctionShadow.FunctionShadowDead();
  export const byEnum = new EnumShadow.EnumShadowDead();
  export namespace AliasTarget {
    export class AliasLocal extends Error {}
  }
  import AliasShadow = AliasTarget;

  export const byNested = new NestedShadow.NestedLocal();
  export const byAlias = new AliasShadow.AliasLocal();
  export const byBare = new BareShadowed();
  export class Sub extends ClassShadow.ClassShadowBase {}
  export function later() {
    return new LaterShadow.LaterShadowDead();
  }
  const LaterShadow = make();
}

// The first block's class is private to it, so the second block's `new` still
// names the import.
export namespace MergedBlocks {
  class MergedShadow {}
}

export namespace MergedBlocks {
  export const second = new MergedShadow.MergedBuilt();
}

// An exported member is shared by every block of a merged namespace, so the
// second block's `new` builds the first block's class, none of the import's.
export namespace SharedBlocks {
  export namespace SharedShadow {
    export class SharedShadowDead extends Error {}
  }
}

export namespace SharedBlocks {
  export const shared = new SharedShadow.SharedShadowDead();
}

// An unexported `Inner` in each block is two namespaces, so the second cannot
// see the first one's member and its `new` still names the import.
export namespace SplitBlocks {
  namespace Inner {
    export namespace SplitShadow {
      export class SplitBuilt extends Error {}
    }
  }
}

export namespace SplitBlocks {
  namespace Inner {
    export const split = new SplitShadow.SplitBuilt();
  }
}
