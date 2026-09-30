// Not flagged: `new this(...)` is an unresolved construction inside the class,
// so it counts as the class building itself.
export class ThisFactoryError extends Error {
  static create(): ThisFactoryError {
    return new this("factory");
  }
}

// Not flagged: the factory names the class.
export class NamedFactoryError extends Error {
  static create(): NamedFactoryError {
    return new NamedFactoryError("factory");
  }
}
