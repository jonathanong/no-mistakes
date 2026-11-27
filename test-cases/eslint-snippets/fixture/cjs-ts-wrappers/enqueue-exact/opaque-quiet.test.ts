const spec = "@example/jobs";
const jobs = require(spec);
const { vi } = require(spec);
const { readFile } = require(spec);
vi.fn();
require(spec).vi.fn();
readFile(jobs);
function take(vi: unknown) {
  vi.fn<typeof jobs.enqueue>();
}
