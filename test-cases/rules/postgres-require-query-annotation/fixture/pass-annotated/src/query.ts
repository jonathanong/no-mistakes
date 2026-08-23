import { query } from "@data-stores/psql";

export function list(limit: number) {
  return query(`/* posts/list */ SELECT id FROM posts ORDER BY id DESC LIMIT ${limit}`);
}
