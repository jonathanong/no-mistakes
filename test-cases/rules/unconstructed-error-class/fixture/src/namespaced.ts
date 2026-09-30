// The graph flattens namespaces out of a class's scope: a class declared in a
// namespace is scoped by its own name, exactly like a top-level class, and its
// members are scoped `Class/member`. Unresolved `new this()` construction is
// therefore credited to the class, not to the enclosing namespace.
export namespace Errors {
  // Not flagged: the static factory builds the class with `new this()`.
  export class TopicError extends Error {
    static create() {
      return new this();
    }
  }

  // Flagged: nested classes are still reported when nothing constructs them.
  export class DeadNamespacedError extends Error {}

  export namespace Inner {
    // Not flagged: same as `TopicError`, one namespace deeper.
    export class DeepError extends Error {
      static create() {
        return (() => new this())();
      }
    }
  }
}

export namespace Dotted.Path {
  // Flagged: a dotted namespace name does not change the outcome.
  export class DeadDottedError extends Error {}
}
