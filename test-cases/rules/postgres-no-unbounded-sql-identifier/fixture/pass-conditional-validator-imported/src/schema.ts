// Both conditional branches return literals, including the nested fallback.
export function schemaOf(value: string) {
  return value === "app" ? "app" : value === "archive" ? `archive` : "fallback";
}
