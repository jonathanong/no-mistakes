function expensive(value) {
  const a = value + value;
  const b = a + a;
  const c = b + b;
  return c + c;
}
expensive("unrelated top-level work");
export const prefix = "/* shared */ SELECT 1";
export function queryText() {
  return prefix;
}
