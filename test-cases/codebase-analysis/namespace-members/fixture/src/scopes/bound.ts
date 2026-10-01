// `bind`, `call` and `apply` receive the member they are called on, so a class
// they name is built through a value the graph does not follow.
export namespace Bound {
  export class BoundClass extends Error {}
}
export const bound = new (Bound.BoundClass.bind(null))();

export namespace Called {
  export class CalledClass extends Error {}
}
export const called = () => Called.CalledClass.call(null);

export namespace Applied {
  export class AppliedClass extends Error {}
}
export const applied = () => Applied.AppliedClass.apply(null, []);

// The same through an import.
export namespace Handed {
  export class HandedClass extends Error {}
}

// A static guard hands nothing on, and a plain function's `bind` names no
// member at all.
export namespace Guarded {
  export class GuardedClass extends Error {
    static is(value: unknown) {
      return value instanceof GuardedClass;
    }
  }
}
export const isGuarded = (value: unknown) => Guarded.GuardedClass.is(value);

const handler = () => undefined;
export const bindHandler = handler.bind(null);
