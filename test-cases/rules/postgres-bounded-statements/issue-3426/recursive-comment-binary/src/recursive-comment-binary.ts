import sql from "sql-template-strings";
import { query } from "@app/db";

export async function ancestorWalk(start: string, max: number) {
  return query(sql`WITH RECURSIVE w AS (
    SELECT id, parent_id, 0 AS depth FROM posts WHERE id = ${start}
    UNION ALL
    SELECT posts.id, posts.parent_id, w.depth + 1
    -- no-mistakes-disable-next-line postgres-bounded-statements: depth is capped for this ancestry walk
    FROM posts JOIN w ON w.parent_id = posts.id WHERE w.depth < ${max + 1}
  ) SELECT * FROM w`);
}

export async function unsuppressedWalk(start: string, max: number) {
  return query(sql`WITH RECURSIVE w AS (
    SELECT id, parent_id, 0 AS depth FROM posts WHERE id = ${start}
    UNION ALL
    SELECT posts.id, posts.parent_id, w.depth + 1
    FROM posts JOIN w ON w.parent_id = posts.id WHERE w.depth < ${max + 1}
  ) SELECT * FROM w`);
}

export async function uncappedPositive(start: string) {
  return query(sql`WITH RECURSIVE w AS (
    SELECT id, parent_id, 0 AS depth FROM posts WHERE id = ${start}
    UNION ALL
    SELECT posts.id, posts.parent_id, w.depth + 1
    FROM posts JOIN w ON w.parent_id = posts.id
  ) SELECT * FROM w`);
}

export async function bypassedCap(start: string, max: number) {
  return query(sql`WITH RECURSIVE w AS (
    SELECT id, parent_id, 0 AS depth FROM posts WHERE id = ${start}
    UNION ALL
    SELECT posts.id, posts.parent_id, w.depth + 1
    FROM posts JOIN w ON w.parent_id = posts.id WHERE w.depth < ${max + 1} OR posts.parent_id IS NOT NULL
  ) SELECT * FROM w`);
}
