import { RateLimitError, Worker } from "bullmq";

function wrap(processor: () => Promise<void>) {
  return processor;
}

const handle = async function handleItems() {
  throw new RateLimitError();
};

async function replaced() {
  throw new RateLimitError();
}

replaced = async () => {};

export const stable = new Worker("items", handle);
export const mutable = new Worker("items", replaced);
export const opaque = new Worker("items", wrap(handle));
export const wrapped = new Worker("widgets", wrap(async () => {
  throw new RateLimitError();
}));
