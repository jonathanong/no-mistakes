// Exported namespaces that only another file touches. `namespace-consumer.ts`
// reaches each one through an import. Four uses leave the graph unable to tell
// which class gets built, so those namespaces stay unreported; the default
// import is followed like a named one.
export namespace ViaAlias {
  // Not flagged: the importing file stores the namespace in a variable.
  export class ViaAliasDead extends Error {}
}

export namespace ViaArgument {
  // Not flagged: the importing file hands the namespace to other code.
  export class ViaArgumentDead extends Error {}
}

export namespace ViaComputed {
  // Not flagged: the importing file indexes the namespace with a variable.
  export class ViaComputedDead extends Error {}
}

export namespace ViaMember {
  // Not flagged: the importing file hands the class itself to other code.
  export class ViaMemberDead extends Error {}
}

namespace Standard {
  // Not flagged: built through the default import as `new Std.Built()`.
  export class Built extends Error {}

  // Flagged: the default import resolves, and nothing builds this class.
  export class StandardDead extends Error {}
}

export { Standard as default };
