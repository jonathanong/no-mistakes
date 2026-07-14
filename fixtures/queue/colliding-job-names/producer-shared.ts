import { alphaQueue, betaQueue } from "./queues";

// Both typed jobs intentionally collapse to the same public edge identity.
export const enqueueBoth = () =>
  Promise.all([alphaQueue.add("send", {}), betaQueue.add("send", {})]);
