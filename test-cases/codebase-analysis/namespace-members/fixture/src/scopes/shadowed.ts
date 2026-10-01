// A parameter named like the namespace, or like one of its classes, is not the
// namespace: neither construction builds a class of it.
export namespace Shadowed {
  export class ShadowDead extends Error {}

  export class InnerShadow extends Error {}

  export function make(InnerShadow: new () => Error) {
    return new InnerShadow();
  }
}

export function build(Shadowed: { ShadowDead: new () => Error }) {
  return new Shadowed.ShadowDead();
}
