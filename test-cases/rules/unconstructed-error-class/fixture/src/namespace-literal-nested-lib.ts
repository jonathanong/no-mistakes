// `namespace-literal-nested.ts` builds `LiteralNestedDead` through a string
// literal that follows another member, which the graph cannot name, so the
// namespace stays quiet.
export namespace LiteralNested {
  export namespace Inner {
    export class LiteralNestedDead extends Error {}
  }
}
