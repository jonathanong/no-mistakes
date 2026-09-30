// Namespaces add no scope component, so both `CollideBase` classes share the
// display scope `CollideBase`. An `extends` (or `new`) naming a scope that two
// declarations share resolves to no edge, so the shared key never merges the
// two parents: the Array-based `B.CollideBase` must not make `B.CollideChild`
// an error class, and must not credit the error class in `A`.
export namespace A {
  // Flagged: an error class that nothing constructs or subclasses.
  export class CollideBase extends Error {}
}

export namespace B {
  export class CollideBase extends Array {}

  // Not flagged: its base is `B.CollideBase`, which is not an error class.
  export class CollideChild extends CollideBase {}
}
