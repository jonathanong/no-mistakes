import { nestedDest, dynamicDest, computedDest, rhs } from "@vendor/projection-controls";
declare const key: string;
for (const item of ["a", "b"]) {
  nestedDest.get(item);
  // The producer accepts only one exact imported-receiver member on the RHS.
  nestedDest.get = rhs.client.get;
}
for (const item of ["a", "b"]) {
  dynamicDest.get(item);
  dynamicDest.get = rhs[key];
}
for (const item of ["a", "b"]) {
  computedDest.get(item);
  computedDest.get = rhs["get"];
}
// Unsupported get replacements must not poison an unwritten sibling member.
nestedDest.set("key", "value");
dynamicDest.set("key", "value");
