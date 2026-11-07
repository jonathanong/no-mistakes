export function getRelationTableNameOrThrow(value: unknown): string {
  return process.env.UNCHECKED_TABLE!;
}
