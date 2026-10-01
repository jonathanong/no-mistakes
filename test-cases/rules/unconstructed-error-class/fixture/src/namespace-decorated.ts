// A bare decorator calls the member it names, so it is not a value use of the
// namespace: `DecoratedDead` is still reported.
export namespace Decorated {
  export function mark(target: unknown) {
    return target;
  }

  export class DecoratedDead extends Error {}
}

@Decorated.mark
export class DecoratedHost {}
