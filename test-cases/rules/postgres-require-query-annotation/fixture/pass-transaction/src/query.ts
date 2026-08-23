import { query } from "@data-stores/psql";

export async function tx() {
  await query(`BEGIN`);
  await query(`/* tx */ COMMIT`);
  await query(`ROLLBACK`);
}
