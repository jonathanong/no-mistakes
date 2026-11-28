import { expect } from "vitest";

declare const record: Record<string, string>;
declare const items: string[];
declare const pair: [string, number];
declare const key: string;

// With noUncheckedIndexedAccess: true, in-bounds tuple access is string (reported)
expect(pair[0]).toBeDefined();

// Dynamic array/record lookups become string | undefined (allowed)
expect(record[key]).toBeDefined();
expect(items[0]).toBeDefined();
