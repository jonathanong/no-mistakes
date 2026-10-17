import { read, sql } from '@example/db';
export const query = sql`SELECT 1`;
function appendKnown(target: unknown) { target.append(sql`; DELETE FROM records`); }
export function conditionalLiteral(flag: boolean, choose: boolean, input: unknown[]) {
  if (flag) {
    const callback = choose ? () => { for (const value of input) appendKnown(query); read(query); } : () => {};
    callback();
  }
}
export function conditionalNames(flag: boolean, choose: boolean, input: unknown[]) {
  if (flag) {
    const unsafe = () => { for (const value of input) appendKnown(query); read(query); };
    const callback = choose ? unsafe : () => {};
    callback();
  }
}
export function uninvokedConditional(flag: boolean, choose: boolean, input: unknown[]) {
  if (flag) {
    const callback = choose ? () => { for (const value of input) appendKnown(query); read(query); } : () => {};
  }
}
export function falseConditional(choose: boolean, input: unknown[]) {
  if (false) {
    const callback = choose ? () => { for (const value of input) appendKnown(query); read(query); } : () => {};
    callback();
  }
  read(query);
}
export function logicalOr(flag: boolean, choose: boolean, input: unknown[]) {
  if (flag) {
    const callback = choose || (() => { for (const value of input) appendKnown(query); read(query); });
    callback();
  }
}
export function logicalAnd(flag: boolean, choose: boolean, input: unknown[]) {
  if (flag) {
    const callback = choose && (() => { for (const value of input) appendKnown(query); read(query); });
    callback();
  }
}
export function logicalNullish(flag: boolean, choose: boolean | null, input: unknown[]) {
  if (flag) {
    const callback = choose ?? (() => { for (const value of input) appendKnown(query); read(query); });
    callback();
  }
}
export function deepConditional(flag: boolean, choose: boolean, input: unknown[]) {
  if (flag) {
    const holder = { child: { child: { child: { child: { child: { child: { child: { child: { child: { child: { child: { child: { child: { child: { child: { child: { child: { child: { child: { child: { child: { child: { child: { child: { child: choose ? () => { for (const value of input) appendKnown(query); read(query); } : () => {} } } } } } } } } } } } } } } } } } } } } } } } } };
    holder.child.child.child.child.child.child.child.child.child.child.child.child.child.child.child.child.child.child.child.child.child.child.child.child.child();
  }
}
