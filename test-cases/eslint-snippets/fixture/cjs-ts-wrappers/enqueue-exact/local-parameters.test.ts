import { vi } from "vitest";
type Parameters<T> = [string];
const spec = "@example/jobs";
const jobs = require(spec);
vi.fn<(...args: Parameters<typeof jobs.enqueue>) => void>();
