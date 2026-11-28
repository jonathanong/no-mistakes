import { vi } from "vitest";
const enq = require("@example/jobs")["enqueue"];
vi.mocked(enq);
