import { vi } from "vitest";
const spec = "@example/jobs";
const jobs = require(spec);
vi.fn<(...args: Parameters<typeof jobs.enqueue>) => void>();
