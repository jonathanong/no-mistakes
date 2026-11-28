// no-mistakes-disable-file test-no-unfailable-assertion -- legacy assertion harness
import { expect } from "vitest";

declare const value: string;

expect(value).toBe(value);
expect("ok").toBeDefined();
