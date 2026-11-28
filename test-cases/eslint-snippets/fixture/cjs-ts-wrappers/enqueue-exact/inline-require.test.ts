import { enqueue } from "@example/jobs";

require("vitest").vi.mocked(enqueue);
