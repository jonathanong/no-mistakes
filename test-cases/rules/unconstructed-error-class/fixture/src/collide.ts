// Namespaces add no scope component, so both `CollideBase` classes share the
// display scope `CollideBase`. The graph still tells them apart by class id:
// `B.CollideChild` extends `B.CollideBase`, which is an Array, so it is not an
// error class. The rule credits by scope, though, so subclassing
// `B.CollideBase` also credits `A.CollideBase`: a dead class that shares its
// scope with a used one stays quiet, never a false finding. `collide-grand.ts`
// holds the top-level check. `BareBase` is the same pair, except that
// `B.BareBase` has no `extends` clause at all.
export namespace A {
  // Not flagged: it shares the scope `CollideBase` with `B.CollideBase`, which
  // `B.CollideChild` subclasses.
  export class CollideBase extends Error {}

  // Not flagged: it shares the scope `BareBase` with `B.BareBase`, which
  // `B.BareChild` subclasses.
  export class BareBase extends Error {}
}

export namespace B {
  export class CollideBase extends Array {}

  // Not flagged: its base is `B.CollideBase`, which is not an error class.
  export class CollideChild extends CollideBase {}

  export class BareBase {}

  // Not flagged: its base is `B.BareBase`, which extends nothing, so it is
  // not an error class either.
  export class BareChild extends BareBase {}
}
