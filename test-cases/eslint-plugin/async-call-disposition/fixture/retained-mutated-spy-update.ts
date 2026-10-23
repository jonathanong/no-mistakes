import { enqueueEmail } from "@app/jobs";
import { vi } from "vitest";
// Repeated spyOn can retrieve the spy whose method was modified.
vi.spyOn(service, "send").mockImplementation++;
vi.spyOn(service, "send").mockImplementation(() => enqueueEmail("1"));
