export function enqueueEmail(entries: string, options: unknown): Promise<void>;
export function enqueueEmail(entries: unknown, options: unknown): Promise<void>;
export function enqueueEmail(entries: unknown, options: unknown): Promise<void> {
  return Promise.resolve();
}
