import sql from "sql-template-strings";

// Every function below contains one expression the model cannot represent
// (a sequence expression), in a different position. Running any of them must
// fail closed instead of producing partial SQL.
export function inMemberObject() {
  return sql`SELECT ${(1, 2).x}`;
}
export function inConditionalTest() {
  return (1, 2) ? sql`a` : sql`b`;
}
export function inConditionalBranch(flag: boolean) {
  return flag ? (1, 2) : sql`b`;
}
export function inAndRight(flag: boolean) {
  return sql`a`.append(flag && (1, 2));
}
export function inOrRight(flag: boolean) {
  return sql`a`.append(flag || (1, 2));
}
export function inNullishRight(a) {
  return sql`a`.append(a ?? (1, 2));
}
export function inNot() {
  return sql`${!(1, 2)}`;
}
export function inEquality() {
  return sql`${(1, 2) === 1}`;
}
export function inAddition() {
  return sql`${(1, 2) + "a"}`;
}
export function inObject() {
  return sql`${{ a: (1, 2) }}`;
}
export function inArray() {
  return sql`${[(1, 2)]}`;
}
export function inTaggedHole() {
  return sql`${(1, 2)}`;
}
export function inUntaggedHole() {
  return sql`a`.append(`${(1, 2)}`);
}
export function inAppendArgument() {
  return sql`a`.append((1, 2));
}
export function inAppendTarget() {
  return (1, 2).append(sql`a`);
}
export function inLoopIterable() {
  for (const item of (1, 2)) {
  }
  return sql`a`;
}
export function inLocal() {
  const value = (1, 2);
  return sql`a`;
}
export function inDefault(value = (1, 2)) {
  return sql`a`;
}
export function inIfTest() {
  if ((1, 2)) {
  }
  return sql`a`;
}
export function inReturn() {
  return (1, 2);
  sql`a`;
}
export function inCallArgument() {
  return other((1, 2));
}
export function inAndTest(flag: boolean) {
  return flag && (1, 2) ? sql`a` : sql`b`;
}
export function inOrTest(flag: boolean) {
  return flag || (1, 2) ? sql`a` : sql`b`;
}
export function inNotTest() {
  return !(1, 2) ? sql`a` : sql`b`;
}
export function inNegatedEqualityTest() {
  return (1, 2) !== 1 ? sql`a` : sql`b`;
}
export function inSpread() {
  return sql`a`.append({ ...(1, 2) }.x);
}
export function inLoopBody(items: string[]) {
  for (const item of items) {
    (1, 2);
  }
  return sql`a`;
}
export function inExpressionStatement() {
  (1, 2);
  return sql`a`;
}
export function inLoopReturn(items: string[]) {
  for (const item of items) {
    return (1, 2);
  }
  return sql`a`;
}
export function inCallInStatement(q) {
  other((1, 2));
  return sql`a`;
}
export function inObjectSpreadValue() {
  return sql`${{ ...{ a: (1, 2) } }}`;
}
