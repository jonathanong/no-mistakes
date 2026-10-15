import { getterReceiver, safeReceiver, first, opaque } from "@vendor/getter-controls";
class Mutator {
  static get touch() {
    // This external execution can mutate the shared receiver. No receiver
    // argument or explicit foreign writer should make this test vacuous.
    opaque();
    return 0;
  }
}
for (const item of ["a", "b"]) {
  getterReceiver.get(item);
  getterReceiver.get = first;
  void Mutator.touch;
}
for (const item of ["a", "b"]) {
  safeReceiver.get(item);
  safeReceiver.get = first;
}
getterReceiver.set("outside", "value");
