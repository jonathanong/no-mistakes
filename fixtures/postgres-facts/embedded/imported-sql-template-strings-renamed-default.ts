import tag from "sql-template-strings";
// Type-only and side-effect imports must not mark `tag` (or `sql`) as a shadow.
import type DefaultSql from "untrusted-tag-library";
import { type sql } from "other-untrusted";
import "sql-template-strings";
import { query } from "@data-stores/psql";

export function load(id: number) {
  return query(tag`SELECT * FROM topics WHERE id = ${id}`);
}
