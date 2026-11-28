export const leafTag = "leaf";

export function executeLeaf(sql: string, run: (statement: string) => unknown) {
  return run(sql);
}
