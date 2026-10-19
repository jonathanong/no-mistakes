import { p6 } from "@example/db";

// Overload signatures have no body. The implementation still writes .query.
function patch(x: { query: unknown }): void;
function patch(x: { query: unknown }): void;
function patch(x: { query: unknown }) {
  x.query = () => undefined;
}

export function a() {
  return p6.query("SELECT id FROM users WHERE x = 1");
}

export function b() {
  patch(p6);
}
