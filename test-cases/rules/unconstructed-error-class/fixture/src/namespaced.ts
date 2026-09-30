// The graph flattens namespaces out of a class's scope: a class declared in a
// namespace is scoped by its own name, exactly like a top-level class, and its
// members are scoped `Class/member`. Unresolved `new this()` construction is
// therefore credited to the class, not to the enclosing namespace. It cannot
// resolve `new Errors.TopicError()` to the namespace member, so the rule never
// reports a namespaced class.
export namespace Errors {
  // Not flagged: the static factory builds the class with `new this()`.
  export class TopicError extends Error {
    static create() {
      return new this();
    }
  }

  // Not flagged: a namespaced class is never reported, even when nothing
  // constructs it.
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
  // Not flagged: a dotted namespace name does not change the outcome.
  export class DeadDottedError extends Error {}
}
