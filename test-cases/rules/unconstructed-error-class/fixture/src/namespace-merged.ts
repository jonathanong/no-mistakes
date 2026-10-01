// Declaration merging that the graph resolves: blocks of one namespace, and a
// dotted declaration beside a nested one, share a single member table.
export namespace Merged {
  // Not flagged: built below as `new Merged.First()`.
  export class First extends Error {}
}

export namespace Merged {
  // Flagged: a second block of the same namespace changes nothing.
  export class Second extends Error {}
}

export const first = new Merged.First();

export namespace Mix.Inner {
  // Not flagged: built below as `new Mix.Inner.DottedPart()`.
  export class DottedPart extends Error {}
}

export namespace Mix {
  export namespace Inner {
    // Flagged: the nested block extends the namespace the dotted one declared.
    export class NestedPart extends Error {}
  }
}

export const dottedPart = new Mix.Inner.DottedPart();
