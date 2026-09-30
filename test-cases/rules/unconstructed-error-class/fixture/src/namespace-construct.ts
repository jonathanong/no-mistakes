// The graph cannot resolve a reference to a namespace member, so neither
// `new Built.Qualified()` nor the bare `new Local()` inside the namespace
// credits its class. The rule therefore never reports a namespaced class,
// instead of calling a class dead that this file builds.
export namespace Built {
  // Not flagged: built below as `new Built.Qualified()`.
  export class Qualified extends Error {}

  // Not flagged: built by `make` as `new Local()`.
  export class Local extends Error {}

  export function make() {
    return new Local();
  }
}

export const qualified = new Built.Qualified();

export namespace Private {
  // Not flagged: not exported from its namespace, yet the graph treats every
  // member of an exported namespace as exported.
  class Hidden extends Error {}

  export function fail(): never {
    throw new Hidden();
  }
}

// Not flagged: subclassed only from inside a namespace. Nothing constructs
// `NsBase`, so it stays unreported only if that `extends` still credits it.
export class NsBase extends Error {}

export namespace Sub {
  export class Child extends NsBase {}
}
