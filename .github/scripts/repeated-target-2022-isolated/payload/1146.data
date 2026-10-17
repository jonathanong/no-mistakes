import {
  opaqueBeforeReceiver,
  opaqueAfterReceiver,
  unrelatedOpaqueReceiver,
  memberBeforeReceiver,
  memberAfterReceiver,
  aliasEscapeReceiver,
  externalEscapeReceiver,
  constructorReceiver,
  matchedReceiver,
  rhs,
} from "@vendor/opaque-producer";
import { opaqueExternal } from "@vendor/opaque-execution";
declare const opaque: (...args: unknown[]) => unknown;
declare const opaqueObject: { consume: (...args: unknown[]) => unknown };
declare const OpaqueConstructor: new (...args: unknown[]) => unknown;
const savedOpaque = opaqueBeforeReceiver.get;
// An opaque call can introduce a replacement outside the retained write set.
for (const item of ["a", "b"]) {
  opaqueBeforeReceiver.get(item);
  opaque(opaqueBeforeReceiver);
  opaqueBeforeReceiver.get = rhs.get;
  savedOpaque(item);
}
// Invalidation must update a region whose explicit write was already retained.
for (const item of ["a", "b"]) {
  opaqueAfterReceiver.get(item);
  opaqueAfterReceiver.get = rhs.get;
  opaque(opaqueAfterReceiver);
}
// A distinct receiver in an independent straightline region keeps its proof.
for (const item of ["a", "b"]) {
  unrelatedOpaqueReceiver.get(item);
  unrelatedOpaqueReceiver.get = rhs.get;
}

// Unknown member execution is opaque both before and after a retained write.
for (const item of ["a", "b"]) {
  memberBeforeReceiver.get(item);
  opaqueObject.consume(memberBeforeReceiver);
  memberBeforeReceiver.get = rhs.get;
}
for (const item of ["a", "b"]) {
  memberAfterReceiver.get(item);
  memberAfterReceiver.get = rhs.get;
  opaqueObject.consume(memberAfterReceiver);
}
const escapedAlias = aliasEscapeReceiver;
for (const item of ["a", "b"]) {
  aliasEscapeReceiver.get(item);
  opaqueObject.consume(escapedAlias);
  aliasEscapeReceiver.get = rhs.get;
}
// A known external callable also has no purity proof in this closed model.
const externalAlias = externalEscapeReceiver;
for (const item of ["a", "b"]) {
  externalEscapeReceiver.get(item);
  opaqueExternal.consume(externalAlias);
  externalEscapeReceiver.get = rhs.get;
}
for (const item of ["a", "b"]) {
  constructorReceiver.get(item);
  new OpaqueConstructor(constructorReceiver);
  constructorReceiver.get = rhs.get;
}
// Transparent method aliases and ordinary arguments remain supported.
for (const item of ["a", "b"]) {
  const method = matchedReceiver.get;
  const alias = method;
  alias(item);
  matchedReceiver.get("literal");
  matchedReceiver.get = rhs.get;
}
