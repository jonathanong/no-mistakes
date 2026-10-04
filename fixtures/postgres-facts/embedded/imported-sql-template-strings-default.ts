import sql from "sql-template-strings";
import { write } from "@example/db";

export async function scratchProbeInsert(mimeType: string): Promise<void> {
  await write(sql`INSERT INTO url_content_types (mime_type) VALUES (${mimeType})`);
}
