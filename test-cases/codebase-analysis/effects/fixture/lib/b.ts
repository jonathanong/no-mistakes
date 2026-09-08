import { start } from "./a";

export function loop() {
  invalidate();
  // import cycle a <-> b must be traversed without infinite recursion
  start();
}

export function invalidate() {}
