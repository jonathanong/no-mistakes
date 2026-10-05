import { openTransaction } from "@example/db/tx/open";

// A subpath factory import is matched too.
export async function moveOrderViaSubpath(orderId: string) {
  await using tx = await openTransaction();
  await tx(`SELECT * FROM orders WHERE id = ANY($1) FOR UPDATE`, [orderId]);
}
