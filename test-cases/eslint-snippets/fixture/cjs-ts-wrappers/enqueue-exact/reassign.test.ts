import { enqueue } from "@example/jobs";
let { vi } = require("vitest");
vi = { mocked() {} };
vi.mocked(enqueue);
let { enqueue: job } = require("@example/jobs");
job = function local() {};
import { vi as runner } from "vitest";
runner.mocked(job);
