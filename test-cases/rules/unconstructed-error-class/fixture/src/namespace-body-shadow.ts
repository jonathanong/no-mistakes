// A local in a namespace body that shares a declared namespace's name is that
// local inside the body, so the `new` there builds nothing of the namespace and
// `ShadowDead` is still reported.
export namespace ShadowErrors {
  export class ShadowDead extends Error {}
}

declare function make(): any;

export namespace ShadowHelpers {
  const ShadowErrors = make();
  export const built = new ShadowErrors.ShadowDead();
}
