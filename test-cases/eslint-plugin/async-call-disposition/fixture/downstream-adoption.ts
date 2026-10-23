import { enqueueEmail } from "@app/jobs";
import { vi } from "vitest";
const pending = [];
vi.spyOn(service, "send").mockImplementation((...args) => {
  const result = Promise.resolve(enqueueEmail(...args)).finally(cleanup);
  pending.push(result);
  return result;
});
