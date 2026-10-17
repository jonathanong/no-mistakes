// Earlier evaluated collection operands must survive a later conditional operand.
export function build(flag?: string): string[] {
  return [outer(flag)];
}
function outer(flag?: string): string {
  return inner(
    "IF",
    [...(flag ? [`LOCK TABLE ${flag};`] : []), "LOCK TABLE t;"],
    ["SELECT 1"],
    ["UPDATE t SET n=1"],
  );
}
function inner(header: string, a: string[], b: string[], c: string[]): string {
  if (a.length) return ["SELECT 1", ...b.map(x => `  ${x}`)].join("\n");
  return ["SELECT 2", ...c].join("\n");
}
