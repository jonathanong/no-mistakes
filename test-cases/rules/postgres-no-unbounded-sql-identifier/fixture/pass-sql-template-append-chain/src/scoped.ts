import { write } from "@example/db";
import sql from "sql-template-strings";
import { eventColumns } from "./columns";

export async function insertScoped(
  scope: string,
  event: { type: string },
  normalized: { id: string },
) {
  return write(sql`INSERT INTO ${scope} (`.append(eventColumns()).append(sql`) VALUES (${normalized.id}, ${event.type})`));
}
