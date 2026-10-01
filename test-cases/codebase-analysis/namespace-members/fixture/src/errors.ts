// Namespace members the graph resolves to the exact class: through a qualified
// path, a bare name in the body, a nested namespace, and a dotted declaration.
export namespace Errors {
  export class Built extends Error {}

  export class Local extends Error {}

  // A bare `new Local()` names the class of the enclosing namespace.
  export function make() {
    return new Local();
  }

  export class Base extends Error {}

  // A bare `extends Base` names the class of the enclosing namespace.
  export class Child extends Base {}

  export namespace Inner {
    export class Deep extends Error {}
  }
}

export namespace A.B {
  export class C extends Error {}
}

export const built = new Errors.Built();
export const deep = new Errors.Inner.Deep();
export const dotted = new A.B.C();

export class TopLevel extends Errors.Base {}

// Two classes of one name, one in each namespace. The exact class id keeps
// them apart: the bare `new Twin()` and `extends Twin` each reach the `Twin`
// of their own namespace.
export namespace One {
  export class Twin extends Error {}

  export const one = () => new Twin();
}

export namespace Two {
  export class Twin extends Array {}

  export class TwinChild extends Twin {}
}
