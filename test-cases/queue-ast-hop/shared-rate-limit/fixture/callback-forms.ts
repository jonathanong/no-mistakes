import { RateLimitError, Worker } from "bullmq";

function wrap(processor: () => Promise<void>) {
  return processor;
}

const handle = async function handleItems() {
  throw new RateLimitError();
};

let stableLet = async () => {
  throw new RateLimitError();
};

async function replaced() {
  throw new RateLimitError();
}

replaced = async () => {};

export const stable = new Worker("items", handle);
export const stableLetWorker = new Worker("items", stableLet);
export const mutable = new Worker("items", replaced);
export const opaque = new Worker("items", wrap(handle));
export const wrapped = new Worker("widgets", wrap(async () => {
  throw new RateLimitError();
}));

export function trailingHelper() {
  return 0;
}

export default function () {
  return 1;
}

declare const globalProcessor: () => Promise<void>;
const extraArgs: [() => Promise<void>] = [handle];
export const spreadArgs = new Worker("items", ...extraArgs);
export const unresolvedProcessor = new Worker("items", globalProcessor);
void stable.rateLimit(1000);
