import { expect } from "vitest";
import { screen } from "@testing-library/react";

declare function getName(): string;
declare function getMaybeName(): string | undefined;
declare function getAny(): any;
declare function getUnknown(): unknown;
declare function getVoid(): void;
declare function getNever(): never;
declare function getIntersection(): { id: string } & { role: string };
declare function getAnyIntersection(): { id: string } & any;
declare function getAsyncName(): Promise<string>;
declare function getAsyncMaybe(): Promise<string | undefined>;
declare const rootEl: HTMLElement;
declare const record: Record<string, string>;
declare const items: string[];
declare const pair: [string, number];
declare const unionItems: string[] | Record<string, string>;
declare const key: string;
declare const idx: number;

// Positive cases (defined-by-type)
expect(getName()).toBeDefined();
expect(getName()).not.toBeUndefined();
expect(screen.queryByText("Save")).toBeDefined();
expect(rootEl.querySelector(".item")).not.toBeUndefined();
expect(getIntersection()).toBeDefined();
expect(getAsyncName()).resolves.toBeDefined();

export function constrained<T extends string>(value: T) {
  expect(value).toBeDefined();
}

// Counterexamples (allowed)
expect(getMaybeName()).toBeDefined();
expect(getMaybeName()).not.toBeUndefined();
expect(getAny()).toBeDefined();
expect(getUnknown()).toBeDefined();
expect(getVoid()).toBeDefined();
expect(getNever()).toBeDefined();
expect(getAnyIntersection()).toBeDefined();
expect(getAsyncMaybe()).resolves.toBeDefined();
expect(getAsyncName()).rejects.toBeDefined();
expect.poll(() => getName()).toBeDefined();
expect(record[key]).toBeDefined();
expect(items[0]).toBeDefined();
expect(items[idx]).not.toBeUndefined();
expect(pair[0]).toBeDefined();
expect(unionItems[0]).toBeDefined();

export function unconstrained<T>(value: T) {
  expect(value).toBeDefined();
}
