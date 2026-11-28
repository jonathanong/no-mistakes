import { vi } from "vitest";
const enqueue = require("@example/jobs").enqueue;
vi.mocked(enqueue);
