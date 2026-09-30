import { ChildError } from "./hierarchy";
import { AliasedError as Renamed } from "./reexported";
import * as errors from "./reexported";
import { BarrelError, StarError } from "./barrel";
import { ConstructedError } from "./errors";

export function fail(kind: number): never {
  switch (kind) {
    case 0:
      throw new ConstructedError("constructed");
    case 1:
      throw new ChildError("child");
    case 2:
      throw new Renamed("alias");
    case 3:
      throw new BarrelError("barrel");
    case 4:
      throw new StarError("star");
    default:
      throw new errors.NamespaceError("namespace");
  }
}
