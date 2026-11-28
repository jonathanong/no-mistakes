const vi = require("vitest").vi;
const V = require("vitest");
const bound = V.vi;
const { enqueue } = require("@example/jobs");
vi.mocked(enqueue);
bound.mocked(enqueue);
