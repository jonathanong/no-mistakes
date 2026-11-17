import { query } from "@example/db";

const relationTables = Object.freeze({
  topic: "relation__topic",
  post: "relation__post",
} as const);

function tableIdentifier(name: string) {
  if (typeof name !== "string" || !Object.hasOwn(relationTables, name)) {
    throw new Error(`unknown ${name}`);
  }
  return relationTables[name as keyof typeof relationTables];
}

// The closed set is not a catalog-key miss. Value holes stay binds so the
// table names remain visible to the catalog check.
export function softDelete(name: string, objs: string[], subs: string[]) {
  return query(
    `WITH locked AS (SELECT r.object_id, r.subject_id FROM ${tableIdentifier(name)} r JOIN unnest($1::uuid[], $2::uuid[]) AS k(object_id, subject_id) USING (object_id, subject_id) WHERE r.object_id = ANY($1::uuid[]) ORDER BY r.object_id, r.subject_id FOR UPDATE OF r) UPDATE ${tableIdentifier(name)} AS t SET deleted_at = now() FROM locked WHERE t.object_id = locked.object_id AND t.subject_id = locked.subject_id`,
    [objs, subs],
  );
}

export function softDeleteExact(name: string, objs: string[], subs: string[]) {
  return query(
    `WITH locked AS (SELECT r.object_id, r.subject_id FROM ${tableIdentifier(name)} r JOIN unnest(${objs}::uuid[], ${subs}::uuid[]) AS k(object_id, subject_id) USING (object_id, subject_id) ORDER BY r.object_id, r.subject_id FOR UPDATE OF r) UPDATE ${tableIdentifier(name)} AS t SET deleted_at = now() FROM locked WHERE t.object_id = locked.object_id AND t.subject_id = locked.subject_id`,
  );
}

export function lockRows(name: string, ids: string[]) {
  return query(
    `SELECT r.object_id, r.subject_id FROM ${tableIdentifier(name)} r WHERE r.object_id = ANY($1::uuid[]) ORDER BY r.object_id, r.subject_id FOR UPDATE OF r`,
    [ids],
  );
}
