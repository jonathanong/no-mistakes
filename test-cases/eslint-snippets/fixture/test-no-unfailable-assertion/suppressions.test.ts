import { expect } from "vitest";

declare const value: string;

// no-mistakes-disable-next-line test-no-unfailable-assertion -- smoke check for assertion harness
expect(value).toBe(value);

expect(value).toBe(value); // no-mistakes-disable-line test-no-unfailable-assertion -- identity check

// Missing reason does not suppress:
// no-mistakes-disable-next-line test-no-unfailable-assertion
expect(value).toBe(value);

// Blank reason does not suppress:
expect(value).toBe(value); // no-mistakes-disable-line test-no-unfailable-assertion --
