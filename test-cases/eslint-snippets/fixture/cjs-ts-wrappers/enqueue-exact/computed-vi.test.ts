import { enqueue } from "@example/jobs";

const vi = require("vitest")["vi"];
vi.mocked(enqueue);
