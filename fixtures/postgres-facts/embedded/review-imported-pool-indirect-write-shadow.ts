import { p7 } from "@example/db";
import { withTx } from "./tx";

// A same-file function hides the imported name, so the call stays Dynamic.
function withTx(x: { query: unknown }) {
  x.query = () => undefined;
}

export function a() {
  return p7.query("SELECT id FROM users WHERE x = 1");
}

export function b() {
  withTx(p7);
}
