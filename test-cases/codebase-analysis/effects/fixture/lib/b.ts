import { start } from "./a";

export function loop() {
  invalidate();
  // import cycle a <-> b must be traversed without infinite recursion
  start();
}

function invalidate() {}
