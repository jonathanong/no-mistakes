export class Before extends Error {}

export namespace Outer {
  export class InNamespace extends Error {}

  export namespace Inner {
    export class InNested extends Error {}
  }
}

export namespace Dotted.Path {
  export class InDotted extends Error {}
}

declare namespace Ambient {
  class InDeclareNamespace extends Error {}
}

declare module "virtual" {
  export class InModule extends Error {}
}

declare global {
  class InGlobal extends Error {}
}

export declare class Declared extends Error {}

// Every block above is closed by now, so a balanced depth marks nothing here.
export class After extends Error {}
