declare function register(value: unknown): void;

// A namespace escapes when some use of it is not a static member access the
// graph resolves to a class. Each class below is named for its namespace.
export namespace Clean {
  export class CleanClass extends Error {}
}
export const clean = new Clean.CleanClass();

export namespace Aliased {
  export class AliasedClass extends Error {}
}
export const alias = Aliased;

export namespace Argument {
  export class ArgumentClass extends Error {}
}
register(Argument);

export namespace Computed {
  export class ComputedClass extends Error {}
}
export const computed = (name: "ComputedClass") => new Computed[name]();

export namespace Lost {
  export class LostClass extends Error {}
}
// `Lost` declares no `Missing`, so the construction could build any class.
export const lost = () => new Lost.Missing();

export class Mixed {}

// A namespace merged with a class: statics and members cannot be told apart.
export namespace Mixed {
  export class MixedClass extends Error {}
}

export namespace Defaulted {
  export class DefaultedClass extends Error {}
}
export default Defaulted;
