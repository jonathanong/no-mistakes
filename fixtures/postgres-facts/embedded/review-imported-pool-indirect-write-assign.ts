import { p2 } from "@example/db";

// Object.assign replaces .query on the imported executor, so the call must stay Dynamic.
export function a() {
  return p2.query("SELECT id FROM users WHERE x = 2");
}

export function b() {
  Object.assign(p2, { query: () => undefined });
}
