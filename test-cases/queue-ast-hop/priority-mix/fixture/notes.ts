// no-mistakes-disable-file queues-check: mixed priorities are intentional
import { notes } from "./queues";

export function enqueuePlainNote(id: string) {
  return notes.add("writeNote", { id });
}

export function enqueueRushNote(id: string) {
  return notes.add("rushNote", { id }, { priority: 9 });
}
