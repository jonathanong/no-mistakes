const spec = "@example/jobs";
const jobs = require(spec);
const { vi } = require(spec);
vi.fn<typeof jobs.enqueue>();
require(spec).vi.fn<typeof jobs.enqueue>();
