// no-mistakes-disable-file postgres-lock-inventory: fixture exception
import { query } from "@example/db";
export async function reserve(id: number) {
  await lockAccountRow(id);
}
