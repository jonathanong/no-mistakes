import { Lib } from "./namespace-lib";
import { Lib as Aliased, Public } from "./namespace-lib";
import * as lib from "./namespace-lib";
import { Lib as BarrelLib } from "./namespace-barrel";

export const used = new Lib.Used();
export const deepUsed = new Lib.Deep.DeepUsed();
export const renamedImport = new Aliased.ViaImportRename();
export const renamedExport = new Public.ViaExportRename();
export const namespaceImport = new lib.Lib.ViaStar();
export const barrel = new BarrelLib.ViaBarrel();

// Not flagged: not exported. It still credits `Lib.Base`.
class UseChild extends Lib.Base {}

export function make() {
  return new UseChild();
}
