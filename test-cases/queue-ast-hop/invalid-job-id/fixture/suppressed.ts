// no-mistakes-disable-file queues-check: synthetic widget ids are intentional
import { widgets } from "./enqueue";

export function enqueueSuppressed() {
  return widgets.add("place", {}, { jobId: "user:1" });
}
