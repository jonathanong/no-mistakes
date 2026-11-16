import SQL from "sql-template-strings";
import * as db from "@example/db";

function where(ref: { kind: string; id: string }) {
  return ref.kind === "a" ? SQL`placement.a_id = ${ref.id}` : SQL`placement.b_id = ${ref.id}`;
}

export async function lockPlacement(ref: { kind: string; id: string }, imageId: string) {
  const stmt = SQL`SELECT placement.id FROM placements placement JOIN surfaces surface ON surface.id = placement.surface_id WHERE `;
  stmt.append(where(ref));
  stmt.append(SQL` AND surface.image_id = ${imageId} ORDER BY placement.id LIMIT 1 FOR UPDATE OF placement`);
  await db.query(stmt);
}

export function unparseableControl() {
  return db.query(SQL`SELECT id FROM placements FOR UPDATE OF`);
}
