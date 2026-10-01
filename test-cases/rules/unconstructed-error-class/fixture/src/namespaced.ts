// A class declared in a namespace is reported like a top-level class: when no
// non-test source builds or subclasses it. The graph resolves each member
// reference below, and `Errors` is only ever the head of a `new` or an
// `extends`, so the namespace never escapes.
export namespace Errors {
  // Not flagged: the static factory builds the class with `new this()`.
  export class TopicError extends Error {
    static create() {
      return new this();
    }
  }

  // Flagged: nothing builds or subclasses it.
  export class DeadNamespacedError extends Error {}

  // Not flagged: built below as `new Errors.Built()`.
  export class Built extends Error {}

  // Not flagged: built by `make` as a bare `new Local()` in the namespace body.
  export class Local extends Error {}

  export function make() {
    return new Local();
  }

  // Not flagged: subclassed by `Child` and, from outside, by `TopLevelChild`.
  export class Base extends Error {}

  // Flagged: nothing builds it, though it still credits `Base`.
  export class Child extends Base {}

  export namespace Inner {
    // Flagged: a nested namespace does not change the outcome.
    export class DeepDeadError extends Error {}

    // Not flagged: built below as `new Errors.Inner.DeepBuilt()`.
    export class DeepBuilt extends Error {}

    // Not flagged: the static factory builds it, as for `TopicError`.
    export class DeepError extends Error {
      static create() {
        return (() => new this())();
      }
    }
  }
}

export const built = new Errors.Built();
export const deep = new Errors.Inner.DeepBuilt();

// Flagged: it extends the namespaced `Errors.Base`, which makes it an error
// class, and nothing builds it.
export class TopLevelChild extends Errors.Base {}

export namespace Dotted.Path {
  // Flagged: a dotted name resolves like a nested namespace.
  export class DeadDottedError extends Error {}

  // Not flagged: built below as `new Dotted.Path.BuiltDotted()`.
  export class BuiltDotted extends Error {}
}

export const dotted = new Dotted.Path.BuiltDotted();
