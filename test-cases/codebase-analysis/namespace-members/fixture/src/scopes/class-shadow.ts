// A class, function, or enum declared in a namespace body hides an import of
// its name inside that body, and so does a nested namespace, an `import`
// alias, and a `const` declared after the code that reads it. The constructions
// and the base below name those values, never the imported ones.
import {
  AliasErrors,
  BareImported,
  ClassErrors,
  EnumErrors,
  FunctionErrors,
  LaterErrors,
  MergedErrors,
  NestedErrors,
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
