// Not flagged: `new this(...)` is an unresolved construction inside the class,
// so it counts as the class building itself.
export class ThisFactoryError extends Error {
  static create(): ThisFactoryError {
    return new this("factory");
  }
}

// Not flagged: `new this()` inside a nested arrow function still builds the
// class.
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
