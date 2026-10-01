declare function register(value: unknown): void;

// A class named bare inside the namespace that declares it is a value, as
// `BareHanded.BareHandedDead` would be: handing it on reaches whatever builds
// it, so its namespace escapes.
export namespace BareHanded {
  export class BareHandedDead extends Error {}
  register(BareHandedDead);
}

// The same from a namespace nested in the one that declares the class.
export namespace BareNested {
  export class BareNestedDead extends Error {}
  export namespace Inner {
    export const handed = register(BareNestedDead);
  }
}

// `bind` hands the class itself on.
export namespace BareBound {
  export class BareBoundDead extends Error {}
  export const bound = BareBoundDead.bind(null);
}

// A parameter that shadows the class name, a guard, and a type hand nothing on.
export namespace BareKept {
  export class BareKeptDead extends Error {}
  export function shadowed(BareKeptDead: unknown): void {
    register(BareKeptDead);
  }
  export function guard(error: unknown): error is BareKeptDead {
    return error instanceof BareKeptDead;
  }
}
