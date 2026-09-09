declare const test: { setTimeout: (ms: number) => void };

export function run(setTimeout: () => void) {
  setTimeout();
  test.setTimeout(30_000);
}
