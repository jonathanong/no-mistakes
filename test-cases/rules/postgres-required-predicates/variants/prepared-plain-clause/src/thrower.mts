export function assertImpossible(value: never): never {
  throw new Error(`unexpected: ${String(value)}`);
}
