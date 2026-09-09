import { importedTarget as localAlias } from "./imported-target.mts";
import { reexportedTarget } from "./reexported-target.mts";
import * as star from "./star-barrel.mts";
import * as directNamespace from "./imported-target.mts";
import { importedTarget as namedValue } from "./imported-target.mts";
import defaultTarget from "./default-target.mts";

function target() {}

function run() {
  target();
  target();
  run();
}

run();
localAlias();
reexportedTarget();
star.starTarget();
directNamespace.importedTarget();
// Named/default bindings are callable values, never namespace objects.
namedValue.member();
defaultTarget.member();
defaultTarget();
