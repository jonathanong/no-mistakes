import { p4 } from "@example/db";

// A destructured binding is not declared elsewhere, so the call stays Dynamic.
const { assign } = Object;

export function a() {
  return p4.query("SELECT id FROM users WHERE x = 1");
}

export function b() {
  assign(p4, { query: () => undefined });
}
