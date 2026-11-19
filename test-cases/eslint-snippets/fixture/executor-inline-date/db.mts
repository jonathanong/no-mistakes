export interface SessionExecutor {
  <T = unknown>(statement: string | { text: string }): Promise<{ rows: T[] }>;
  commit(): void;
}
