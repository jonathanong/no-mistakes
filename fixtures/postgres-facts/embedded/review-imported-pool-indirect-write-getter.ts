import { p2 } from "@example/db";

// Returning the import lets another function write .query, so the call must stay Dynamic.
function get() {
  return p2;
}

export function a() {
  return p2.query("SELECT id FROM users WHERE x = 2");
}

export function b() {
  get().query = () => undefined;
}
