import { read, sql } from '@example/db';
export const query = sql`SELECT 1`;
function appendKnown(target: unknown) { target.append(sql`; DELETE FROM records`); }
export function recordAlias(flag: boolean, input: unknown[]) {
  if (flag) {
    const callback = () => { for (const value of input) appendKnown(query); read(query); };
    const holder = { run: callback };
    const alias = holder;
    alias.run();
  }
}
export function arrayAlias(flag: boolean, input: unknown[]) {
  if (flag) {
    const callback = () => { for (const value of input) appendKnown(query); read(query); };
    const holder = [callback];
    const alias = holder;
    alias[0]();
  }
}
export function uninvokedAlias(flag: boolean, input: unknown[]) {
  if (flag) {
    const callback = () => { for (const value of input) appendKnown(query); read(query); };
    const holder = { run: callback };
    const alias = holder;
  }
}
export function falseAlias(input: unknown[]) {
  if (false) {
    const callback = () => { for (const value of input) appendKnown(query); read(query); };
    const holder = [callback];
    const alias = holder;
    alias[0]();
  }
  read(query);
}
export function publishedInsideArm(flag: boolean, input: unknown[]) {
  if (flag) {
    const holder = [() => { read(query); }];
    const alias = holder;
    // Publishing through an alias must update the same prepared container.
    alias[0] = () => { for (const value of input) appendKnown(query); read(query); };
    holder[0]();
  }
}
export function publishedOuterAlias(flag: boolean, input: unknown[]) {
  const holder = [() => { read(query); }];
  const alias = holder;
  if (flag) {
    const trigger = 0;
    alias[0] = () => { for (const value of input) appendKnown(query); read(query); };
    holder[0]();
  }
}
