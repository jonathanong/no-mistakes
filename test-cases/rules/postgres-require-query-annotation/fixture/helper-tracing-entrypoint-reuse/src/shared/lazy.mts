export function runLazy(sql: string, run: (statement: string) => unknown) {
  return run(sql);
}
