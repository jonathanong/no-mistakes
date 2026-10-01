// Namespace members built from other files, through each way to reach a
// namespace: a named import, a renamed import, a namespace import, a barrel,
// and a namespace exported under another name.
export namespace Lib {
  // Not flagged: `namespace-use.ts` builds it as `new Lib.Used()`.
  export class Used extends Error {}

  // Flagged: no file builds or subclasses it.
  export class DeadLibError extends Error {}

  // Not flagged: built through a renamed import, `new Aliased.ViaImportRename()`.
  export class ViaImportRename extends Error {}

  // Not flagged: built through a namespace import, `new lib.Lib.ViaStar()`.
  export class ViaStar extends Error {}

  // Not flagged: built through the barrel as `new BarrelLib.ViaBarrel()`.
  export class ViaBarrel extends Error {}

  // Flagged: only a test builds it, and a test does not count.
  export class TestOnlyBuilt extends Error {}

  // Not flagged: `namespace-use.ts` extends it as `Lib.Base`.
  export class Base extends Error {}

  export namespace Deep {
    // Not flagged: built as `new Lib.Deep.DeepUsed()`.
    export class DeepUsed extends Error {}

    // Flagged: no file builds it.
    export class DeepDead extends Error {}
  }
}

namespace Renamed {
  // Not flagged: built as `new Public.ViaExportRename()`.
  export class ViaExportRename extends Error {}

  // Flagged: the export clause below exports the namespace as `Public`.
  export class RenamedDead extends Error {}
}

export { Renamed as Public };
