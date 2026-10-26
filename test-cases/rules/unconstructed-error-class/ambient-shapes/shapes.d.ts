// Script on purpose: a top-level import would hide these ambient modules.
// An export list turns off implicit ambient exports, so unexported Inner stays private.
// `spec:multi` names two classes, so a missing export is ambiguous rather than absent.
declare class Orphan {}

declare namespace Outer {
  export namespace Inner {
    export class NestedBase {}
  }
}

declare namespace Sealed {
  export class Marker {}
  export { Marker as Shown };
  namespace Inner {
    export class NestedBase {}
  }
}

declare module "spec:direct" {
  export class Direct {}
}

declare module "spec:nested" {
  export = Outer;
}

declare module "spec:sealed" {
  export = Sealed;
}

declare module "spec:multi" {
  export class Left {}
  export class Right {}
}
