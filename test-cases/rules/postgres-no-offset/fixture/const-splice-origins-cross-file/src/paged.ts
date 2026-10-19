import { read } from "@example/db";

export function page() {
  return read(`SELECT id FROM posts ORDER BY id OFFSET 10`);
}
