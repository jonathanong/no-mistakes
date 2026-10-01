import * as mod from "./errors";
import { Errors as Renamed, A } from "./errors";

// Each reaches an exported namespace of `errors.ts` through an import: a
// renamed binding, a named binding of a dotted namespace, and a module
// namespace import.
export const viaRename = () => new Renamed.Inner.Deep();
export const viaDotted = () => new A.B.C();
export const viaModule = () => new mod.Errors.Built();

export class Sub extends Renamed.Base {}
