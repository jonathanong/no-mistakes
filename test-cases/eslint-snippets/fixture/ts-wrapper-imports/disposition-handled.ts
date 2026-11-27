import { run } from "@example/jobs";

export async function batched() {
  await Promise.all([run()] as const);
}

export async function caught() {
  run().catch(() => {}) as Promise<void>;
}

export async function drained() {
  const pending: Promise<void>[] = [];
  pending.push(run());
  await Promise.all(pending as Promise<void>[]);
}
