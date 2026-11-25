import queueModule, * as queue from "bullmq";

export const itemsWorker = new queue.Worker("items", async function handleItems() {
  if (queueModule) throw queue.RateLimitError();
  throw queue.Worker.RateLimitError();
});

function getQueue() {
  return queue;
}

export const widgetsWorker = new queue.Worker("widgets", async () => {
  throw getQueue().Worker.RateLimitError();
});
