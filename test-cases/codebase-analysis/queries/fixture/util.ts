export function used(a: string, b: { x: number }) {
  return a + b.x;
}

// `dead` is intentionally never imported anywhere — it exercises the
// dead-export detection. Do not add an importer for it.
export const dead = 42;

export function helper() {
  return 1;
}
