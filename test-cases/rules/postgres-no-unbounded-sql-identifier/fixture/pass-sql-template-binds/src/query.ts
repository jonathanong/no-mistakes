import { beginTransaction } from "@example/db";
import sql from "sql-template-strings";

declare const input: { days: number; label: string };
declare function computePermissions(): { perms: string[] };

export async function createKey(userId: string, prefix: string, hash: string, type: string) {
  await using query = await beginTransaction();
  const { days, label } = input;
  const permissionResult = computePermissions();
  const perms = permissionResult.perms;
  await query(sql`/* createKey */
  WITH new_key AS (SELECT uuidv7() AS id)
  INSERT INTO keys (id, user_id, prefix, hash, type, label, scopes, expires_at)
  SELECT new_key.id, ${userId}::uuid, ${prefix}, ${hash}, ${type}, ${label}, ${perms}::scope_enum[],
    CASE WHEN ${days}::integer IS NULL THEN NULL ELSE uuid_extract_timestamp(new_key.id) + ${days}::integer * INTERVAL '24 hours' END
  FROM new_key RETURNING id`);
}

export async function createRequest(userId: string, requestedById: string) {
  await using query = await beginTransaction();
  await query(sql`/* createRequest */
  WITH lock AS MATERIALIZED (SELECT fn_lock_user(${userId}))
  INSERT INTO requests (user_id, requested_by_id) SELECT ${userId}, ${requestedById} FROM lock RETURNING id`);
}

export async function permissions(id: string) {
  await using query = await beginTransaction();
  const permissionResult = computePermissions();
  await query(sql`SELECT id FROM keys WHERE id = ${id} AND scopes = ${permissionResult.perms}`);
}
