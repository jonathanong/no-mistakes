import { enqueueRefund } from "@app/jobs";
import { vi } from "vitest";
const owned = [];
// The returned promise is handled independently of the optional collection.
vi.spyOn(service, "refund").mockImplementation(data => {
  const p = enqueueRefund(data);
  if (cond) owned.push(p);
  return p;
});
