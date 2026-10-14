import { read, sql } from '@example/db';
export const query = sql`SELECT 1`;
function appendKnown(target: unknown) { target.append(sql`; DELETE FROM records`); }
function invoke(flag: boolean, unused: unknown, callback: () => void) {
  if (flag) { const local = 1; arguments[2](); }
}
export function argumentsCallback(flag: boolean, input: unknown[]) {
  const callback = () => { for (const value of input) appendKnown(query); read(query); };
  invoke(flag, 0, callback);
}
function uninvoked(flag: boolean, unused: unknown, callback: () => void) {
  if (flag) { const local = 1; }
}
export function unusedArguments(flag: boolean, input: unknown[]) {
  const callback = () => { for (const value of input) appendKnown(query); read(query); };
  uninvoked(flag, 0, callback);
}
function staticFalse(unused: unknown, callback: () => void) {
  if (false) { const local = 1; arguments[1](); }
}
export function falseArguments(input: unknown[]) {
  const callback = () => { for (const value of input) appendKnown(query); read(query); };
  staticFalse(0, callback);
  read(query);
}
function aliasInvoke(flag: boolean, unused: unknown, callback: () => void) {
  const slots = arguments;
  if (flag) { const local = 1; slots[2](); }
}
export function aliasedArguments(flag: boolean, input: unknown[]) {
  const callback = () => { for (const value of input) appendKnown(query); read(query); };
  aliasInvoke(flag, 0, callback);
}
