export namespace Backstop {
  // Not flagged: `namespace-escape-use.ts` builds `Backstop.Missing`, a member
  // the graph cannot find. A construction it cannot resolve might be this
  // class, so the namespace counts as escaped.
  export class BackstopDead extends Error {}
}
