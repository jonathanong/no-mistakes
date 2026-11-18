export function runImportedSearch(
  statement: string,
  execute: (statement: string) => Promise<void>,
) {
  return execute(statement);
}
