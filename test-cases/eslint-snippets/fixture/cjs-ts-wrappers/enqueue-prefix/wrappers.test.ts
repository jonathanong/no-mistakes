import { vi } from "vitest";
import { enqueueJob } from "@example/jobs";
(vi.mocked as typeof vi.mocked)(enqueueJob);
(vi as typeof vi).mocked(enqueueJob);
(vi.fn<typeof enqueueJob>)();
