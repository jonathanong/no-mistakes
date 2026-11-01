import { write } from "@example/db";
import sql from "sql-template-strings";
import { eventColumns } from "./columns";

export async function insertEvent(
  scope: string,
  event: { type: string },
  normalized: { id: string },
  payload: string,
  receivedAt: string,
) {
  const r = await write(sql`/* insertEvent */
  WITH t AS (INSERT INTO event_types (type) VALUES (${event.type}) RETURNING id)
  , ins AS (INSERT INTO ${scope} (`.append(eventColumns()).append(sql`) VALUES (${normalized.id}, ${payload}::jsonb, ${receivedAt}) RETURNING id)
  SELECT 1`));
  return r;
}
