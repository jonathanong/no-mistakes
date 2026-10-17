import { read, sql } from '@example/db';
export const query = sql`SELECT 1`;
function appendKnown(target: unknown) { target.append(sql`; DELETE FROM records`); }
export function computedObject(flag: boolean, input: unknown[]) {
  if (flag) {
    const callback = () => { for (const value of input) appendKnown(query); read(query); };
    const holder = { ['run']: callback };
    holder.run();
  }
}
export function spreadObject(flag: boolean, input: unknown[]) {
  if (flag) {
    const callback = () => { for (const value of input) appendKnown(query); read(query); };
    const holder = { ...{}, run: callback };
    holder.run();
  }
}
export function namedSpread(flag: boolean, input: unknown[]) {
  if (flag) {
    const callback = () => { for (const value of input) appendKnown(query); read(query); };
    const base = { run: callback }; const holder = { ...base };
    holder.run();
  }
}
export function objectAlias(flag: boolean, input: unknown[]) {
  if (flag) {
    const callback = () => { for (const value of input) appendKnown(query); read(query); };
    const holder = { ['run']: callback }; const alias = holder;
    alias.run();
  }
}
export function uninvokedObject(flag: boolean, input: unknown[]) {
  if (flag) {
    const callback = () => { for (const value of input) appendKnown(query); read(query); };
    const holder = { ['run']: callback };

  }
}
export function falseObject(input: unknown[]) {
  if (false) {
    const callback = () => { for (const value of input) appendKnown(query); read(query); };
    const holder = { ['run']: callback };
    holder.run();
  }
  read(query);
}
export function deepObject(flag: boolean, input: unknown[]) {
  if (flag) {
    const holder = { child: { child: { child: { child: { child: { child: { child: { child: { child: { child: { child: { child: { child: { child: { child: { child: { child: { child: { child: { child: { child: { child: { child: { child: { child: { ['run']: () => { for (const value of input) appendKnown(query); read(query); } } } } } } } } } } } } } } } } } } } } } } } } } } };
    holder.child.child.child.child.child.child.child.child.child.child.child.child.child.child.child.child.child.child.child.child.child.child.child.child.child.run();
  }
}
