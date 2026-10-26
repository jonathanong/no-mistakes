// An unknown constructor in a member must not credit its enclosing error class.
export class DynamicError extends Error {
  static make(ctor: any) { return new ctor(); }
}
// A nested class's unknown constructor must not credit the outer error class.
export class OuterError extends Error {
  static make() { class Nested { make(ctor: any) { return new ctor(); } } return Nested; }
}

// Runtime counterexample: the unknown constructor really can be the enclosing class.
export const built = DynamicError.make(DynamicError);
