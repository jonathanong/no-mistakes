import { enqueue } from "@example/jobs";

require("other").vi.mocked(enqueue);
