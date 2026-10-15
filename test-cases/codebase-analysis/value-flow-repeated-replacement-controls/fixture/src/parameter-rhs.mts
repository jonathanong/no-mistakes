import { parameterDest, rhs } from "@vendor/parameter-controls";
function runParameter(candidate: (key: string) => unknown) {
  parameterDest.get("key");
  // Parameter agreement is outside the replacement producer's stable-leaf proof.
  parameterDest.get = candidate;
}
runParameter(rhs.get);
runParameter(rhs.get);
parameterDest.set("key", "value");
