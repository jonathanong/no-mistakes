import { query } from "@example/db";

export async function tx() {
  await query(`BEGIN`);
  await query(`/* tx */ COMMIT`);
  await query(`ROLLBACK`);
}
