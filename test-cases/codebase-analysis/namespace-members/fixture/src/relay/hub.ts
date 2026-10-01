// Namespaces that `barrel.ts` and `stars.ts` pass on, each in its own way.
export namespace Hub {
  export class HubClass extends Error {}
}

export namespace Other {
  export class OtherClass extends Error {}
}

export namespace Spare {
  export class SpareClass extends Error {}
}

namespace Pivot {
  export class PivotClass extends Error {}
}

export { Pivot as default };
