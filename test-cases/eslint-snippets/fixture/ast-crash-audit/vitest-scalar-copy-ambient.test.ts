import { vi } from "vitest";

declare const item: Record<string, number>;
const original = { testTimeout: 999_999, ...item };
const copy = { ...original };
copy.testTimeout += 1;
vi.setConfig(original);
