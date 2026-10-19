import { p5 } from "@example/db";

// A same-file helper writes the parameter's .query, so the call must stay Dynamic.
function patch(x: { query: unknown }) {
  x.query = () => undefined;
}

export function a() {
  return p5.query("SELECT id FROM users WHERE x = 5");
}

export function b() {
  patch(p5);
}
