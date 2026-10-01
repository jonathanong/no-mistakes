// A member its namespace does not export is not an export of the file, so it is
// never reported, and neither is a member of a namespace that is not exported.
export namespace Private {
  // Not flagged: not exported from its namespace.
  class Hidden extends Error {}

  export function fail(): never {
    throw new Hidden();
  }
}

namespace Internal {
  // Not flagged: the namespace itself is not exported.
  export class InternalError extends Error {}
}

// Not flagged: subclassed only from inside a namespace. Nothing constructs
// `NsBase`, so it stays unreported only if that `extends` still credits it.
export class NsBase extends Error {}

export namespace Sub {
  // Flagged: nothing builds it, though it still credits `NsBase`.
  export class Child extends NsBase {}
}

export namespace Scoped {
  export function make() {
    // Not flagged: declared in a function, not a member of the namespace.
    class LocalError extends Error {}
    return new LocalError();
  }

  // Not flagged: a class expression is not a member of the namespace.
  export const Expression = class ExpressionMemberError extends Error {};
}
