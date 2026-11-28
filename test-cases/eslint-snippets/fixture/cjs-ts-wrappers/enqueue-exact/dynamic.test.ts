import { vi } from "vitest";
const spec = "@example/jobs";
const jobs = require(spec);
vi.mocked(jobs.enqueue);
vi.mocked(require("@example/jobs").enqueue);
vi.fn<typeof jobs.enqueue>();
// no-mistakes-disable-next-line test-no-typed-enqueue-fake -- dynamic job module
vi.mocked(jobs.enqueue);
vi.mocked(require(spec)["enqueue"]);
vi.mocked(jobs["enqueue"]);
