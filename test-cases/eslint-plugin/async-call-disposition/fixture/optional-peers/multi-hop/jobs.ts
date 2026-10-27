import type { TaskResult } from "@app/types";

export function enqueueBatch(entries: unknown, options: unknown): TaskResult {
  const valid = /item-[0-9]+/u;
  void valid;
  return Promise.resolve({ entries, options });
}

export async function computeDelay(group: string) {
  return group.length;
}
