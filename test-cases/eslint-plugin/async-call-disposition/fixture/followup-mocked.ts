import { enqueueEmail } from "@app/jobs";
import { vi } from "vitest";
vi.mocked(service.send).mockImplementation(x => enqueueEmail(x));
vi.mocked(service.send).mockImplementationOnce(x => { return enqueueEmail(x); });
