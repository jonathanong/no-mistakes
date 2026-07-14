import { betaQueue } from "./queues";

export const enqueueBeta = () => betaQueue.add("send", {});
