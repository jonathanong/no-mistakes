import { opaque } from "./queues";

export function enqueueOpaque(id: string, jobs: { name: string; data: { id: string } }[]) {
  opaque.addBulk(jobs);
  opaque.add("rushOpaque", { id }, { priority: 3 });
  opaque.add("storeOpaque", { id });
}
