// A class, function, or enum declared in a namespace body hides an import of
// its name inside that body, and so does a nested namespace, an `import`
// alias, and a `const` declared after the code that reads it. The constructions
// and the base below name those values, never the imported ones.
import {
  AliasErrors,
  BareImported,
  ClassErrors,
  DeepErrors,
  DottedErrors,
  EnumErrors,
  FunctionErrors,
  LaterErrors,
  MergedErrors,
  NestedErrors,
  SharedErrors,
  SplitErrors,
} from "./class-shadow-lib";

declare function make(): any;

export namespace ShadowBodies {
  class ClassErrors {
    static ClassDead = Error;
    static ClassBase = Error;
  }
  function FunctionErrors() {}
  function BareImported() {}
  enum EnumErrors {
    EnumDead,
  }
  export namespace NestedErrors {
    export class NestedLocal extends Error {}
  }

  export const byClass = new ClassErrors.ClassDead();
  export const byFunction = new FunctionErrors.FunctionDead();
  export const byEnum = new EnumErrors.EnumDead();
  export namespace AliasTarget {
    export class AliasLocal extends Error {}
  }
  import AliasErrors = AliasTarget;

  export const byNested = new NestedErrors.NestedLocal();
  export const byAlias = new AliasErrors.AliasLocal();
  export const byBare = new BareImported();
  export class Sub extends ClassErrors.ClassBase {}
  export function later() {
    return new LaterErrors.LaterDead();
  }
  const LaterErrors = make();
}

// The first block's class is private to it, so the second block's `new` still
// names the import.
export namespace MergedBlocks {
  class MergedErrors {}
}

export namespace MergedBlocks {
  export const second = new MergedErrors.MergedDead();
}

// An exported member is shared by every block of a merged namespace, so the
// second block's `new` builds the first block's nested class, not the import.
export namespace SharedBlocks {
  export namespace SharedErrors {
    export class SharedDead extends Error {}
  }
}

export namespace SharedBlocks {
  export const shared = new SharedErrors.SharedDead();
}

// So are the exported members of an exported namespace merged one level down.
export namespace DeepBlocks {
  export namespace Inner {
    export namespace DeepErrors {
      export class DeepDead extends Error {}
    }
  }
}

export namespace DeepBlocks {
  export namespace Inner {
    export const deep = new DeepErrors.DeepDead();
  }
}

// A dotted declaration exports its last name from the namespace before it.
export namespace DottedBlocks.DottedErrors {
  export class DottedDead extends Error {}
}

export namespace DottedBlocks {
  export const dotted = new DottedErrors.DottedDead();
}

// An unexported `Inner` in each block is two namespaces, so the second cannot
// see the first one's member and its `new` still names the import.
export namespace SplitBlocks {
  namespace Inner {
    export namespace SplitErrors {
      export class SplitDead extends Error {}
    }
  }
}

export namespace SplitBlocks {
  namespace Inner {
    export const split = new SplitErrors.SplitDead();
  }
}
