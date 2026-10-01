// A bare decorator calls the member it names, so it reads the namespace as a
// call head and not as a value. `DecoratedDead` is still built by nothing.
export namespace Decorated {
  export function mark(target: unknown) {
    return target;
  }

  export class DecoratedDead extends Error {}
}

@Decorated.mark
export class Host {
  @Decorated.mark
  method() {}
}
