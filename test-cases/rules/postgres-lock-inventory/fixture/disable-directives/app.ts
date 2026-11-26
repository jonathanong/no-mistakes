import { query } from "@example/db";
export async function reserve(id: number) {
  await lockAccountRow(id);
}

export async function reserveInline(id: number) {
  await lockAccountRow(id); // no-mistakes-disable-line postgres-lock-inventory: fixture exception
}
