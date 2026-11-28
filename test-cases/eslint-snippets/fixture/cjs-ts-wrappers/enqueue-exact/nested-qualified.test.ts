import { vi } from "vitest";
const spec = "@example/jobs";
const jobs = require(spec);
vi.fn<typeof jobs.group.enqueue>();
