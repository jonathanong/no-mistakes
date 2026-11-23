import { bundles } from "./queues";

export function enqueueFarBundle(job: object) {
  bundles.addBulk([{ ...job }]);
}
