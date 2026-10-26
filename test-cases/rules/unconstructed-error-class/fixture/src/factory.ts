// An unresolved `new this(...)` reports uncertainty, and does not prove that
// the error class is constructed.
export class ThisFactoryError extends Error {
  static create(): ThisFactoryError {
    return new this("factory");
  }
}

// An unresolved constructor inside a nested arrow also reports uncertainty,
// without crediting the enclosing class.
export class NestedFactoryError extends Error {
  static create(): NestedFactoryError {
    return (() => new this())();
  }
}

// Not flagged: the factory names the class.
export class NamedFactoryError extends Error {
  static create(): NamedFactoryError {
    return new NamedFactoryError("factory");
  }
}
