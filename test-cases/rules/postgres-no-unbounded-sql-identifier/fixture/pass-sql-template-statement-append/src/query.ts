import { beginTransaction } from "@example/db";
import sql from "sql-template-strings";
import { scoped } from "./scoped";

function owned(userId: string) {
  return sql` AND owner_id = ${userId}`;
}

export async function insertEvent(userId: string, flag: boolean) {
  await using query = await beginTransaction();
  const insert = sql`INSERT INTO events (user_id) VALUES (${userId})`;
  insert.append(owned(userId));
  if (flag) {
    insert.append(scoped(userId));
  }
  const first = await query(insert);
  const upsert = sql`INSERT INTO events (user_id) VALUES (${userId})`;
  upsert.append(scoped(userId));
  return query(upsert);
}
