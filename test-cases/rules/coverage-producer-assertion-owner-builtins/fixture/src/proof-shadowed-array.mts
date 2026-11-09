declare function run(value?: unknown): void;

export function shadowedArray(Array: { from(source: unknown, callback: unknown): void }, rows: number[]) {
  Array.from(rows, run);
}
