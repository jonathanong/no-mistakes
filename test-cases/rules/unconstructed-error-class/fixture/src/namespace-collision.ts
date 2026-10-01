declare function register(value: unknown): void;

// A value that shares a segment name with a namespace is not that namespace:
// `Inner` below is a plain constant, and `Collide.Inner` is never read.
export namespace Collide {
  export namespace Inner {
    export class CollideDead extends Error {}
  }
}

const Inner = {};
register(Inner);

// A parameter named like the namespace is not the namespace either.
export namespace Hide {
  export class HideDead extends Error {}
}

export function hide(Hide: unknown) {
  register(Hide);
}

// Inside `Reach`, a bare `Inner` is the namespace `Reach.Inner`, which this
// reads as a value.
export namespace Reach {
  export namespace Inner {
    export class ReachDead extends Error {}
  }

  export const leaked = Inner;
}
