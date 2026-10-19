import { p6 } from "@example/db";

// Object.defineProperty replaces .query on the imported executor, so the call must stay Dynamic.
export function a() {
  return p6.query("SELECT id FROM users WHERE x = 6");
}

export function b() {
  Object.defineProperty(p6, "query", { value: () => undefined });
}
