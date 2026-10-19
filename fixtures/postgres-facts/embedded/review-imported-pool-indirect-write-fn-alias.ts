import { p2 } from "@example/db";

// An alias of a same-file writer is not declared elsewhere, so the call stays Dynamic.
function patch(x: { query: unknown }) {
  x.query = () => undefined;
}
const g = patch;

export function a() {
  return p2.query("SELECT id FROM users WHERE x = 1");
}

export function b() {
  g(p2);
}
