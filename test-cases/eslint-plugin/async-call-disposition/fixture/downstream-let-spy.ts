import { enqueueEmail as original } from "@app/jobs";
import { vi } from "vitest";
let spy = vi.spyOn(service, "send");
spy.mockImplementation((a, b) => { if (selected) return original(a, b); return other(); });
let mock = vi.fn();
mock.mockImplementationOnce((a, b) => { if (selected) return original(a, b); return other(); });
