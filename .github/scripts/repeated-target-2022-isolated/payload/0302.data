import { query } from "@example/db";
import { openTransaction } from "../transaction";

export async function mixed() {
  await query(`SELECT id FROM confirmed_first`);
  const tx = await openTransaction();
  await tx(`SELECT id FROM relative_second`);
}
