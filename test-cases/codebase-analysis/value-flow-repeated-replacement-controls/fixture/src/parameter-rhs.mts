import { parameterDest, rhs } from "@vendor/parameter-controls";
function runParameter(candidate: (key: string) => unknown = rhs.get) {
  parameterDest.get("key");
  // Parameter agreement is outside the replacement producer's stable-leaf proof.
  parameterDest.get = candidate;
}
// Closed-owner evidence requires contiguous no-argument physical calls.
runParameter();
runParameter();
parameterDest.set("key", "value");
