// Each namespace here is used in a way the graph cannot follow to a class, so
// its dead class stays unreported rather than risk a false finding. Without
// the use named in its comment, each class would be flagged.

declare function register(value: unknown): void;

export namespace Aliased {
  // Not flagged: `alias` holds the namespace and builds the class through it.
  export class AliasedDead extends Error {}
}
const alias = Aliased;
export const viaAlias = () => new alias.AliasedDead();

export namespace Picked {
  // Not flagged: the class itself is stored in a variable.
  export class PickedDead extends Error {}
}
export const Stored = Picked.PickedDead;

export namespace Destructured {
  // Not flagged: destructuring reads the class out of the namespace.
  export class DestructuredDead extends Error {}
}
const { DestructuredDead } = Destructured;
export const viaDestructuring = () => new DestructuredDead();

export namespace Passed {
  // Not flagged: the whole namespace is handed to other code.
  export class PassedDead extends Error {}
}
register(Passed);

export namespace Argument {
  // Not flagged: the class is handed to other code.
  export class ArgumentDead extends Error {}
}
register(Argument.ArgumentDead);

export namespace Computed {
  // Not flagged: a computed access can name any member.
  export class ComputedDead extends Error {}
}
export const viaComputed = (name: "ComputedDead") => new Computed[name]();

export class Mixed {}

export namespace Mixed {
  // Not flagged: the namespace merges with a class of the same name, whose
  // statics the graph cannot tell apart from namespace members.
  export class MixedDead extends Error {}
}

export namespace Defaulted {
  // Not flagged: the namespace is the default export.
  export class DefaultedDead extends Error {}
}
export default Defaulted;

export namespace Gap {
  // Not flagged: `Gap` declares no `Missing`, so the construction below could
  // build any class.
  export class GapDead extends Error {}
}
export const viaGap = () => new Gap.Missing.Factory();
