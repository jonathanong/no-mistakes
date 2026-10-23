import { enqueueEmail } from "@app/jobs";
import { vi } from "vitest";
const mock = vi.fn();
// mocked preserves this function's rewritten method; it cannot restore provenance.
mock.mockImplementation = unknown;
vi.mocked(mock).mockImplementation(x => enqueueEmail(x));
service.send.mockImplementation = unknown;
vi.mocked(service.send).mockImplementationOnce(x => enqueueEmail(x));
