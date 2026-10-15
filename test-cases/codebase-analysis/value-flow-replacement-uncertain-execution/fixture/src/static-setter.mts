import { setterReceiver, first, opaque } from "@vendor/setter-controls";
class Setter {
  static set touch(value: unknown) {
    opaque(value);
  }
}
for (const item of ["a", "b"]) {
  setterReceiver.get(item);
  setterReceiver.get = first;
  // The collector already rejects this unrepresented write. Retain that
  // conservative boundary while admission also accounts for executable Set.
  Setter.touch = item;
}
