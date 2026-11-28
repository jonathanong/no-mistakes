import { vi } from "vitest";
const spec = "@example/jobs";
let { enqueue } = require(spec);
vi.mocked(enqueue);
let { enqueue: job } = require(spec);
job = function local() {};
vi.mocked(job);
