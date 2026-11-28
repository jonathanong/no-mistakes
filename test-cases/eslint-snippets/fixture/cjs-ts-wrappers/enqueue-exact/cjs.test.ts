const vt = require("vitest");
const jobs = require("@example/jobs");
vt.vi.mocked(jobs.enqueue);
const { vi: v } = require("vitest");
const { enqueue: enq } = require("@example/jobs");
v.mocked(enq);
import vt2 = require("vitest");
import jobs2 = require("@example/jobs");
vt2.vi.mocked(jobs2.enqueue);
