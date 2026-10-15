import { receiver } from "@vendor/opaque-cold";
declare const opaqueObject: { consume: (...args: unknown[]) => unknown };
declare const opaque: (...args: unknown[]) => unknown;
// Unknown calls alone do not allocate replacement facts or a hot catalog.
for (const item of ["a", "b"]) {
  opaque(receiver);
  opaqueObject.consume(receiver);
  receiver.get(item);
}
