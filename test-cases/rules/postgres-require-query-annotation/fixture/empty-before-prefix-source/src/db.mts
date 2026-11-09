export function write(statement: unknown): unknown {
  return statement;
}

export function sql(parts: TemplateStringsArray): string {
  return parts[0];
}
