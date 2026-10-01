// `bind`, `call` and `apply` receive the member they are called on, so a class
// they name is built through a value the rule cannot follow.
export namespace Bound {
  export class BoundDead extends Error {}
}
export const bound = new (Bound.BoundDead.bind(null))();

export namespace Called {
  export class CalledDead extends Error {}
}
export const called = () => Called.CalledDead.call(null);

export namespace Applied {
  export class AppliedDead extends Error {}
}
export const applied = () => Applied.AppliedDead.apply(null, []);

// Used through an import in `namespace-bound-use.ts`.
export namespace Handed {
  export class HandedDead extends Error {}
}

// A static guard hands nothing on: the class is still never built.
export namespace Guarded {
  export class GuardedDead extends Error {
    static is(value: unknown) {
      return value instanceof GuardedDead;
    }
  }
}
export const isGuarded = (value: unknown) => Guarded.GuardedDead.is(value);
