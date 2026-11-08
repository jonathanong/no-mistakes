import { AsyncLocalStorage } from 'node:async_hooks';
import { join } from 'node:path';
const storage = new AsyncLocalStorage<string>();
export function format(values: number[], text: string): string {
  const seen = new Set<number>();
  const kept = values.filter((v) => Number.isFinite(v) && !seen.has(v)).map((v) => v * 2);
  const parts = text.trim().split(',').slice(1).map((p: string) => p.padStart(2, '0'));
  console.log(JSON.stringify(kept), Date.UTC(2020, 1, 1), /a/.test(text), parts.join('-'));
  return join(parts.join('/'), String(new Date(0).getTime()), String(storage.getStore()));
}
export async function settle(task: () => Promise<number>) {
  return Promise.allSettled([Promise.resolve(1), task()]);
}
