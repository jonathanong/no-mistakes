import { vi } from "vitest";
const spec = "@example/jobs";
const jobs = require(spec);
vi.mocked(jobs.send);
vi.fn<typeof jobs.send>();
vi.fn<(...args: Parameters<typeof jobs.send>) => void>();
