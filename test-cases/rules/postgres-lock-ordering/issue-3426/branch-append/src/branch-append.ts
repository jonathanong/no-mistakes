import sql from "sql-template-strings";
import { query } from "@app/db";
import { where, type Ref } from "./where";

export async function lockOne(ref: Ref, id: string) {
  const st = sql`SELECT s.id FROM s JOIN p ON p.id = s.p_id WHERE `;
  st.append(where(ref));
  st.append(sql` AND s.image_id = ${id} ORDER BY p.id DESC LIMIT 1 FOR UPDATE OF p`);
  await query(st);
}

export async function unorderedLock(ids: string[]) {
  await query(sql`SELECT p.id FROM p WHERE p.id = ANY(${ids}::uuid[]) FOR UPDATE OF p`);
}
