import { pool } from "@example/db";

// must stay Dynamic: the tag writes the interpolated receiver's .query
function tag(_strings: TemplateStringsArray, x: { query: unknown }) {
  x.query = () => undefined;
}

export function a() {
  return pool.query("SELECT id FROM users WHERE x = 1");
}

export function b() {
  tag`x${pool}`;
}
