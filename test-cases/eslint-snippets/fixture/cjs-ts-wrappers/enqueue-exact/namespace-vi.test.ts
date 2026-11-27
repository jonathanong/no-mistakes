const V = require("vitest");
const { vi } = V;
const { enqueue } = require("@example/jobs");
vi.mocked(enqueue);
