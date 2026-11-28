const frameworkName = "vitest";
const jobName = "@example/jobs";
const V = require(frameworkName);
const { vi } = V;
const jobs = require(jobName);
vi.mocked(jobs.enqueue);
