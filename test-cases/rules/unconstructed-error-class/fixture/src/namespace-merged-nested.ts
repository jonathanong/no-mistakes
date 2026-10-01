// A class, function or enum merged with a nested namespace of the same name,
// in one body or across blocks of the parent: a static member access through
// the merged name builds the namespace's class, wherever it is written.
export namespace MergedNested {
  export class Holder {}

  export namespace Holder {
    // Not flagged: built below through the merged `Holder`.
    export class Built extends Error {}

    // Not flagged: subclassed below through the merged `Holder`.
    export class Base extends Error {}

    // Flagged: nothing builds it.
    export class Unused extends Error {}
  }

  export const built = new Holder.Built();

  // Flagged: nothing builds it.
  export class Child extends Holder.Base {}
}

export namespace MergedNestedFunction {
  export function make() {}
}

export namespace MergedNestedFunction {
  export namespace make {
    // Not flagged: built below through the merged `make`.
    export class Made extends Error {}
  }

  export function build() {
    return new make.Made();
  }
}

export namespace MergedNestedEnum {
  export enum Kind {
    Plain,
  }

  export namespace Kind {
    // Not flagged: built below through the merged `Kind`.
    export class KindBuilt extends Error {}
  }

  export function build() {
    return new Kind.KindBuilt();
  }
}
