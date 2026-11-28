import { expect } from "vitest";

declare function nextCount(): number;
declare function nextName(): string;
declare function getItem(index: number): string;
declare const value: string;
declare const other: string;
declare const user: { id: string };
declare const items: Array<{ name: string }>;
declare const idx: number;
declare const pending: Promise<string>;

// Positive cases (same-value)
expect("ok").toBe("ok");
expect(42).toBeDefined();
expect(true).toBeTruthy();
expect(-1).toEqual(-1);
expect(1n).toBe(1n);
expect(`static`).toBe("static");
expect(undefined).toBeUndefined();

const expected = "ready";
const alias = expected;
expect(expected).toBe("ready");
expect(alias).toBeDefined();

expect(value).toBe(value);
expect(user.id).toEqual(user.id);
expect(items[0].name).toStrictEqual(items[0].name);

export class Holder {
  id = "1";
  check() {
    expect(this.id).toBe(this.id);
  }
}

// Counterexamples (allowed)
let count = 0;
count = nextCount();
expect(count).toBe(0);

let current = "init";
const before = current;
current = nextName();
expect(current).toBe(before);

const box = { value: 1 };
box.value = nextCount();
expect(box.value).toBe(1);

expect("ok").toBe(value);
expect(value).toBe(other);
expect(value).not.toBe(value);
expect(pending).resolves.toBe(pending);
expect(getItem(0)).toBe(getItem(0));
expect(items[idx]).toBe(items[idx]);

const pattern = /foo/g;
expect(pattern).toBe(other);

function shadowedExpect() {
  const expect = (v: unknown) => ({ toBe: (_: unknown) => v });
  expect(1).toBe(1);
}
void shadowedExpect;
