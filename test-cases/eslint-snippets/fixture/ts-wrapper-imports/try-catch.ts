import { handleRateLimit } from "@example/rate";

declare function work(): Promise<number>;

export async function constructed(error: unknown) {
  try {
    return new (Promise as typeof Promise)((resolve) => resolve(1));
  } catch {
    handleRateLimit(error);
  }
}

export async function awaited(error: unknown) {
  try {
    const p = work();
    (await p) as unknown;
    return p;
  } catch {
    handleRateLimit(error);
  }
}
