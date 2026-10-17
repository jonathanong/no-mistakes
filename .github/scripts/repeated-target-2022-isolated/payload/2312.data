import {
  closedOpaqueReceiver,
  nestedOpaqueReceiver,
  regionOwnerReceiver,
  wideOwnerReceiver,
  rhs,
} from "@vendor/opaque-producer";
import { opaqueExternal } from "@vendor/opaque-execution";
// This owner comes first so no earlier replacement facts reject its closed
// proof during loop setup. Match the actual region before the wider owner
// group: these tracked calls must not make the enclosing owner opaque.
function runRegions() {
  for (const item of ["a", "b"]) {
    regionOwnerReceiver.get(item);
    regionOwnerReceiver.get = rhs.get;
  }
  wideOwnerReceiver.get("key");
  wideOwnerReceiver.get = rhs.get;
}
runRegions();
runRegions();

// A known external member can replace get, even in a closed local owner.
function runOpaque() {
  closedOpaqueReceiver.get("key");
  opaqueExternal.consume(closedOpaqueReceiver);
  closedOpaqueReceiver.get = rhs.get;
}
runOpaque();
runOpaque();

// The do-once region is not a backedge itself, but its opaque execution can
// replace the value observed on the closed owner's next invocation.
function runNestedOpaque() {
  nestedOpaqueReceiver.get("key");
  nestedOpaqueReceiver.get = rhs.get;
  do {
    opaqueExternal.consume(nestedOpaqueReceiver);
  } while (false);
}
runNestedOpaque();
runNestedOpaque();
